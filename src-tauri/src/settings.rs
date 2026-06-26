use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::State;

pub struct SettingsState {
    pub settings: Arc<Mutex<AppSettings>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub auto_start: bool,
    pub close_to_tray: bool,
    #[serde(default)]
    pub shortcuts: HashMap<String, String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        let mut shortcuts = HashMap::new();
        shortcuts.insert("clipboard".to_string(), "Ctrl+Shift+V".to_string());
        Self {
            auto_start: false,
            close_to_tray: true,
            shortcuts,
        }
    }
}

impl AppSettings {
    fn path() -> std::path::PathBuf {
        let dir = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("tool-kit");
        std::fs::create_dir_all(&dir).ok();
        dir.join("settings.json")
    }

    pub fn load() -> Self {
        let path = Self::path();
        let mut settings: AppSettings = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        // Sync auto_start with the actual OS state on load
        if let Ok(enabled) = query_autostart_enabled() {
            settings.auto_start = enabled;
        }

        // Ensure clipboard shortcut exists with default
        if !settings.shortcuts.contains_key("clipboard") {
            settings.shortcuts.insert("clipboard".to_string(), "Ctrl+Shift+V".to_string());
        }

        settings
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::path();
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }
}

const AUTOSTART_APP_NAME: &str = "Tool Kit";

fn build_autostart() -> Result<auto_launch::AutoLaunch, String> {
    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    auto_launch::AutoLaunchBuilder::new()
        .set_app_name(AUTOSTART_APP_NAME)
        .set_app_path(exe_path.to_str().ok_or("Invalid exe path")?)
        .build()
        .map_err(|e| e.to_string())
}

fn query_autostart_enabled() -> Result<bool, String> {
    let autolaunch = build_autostart()?;
    autolaunch.is_enabled().map_err(|e| e.to_string())
}

fn update_autostart(enabled: bool) -> Result<(), String> {
    let autolaunch = build_autostart()?;

    if enabled {
        autolaunch
            .enable()
            .map_err(|e| format!("启用开机自启动失败: {}", e))?;
    } else if autolaunch.is_enabled().unwrap_or(false) {
        autolaunch
            .disable()
            .map_err(|e| format!("禁用开机自启动失败: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_settings(state: State<'_, SettingsState>) -> Result<AppSettings, String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?.clone();

    if let Ok(enabled) = query_autostart_enabled() {
        settings.auto_start = enabled;
    }

    Ok(settings)
}

#[tauri::command]
pub fn save_settings(
    state: State<'_, SettingsState>,
    auto_start: bool,
    close_to_tray: bool,
    shortcuts: HashMap<String, String>,
) -> Result<AppSettings, String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.auto_start = auto_start;
    settings.close_to_tray = close_to_tray;
    settings.shortcuts = shortcuts;
    settings.save()?;

    update_autostart(auto_start)?;

    Ok(settings.clone())
}

#[tauri::command]
pub fn update_shortcuts(
    app: tauri::AppHandle,
    state: State<'_, SettingsState>,
    shortcuts: HashMap<String, String>,
) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.shortcuts = shortcuts;
    settings.save()?;

    // Re-register shortcuts in the app
    crate::register_shortcuts_for_app(&app, &settings.shortcuts);

    Ok(())
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

const GITHUB_REPO: &str = "caojiabin2012/tool-kit";

fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|part| part.parse().ok())
            .collect()
    };
    let latest_parts = parse(latest);
    let current_parts = parse(current);
    let len = latest_parts.len().max(current_parts.len());

    for i in 0..len {
        let latest_part = *latest_parts.get(i).unwrap_or(&0);
        let current_part = *current_parts.get(i).unwrap_or(&0);
        if latest_part > current_part {
            return true;
        }
        if latest_part < current_part {
            return false;
        }
    }

    false
}

fn pick_download_url(assets: &[GitHubAsset]) -> Option<String> {
    let preferred_suffixes: &[&str] = if cfg!(target_os = "windows") {
        &["-setup.exe", ".msi"]
    } else if cfg!(target_os = "macos") {
        &[".dmg"]
    } else if cfg!(target_os = "linux") {
        &[".AppImage", ".deb"]
    } else {
        &["-setup.exe", ".msi", ".dmg", ".AppImage", ".deb"]
    };

    for suffix in preferred_suffixes {
        if let Some(asset) = assets.iter().find(|a| a.name.ends_with(suffix)) {
            return Some(asset.browser_download_url.clone());
        }
    }

    assets.first().map(|a| a.browser_download_url.clone())
}

#[tauri::command]
pub async fn check_for_update() -> Result<Option<UpdateInfo>, String> {
    let url = format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest");
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("User-Agent", "tool-kit-app")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("无法连接更新服务器: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("检查更新失败 (HTTP {})", resp.status()));
    }

    let release: GitHubRelease = resp
        .json()
        .await
        .map_err(|e| format!("解析更新信息失败: {e}"))?;
    let current = env!("CARGO_PKG_VERSION").to_string();
    let latest = release.tag_name.trim_start_matches('v').to_string();

    if is_newer_version(&latest, &current) {
        Ok(Some(UpdateInfo {
            current_version: current,
            latest_version: latest,
            download_url: pick_download_url(&release.assets),
            release_notes: release.body,
        }))
    } else {
        Ok(None)
    }
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: Option<String>,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[tauri::command]
pub async fn download_and_install_update(
    app: tauri::AppHandle,
    download_url: String,
) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let file_name = download_url
        .rsplit('/')
        .next()
        .unwrap_or("update.exe");
    let installer_path = temp_dir.join(file_name);

    let client = reqwest::Client::new();
    let resp = client
        .get(&download_url)
        .header("User-Agent", "tool-kit-app")
        .send()
        .await
        .map_err(|e| format!("下载失败: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("下载失败: {}", resp.status()));
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("读取下载数据失败: {e}"))?;

    std::fs::write(&installer_path, &bytes)
        .map_err(|e| format!("保存安装包失败: {e}"))?;

    log::info!("Update installer saved to: {}", installer_path.display());

    launch_installer(&installer_path, file_name)?;

    let app_handle = app.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
        app_handle.exit(0);
    });

    Ok("安装程序已启动，应用即将退出以完成更新".to_string())
}

#[cfg(windows)]
fn launch_installer(installer_path: &std::path::Path, file_name: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let path_str = installer_path.to_string_lossy().replace('\'', "''");
    let _ = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!("Unblock-File -LiteralPath '{path_str}'"),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .status();

    let path = installer_path
        .to_str()
        .ok_or_else(|| "安装包路径无效".to_string())?;

    if file_name.ends_with(".msi") {
        std::process::Command::new("msiexec")
            .args(["/i", path, "/passive", "/norestart"])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("启动安装器失败: {e}"))?;
    } else {
        std::process::Command::new(path)
            .arg("/S")
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("启动安装器失败: {e}"))?;
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn launch_installer(installer_path: &std::path::Path, _file_name: &str) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mut perms = std::fs::metadata(installer_path)
        .map_err(|e| format!("读取安装包权限失败: {e}"))?
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(installer_path, perms)
        .map_err(|e| format!("设置安装包权限失败: {e}"))?;

    std::process::Command::new(installer_path)
        .spawn()
        .map_err(|e| format!("启动安装器失败: {e}"))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn launch_installer(installer_path: &std::path::Path, _file_name: &str) -> Result<(), String> {
    std::process::Command::new("open")
        .arg(installer_path)
        .spawn()
        .map_err(|e| format!("启动安装器失败: {e}"))?;
    Ok(())
}

#[derive(Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
}
