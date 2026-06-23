use std::sync::{Arc, Mutex};
use tauri::Manager;
use crate::clipboard::{Database, ClipboardMonitor, ClipboardDbState};

mod clipboard;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("tool-kit");

    std::fs::create_dir_all(&app_dir).ok();

    let db_path = app_dir.join("clipboard.db");
    let db_path_str = db_path.to_str().unwrap_or("clipboard.db").to_string();

    let db = Database::new(&db_path_str).expect("Failed to initialize database");
    let db = Arc::new(Mutex::new(db));

    let monitor = ClipboardMonitor::new(db.clone());
    monitor.start();

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(ClipboardDbState { db })
        .invoke_handler(tauri::generate_handler![
            clipboard::get_clipboard_items,
            clipboard::get_clipboard_item_detail,
            clipboard::delete_clipboard_item,
            clipboard::toggle_pin_item,
            clipboard::clear_clipboard_history,
            clipboard::copy_to_clipboard,
            clipboard::paste_from_clipboard,
        ])
        .setup(|app| {
            use tauri_plugin_global_shortcut::GlobalShortcutExt;

            let app_handle = app.handle().clone();

            app.global_shortcut().on_shortcut(
                "Control+Shift+V",
                move |_app, _shortcut, event| {
                    use tauri_plugin_global_shortcut::ShortcutState;
                    if event.state == ShortcutState::Pressed {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                },
            )?;

            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
