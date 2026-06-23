use std::sync::{Arc, Mutex};
use tauri::State;
use crate::clipboard::database::{Database, ClipboardItemPreview, ClipboardItem};
use serde::{Deserialize, Serialize};

pub struct ClipboardDbState {
    pub db: Arc<Mutex<Database>>,
}

#[derive(Debug, Deserialize)]
pub struct GetItemsParams {
    pub content_type: Option<String>,
    pub search_query: Option<String>,
    pub time_filter: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct GetItemsResult {
    pub items: Vec<ClipboardItemPreview>,
    pub total: i64,
}

#[tauri::command]
pub fn get_clipboard_items(
    state: State<'_, ClipboardDbState>,
    params: GetItemsParams,
) -> Result<GetItemsResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let content_type = params.content_type.as_deref();
    let search_query = params.search_query.as_deref();
    let time_filter = params.time_filter.as_deref();
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    let items = db.get_items(content_type, search_query, time_filter, limit, offset)
        .map_err(|e| e.to_string())?;
    let total = db.get_total_count(content_type, search_query, time_filter)
        .map_err(|e| e.to_string())?;

    Ok(GetItemsResult { items, total })
}

#[tauri::command]
pub fn get_clipboard_item_detail(
    state: State<'_, ClipboardDbState>,
    id: i64,
) -> Result<Option<ClipboardItem>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_item_detail(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_clipboard_item(
    state: State<'_, ClipboardDbState>,
    id: i64,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_item(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_pin_item(
    state: State<'_, ClipboardDbState>,
    id: i64,
) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.toggle_pin(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_clipboard_history(
    state: State<'_, ClipboardDbState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.clear_all().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn paste_from_clipboard() -> Result<String, String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.get_text().map_err(|e| e.to_string())
}
