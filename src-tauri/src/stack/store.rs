use std::fs;
use std::path::{Path, PathBuf};

use crate::stack::types::{StackSettings, StackStore};

pub fn stack_store_path() -> PathBuf {
    crate::app_paths::app_data_dir()
        .join("stack")
        .join("stack-env.json")
}

pub fn load_store() -> StackStore {
    let path = stack_store_path();
    if !path.exists() {
        return StackStore::default();
    }
    match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => StackStore::default(),
    }
}

pub fn save_store(store: &StackStore) -> Result<(), String> {
    let path = stack_store_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn require_install_root() -> Result<PathBuf, String> {
    let store = load_store();
    store
        .install_root
        .ok_or_else(|| "请先设置环境安装目录".into())
        .map(PathBuf::from)
}

pub fn downloads_dir(install_root: &Path) -> PathBuf {
    install_root.join("downloads")
}

use crate::stack::types::DbEngine;

pub fn mysql_paths(install_root: &Path, engine: DbEngine) -> (PathBuf, PathBuf, PathBuf) {
    let base = install_root.join("mysql");
    let data = match engine {
        DbEngine::Mysql => base.join("data"),
        DbEngine::MariaDb => base.join("data-mariadb"),
    };
    (
        base.join("my.ini"),
        data,
        base.join("logs"),
    )
}

pub fn nginx_conf_path(install_root: &Path) -> PathBuf {
    install_root.join("nginx").join("devtools-nginx.conf")
}

/// 配置文件必须放在 nginx.exe 同目录，include conf/* 才能正确解析
pub fn nginx_runtime_conf_path(home_dir: &Path) -> PathBuf {
    home_dir.join("devtools-nginx.conf")
}

pub fn php_ini_path(install_root: &Path) -> PathBuf {
    install_root.join("php").join("devtools-php.ini")
}

/// php.ini 放在 php-cgi.exe 同目录，避免 Windows 下 -c 绝对路径异常
pub fn php_runtime_ini_path(home_dir: &Path) -> PathBuf {
    home_dir.join("devtools-php.ini")
}

pub fn redis_conf_path(install_root: &Path) -> PathBuf {
    install_root.join("redis").join("devtools-redis.conf")
}

/// msys2 版 Redis 只能用相对路径读配置，必须放在 redis-server.exe 同目录
pub fn redis_runtime_conf_path(home_dir: &Path) -> PathBuf {
    home_dir.join("devtools-redis.conf")
}

pub fn resolve_www_root(install_root: &Path, settings: &StackSettings) -> PathBuf {
    let sub = settings.www_subdir.trim().trim_matches(['/', '\\']);
    if sub.is_empty() {
        install_root.join("www").join("default")
    } else {
        install_root.join(sub.replace('/', std::path::MAIN_SEPARATOR_STR))
    }
}

pub fn www_root(install_root: &Path) -> PathBuf {
    let store = load_store();
    resolve_www_root(install_root, &store.settings)
}

pub fn selected_version_id(component: &str) -> Result<String, String> {
    let store = load_store();
    if let Some(id) = store.version_prefs.get(component) {
        if crate::stack::manifest::find_version(component, id).is_ok() {
            return Ok(id.clone());
        }
    }
    Ok(crate::stack::manifest::default_version_id(component)?.to_string())
}

pub fn set_version_pref(component: &str, version_id: &str) -> Result<(), String> {
    crate::stack::manifest::find_version(component, version_id)?;
    let mut store = load_store();
    store.version_prefs.insert(component.to_string(), version_id.to_string());
    save_store(&store)
}

pub fn zip_cache_path_for(_component: &str, filename: &str) -> Result<PathBuf, String> {
    let root = require_install_root()?;
    Ok(downloads_dir(&root).join(filename))
}

pub fn zip_cache_path_for_store(store: &StackStore, filename: &str) -> Result<PathBuf, String> {
    let root = store
        .install_root
        .as_ref()
        .ok_or_else(|| "请先设置环境安装目录".to_string())?;
    Ok(downloads_dir(Path::new(root)).join(filename))
}
