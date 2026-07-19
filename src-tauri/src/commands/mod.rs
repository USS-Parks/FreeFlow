pub mod audio;
pub mod dictionary;
pub mod history;
pub mod models;
pub mod transcription;

use crate::settings::{get_settings, write_settings, AppSettings, LogLevel, OnboardingStage};
use crate::utils::cancel_current_operation;
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
#[specta::specta]
pub fn cancel_operation(app: AppHandle) {
    cancel_current_operation(&app);
}

#[tauri::command]
#[specta::specta]
pub fn is_portable() -> bool {
    crate::portable::is_portable()
}

#[tauri::command]
#[specta::specta]
pub fn get_app_dir_path(app: AppHandle) -> Result<String, String> {
    let app_data_dir = crate::portable::app_data_dir(&app)
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    Ok(app_data_dir.to_string_lossy().to_string())
}

#[tauri::command]
#[specta::specta]
pub fn get_app_settings(app: AppHandle) -> Result<AppSettings, String> {
    Ok(get_settings(&app))
}

#[tauri::command]
#[specta::specta]
pub fn get_default_settings() -> Result<AppSettings, String> {
    Ok(crate::settings::get_default_settings())
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct OnboardingDiagnostics {
    pub stage: OnboardingStage,
    pub completed: bool,
    pub model_selected: bool,
    pub autostart_requested: bool,
    pub autostart_enabled: bool,
    pub app_data_path: String,
    pub portable: bool,
}

#[tauri::command]
#[specta::specta]
pub fn set_onboarding_stage(app: AppHandle, stage: OnboardingStage) -> Result<(), String> {
    if stage == OnboardingStage::Complete {
        return Err("Use complete_onboarding to finish setup".to_string());
    }
    let mut settings = get_settings(&app);
    settings.onboarding_completed = false;
    settings.onboarding_stage = stage;
    write_settings(&app, settings);
    Ok(())
}

fn mark_onboarding_complete(settings: &mut AppSettings) -> Result<(), String> {
    if settings.selected_model.is_empty() {
        return Err("A local transcription model must be selected first".to_string());
    }
    settings.onboarding_completed = true;
    settings.onboarding_stage = OnboardingStage::Complete;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn complete_onboarding(app: AppHandle) -> Result<(), String> {
    let mut settings = get_settings(&app);
    mark_onboarding_complete(&mut settings)?;
    write_settings(&app, settings);
    Ok(())
}

#[cfg(test)]
mod onboarding_tests {
    use super::*;

    #[test]
    fn completion_requires_a_selected_local_model() {
        let mut settings = crate::settings::get_default_settings();
        assert!(mark_onboarding_complete(&mut settings).is_err());
        assert!(!settings.onboarding_completed);
        assert_eq!(settings.onboarding_stage, OnboardingStage::Welcome);
    }

    #[test]
    fn completion_updates_stage_and_compatibility_flag_together() {
        let mut settings = crate::settings::get_default_settings();
        settings.selected_model = "local-model".to_string();
        settings.onboarding_stage = OnboardingStage::FirstDictation;

        mark_onboarding_complete(&mut settings).unwrap();

        assert!(settings.onboarding_completed);
        assert_eq!(settings.onboarding_stage, OnboardingStage::Complete);
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_onboarding_diagnostics(app: AppHandle) -> Result<OnboardingDiagnostics, String> {
    let settings = get_settings(&app);
    let app_data_path = crate::portable::app_data_dir(&app)
        .map_err(|error| format!("Failed to resolve app data directory: {error}"))?
        .to_string_lossy()
        .to_string();
    let autostart_enabled = app
        .autolaunch()
        .is_enabled()
        .map_err(|error| format!("Failed to inspect launch-at-login state: {error}"))?;

    Ok(OnboardingDiagnostics {
        stage: settings.onboarding_stage,
        completed: settings.onboarding_completed,
        model_selected: !settings.selected_model.is_empty(),
        autostart_requested: settings.autostart_enabled,
        autostart_enabled,
        app_data_path,
        portable: crate::portable::is_portable(),
    })
}

#[tauri::command]
#[specta::specta]
pub fn copy_text_to_clipboard(app: AppHandle, text: String) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|error| format!("Failed to copy text: {error}"))
}

#[tauri::command]
#[specta::specta]
pub fn get_log_dir_path(app: AppHandle) -> Result<String, String> {
    let log_dir = crate::portable::app_log_dir(&app)
        .map_err(|e| format!("Failed to get log directory: {}", e))?;

    Ok(log_dir.to_string_lossy().to_string())
}

#[specta::specta]
#[tauri::command]
pub fn set_log_level(app: AppHandle, level: LogLevel) -> Result<(), String> {
    let tauri_log_level: tauri_plugin_log::LogLevel = level.into();
    let log_level: log::Level = tauri_log_level.into();
    // Update the file log level atomic so the filter picks up the new level
    crate::FILE_LOG_LEVEL.store(
        log_level.to_level_filter() as u8,
        std::sync::atomic::Ordering::Relaxed,
    );

    let mut settings = get_settings(&app);
    settings.log_level = level;
    write_settings(&app, settings);

    Ok(())
}

#[specta::specta]
#[tauri::command]
pub fn open_recordings_folder(app: AppHandle) -> Result<(), String> {
    let app_data_dir = crate::portable::app_data_dir(&app)
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let recordings_dir = app_data_dir.join("recordings");

    let path = recordings_dir.to_string_lossy().as_ref().to_string();
    app.opener()
        .open_path(path, None::<String>)
        .map_err(|e| format!("Failed to open recordings folder: {}", e))?;

    Ok(())
}

#[specta::specta]
#[tauri::command]
pub fn open_log_dir(app: AppHandle) -> Result<(), String> {
    let log_dir = crate::portable::app_log_dir(&app)
        .map_err(|e| format!("Failed to get log directory: {}", e))?;

    let path = log_dir.to_string_lossy().as_ref().to_string();
    app.opener()
        .open_path(path, None::<String>)
        .map_err(|e| format!("Failed to open log directory: {}", e))?;

    Ok(())
}

#[specta::specta]
#[tauri::command]
pub fn open_app_data_dir(app: AppHandle) -> Result<(), String> {
    let app_data_dir = crate::portable::app_data_dir(&app)
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let path = app_data_dir.to_string_lossy().as_ref().to_string();
    app.opener()
        .open_path(path, None::<String>)
        .map_err(|e| format!("Failed to open app data directory: {}", e))?;

    Ok(())
}

/// Check if Apple Intelligence is available on this device.
/// Called by the frontend when the user selects Apple Intelligence provider.
#[specta::specta]
#[tauri::command]
pub fn check_apple_intelligence_available() -> bool {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        crate::apple_intelligence::check_apple_intelligence_availability()
    }
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    {
        false
    }
}

/// Try to initialize Enigo (keyboard/mouse simulation).
/// On macOS, this will return an error if accessibility permissions are not granted.
#[specta::specta]
#[tauri::command]
pub fn initialize_enigo(app: AppHandle) -> Result<(), String> {
    use crate::input::EnigoState;

    // Check if already initialized
    if app.try_state::<EnigoState>().is_some() {
        log::debug!("Enigo already initialized");
        return Ok(());
    }

    // Try to initialize
    match EnigoState::new() {
        Ok(enigo_state) => {
            app.manage(enigo_state);
            log::info!("Enigo initialized successfully after permission grant");
            Ok(())
        }
        Err(e) => {
            if cfg!(target_os = "macos") {
                log::warn!(
                    "Failed to initialize Enigo: {} (accessibility permissions may not be granted)",
                    e
                );
            } else {
                log::warn!("Failed to initialize Enigo: {}", e);
            }
            Err(format!("Failed to initialize input system: {}", e))
        }
    }
}

/// Marker state to track if shortcuts have been initialized.
pub struct ShortcutsInitialized;

/// Initialize keyboard shortcuts.
/// On macOS, this should be called after accessibility permissions are granted.
/// This is idempotent - calling it multiple times is safe.
#[specta::specta]
#[tauri::command]
pub fn initialize_shortcuts(app: AppHandle) -> Result<(), String> {
    // Check if already initialized
    if app.try_state::<ShortcutsInitialized>().is_some() {
        log::debug!("Shortcuts already initialized");
        return Ok(());
    }

    // Initialize shortcuts
    crate::shortcut::init_shortcuts(&app);

    // Mark as initialized
    app.manage(ShortcutsInitialized);

    log::info!("Shortcuts initialized successfully");
    Ok(())
}
