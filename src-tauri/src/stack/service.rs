use std::path::Path;

use crate::stack::config::sync_configs;
use crate::stack::manifest;
use crate::stack::process_util::is_port_listening;
use crate::stack::store::{load_store, mysql_paths, require_install_root, resolve_www_root, save_store};
use crate::stack::types::{DbEngine, InstallComponentParams, StackEnvInfo, UpdateStackSettingsParams};

pub fn install(params: &InstallComponentParams) -> Result<(), String> {
    let (comp, ver) = manifest::resolve_version(&params.component, params.version_id.as_deref())?;
    let port = params.port.unwrap_or(comp.default_port);
    let source = params.source_path.as_deref();
    let version_id = params.version_id.as_deref();

    match params.component.as_str() {
        "mysql" => {
            let engine = ver
                .engine
                .or(params.engine)
                .unwrap_or(DbEngine::Mysql);
            crate::stack::mysql::install::install(source, port, engine, ver.label, version_id)?;
        }
        "nginx" => {
            crate::stack::nginx::install(source, port, ver.label, version_id)?;
        }
        "php" => {
            crate::stack::php::install(source, port, ver.label, version_id)?;
        }
        "redis" => {
            crate::stack::redis::install(source, port, ver.label, version_id)?;
        }
        _ => return Err(format!("未知组件: {}", params.component)),
    }

    let store = load_store();
    if let Some(root) = store.install_root.as_deref() {
        let _ = sync_configs(&store, Path::new(root));
    }
    Ok(())
}

pub fn start(component: &str) -> Result<(), String> {
    match component {
        "mysql" => {
            crate::stack::mysql::process::start()?;
        }
        "nginx" => {
            crate::stack::nginx::start()?;
        }
        "php" => {
            crate::stack::php::start()?;
        }
        "redis" => {
            crate::stack::redis::start()?;
        }
        _ => return Err(format!("未知组件: {component}")),
    }
    Ok(())
}

pub fn stop(component: &str) -> Result<(), String> {
    match component {
        "mysql" => {
            crate::stack::mysql::process::stop_from_store()?;
        }
        "nginx" => {
            crate::stack::nginx::stop_from_store()?;
        }
        "php" => {
            crate::stack::php::stop_from_store()?;
        }
        "redis" => {
            crate::stack::redis::stop_from_store()?;
        }
        _ => return Err(format!("未知组件: {component}")),
    }
    Ok(())
}

pub fn uninstall(component: &str) -> Result<(), String> {
    match component {
        "mysql" => crate::stack::mysql::install::uninstall()?,
        "nginx" => crate::stack::nginx::uninstall()?,
        "php" => crate::stack::php::uninstall()?,
        "redis" => crate::stack::redis::uninstall()?,
        _ => return Err(format!("未知组件: {component}")),
    }
    Ok(())
}

pub fn start_all() -> Result<(), String> {
    let mut errors = Vec::new();
    for id in ["redis", "mysql", "php", "nginx"] {
        let store = crate::stack::store::load_store();
        let installed = match id {
            "mysql" => store.mysql.is_some(),
            "nginx" => store.nginx.is_some(),
            "php" => store.php.is_some(),
            "redis" => store.redis.is_some(),
            _ => false,
        };
        if installed {
            if let Err(err) = start(id) {
                errors.push(format!("{id}: {err}"));
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

pub fn stop_all() -> Result<(), String> {
    for id in ["nginx", "php", "mysql", "redis"] {
        let store = crate::stack::store::load_store();
        let installed = match id {
            "mysql" => store.mysql.is_some(),
            "nginx" => store.nginx.is_some(),
            "php" => store.php.is_some(),
            "redis" => store.redis.is_some(),
            _ => false,
        };
        if installed {
            let _ = stop(id);
        }
    }
    Ok(())
}

pub fn update_settings(params: &UpdateStackSettingsParams) -> Result<(), String> {
    let mut store = load_store();
    let root = require_install_root()?;

    if let Some(subdir) = params.www_subdir.as_deref() {
        let trimmed = subdir.trim().trim_matches(['/', '\\']);
        if trimmed.is_empty() {
            return Err("网站目录不能为空".into());
        }
        store.settings.www_subdir = trimmed.replace('\\', "/");
    }

    save_store(&store)?;
    sync_configs(&store, &root)?;
    Ok(())
}

pub fn set_component_port(component: &str, port: u16) -> Result<(), String> {
    if port == 0 {
        return Err("端口无效".into());
    }

    let mut store = load_store();
    let root = require_install_root()?;

    let running = is_component_running(&store, component)?;
    if running {
        return Err("请先停止该服务再修改端口".into());
    }

    ensure_port_free(&store, component, port)?;

    match component {
        "mysql" => {
            let mysql = store.mysql.as_mut().ok_or("MySQL 尚未安装")?;
            mysql.port = port;
        }
        "nginx" => {
            let nginx = store.nginx.as_mut().ok_or("Nginx 尚未安装")?;
            nginx.port = port;
        }
        "php" => {
            let php = store.php.as_mut().ok_or("PHP 尚未安装")?;
            php.port = port;
        }
        "redis" => {
            let redis = store.redis.as_mut().ok_or("Redis 尚未安装")?;
            redis.port = port;
        }
        _ => return Err(format!("未知组件: {component}")),
    }

    save_store(&store)?;
    sync_configs(&store, &root)?;
    Ok(())
}

pub fn open_component_config(component: &str) -> Result<(), String> {
    let store = load_store();
    let root = store.install_root.as_ref().ok_or("尚未设置安装目录")?;
    let install_root = Path::new(root);

    let path = match component {
        "mysql" => {
            let engine = store
                .mysql
                .as_ref()
                .map(|m| m.engine)
                .unwrap_or(DbEngine::Mysql);
            mysql_paths(install_root, engine).0
        }
        "nginx" => {
            let nginx = store.nginx.as_ref().ok_or("Nginx 尚未安装")?;
            let path = crate::stack::nginx::runtime_conf_path(nginx);
            if !path.exists() {
                let php_port = store.php.as_ref().map(|p| p.port).unwrap_or(9000);
                let www = resolve_www_root(install_root, &store.settings);
                let _ = crate::stack::nginx::write_config(nginx, php_port, &www);
            }
            path
        }
        "php" => {
            let php = store.php.as_ref().ok_or("PHP 尚未安装")?;
            let path = crate::stack::php::runtime_ini_path(php);
            if !path.exists() {
                let _ = crate::stack::php::write_config(install_root, php);
            }
            path
        }
        "redis" => {
            let redis = store.redis.as_ref().ok_or("Redis 尚未安装")?;
            let path = crate::stack::redis::runtime_conf_path(redis);
            if !path.exists() {
                let _ = crate::stack::redis::write_config(install_root, redis);
            }
            path
        }
        _ => return Err(format!("未知组件: {component}")),
    };

    if !path.exists() {
        return Err(format!("配置文件不存在: {}", path.display()));
    }
    open::that(path).map_err(|e| e.to_string())
}

pub fn open_component_log(component: &str) -> Result<(), String> {
    use std::fs;

    let store = load_store();
    let root = store.install_root.as_ref().ok_or("尚未设置安装目录")?;
    let install_root = Path::new(root);

    let path = match component {
        "mysql" => {
            let engine = store
                .mysql
                .as_ref()
                .map(|m| m.engine)
                .unwrap_or(DbEngine::Mysql);
            mysql_paths(install_root, engine).2.join("error.log")
        }
        "nginx" => store
            .nginx
            .as_ref()
            .map(|n| crate::stack::nginx::log_path(n))
            .ok_or("Nginx 尚未安装")?,
        "php" => crate::stack::php::log_path(install_root),
        "redis" => return Err("Redis 日志请查看数据目录".into()),
        _ => return Err(format!("未知组件: {component}")),
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if !path.exists() {
        fs::write(&path, "").map_err(|e| e.to_string())?;
    }
    open::that(path).map_err(|e| e.to_string())
}

pub fn open_site() -> Result<(), String> {
    let store = load_store();
    let nginx = store.nginx.as_ref().ok_or("Nginx 尚未安装")?;
    let url = format!("http://127.0.0.1:{}", nginx.port);
    open::that(url).map_err(|e| e.to_string())
}

pub fn build_env_info(store: &crate::stack::types::StackStore) -> StackEnvInfo {
    let www = store
        .install_root
        .as_ref()
        .map(|r| resolve_www_root(Path::new(r), &store.settings))
        .and_then(|p| p.to_str().map(String::from));

    let site_url = store
        .nginx
        .as_ref()
        .map(|n| format!("http://127.0.0.1:{}", n.port));

    StackEnvInfo {
        site_url,
        www_root: www,
        mysql_host: "127.0.0.1".into(),
        mysql_port: store.mysql.as_ref().map(|m| m.port),
        mysql_user: "root".into(),
        mysql_password: String::new(),
        php_fastcgi: store
            .php
            .as_ref()
            .map(|p| format!("127.0.0.1:{}", p.port)),
        redis_addr: store
            .redis
            .as_ref()
            .map(|r| format!("127.0.0.1:{}", r.port)),
    }
}

fn is_component_running(
    store: &crate::stack::types::StackStore,
    component: &str,
) -> Result<bool, String> {
    let status = match component {
        "mysql" => store
            .mysql
            .as_ref()
            .map(|m| crate::stack::mysql::process::status(m)),
        "nginx" => store
            .nginx
            .as_ref()
            .map(|n| crate::stack::nginx::status(n)),
        "php" => store.php.as_ref().map(|p| crate::stack::php::status(p)),
        "redis" => store
            .redis
            .as_ref()
            .map(|r| crate::stack::redis::status(r)),
        _ => return Err(format!("未知组件: {component}")),
    };
    Ok(matches!(
        status,
        Some(crate::stack::types::ServiceStatus::Running)
    ))
}

fn ensure_port_free(
    store: &crate::stack::types::StackStore,
    component: &str,
    port: u16,
) -> Result<(), String> {
    let conflicts = [
        ("mysql", store.mysql.as_ref().map(|m| m.port)),
        ("nginx", store.nginx.as_ref().map(|n| n.port)),
        ("php", store.php.as_ref().map(|p| p.port)),
        ("redis", store.redis.as_ref().map(|r| r.port)),
    ];

    for (id, existing) in conflicts {
        if id == component {
            continue;
        }
        if existing == Some(port) {
            return Err(format!("端口 {port} 已被 {id} 使用"));
        }
    }

    if is_port_listening(port) {
        return Err(format!("端口 {port} 已被其他程序占用"));
    }
    Ok(())
}

pub fn component_paths(
    store: &crate::stack::types::StackStore,
    component: &str,
) -> (Option<String>, Option<String>) {
    let Some(root) = store.install_root.as_ref() else {
        return (None, None);
    };
    let install_root = Path::new(root);

    match component {
        "mysql" => {
            let engine = store
                .mysql
                .as_ref()
                .map(|m| m.engine)
                .unwrap_or(DbEngine::Mysql);
            let (conf, _, logs) = mysql_paths(install_root, engine);
            (
                Some(conf.to_string_lossy().into_owned()),
                Some(logs.join("error.log").to_string_lossy().into_owned()),
            )
        }
        "nginx" => store.nginx.as_ref().map_or((None, None), |n| {
            (
                Some(
                    crate::stack::nginx::runtime_conf_path(n)
                        .to_string_lossy()
                        .into_owned(),
                ),
                Some(crate::stack::nginx::log_path(n).to_string_lossy().into_owned()),
            )
        }),
        "php" => store.php.as_ref().map_or((None, None), |p| {
            (
                Some(
                    crate::stack::php::runtime_ini_path(p)
                        .to_string_lossy()
                        .into_owned(),
                ),
                Some(
                    crate::stack::php::log_path(install_root)
                        .to_string_lossy()
                        .into_owned(),
                ),
            )
        }),
        "redis" => store.redis.as_ref().map_or((None, None), |r| {
            let conf = crate::stack::redis::runtime_conf_path(r);
            (Some(conf.to_string_lossy().into_owned()), None)
        }),
        _ => (None, None),
    }
}
