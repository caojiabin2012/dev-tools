use tauri::AppHandle;

use crate::stack::config::sync_all_configs;
use crate::stack::download::download_component;
use crate::stack::service::{
    install, open_component_config, open_component_log, open_site, set_component_port, start,
    start_all, stop, stop_all, uninstall, update_settings,
};
use crate::stack::state::build_stack_state;
use crate::stack::store::{load_store, save_store, set_version_pref};
use crate::stack::types::{
    InstallComponentParams, SetComponentPortParams, SetComponentVersionParams, StackState,
    UpdateStackSettingsParams,
};

#[tauri::command]
pub fn stack_get_state() -> StackState {
    build_stack_state()
}

#[tauri::command]
pub fn stack_set_install_root(path: String) -> Result<StackState, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("安装目录不能为空".into());
    }
    let mut store = load_store();
    store.install_root = Some(trimmed.to_string());
    save_store(&store)?;
    Ok(build_stack_state())
}

#[tauri::command]
pub fn stack_pick_install_root() -> Result<Option<String>, String> {
    let picked = rfd::FileDialog::new()
        .set_title("选择环境安装目录")
        .pick_folder();
    Ok(picked.map(|p| p.to_string_lossy().into_owned()))
}

#[tauri::command]
pub fn stack_pick_www_subdir() -> Result<Option<String>, String> {
    let store = load_store();
    let root = store.install_root.as_ref().ok_or("尚未设置安装目录")?;
    let picked = rfd::FileDialog::new()
        .set_title("选择网站根目录")
        .set_directory(root)
        .pick_folder();
    Ok(picked.map(|p| {
        let picked_str = p.to_string_lossy();
        let root_path = std::path::Path::new(root);
        if let Ok(rel) = p.strip_prefix(root_path) {
            rel.to_string_lossy().replace('\\', "/")
        } else {
            picked_str.into_owned()
        }
    }))
}

#[tauri::command]
pub fn stack_pick_component_source() -> Result<Option<String>, String> {
    let picked = rfd::FileDialog::new()
        .set_title("选择组件 zip 或已解压目录")
        .add_filter("ZIP", &["zip"])
        .pick_file()
        .or_else(|| {
            rfd::FileDialog::new()
                .set_title("选择已解压目录")
                .pick_folder()
        });
    Ok(picked.map(|p| p.to_string_lossy().into_owned()))
}

#[tauri::command]
pub fn stack_set_component_version(params: SetComponentVersionParams) -> Result<StackState, String> {
    set_version_pref(&params.component, &params.version_id)?;
    Ok(build_stack_state())
}

#[tauri::command]
pub async fn stack_download_component(
    app: AppHandle,
    component: String,
    version_id: Option<String>,
) -> Result<StackState, String> {
    let app_clone = app.clone();
    let version = version_id.clone();
    tauri::async_runtime::spawn_blocking(move || {
        download_component(&app_clone, &component, version.as_deref())
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(build_stack_state())
}

#[tauri::command]
pub fn stack_install_component(params: InstallComponentParams) -> Result<StackState, String> {
    install(&params)?;
    Ok(build_stack_state())
}

#[tauri::command]
pub fn stack_start_component(component: String) -> Result<StackState, String> {
    start(&component)?;
    Ok(build_stack_state())
}

#[tauri::command]
pub fn stack_stop_component(component: String) -> Result<StackState, String> {
    stop(&component)?;
    Ok(build_stack_state())
}

#[tauri::command]
pub fn stack_uninstall_component(component: String) -> Result<StackState, String> {
    uninstall(&component)?;
    Ok(build_stack_state())
}

#[tauri::command]
pub fn stack_start_all() -> Result<StackState, String> {
    start_all()?;
    Ok(build_stack_state())
}

#[tauri::command]
pub fn stack_stop_all() -> Result<StackState, String> {
    stop_all()?;
    Ok(build_stack_state())
}

#[tauri::command]
pub fn stack_update_settings(params: UpdateStackSettingsParams) -> Result<StackState, String> {
    update_settings(&params)?;
    Ok(build_stack_state())
}

#[tauri::command]
pub fn stack_set_component_port(params: SetComponentPortParams) -> Result<StackState, String> {
    set_component_port(&params.component, params.port)?;
    Ok(build_stack_state())
}

#[tauri::command]
pub fn stack_regenerate_configs() -> Result<StackState, String> {
    sync_all_configs()?;
    Ok(build_stack_state())
}

#[tauri::command]
pub fn stack_open_component_config(component: String) -> Result<(), String> {
    open_component_config(&component)
}

#[tauri::command]
pub fn stack_open_component_log(component: String) -> Result<(), String> {
    open_component_log(&component)
}

#[tauri::command]
pub fn stack_open_site() -> Result<(), String> {
    open_site()
}

#[tauri::command]
pub fn stack_open_install_root() -> Result<(), String> {
    let store = load_store();
    let root = store.install_root.ok_or("尚未设置安装目录")?;
    open::that(root).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stack_open_www_root() -> Result<(), String> {
    use std::path::Path;
    let store = load_store();
    let root = store.install_root.as_ref().ok_or("尚未设置安装目录")?;
    let www = crate::stack::store::www_root(Path::new(root));
    std::fs::create_dir_all(&www).map_err(|e| e.to_string())?;
    open::that(www).map_err(|e| e.to_string())
}
