use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
use crate::clipboard::{Database, ClipboardDbState, ClipboardMonitor};
use crate::settings::{SettingsState, AppSettings};

mod clipboard;
mod ocr;
mod settings;

fn show_main_window(window: &tauri::WebviewWindow) {
    let _ = window.set_skip_taskbar(false);
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

fn hide_main_window(window: &tauri::WebviewWindow) {
    let _ = window.hide();
    let _ = window.set_skip_taskbar(true);
}

fn toggle_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            hide_main_window(&window);
        } else {
            show_main_window(&window);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Log panics to file so we can diagnose crashes
    std::panic::set_hook(Box::new(|info| {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("unknown");
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Box<dyn Any>".to_string()
        };
        let location = info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column())).unwrap_or_default();
        let msg = format!("PANIC on thread '{}': {} at {}", thread_name, payload, location);
        log::error!("{}", msg);
        // Also write to crash file for diagnosis
        if let Some(app_dir) = dirs::data_local_dir() {
            let crash_path = app_dir.join("tool-kit").join("crash.log");
            let _ = std::fs::write(&crash_path, &msg);
        }
    }));

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
    log::info!("Clipboard monitor started");

    let app_settings = AppSettings::load();
    let settings = Arc::new(Mutex::new(app_settings));
    let settings_for_tray = settings.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default()
            .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
            .build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .manage(ClipboardDbState { db })
        .manage(SettingsState { settings })
        .invoke_handler(tauri::generate_handler![
            clipboard::get_clipboard_items,
            clipboard::get_clipboard_item_detail,
            clipboard::delete_clipboard_item,
            clipboard::toggle_pin_item,
            clipboard::clear_clipboard_history,
            clipboard::copy_to_clipboard,
            clipboard::paste_from_clipboard,
            clipboard::copy_image_to_clipboard,
            clipboard::open_file,
            clipboard::open_file_containing_folder,
            clipboard::ocr_image,
            settings::get_settings,
            settings::save_settings,
            settings::get_app_version,
            settings::check_for_update,
        ])
        .setup(|app| {
            use tauri_plugin_global_shortcut::GlobalShortcutExt;

            let app_handle = app.handle().clone();

            let show_item = MenuItem::with_id(app, "tray_show", "显示窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            app.on_menu_event(|app, event| {
                match event.id().as_ref() {
                    "tray_show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            show_main_window(&window);
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                }
            });

            if let Some(tray) = app.tray_by_id("main") {
                tray.set_menu(Some(tray_menu))?;
                tray.set_show_menu_on_left_click(false)?;

                let handle = app_handle.clone();
                tray.on_tray_icon_event(move |_tray, event| {
                    match event {
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        }
                        | TrayIconEvent::DoubleClick {
                            button: MouseButton::Left,
                            ..
                        } => toggle_main_window(&handle),
                        _ => {}
                    }
                });
            }

            // Close to tray behavior
            let main_window = app.get_webview_window("main").unwrap();
            let window_for_close = main_window.clone();
            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    let close_to_tray = settings_for_tray.lock().unwrap().close_to_tray;
                    if close_to_tray {
                        api.prevent_close();
                        hide_main_window(&window_for_close);
                    }
                }
            });

            let _ = app.global_shortcut().on_shortcut(
                "Control+Shift+V",
                move |_app, _shortcut, event| {
                    use tauri_plugin_global_shortcut::ShortcutState;
                    if event.state == ShortcutState::Pressed {
                        toggle_main_window(&app_handle);
                    }
                },
            );

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
