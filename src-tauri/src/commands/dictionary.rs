use crate::managers::dictionary::{
    DictionaryEngineSupport, DictionaryEntry, DictionaryManager, DictionarySort,
};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub fn get_dictionary_entries(
    manager: State<'_, Arc<DictionaryManager>>,
    query: Option<String>,
    sort: Option<DictionarySort>,
) -> Result<Vec<DictionaryEntry>, String> {
    manager
        .list(query.as_deref(), sort.unwrap_or(DictionarySort::Starred))
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn create_dictionary_entry(
    manager: State<'_, Arc<DictionaryManager>>,
    spoken_form: String,
    replacement: String,
    starred: bool,
) -> Result<DictionaryEntry, String> {
    manager
        .create(&spoken_form, &replacement, starred)
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn update_dictionary_entry(
    manager: State<'_, Arc<DictionaryManager>>,
    id: i64,
    spoken_form: String,
    replacement: String,
    starred: bool,
) -> Result<DictionaryEntry, String> {
    manager
        .update(id, &spoken_form, &replacement, starred)
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn delete_dictionary_entry(
    manager: State<'_, Arc<DictionaryManager>>,
    id: i64,
) -> Result<(), String> {
    manager.delete(id).map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn export_dictionary_csv(manager: State<'_, Arc<DictionaryManager>>) -> Result<String, String> {
    manager.export_csv().map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn import_dictionary_csv(
    manager: State<'_, Arc<DictionaryManager>>,
    csv: String,
) -> Result<usize, String> {
    manager.import_csv(&csv).map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn get_dictionary_engine_support() -> DictionaryEngineSupport {
    DictionaryEngineSupport {
        whisper_initial_prompt_only: true,
        deterministic_replacement: true,
    }
}
