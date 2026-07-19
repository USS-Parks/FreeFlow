use crate::actions::process_transcription_output_for_application;
use crate::managers::{
    history::{HistoryManager, PaginatedHistory},
    transcription::{TranscriptionManager, TranscriptionProgressEvent, TranscriptionProgressStage},
};
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
#[specta::specta]
pub async fn get_history_entries(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    cursor: Option<i64>,
    limit: Option<usize>,
    query: Option<String>,
) -> Result<PaginatedHistory, String> {
    history_manager
        .get_history_entries(cursor, limit, query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_history_entry_saved(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    id: i64,
) -> Result<(), String> {
    history_manager
        .toggle_saved_status(id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_audio_file_path(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    file_name: String,
) -> Result<String, String> {
    let path = history_manager
        .get_audio_file_path(&file_name)
        .map_err(|error| error.to_string())?;
    path.to_str()
        .ok_or_else(|| "Invalid file path".to_string())
        .map(|s| s.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn read_audio_file(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    file_name: String,
) -> Result<Vec<u8>, String> {
    let path = history_manager
        .get_audio_file_path(&file_name)
        .map_err(|error| error.to_string())?;
    std::fs::read(path).map_err(|error| format!("Failed to read recording: {error}"))
}

#[tauri::command]
#[specta::specta]
pub async fn delete_history_entry(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    id: i64,
) -> Result<(), String> {
    history_manager
        .delete_entry(id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn clear_history(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
) -> Result<usize, String> {
    history_manager
        .clear_history()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn retry_history_entry_transcription(
    app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    transcription_manager: State<'_, Arc<TranscriptionManager>>,
    id: i64,
) -> Result<(), String> {
    let never_store = crate::settings::get_history_storage_mode(&app)
        == crate::settings::HistoryStorageMode::NeverStore;
    let entry = history_manager
        .get_entry_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("History entry {} not found", id))?;

    let audio_path = history_manager
        .get_audio_file_path(&entry.file_name)
        .map_err(|error| error.to_string())?;
    let samples = crate::audio_toolkit::read_wav_samples(&audio_path)
        .map_err(|e| format!("Failed to load audio: {}", e))?;

    if samples.is_empty() {
        if never_store {
            history_manager
                .delete_entry(id)
                .await
                .map_err(|error| error.to_string())?;
        }
        return Err("Recording has no audio samples".to_string());
    }

    transcription_manager.initiate_model_load();
    TranscriptionProgressEvent::publish(
        &app,
        Some(id),
        TranscriptionProgressStage::Recognizing,
        45,
        None,
    );

    let tm = Arc::clone(&transcription_manager);
    let outcome = tauri::async_runtime::spawn_blocking(move || tm.transcribe_detailed(samples))
        .await
        .map_err(|e| format!("Transcription task panicked: {}", e))?;
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(error) => {
            let message = error.to_string();
            if never_store {
                history_manager
                    .delete_entry(id)
                    .await
                    .map_err(|delete_error| delete_error.to_string())?;
            } else {
                let _ = history_manager.mark_transcription_failed(id, &message);
            }
            TranscriptionProgressEvent::publish(
                &app,
                Some(id),
                TranscriptionProgressStage::Failed,
                100,
                Some(message.clone()),
            );
            return Err(message);
        }
    };

    if outcome.text.is_empty() {
        if never_store {
            history_manager
                .delete_entry(id)
                .await
                .map_err(|error| error.to_string())?;
        } else {
            let _ = history_manager.mark_transcription_failed(id, "Recording contains no speech");
        }
        TranscriptionProgressEvent::publish(
            &app,
            Some(id),
            TranscriptionProgressStage::Failed,
            100,
            Some("Recording contains no speech".to_string()),
        );
        return Err("Recording contains no speech".to_string());
    }

    if entry.post_process_requested {
        TranscriptionProgressEvent::publish(
            &app,
            Some(id),
            TranscriptionProgressStage::PostProcessing,
            80,
            None,
        );
    }
    let processed = process_transcription_output_for_application(
        &app,
        &outcome.text,
        entry.post_process_requested,
        entry.application_id.as_deref(),
    )
    .await;
    if never_store {
        history_manager
            .delete_entry(id)
            .await
            .map_err(|e| e.to_string())?;
    } else {
        history_manager
            .complete_transcription(
                id,
                &outcome,
                processed.post_processed_text,
                processed.post_process_prompt,
            )
            .map_err(|e| e.to_string())?;
    }
    TranscriptionProgressEvent::publish(
        &app,
        Some(id),
        TranscriptionProgressStage::Completed,
        100,
        None,
    );
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn update_history_limit(
    app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    limit: usize,
) -> Result<(), String> {
    let mut settings = crate::settings::get_settings(&app);
    settings.history_limit = limit;
    crate::settings::write_settings(&app, settings);

    history_manager
        .cleanup_old_entries()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn update_recording_retention_period(
    app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    period: String,
) -> Result<(), String> {
    use crate::settings::RecordingRetentionPeriod;

    let retention_period = match period.as_str() {
        "never" => RecordingRetentionPeriod::Never,
        "preserve_limit" => RecordingRetentionPeriod::PreserveLimit,
        "days3" => RecordingRetentionPeriod::Days3,
        "weeks2" => RecordingRetentionPeriod::Weeks2,
        "months3" => RecordingRetentionPeriod::Months3,
        _ => return Err(format!("Invalid retention period: {}", period)),
    };

    let mut settings = crate::settings::get_settings(&app);
    settings.recording_retention_period = retention_period;
    crate::settings::write_settings(&app, settings);

    history_manager
        .cleanup_old_entries()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn update_history_storage_mode(
    app: AppHandle,
    mode: crate::settings::HistoryStorageMode,
) -> Result<(), String> {
    let mut settings = crate::settings::get_settings(&app);
    settings.history_storage_mode = mode;
    crate::settings::write_settings(&app, settings);
    Ok(())
}
