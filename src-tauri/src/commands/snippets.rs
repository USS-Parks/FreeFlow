use crate::managers::snippets::{Snippet, SnippetManager, SnippetSort};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub fn get_snippets(
    manager: State<'_, Arc<SnippetManager>>,
    query: Option<String>,
    sort: Option<SnippetSort>,
) -> Result<Vec<Snippet>, String> {
    manager
        .list(query.as_deref(), sort.unwrap_or(SnippetSort::Updated))
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn create_snippet(
    manager: State<'_, Arc<SnippetManager>>,
    name: String,
    trigger_phrase: String,
    expansion: String,
) -> Result<Snippet, String> {
    manager
        .create(&name, &trigger_phrase, &expansion)
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn update_snippet(
    manager: State<'_, Arc<SnippetManager>>,
    id: i64,
    name: String,
    trigger_phrase: String,
    expansion: String,
) -> Result<Snippet, String> {
    manager
        .update(id, &name, &trigger_phrase, &expansion)
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn delete_snippet(manager: State<'_, Arc<SnippetManager>>, id: i64) -> Result<(), String> {
    manager.delete(id).map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn export_snippets_json(manager: State<'_, Arc<SnippetManager>>) -> Result<String, String> {
    manager.export_json().map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn import_snippets_json(
    manager: State<'_, Arc<SnippetManager>>,
    json: String,
) -> Result<usize, String> {
    manager
        .import_json(&json)
        .map_err(|error| error.to_string())
}
