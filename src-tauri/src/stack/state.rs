use crate::stack::download::is_downloaded_for_store;
use crate::stack::manifest;
use crate::stack::process_util::{PortSnapshot, service_status_with_snapshot};
use crate::stack::service::{build_env_info, component_paths};
use crate::stack::store::{load_store, zip_cache_path_for_store};
use crate::stack::types::{ComponentView, ServiceStatus, StackState, StackStore, VersionOption};

pub fn build_stack_state() -> StackState {
    let store = load_store();
    let settings = store.settings.clone();
    let env_info = build_env_info(&store);
    let port_snapshot = PortSnapshot::capture();
    let components = manifest::WINDOWS_COMPONENTS
        .iter()
        .filter_map(|m| build_component_view(m.id, &store, &port_snapshot).ok())
        .collect();
    StackState {
        install_root: store.install_root.clone(),
        settings,
        env_info,
        components,
    }
}

fn resolve_selected_version_id(store: &StackStore, component_id: &str) -> Result<String, String> {
    let comp = manifest::get_component(component_id)?;
    if let Some(id) = store.version_prefs.get(component_id) {
        if manifest::find_version(component_id, id).is_ok() {
            return Ok(id.clone());
        }
    }
    Ok(comp.default_version_id.to_string())
}

fn build_component_view(id: &str, store: &StackStore, port_snapshot: &PortSnapshot) -> Result<ComponentView, String> {
    let comp = manifest::get_component(id)?;
    let version_id = resolve_selected_version_id(store, id)?;
    let (_, ver) = manifest::resolve_version(id, Some(&version_id))?;

    let available_versions: Vec<VersionOption> = comp
        .versions
        .iter()
        .map(|v| VersionOption {
            id: v.id.to_string(),
            label: v.label.to_string(),
            engine: v.engine.map(|e| match e {
                crate::stack::types::DbEngine::Mysql => "mysql".to_string(),
                crate::stack::types::DbEngine::MariaDb => "mariadb".to_string(),
            }),
        })
        .collect();

    let downloaded = is_downloaded_for_store(store, id, &version_id);
    let download_path = zip_cache_path_for_store(store, &ver.filename)
        .ok()
        .filter(|p| p.exists())
        .map(|p| p.to_string_lossy().into_owned());

    let (config_path, log_path) = if store.install_root.is_some()
        && matches!(id, "mysql" | "nginx" | "php" | "redis")
    {
        let installed = match id {
            "mysql" => store.mysql.is_some(),
            "nginx" => store.nginx.is_some(),
            "php" => store.php.is_some(),
            "redis" => store.redis.is_some(),
            _ => false,
        };
        if installed {
            component_paths(store, id)
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    match id {
        "mysql" => Ok(component_from_install(
            id,
            comp.name,
            &version_id,
            ver.label,
            available_versions,
            comp.default_port,
            downloaded,
            download_path,
            config_path,
            log_path,
            store.mysql.as_ref().map(|m| {
                (
                    m.port,
                    m.home_dir.clone(),
                    m.pid,
                    service_status_with_snapshot(port_snapshot, m.port, m.pid),
                    Some(format!("127.0.0.1:{} · root 无密码", m.port)),
                    m.version_label.clone(),
                )
            }),
        )),
        "nginx" => Ok(component_from_install(
            id,
            comp.name,
            &version_id,
            ver.label,
            available_versions,
            comp.default_port,
            downloaded,
            download_path,
            config_path,
            log_path,
            store.nginx.as_ref().map(|n| {
                (
                    n.port,
                    n.home_dir.clone(),
                    n.pid,
                    service_status_with_snapshot(port_snapshot, n.port, n.pid),
                    Some(format!("http://127.0.0.1:{}", n.port)),
                    n.version_label.clone(),
                )
            }),
        )),
        "php" => Ok(component_from_install(
            id,
            comp.name,
            &version_id,
            ver.label,
            available_versions,
            comp.default_port,
            downloaded,
            download_path,
            config_path,
            log_path,
            store.php.as_ref().map(|p| {
                (
                    p.port,
                    p.home_dir.clone(),
                    p.pid,
                    service_status_with_snapshot(port_snapshot, p.port, p.pid),
                    Some(format!("FastCGI 127.0.0.1:{}", p.port)),
                    p.version_label.clone(),
                )
            }),
        )),
        "redis" => Ok(component_from_install(
            id,
            comp.name,
            &version_id,
            ver.label,
            available_versions,
            comp.default_port,
            downloaded,
            download_path,
            config_path,
            log_path,
            store.redis.as_ref().map(|r| {
                (
                    r.port,
                    r.home_dir.clone(),
                    r.pid,
                    service_status_with_snapshot(port_snapshot, r.port, r.pid),
                    Some(format!("127.0.0.1:{}", r.port)),
                    r.version_label.clone(),
                )
            }),
        )),
        _ => Err(format!("未知组件: {id}")),
    }
}

fn component_from_install(
    id: &str,
    name: &str,
    selected_version_id: &str,
    selected_version_label: &str,
    available_versions: Vec<VersionOption>,
    default_port: u16,
    downloaded: bool,
    download_path: Option<String>,
    config_path: Option<String>,
    log_path: Option<String>,
    installed: Option<(u16, String, Option<u32>, ServiceStatus, Option<String>, String)>,
) -> ComponentView {
    if let Some((port, home_dir, pid, status, hint, installed_version)) = installed {
        ComponentView {
            id: id.to_string(),
            name: name.to_string(),
            selected_version_id: selected_version_id.to_string(),
            selected_version_label: installed_version,
            available_versions,
            default_port,
            downloaded,
            download_path,
            installed: true,
            status,
            port: Some(port),
            home_dir: Some(home_dir),
            pid,
            hint,
            config_path,
            log_path,
        }
    } else {
        ComponentView {
            id: id.to_string(),
            name: name.to_string(),
            selected_version_id: selected_version_id.to_string(),
            selected_version_label: selected_version_label.to_string(),
            available_versions,
            default_port,
            downloaded,
            download_path,
            installed: false,
            status: ServiceStatus::NotInstalled,
            port: None,
            home_dir: None,
            pid: None,
            hint: if downloaded {
                Some("安装包已就绪，点击安装".into())
            } else {
                Some("选择版本后下载或导入本地 zip".into())
            },
            config_path: None,
            log_path: None,
        }
    }
}
