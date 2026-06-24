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
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_start: false,
            close_to_tray: true,
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
) -> Result<AppSettings, String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.auto_start = auto_start;
    settings.close_to_tray = close_to_tray;
    settings.save()?;

    update_autostart(auto_start)?;

    Ok(settings.clone())
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub async fn check_for_update() -> Result<Option<UpdateInfo>, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.github.com/repos/nicepkg/tool-kit/releases/latest")
        .header("User-Agent", "tool-kit-app")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Ok(None);
    }

    let release: GitHubRelease = resp.json().await.map_err(|e| e.to_string())?;
    let current = env!("CARGO_PKG_VERSION").to_string();
    let latest = release.tag_name.trim_start_matches('v').to_string();

    if latest != current {
        let download_url = release.assets.iter()
            .find(|a| {
                a.name.ends_with(".msi")
                    || a.name.ends_with("-setup.exe")
                    || a.name.ends_with(".dmg")
                    || a.name.ends_with(".AppImage")
            })
            .map(|a| a.browser_download_url.clone());

        Ok(Some(UpdateInfo {
            current_version: current,
            latest_version: latest,
            download_url,
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

#[derive(Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
}
