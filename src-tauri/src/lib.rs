mod actions;
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod apple_intelligence;
mod asr_evaluation;
mod audio_feedback;
pub mod audio_toolkit;
mod catalog;
pub mod cli;
mod clipboard;
mod commands;
pub mod contracts;
mod helpers;
mod input;
mod llm_client;
mod managers;
mod model_install;
mod overlay;
mod platform_context;
pub mod portable;
mod settings;
mod shortcut;
mod signal_handle;
pub mod storage;
mod transcription_coordinator;
mod tray;
mod tray_i18n;
mod utils;

pub use cli::CliArgs;
#[cfg(debug_assertions)]
use specta_typescript::{BigIntExportBehavior, Typescript};
use tauri_specta::{collect_commands, collect_events, Builder};

use env_filter::Builder as EnvFilterBuilder;
use managers::audio::AudioRecordingManager;
use managers::dictionary::DictionaryManager;
use managers::history::HistoryManager;
use managers::model::ModelManager;
use managers::snippets::SnippetManager;
use managers::transcription::TranscriptionManager;
#[cfg(unix)]
use signal_hook::consts::{SIGUSR1, SIGUSR2};
#[cfg(unix)]
use signal_hook::iterator::Signals;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use tauri::image::Image;
pub use transcription_coordinator::TranscriptionCoordinator;

use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Listener, Manager};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_log::{Builder as LogBuilder, RotationStrategy, Target, TargetKind};

use crate::settings::get_settings;

// Global atomic to store the file log level filter
// We use u8 to store the log::LevelFilter as a number
pub static FILE_LOG_LEVEL: AtomicU8 = AtomicU8::new(log::LevelFilter::Debug as u8);

/// When `true`, log records are also forwarded to the webview via the
/// `log://log` event for the debug panel's live log viewer. Gated on debug
/// mode — the live log viewer is its only consumer and only exists in debug
/// mode — so normal runs never broadcast log records (which can include file
/// paths or transcribed text) onto the frontend event bus. Synced at startup
/// and whenever debug mode is toggled (see `shortcut::change_debug_mode_setting`).
pub static WEBVIEW_LOG_STREAMING: AtomicBool = AtomicBool::new(false);

fn level_filter_from_u8(value: u8) -> log::LevelFilter {
    match value {
        0 => log::LevelFilter::Off,
        1 => log::LevelFilter::Error,
        2 => log::LevelFilter::Warn,
        3 => log::LevelFilter::Info,
        4 => log::LevelFilter::Debug,
        5 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Trace,
    }
}

fn build_console_filter() -> env_filter::Filter {
    let mut builder = EnvFilterBuilder::new();

    match std::env::var("RUST_LOG") {
        Ok(spec) if !spec.trim().is_empty() => {
            if let Err(err) = builder.try_parse(&spec) {
                log::warn!(
                    "Ignoring invalid RUST_LOG value '{}': {}. Falling back to info-level console logging",
                    spec,
                    err
                );
                builder.filter_level(log::LevelFilter::Info);
            }
        }
        _ => {
            builder.filter_level(log::LevelFilter::Info);
        }
    }

    builder.build()
}

fn show_main_window(app: &AppHandle) {
    if let Some(main_window) = app.get_webview_window("main") {
        if let Err(e) = main_window.unminimize() {
            log::error!("Failed to unminimize webview window: {}", e);
        }
        if let Err(e) = main_window.show() {
            log::error!("Failed to show webview window: {}", e);
        }
        if let Err(e) = main_window.set_focus() {
            log::error!("Failed to focus webview window: {}", e);
        }
        #[cfg(target_os = "macos")]
        {
            if let Err(e) = app.set_activation_policy(tauri::ActivationPolicy::Regular) {
                log::error!("Failed to set activation policy to Regular: {}", e);
            }
        }
        return;
    }

    let webview_labels = app.webview_windows().keys().cloned().collect::<Vec<_>>();
    log::error!(
        "Main window not found. Webview labels: {:?}",
        webview_labels
    );
}

#[allow(unused_variables)]
fn should_force_show_permissions_window(app: &AppHandle) -> bool {
    #[cfg(target_os = "windows")]
    {
        let model_manager = app.state::<Arc<ModelManager>>();
        let has_downloaded_models = model_manager
            .get_available_models()
            .iter()
            .any(|model| model.is_downloaded);

        if !has_downloaded_models {
            return false;
        }

        let status = commands::audio::get_windows_microphone_permission_status();
        if status.supported && status.overall_access == commands::audio::PermissionAccess::Denied {
            log::info!(
                "Windows microphone permissions are denied; forcing main window visible for onboarding"
            );
            return true;
        }
    }

    false
}

fn initialize_core_logic(app_handle: &AppHandle) {
    // Note: Enigo (keyboard/mouse simulation) is NOT initialized here.
    // The frontend is responsible for calling the `initialize_enigo` command
    // after onboarding completes. This avoids triggering permission dialogs
    // on macOS before the user is ready.

    // Initialize the managers. The audio recorder receives the streaming router
    // explicitly, so always-on microphone startup can wire live-preview frames
    // even before Tauri state is populated.
    let model_manager =
        Arc::new(ModelManager::new(app_handle).expect("Failed to initialize model manager"));
    let transcription_manager = Arc::new(
        TranscriptionManager::new(app_handle, model_manager.clone())
            .expect("Failed to initialize transcription manager"),
    );
    let recording_manager = Arc::new(
        AudioRecordingManager::new(app_handle, transcription_manager.stream_router())
            .expect("Failed to initialize recording manager"),
    );
    let history_manager =
        Arc::new(HistoryManager::new(app_handle).expect("Failed to initialize history manager"));
    let dictionary_manager = Arc::new(
        DictionaryManager::new(app_handle).expect("Failed to initialize dictionary manager"),
    );
    let snippet_manager =
        Arc::new(SnippetManager::new(app_handle).expect("Failed to initialize snippet manager"));

    // Initialize the transcribe-cpp native backend (logging + backend module
    // registration) once, before any whisper model is loaded.
    managers::transcription::init_transcribe_backend();

    // Apply accelerator preferences before any model loads
    managers::transcription::apply_accelerator_settings(app_handle);

    // Add managers to Tauri's managed state
    app_handle.manage(recording_manager.clone());
    app_handle.manage(model_manager.clone());
    app_handle.manage(transcription_manager.clone());
    app_handle.manage(history_manager.clone());
    app_handle.manage(dictionary_manager.clone());
    app_handle.manage(snippet_manager.clone());
    app_handle.manage(clipboard::InsertionState::new());
    app_handle.manage(tray::CurrentTrayIconState::new());

    // Note: Shortcuts are NOT initialized here.
    // The frontend is responsible for calling the `initialize_shortcuts` command
    // after permissions are confirmed (on macOS) or after onboarding completes.
    // This matches the pattern used for Enigo initialization.

    #[cfg(unix)]
    let signals = Signals::new([SIGUSR1, SIGUSR2]).unwrap();
    // Set up signal handlers for toggling transcription
    #[cfg(unix)]
    signal_handle::setup_signal_handler(app_handle.clone(), signals);

    // Apply macOS Accessory policy if starting hidden and tray is available.
    // If the tray icon is disabled, keep the dock icon so the user can reopen.
    #[cfg(target_os = "macos")]
    {
        let settings = settings::get_settings(app_handle);
        if settings.start_hidden && settings.show_tray_icon {
            let _ = app_handle.set_activation_policy(tauri::ActivationPolicy::Accessory);
        }
    }
    // Get the current theme to set the appropriate initial icon
    let initial_theme = tray::get_current_theme(app_handle);

    // Choose the appropriate initial icon based on theme
    let initial_icon_path = tray::get_icon_path(initial_theme, tray::TrayIconState::Idle);

    let tray = TrayIconBuilder::new()
        .icon(
            Image::from_path(
                app_handle
                    .path()
                    .resolve(initial_icon_path, tauri::path::BaseDirectory::Resource)
                    .unwrap(),
            )
            .unwrap(),
        )
        .tooltip(tray::tray_tooltip())
        .show_menu_on_left_click(true)
        .icon_as_template(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" | "open_hub" => {
                show_main_window(app);
            }
            "toggle_dictation" => {
                app.state::<TranscriptionCoordinator>().send_input(
                    "transcribe",
                    "tray",
                    true,
                    false,
                );
            }
            "history" => {
                show_main_window(app);
                let _ = app.emit("navigate-section", "history");
            }
            "microphone_settings" | "language_settings" => {
                show_main_window(app);
                let _ = app.emit("navigate-section", "general");
            }
            "copy_last_transcript" => {
                tray::copy_last_transcript(app);
            }
            "paste_last_transcript" => {
                tray::paste_last_transcript(app);
            }
            "unload_model" => {
                let transcription_manager = app.state::<Arc<TranscriptionManager>>();
                if !transcription_manager.is_model_loaded() {
                    log::warn!("No model is currently loaded.");
                    return;
                }
                match transcription_manager.unload_model() {
                    Ok(()) => log::info!("Model unloaded via tray."),
                    Err(e) => log::error!("Failed to unload model via tray: {}", e),
                }
            }
            "cancel" => {
                use crate::utils::cancel_current_operation;

                // Use centralized cancellation that handles all operations
                cancel_current_operation(app);
            }
            "quit" => {
                app.exit(0);
            }
            id if id.starts_with("model_select:") => {
                let model_id = id.strip_prefix("model_select:").unwrap().to_string();
                let current_model = settings::get_settings(app).selected_model;
                if model_id == current_model {
                    return;
                }
                let app_clone = app.clone();
                std::thread::spawn(move || {
                    match commands::models::switch_active_model(&app_clone, &model_id) {
                        Ok(()) => {
                            log::info!("Model switched to {} via tray.", model_id);
                        }
                        Err(e) => {
                            log::error!("Failed to switch model via tray: {}", e);
                        }
                    }
                    tray::update_tray_menu(&app_clone, None);
                });
            }
            _ => {}
        })
        .build(app_handle)
        .unwrap();
    app_handle.manage(tray);

    // Initialize tray menu with idle state
    utils::update_tray_menu(app_handle, None);

    // Apply show_tray_icon setting
    let settings = settings::get_settings(app_handle);
    if !settings.show_tray_icon {
        tray::set_tray_visibility(app_handle, false);
    }

    // Refresh tray menu when model state changes
    let app_handle_for_listener = app_handle.clone();
    app_handle.listen("model-state-changed", move |_| {
        tray::update_tray_menu(&app_handle_for_listener, None);
    });

    // Get the autostart manager and configure based on user setting
    let autostart_manager = app_handle.autolaunch();
    let settings = settings::get_settings(app_handle);

    if settings.autostart_enabled {
        // Enable autostart if user has opted in
        let _ = autostart_manager.enable();
    } else {
        // Disable autostart if user has opted out
        let _ = autostart_manager.disable();
    }

    // Create the recording overlay window (hidden by default)
    utils::create_recording_overlay(app_handle);

    let overlay_dock_handle = app_handle.clone();
    app_handle.listen("overlay-drag-started", |_| {
        overlay::begin_overlay_drag();
    });
    app_handle.listen("overlay-drag-finished", move |_| {
        overlay::finish_overlay_drag(&overlay_dock_handle);
    });
}

#[tauri::command]
#[specta::specta]
fn show_main_window_command(app: AppHandle) -> Result<(), String> {
    show_main_window(&app);
    Ok(())
}

/// Headless one-shot transcription for the `--transcribe-file` / `--list-devices`
/// path. Drives the same `TranscriptionManager::transcribe` the app uses; no
/// mic, no VAD, no download. Returns a process exit code (0 ok, 1 runtime
/// failure, 2 bad input/usage).
fn run_headless_transcription(app: &AppHandle, args: &CliArgs) -> i32 {
    use std::time::Instant;

    if let Some(source_path) = args.install_model_file.as_deref() {
        let model_id = args
            .model
            .as_deref()
            .expect("clap requires --model with --install-model-file");
        let accepted_manifest_digest = args
            .accept_model_manifest_digest
            .as_deref()
            .expect("clap requires manifest acceptance with --install-model-file");
        let model_manager = app.state::<Arc<ModelManager>>();
        match model_manager.install_model_from_file(model_id, source_path, accepted_manifest_digest)
        {
            Ok(()) => {
                if args.json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "schema_version": 1,
                            "model_id": model_id,
                            "source": source_path,
                            "manifest_digest": accepted_manifest_digest,
                            "installed": true,
                        })
                    );
                } else {
                    println!("Installed and verified model '{model_id}'.");
                }
                return 0;
            }
            Err(error) => {
                eprintln!("error: local model install failed: {error:#}");
                return 1;
            }
        }
    }

    if let Some(seconds) = args.verify_audio {
        return run_headless_audio_verification(app, seconds.max(1), args.repeat, args.json);
    }

    // --list-devices: print registered compute devices (with indices) and exit.
    // Useful on multi-GPU machines to discover the index for --device-index.
    if args.list_devices {
        let devices = crate::managers::transcription::describe_compute_devices();
        if devices.is_empty() {
            println!("No transcribe-cpp compute devices registered.");
        } else {
            println!("transcribe-cpp compute devices:");
            for d in &devices {
                println!("  {}", d);
            }
        }
        if args.transcribe_file.is_none() {
            return 0;
        }
    }

    // --list-models: print the model registry (catalog + on-disk + custom) with
    // their ids — the same ids `--model` accepts — then exit. `--json` emits the
    // full ModelInfo array for scripting.
    if args.list_models {
        let model_manager = app.state::<Arc<ModelManager>>();
        let models = model_manager.get_available_models();
        if args.json {
            match serde_json::to_string_pretty(&models) {
                Ok(s) => println!("{}", s),
                Err(e) => {
                    eprintln!("error: failed to serialize models: {}", e);
                    return 1;
                }
            }
        } else if models.is_empty() {
            println!("No models available.");
        } else {
            println!("Available models (✓ = installed):");
            let width = models.iter().map(|m| m.id.len()).max().unwrap_or(0);
            for m in &models {
                let mark = if m.is_downloaded { "✓" } else { " " };
                let rec = if m.is_recommended {
                    "  [recommended]"
                } else {
                    ""
                };
                println!(
                    "  {}  {:<width$}  {}{}",
                    mark,
                    m.id,
                    m.name,
                    rec,
                    width = width
                );
            }
        }
        if args.transcribe_file.is_none() {
            return 0;
        }
    }

    if let Some(manifest_path) = args.evaluate_corpus.as_deref() {
        return run_headless_corpus_evaluation(app, args, manifest_path);
    }

    let Some(wav) = args.transcribe_file.clone() else {
        return 0;
    };

    // read_wav_samples reads 16-bit int samples and does no validation; the app
    // only ever saves 16 kHz mono 16-bit PCM, so reject anything else rather than
    // transcribe garbage / mis-time / mis-decode.
    match hound::WavReader::open(&wav) {
        Ok(reader) => {
            let spec = reader.spec();
            if spec.sample_rate != 16_000
                || spec.channels != 1
                || spec.bits_per_sample != 16
                || spec.sample_format != hound::SampleFormat::Int
            {
                eprintln!(
                    "error: expected 16 kHz mono 16-bit PCM WAV, got {} Hz / {} ch / {}-bit {:?}",
                    spec.sample_rate, spec.channels, spec.bits_per_sample, spec.sample_format
                );
                return 2;
            }
        }
        Err(e) => {
            eprintln!("error: cannot open {}: {}", wav.display(), e);
            return 2;
        }
    }

    let samples = match crate::audio_toolkit::read_wav_samples(&wav) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: failed to read {}: {}", wav.display(), e);
            return 2;
        }
    };
    let audio_secs = samples.len() as f64 / 16_000.0;

    let tm = app.state::<Arc<TranscriptionManager>>();

    let model_id = args
        .model
        .clone()
        .unwrap_or_else(|| get_settings(app).selected_model);
    if model_id.is_empty() {
        eprintln!("error: no model selected (pass --model or pick one in the app)");
        return 2;
    }

    // --device-index hard-selects a compute device by its --list-devices registry
    // index (transcribe-cpp / whisper-family models only; not persisted). Omit it
    // to use the persisted accelerator setting.
    let device_index = args.device_index;
    let requested_device = match device_index {
        Some(idx) => format!("index {}", idx),
        None => "settings".to_string(),
    };

    // Cold load (timed).
    let load_start = Instant::now();
    if let Err(e) = tm.load_model_with_device(&model_id, device_index) {
        eprintln!("error: load_model('{}') failed: {}", model_id, e);
        return 1;
    }
    let load_ms = load_start.elapsed().as_millis() as u64;
    let bound_backend = tm.current_backend();

    let runs = args.repeat.unwrap_or(1).max(1);
    let mut times_ms: Vec<u64> = Vec::new();
    let mut last_outcome = None;
    for i in 0..runs {
        // If the model's unload-timeout is "Immediately", transcribe() unloads
        // the engine after each run; reload (untimed) so repeats keep working
        // and the inference timing below stays clean.
        if !tm.is_model_loaded() {
            if let Err(e) = tm.load_model_with_device(&model_id, device_index) {
                eprintln!("error: reload before run {} failed: {}", i + 1, e);
                return 1;
            }
        }
        let t = Instant::now();
        match tm.transcribe_detailed(samples.clone()) {
            Ok(outcome) => last_outcome = Some(outcome),
            Err(e) => {
                eprintln!("error: transcribe failed: {}", e);
                return 1;
            }
        }
        times_ms.push(t.elapsed().as_millis() as u64);
    }
    let best_ms = times_ms.iter().copied().min().unwrap_or(0);
    let rtf = if best_ms > 0 {
        audio_secs / (best_ms as f64 / 1000.0)
    } else {
        0.0
    };
    let outcome = last_outcome.expect("at least one transcription run");

    if args.json {
        println!(
            "{}",
            serde_json::json!({
                "model": model_id,
                "requested_device": requested_device,
                "bound_backend": bound_backend,
                "audio_secs": audio_secs,
                "load_ms": load_ms,
                "transcribe_ms": times_ms,
                "best_ms": best_ms,
                "rtf": rtf,
                "raw_text": outcome.raw_text,
                "text": outcome.text,
                "requested_language": outcome.requested_language,
                "effective_language": outcome.effective_language,
                "detected_language": outcome.detected_language,
            })
        );
    } else {
        println!(
            "model={} device={} backend={} audio={:.2}s load={}ms best={}ms rtf={:.2}x",
            model_id,
            requested_device,
            bound_backend.as_deref().unwrap_or("?"),
            audio_secs,
            load_ms,
            best_ms,
            rtf,
        );
        println!("text: {}", outcome.text);
    }
    0
}

fn read_evaluation_wav(path: &std::path::Path) -> Result<Vec<f32>, String> {
    let reader = hound::WavReader::open(path)
        .map_err(|error| format!("cannot open {}: {error}", path.display()))?;
    let spec = reader.spec();
    if spec.sample_rate != 16_000
        || spec.channels != 1
        || spec.bits_per_sample != 16
        || spec.sample_format != hound::SampleFormat::Int
    {
        return Err(format!(
            "expected 16 kHz mono 16-bit PCM WAV for {}, got {} Hz / {} ch / {}-bit {:?}",
            path.display(),
            spec.sample_rate,
            spec.channels,
            spec.bits_per_sample,
            spec.sample_format
        ));
    }
    crate::audio_toolkit::read_wav_samples(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))
}

fn run_headless_corpus_evaluation(
    app: &AppHandle,
    args: &CliArgs,
    manifest_path: &std::path::Path,
) -> i32 {
    use crate::asr_evaluation::{
        percentile, required_terms_present, resident_memory_bytes, word_errors, EvaluationSummary,
        EvaluationThresholdsResult, ItemScore,
    };

    let manifest = match crate::asr_evaluation::load_manifest(manifest_path) {
        Ok(manifest) => manifest,
        Err(error) => {
            eprintln!("error: {error:#}");
            return 2;
        }
    };
    let model_id = args
        .model
        .clone()
        .unwrap_or_else(|| get_settings(app).selected_model);
    if model_id.is_empty() {
        eprintln!("error: no model selected (pass --model or pick one in the app)");
        return 2;
    }

    let network_denial = match args.require_network_denied.as_deref() {
        Some(target) => match crate::asr_evaluation::require_network_denied(target) {
            Ok(evidence) => Some(evidence),
            Err(error) => {
                eprintln!("error: {error:#}");
                return 1;
            }
        },
        None => None,
    };

    let manager = app.state::<Arc<TranscriptionManager>>();
    let resident_memory_before_load_bytes = resident_memory_bytes();
    if let Err(error) = manager.load_model_with_device(&model_id, args.device_index) {
        eprintln!("error: load_model('{model_id}') failed: {error}");
        return 1;
    }
    let resident_memory_after_load_bytes = resident_memory_bytes();
    let manifest_dir = manifest_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let mut scores = Vec::with_capacity(manifest.items.len());
    let mut aggregate_raw_errors = crate::asr_evaluation::WordErrors::default();
    let mut aggregate_corrected_errors = crate::asr_evaluation::WordErrors::default();
    let mut successful_tasks = 0usize;
    let mut latencies = Vec::with_capacity(manifest.items.len());

    for item in &manifest.items {
        let audio_path = if item.audio.is_absolute() {
            item.audio.clone()
        } else {
            manifest_dir.join(&item.audio)
        };
        let samples = match read_evaluation_wav(&audio_path) {
            Ok(samples) if !samples.is_empty() => samples,
            Ok(_) => {
                eprintln!("error: corpus item '{}' has no samples", item.id);
                return 2;
            }
            Err(error) => {
                eprintln!("error: corpus item '{}': {error}", item.id);
                return 2;
            }
        };
        if !manager.is_model_loaded() {
            if let Err(error) = manager.load_model_with_device(&model_id, args.device_index) {
                eprintln!(
                    "error: reload before corpus item '{}' failed: {error}",
                    item.id
                );
                return 1;
            }
        }
        let outcome =
            match manager.transcribe_detailed_with_language(samples, item.language.clone()) {
                Ok(outcome) => outcome,
                Err(error) => {
                    eprintln!(
                        "error: corpus item '{}' transcription failed: {error}",
                        item.id
                    );
                    return 1;
                }
            };
        let raw_errors = word_errors(&item.reference, &outcome.raw_text);
        let corrected_errors = word_errors(&item.reference, &outcome.text);
        aggregate_raw_errors.substitutions += raw_errors.substitutions;
        aggregate_raw_errors.deletions += raw_errors.deletions;
        aggregate_raw_errors.insertions += raw_errors.insertions;
        aggregate_raw_errors.reference_words += raw_errors.reference_words;
        aggregate_corrected_errors.substitutions += corrected_errors.substitutions;
        aggregate_corrected_errors.deletions += corrected_errors.deletions;
        aggregate_corrected_errors.insertions += corrected_errors.insertions;
        aggregate_corrected_errors.reference_words += corrected_errors.reference_words;
        let task_success = required_terms_present(&outcome.text, &item.required_terms);
        successful_tasks += usize::from(task_success);
        latencies.push(outcome.transcription_ms);
        scores.push(ItemScore {
            id: item.id.clone(),
            audio: audio_path.display().to_string(),
            reference: item.reference.clone(),
            hypothesis: outcome.text,
            raw_hypothesis: outcome.raw_text,
            requested_language: outcome.requested_language,
            effective_language: outcome.effective_language,
            detected_language: outcome.detected_language,
            raw_substitutions: raw_errors.substitutions,
            raw_deletions: raw_errors.deletions,
            raw_insertions: raw_errors.insertions,
            raw_wer: raw_errors.wer(),
            corrected_substitutions: corrected_errors.substitutions,
            corrected_deletions: corrected_errors.deletions,
            corrected_insertions: corrected_errors.insertions,
            reference_words: corrected_errors.reference_words,
            corrected_wer: corrected_errors.wer(),
            task_success,
            audio_duration_ms: outcome.audio_duration_ms,
            transcription_ms: outcome.transcription_ms,
            real_time_factor: outcome.real_time_factor,
        });
    }

    let raw_wer = aggregate_raw_errors.wer();
    let corrected_wer = aggregate_corrected_errors.wer();
    let task_success = successful_tasks as f64 / scores.len() as f64;
    let latency_p50_ms = percentile(&latencies, 0.50);
    let latency_p95_ms = percentile(&latencies, 0.95);
    let resident_memory_after_evaluation_bytes = resident_memory_bytes();
    let thresholds = EvaluationThresholdsResult {
        max_wer: manifest.thresholds.max_wer,
        min_task_success: manifest.thresholds.min_task_success,
        max_p50_ms: manifest.thresholds.max_p50_ms,
        max_p95_ms: manifest.thresholds.max_p95_ms,
        max_idle_rss_bytes: manifest.thresholds.max_idle_rss_bytes,
        max_loaded_rss_bytes: manifest.thresholds.max_loaded_rss_bytes,
        raw_wer_passed: raw_wer <= manifest.thresholds.max_wer,
        task_success_passed: task_success >= manifest.thresholds.min_task_success,
        p50_passed: latency_p50_ms <= manifest.thresholds.max_p50_ms,
        p95_passed: latency_p95_ms <= manifest.thresholds.max_p95_ms,
        idle_rss_passed: resident_memory_before_load_bytes
            .is_some_and(|bytes| bytes <= manifest.thresholds.max_idle_rss_bytes),
        loaded_rss_passed: resident_memory_after_load_bytes
            .into_iter()
            .chain(resident_memory_after_evaluation_bytes)
            .all(|bytes| bytes <= manifest.thresholds.max_loaded_rss_bytes)
            && resident_memory_after_load_bytes.is_some()
            && resident_memory_after_evaluation_bytes.is_some(),
    };
    let passed = thresholds.raw_wer_passed
        && thresholds.task_success_passed
        && thresholds.p50_passed
        && thresholds.p95_passed
        && thresholds.idle_rss_passed
        && thresholds.loaded_rss_passed;
    let summary = EvaluationSummary {
        schema_version: 1,
        corpus_name: manifest.name,
        corpus_source: manifest.source,
        corpus_license: manifest.license,
        model_id,
        item_count: scores.len(),
        raw_wer,
        corrected_wer,
        task_success,
        latency_p50_ms,
        latency_p95_ms,
        resident_memory_before_load_bytes,
        resident_memory_after_load_bytes,
        resident_memory_after_evaluation_bytes,
        network_denial,
        thresholds,
        passed,
        items: scores,
    };
    match serde_json::to_string_pretty(&summary) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("error: failed to serialize corpus result: {error}");
            return 1;
        }
    }
    i32::from(!passed)
}

fn run_headless_audio_verification(
    app: &AppHandle,
    seconds: u64,
    repeat: Option<usize>,
    json: bool,
) -> i32 {
    use crate::audio_toolkit::VadPolicy;
    use std::time::{Duration, Instant};

    let manager = app.state::<Arc<AudioRecordingManager>>();
    let settings = get_settings(app);
    let requested_device = settings
        .selected_microphone
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let runs = repeat.unwrap_or(1).max(1);
    let mut microphone_ready_ms = Vec::with_capacity(runs);
    let mut sample_counts = Vec::with_capacity(runs);
    let mut peaks = Vec::with_capacity(runs);

    for run in 0..runs {
        let binding_id = format!("ff-v2-live-{run}");
        let started = Instant::now();
        if let Err(error) = manager.try_start_recording(&binding_id, VadPolicy::Disabled) {
            eprintln!("error: microphone start failed: {error}");
            return 1;
        }
        microphone_ready_ms.push(started.elapsed().as_secs_f64() * 1_000.0);
        std::thread::sleep(Duration::from_secs(seconds));
        let generation = manager.cancel_generation();
        let Some(samples) = manager.stop_recording(&binding_id, generation) else {
            eprintln!("error: microphone stop returned no capture");
            return 1;
        };
        let peak = samples
            .iter()
            .map(|sample| sample.abs())
            .fold(0.0_f32, f32::max);
        if samples.len() < seconds as usize * 8_000 {
            eprintln!(
                "error: capture {} returned only {} samples for {} second(s)",
                run + 1,
                samples.len(),
                seconds
            );
            return 1;
        }
        sample_counts.push(samples.len());
        peaks.push(peak);
    }

    let cancel_binding = "ff-v2-live-cancel";
    if let Err(error) = manager.try_start_recording(cancel_binding, VadPolicy::Disabled) {
        eprintln!("error: cancellation probe start failed: {error}");
        return 1;
    }
    std::thread::sleep(Duration::from_millis(100));
    manager.cancel_recording();
    if manager.is_recording() {
        eprintln!("error: cancellation probe left the manager recording");
        return 1;
    }

    microphone_ready_ms.sort_by(|left, right| left.total_cmp(right));
    let p95_index = ((microphone_ready_ms.len() as f64 * 0.95).ceil() as usize)
        .saturating_sub(1)
        .min(microphone_ready_ms.len().saturating_sub(1));
    let microphone_ready_p95_ms = microphone_ready_ms[p95_index];
    let result = serde_json::json!({
        "requested_device": requested_device,
        "runs": runs,
        "seconds_per_run": seconds,
        "microphone_ready_ms": microphone_ready_ms,
        "microphone_ready_p95_ms": microphone_ready_p95_ms,
        "feedback_gate_measured": false,
        "sample_counts": sample_counts,
        "peaks": peaks,
        "cancellation_returned_idle": true,
    });

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&result).unwrap_or_default()
        );
    } else {
        println!(
            "microphone={} runs={} ready_p95={:.2}ms samples={:?} peaks={:?} cancel=idle",
            requested_device, runs, microphone_ready_p95_ms, sample_counts, peaks
        );
    }

    0
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(cli_args: CliArgs) {
    // Detect portable mode before anything else
    portable::init();

    // Parse console logging directives from RUST_LOG, falling back to info-level logging
    // when the variable is unset
    let console_filter = build_console_filter();

    let specta_builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            shortcut::change_binding,
            shortcut::reset_binding,
            shortcut::change_ptt_setting,
            shortcut::change_audio_feedback_setting,
            shortcut::change_audio_feedback_volume_setting,
            shortcut::change_sound_theme_setting,
            shortcut::change_theme_setting,
            shortcut::change_start_hidden_setting,
            shortcut::change_autostart_setting,
            shortcut::change_translate_to_english_setting,
            shortcut::change_selected_language_setting,
            shortcut::change_overlay_position_setting,
            shortcut::change_overlay_style_setting,
            shortcut::change_debug_mode_setting,
            shortcut::change_word_correction_threshold_setting,
            shortcut::change_extra_recording_buffer_setting,
            shortcut::change_paste_delay_ms_setting,
            shortcut::change_paste_delay_after_ms_setting,
            shortcut::change_paste_method_setting,
            shortcut::get_available_typing_tools,
            shortcut::change_typing_tool_setting,
            shortcut::change_external_script_path_setting,
            shortcut::change_clipboard_handling_setting,
            shortcut::change_auto_submit_setting,
            shortcut::change_auto_submit_key_setting,
            shortcut::change_post_process_enabled_setting,
            shortcut::change_experimental_enabled_setting,
            shortcut::change_post_process_base_url_setting,
            shortcut::change_post_process_api_key_setting,
            shortcut::change_post_process_model_setting,
            shortcut::set_post_process_provider,
            shortcut::fetch_post_process_models,
            shortcut::add_post_process_prompt,
            shortcut::update_post_process_prompt,
            shortcut::delete_post_process_prompt,
            shortcut::set_post_process_selected_prompt,
            shortcut::update_custom_words,
            shortcut::suspend_binding,
            shortcut::resume_binding,
            shortcut::change_mute_while_recording_setting,
            shortcut::change_append_trailing_space_setting,
            shortcut::change_lazy_stream_close_setting,
            shortcut::change_vad_enabled_setting,
            shortcut::change_app_language_setting,
            shortcut::change_show_whats_new_on_update_setting,
            shortcut::change_whats_new_last_seen_version_setting,
            shortcut::change_keyboard_implementation_setting,
            shortcut::get_keyboard_implementation,
            shortcut::change_show_tray_icon_setting,
            shortcut::change_transcribe_accelerator_setting,
            shortcut::change_ort_accelerator_setting,
            shortcut::change_transcribe_gpu_device,
            shortcut::get_available_accelerators,
            shortcut::handy_keys::start_handy_keys_recording,
            shortcut::handy_keys::stop_handy_keys_recording,
            show_main_window_command,
            commands::cancel_operation,
            commands::is_portable,
            commands::get_app_dir_path,
            commands::get_app_settings,
            commands::get_default_settings,
            commands::set_onboarding_stage,
            commands::complete_onboarding,
            commands::get_onboarding_diagnostics,
            commands::get_log_dir_path,
            commands::copy_text_to_clipboard,
            commands::set_log_level,
            commands::open_recordings_folder,
            commands::open_log_dir,
            commands::open_app_data_dir,
            commands::check_apple_intelligence_available,
            commands::initialize_enigo,
            commands::initialize_shortcuts,
            commands::models::get_available_models,
            commands::models::get_model_info,
            commands::models::get_model_install_plan,
            commands::models::download_model,
            commands::models::install_model_from_file,
            commands::models::delete_model,
            commands::models::cancel_download,
            commands::models::set_active_model,
            commands::models::get_current_model,
            commands::models::get_transcription_model_status,
            commands::models::is_model_loading,
            commands::models::rescan_local_models,
            commands::audio::update_microphone_mode,
            commands::audio::get_microphone_mode,
            commands::audio::get_windows_microphone_permission_status,
            commands::audio::open_microphone_privacy_settings,
            commands::audio::get_available_microphones,
            commands::audio::get_microphone_diagnostics,
            commands::audio::get_dictation_state,
            commands::audio::set_selected_microphone,
            commands::audio::get_selected_microphone,
            commands::audio::get_available_output_devices,
            commands::audio::set_selected_output_device,
            commands::audio::get_selected_output_device,
            commands::audio::play_test_sound,
            commands::audio::check_custom_sounds,
            commands::audio::set_clamshell_microphone,
            commands::audio::get_clamshell_microphone,
            commands::audio::is_recording,
            commands::transcription::set_model_unload_timeout,
            commands::transcription::get_model_load_status,
            commands::transcription::unload_model_manually,
            commands::dictionary::get_dictionary_entries,
            commands::dictionary::create_dictionary_entry,
            commands::dictionary::update_dictionary_entry,
            commands::dictionary::delete_dictionary_entry,
            commands::dictionary::export_dictionary_csv,
            commands::dictionary::import_dictionary_csv,
            commands::dictionary::get_dictionary_engine_support,
            commands::snippets::get_snippets,
            commands::snippets::create_snippet,
            commands::snippets::update_snippet,
            commands::snippets::delete_snippet,
            commands::snippets::export_snippets_json,
            commands::snippets::import_snippets_json,
            commands::history::get_history_entries,
            commands::history::toggle_history_entry_saved,
            commands::history::get_audio_file_path,
            commands::history::read_audio_file,
            commands::history::delete_history_entry,
            commands::history::clear_history,
            commands::history::retry_history_entry_transcription,
            commands::history::update_history_limit,
            commands::history::update_recording_retention_period,
            commands::history::update_history_storage_mode,
            helpers::clamshell::is_laptop,
        ])
        .events(collect_events![
            managers::history::HistoryUpdatePayload,
            managers::transcription::TranscriptionProgressEvent,
            managers::transcription::StreamTextEvent,
            managers::transcription::StreamPhaseEvent,
            transcription_coordinator::DictationStateEvent,
        ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    specta_builder
        .export(
            Typescript::default().bigint(BigIntExportBehavior::Number),
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    let invoke_handler = specta_builder.invoke_handler();

    // The headless path must run as its own instance (see the single-instance
    // note below), not forward to an already-running app.
    let headless_mode = cli_args.transcribe_file.is_some()
        || cli_args.evaluate_corpus.is_some()
        || cli_args.install_model_file.is_some()
        || cli_args.list_devices
        || cli_args.list_models
        || cli_args.verify_audio.is_some();

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .device_event_filter(tauri::DeviceEventFilter::Always)
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            LogBuilder::new()
                .level(log::LevelFilter::Trace) // Set to most verbose level globally
                .max_file_size(500_000)
                .rotation_strategy(RotationStrategy::KeepOne)
                .clear_targets()
                .targets([
                    // Console output respects RUST_LOG environment variable. In
                    // headless mode (--transcribe-file/--list-devices/--list-models)
                    // stdout carries only the result (JSON or plain), so send console
                    // logs to stderr instead to keep stdout clean for CI parsing.
                    Target::new(if headless_mode {
                        TargetKind::Stderr
                    } else {
                        TargetKind::Stdout
                    })
                    .filter({
                        let console_filter = console_filter.clone();
                        move |metadata| console_filter.enabled(metadata)
                    }),
                    // File logs respect the user's settings (stored in FILE_LOG_LEVEL atomic)
                    Target::new(if let Some(data_dir) = portable::data_dir() {
                        TargetKind::Folder {
                            path: data_dir.join("logs"),
                            file_name: Some("freeflow".into()),
                        }
                    } else {
                        TargetKind::LogDir {
                            file_name: Some("freeflow".into()),
                        }
                    })
                    .filter(|metadata| {
                        let file_level = FILE_LOG_LEVEL.load(Ordering::Relaxed);
                        metadata.level() <= level_filter_from_u8(file_level)
                    }),
                    // Stream logs to the webview (via the `log://log` event) so the
                    // debug panel's live log viewer can show them in real time. Only
                    // active while debug mode is on (its sole consumer), and shares the
                    // file log level so the "Log Level" setting controls verbosity.
                    Target::new(TargetKind::Webview).filter(|metadata| {
                        WEBVIEW_LOG_STREAMING.load(Ordering::Relaxed)
                            && metadata.level()
                                <= level_filter_from_u8(FILE_LOG_LEVEL.load(Ordering::Relaxed))
                    }),
                ])
                .build(),
        );

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    // Single-instance forwards CLI args to an already-running FreeFlow and exits.
    // That would make the headless path
    // (--transcribe-file/--list-devices/--list-models) a silent no-op whenever the
    // app is already open, so skip it in headless mode and run a standalone
    // instance instead.
    if !headless_mode {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if args.iter().any(|a| a == "--toggle-transcription") {
                signal_handle::send_transcription_input(app, "transcribe", "CLI");
            } else if args.iter().any(|a| a == "--toggle-post-process") {
                signal_handle::send_transcription_input(app, "transcribe_with_post_process", "CLI");
            } else if args.iter().any(|a| a == "--cancel") {
                crate::utils::cancel_current_operation(app);
            } else {
                show_main_window(app);
            }
        }));
    }

    builder
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_macos_permissions::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .manage(cli_args.clone())
        .setup(move |app| {
            specta_builder.mount_events(app);

            // Headless one-shot path (`--transcribe-file` / `--list-devices` /
            // `--list-models`): initialize only what transcription needs — the
            // path services, the model + transcription managers, and the
            // transcribe-cpp backend + accelerator settings — then run on a worker
            // thread and exit. Deliberately skips the window, tray, overlay, audio
            // recorder (so it never opens the mic, even with always_on_microphone),
            // signal handlers, and autostart that initialize_core_logic sets up.
            if headless_mode {
                let app_handle = app.handle().clone();
                let model_manager = Arc::new(
                    ModelManager::new(&app_handle).expect("Failed to initialize model manager"),
                );
                let transcription_manager = Arc::new(
                    TranscriptionManager::new(&app_handle, model_manager.clone())
                        .expect("Failed to initialize transcription manager"),
                );
                app_handle.manage(model_manager);
                app_handle.manage(transcription_manager);
                if cli_args.verify_audio.is_some() {
                    let transcription_manager = app_handle.state::<Arc<TranscriptionManager>>();
                    let recording_manager = Arc::new(
                        AudioRecordingManager::new(
                            &app_handle,
                            transcription_manager.stream_router(),
                        )
                        .expect("Failed to initialize recording manager for audio verification"),
                    );
                    app_handle.manage(recording_manager);
                }
                managers::transcription::init_transcribe_backend();
                managers::transcription::apply_accelerator_settings(&app_handle);

                let handle = app_handle.clone();
                let args = cli_args.clone();
                std::thread::spawn(move || {
                    let code = run_headless_transcription(&handle, &args);
                    // Drop the loaded engine before teardown: ggml-metal's global
                    // device free asserts (SIGABRT) if a model's Metal resources
                    // are still alive at C++ static-destructor time.
                    if let Some(tm) = handle.try_state::<Arc<TranscriptionManager>>() {
                        let _ = tm.unload_model();
                    }
                    // process::exit (not app.exit, which exits 0 regardless) so the
                    // exit code propagates to the shell for CI gating. Flush first
                    // since process::exit runs no destructors / buffer flushes.
                    use std::io::Write;
                    let _ = std::io::stdout().flush();
                    let _ = std::io::stderr().flush();
                    std::process::exit(code);
                });
                return Ok(());
            }

            // Create main window programmatically so we can set data_directory
            // for portable mode (redirects WebView2 cache to portable Data dir)
            let mut win_builder =
                tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("/".into()))
                    .title("FreeFlow")
                    .inner_size(680.0, 570.0)
                    .min_inner_size(680.0, 570.0)
                    .resizable(true)
                    .maximizable(false)
                    .visible(false);

            if let Some(data_dir) = portable::data_dir() {
                win_builder = win_builder.data_directory(data_dir.join("webview"));
            }

            win_builder.build()?;

            let mut settings = get_settings(app.handle());

            // Apply the persisted appearance theme to the Windows title bar before
            // the window is shown, so it matches the in-app palette without a flash
            // of the wrong theme. On macOS/Linux, Tauri themes are app-wide and
            // would also affect windows that intentionally keep the system theme.
            #[cfg(target_os = "windows")]
            shortcut::apply_window_theme(app.handle(), settings.theme);

            // CLI --debug flag overrides debug_mode and log level (runtime-only, not persisted)
            if cli_args.debug {
                settings.debug_mode = true;
                settings.log_level = settings::LogLevel::Trace;
            }

            let tauri_log_level: tauri_plugin_log::LogLevel = settings.log_level.into();
            let file_log_level: log::Level = tauri_log_level.into();
            // Store the file log level in the atomic for the filter to use
            FILE_LOG_LEVEL.store(file_log_level.to_level_filter() as u8, Ordering::Relaxed);
            // Only forward logs to the webview while debug mode is on (the live log
            // viewer is the sole consumer and only exists in debug mode). This also
            // honors the runtime `--debug` override applied to `settings` above.
            WEBVIEW_LOG_STREAMING.store(settings.debug_mode, Ordering::Relaxed);
            let app_handle = app.handle().clone();
            app.manage(TranscriptionCoordinator::new(app_handle.clone()));

            initialize_core_logic(&app_handle);

            // Populate the overlay-enabled cache from initial settings so the
            // audio path (overlay::emit_levels, called ~24 Hz during recording)
            // can do a single atomic load instead of reading the Tauri store.
            // Kept in sync by shortcut::change_overlay_style_setting.
            overlay::update_overlay_enabled_cache(
                settings.overlay_style != settings::OverlayStyle::None,
            );

            // Pre-warm GPU/accelerator enumeration on a background thread. The first
            // get_available_accelerators call enumerates ORT execution providers and
            // transcribe-cpp compute devices, which can take a moment; without this
            // the cost is paid synchronously when the user first opens Advanced
            // settings, freezing the UI. Result is cached in a OnceLock.
            std::thread::spawn(|| {
                let _ = crate::managers::transcription::get_available_accelerators();
            });

            // Hide tray icon if --no-tray was passed
            if cli_args.no_tray {
                tray::set_tray_visibility(&app_handle, false);
            }

            // Show main window only if not starting hidden.
            // CLI --start-hidden flag overrides the setting.
            // But if permission onboarding is required, always show the window.
            let should_hide = settings.start_hidden || cli_args.start_hidden;
            let should_force_show = should_force_show_permissions_window(&app_handle);

            // If start_hidden but tray is disabled, we must show the window
            // anyway. Without a tray icon, the dock is the only way back in.
            let tray_available = settings.show_tray_icon && !cli_args.no_tray;
            if should_force_show || !should_hide || !tray_available {
                show_main_window(&app_handle);
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _res = window.hide();

                #[cfg(target_os = "macos")]
                {
                    let settings = get_settings(window.app_handle());
                    let tray_visible =
                        settings.show_tray_icon && !window.app_handle().state::<CliArgs>().no_tray;
                    if tray_visible {
                        // Tray is available: hide the dock icon, app lives in the tray
                        let res = window
                            .app_handle()
                            .set_activation_policy(tauri::ActivationPolicy::Accessory);
                        if let Err(e) = res {
                            log::error!("Failed to set activation policy: {}", e);
                        }
                    }
                    // No tray: keep the dock icon visible so the user can reopen
                }
            }
            tauri::WindowEvent::ThemeChanged(theme) => {
                log::info!("Theme changed to: {:?}", theme);
                // Re-apply the current tray state with the new theme's icon set
                utils::refresh_tray_icon(window.app_handle());
            }
            tauri::WindowEvent::Moved(_) if window.label() == "recording_overlay" => {
                overlay::handle_overlay_moved(window.app_handle());
            }
            _ => {}
        })
        .invoke_handler(invoke_handler)
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| match &event {
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Reopen { .. } => {
                show_main_window(app);
            }
            // Teardown transcribe.cpp before exit
            tauri::RunEvent::Exit => {
                if let Some(tm) = app.try_state::<Arc<TranscriptionManager>>() {
                    let _ = tm.unload_model();
                }
            }
            _ => {}
        });
}
