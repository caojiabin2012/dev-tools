use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
use crate::clipboard::{Database, ClipboardDbState, ClipboardMonitor};
use crate::qrcode::{QrcodeDatabase, QrcodeDbState};
use crate::settings::{SettingsState, AppSettings};

mod app_paths;
mod clipboard;
mod diagnostics;
mod ocr;
mod qrcode;
mod settings;
mod window_chrome;


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

pub fn register_shortcuts_for_app(app: &tauri::AppHandle, shortcuts: &std::collections::HashMap<String, String>) {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

    let _ = app.global_shortcut().unregister_all();

    for (tool_id, shortcut_str) in shortcuts {
        let id = tool_id.clone();
        let handle = app.clone();
        let result = app.global_shortcut().on_shortcut(
            shortcut_str.as_str(),
            move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    if let Some(window) = handle.get_webview_window("main") {
                        show_main_window(&window);
                        let _ = handle.emit("shortcut-triggered", &id);
                    }
                }
            },
        );
        if let Err(e) = result {
            log::warn!("Failed to register shortcut {} for {}: {}", shortcut_str, tool_id, e);
        }
    }
}

fn register_shortcuts(app: &tauri::App, shortcuts: &std::collections::HashMap<String, String>) {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

    let _ = app.global_shortcut().unregister_all();

    for (tool_id, shortcut_str) in shortcuts {
        let id = tool_id.clone();
        let handle = app.handle().clone();
        let result = app.global_shortcut().on_shortcut(
            shortcut_str.as_str(),
            move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    if let Some(window) = handle.get_webview_window("main") {
                        show_main_window(&window);
                        let _ = handle.emit("shortcut-triggered", &id);
                    }
                }
            },
        );
        if let Err(e) = result {
            log::warn!("Failed to register shortcut {} for {}: {}", shortcut_str, tool_id, e);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    diagnostics::install_panic_hook();

    let log_dir = crate::app_paths::logs_dir();

    let app_dir = crate::app_paths::app_data_dir();

    let db_path = app_dir.join("clipboard.db");
    let db_path_str = db_path.to_str().unwrap_or("clipboard.db").to_string();

    let db = Database::new(&db_path_str).expect("Failed to initialize database");
    let db = Arc::new(Mutex::new(db));

    let qr_db_path = app_dir.join("qrcode.db");
    let qr_db_path_str = qr_db_path.to_str().unwrap_or("qrcode.db").to_string();
    let qr_db = QrcodeDatabase::new(&qr_db_path_str).expect("Failed to initialize qrcode database");
    let qr_db = Arc::new(Mutex::new(qr_db));

    let monitor = ClipboardMonitor::new(db.clone());
    monitor.start();
    log::info!("Clipboard monitor started");

    let app_settings = AppSettings::load();
    let shortcuts = app_settings.shortcuts.clone();
    let settings = Arc::new(Mutex::new(app_settings));
    let settings_for_tray = settings.clone();

    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                show_main_window(&window);
            }
        }));
    }

    builder
        .plugin(
            tauri_plugin_log::Builder::default()
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .max_file_size(512_000)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(3))
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Folder {
                        path: log_dir,
                        file_name: Some("app".into()),
                    }),
                ])
                .build(),
        )
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .manage(ClipboardDbState { db })
        .manage(QrcodeDbState { db: qr_db })
        .manage(SettingsState { settings })
        .invoke_handler(tauri::generate_handler![
            clipboard::get_clipboard_items,
            clipboard::get_clipboard_item_detail,
            clipboard::get_clipboard_image_data_url,
            clipboard::delete_clipboard_item,
            clipboard::toggle_pin_item,
            clipboard::clear_clipboard_history,
            clipboard::copy_to_clipboard,
            clipboard::paste_from_clipboard,
            clipboard::copy_image_to_clipboard,
            clipboard::copy_files_to_clipboard,
            clipboard::get_file_paths_status,
            clipboard::open_file,
            clipboard::open_file_containing_folder,
            clipboard::ocr_image,
            settings::get_settings,
            settings::save_settings,
            settings::get_app_version,
            settings::install_update_and_restart,
            settings::update_shortcuts,
            diagnostics::record_client_error,
            window_chrome::sync_window_theme,
            qrcode::decode_qr_and_save,
            qrcode::list_qr_decode_items,
            qrcode::get_qr_decode_item,
            qrcode::delete_qr_decode_item,
            qrcode::clear_qr_decode_items,
            qrcode::generate_qr_and_save,
            qrcode::list_qr_generate_items,
            qrcode::get_qr_generate_item,
            qrcode::delete_qr_generate_item,
            qrcode::clear_qr_generate_items,
            qrcode::copy_qr_generate_to_clipboard,
            qrcode::save_qr_generate_file,
        ])
        .setup(move |app| {
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

            // Register all shortcuts
            register_shortcuts(app, &shortcuts);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
