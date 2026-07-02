use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::stack::download::resolve_source;
use crate::stack::extract::{extract_zip, find_home_with_binary};
use crate::stack::process_util::{
    check_port_before_start_with_process, find_pid_by_port, is_port_listening, kill_pid,
    run_command_in, service_status, spawn_service_in, tail_file, wait_for_port,
};
use crate::stack::store::{load_store, nginx_runtime_conf_path, require_install_root, resolve_www_root, save_store, www_root};
use crate::stack::types::NginxInstall;

const RUNTIME_CONF_NAME: &str = "devtools-nginx.conf";

pub fn install(
    source_path: Option<&str>,
    port: u16,
    version_name: &str,
    version_id: Option<&str>,
) -> Result<NginxInstall, String> {
    let mut store = load_store();
    let install_root = require_install_root()?;
    let source = resolve_source("nginx", source_path, version_id)?;
    let base = install_root.join("nginx");
    fs::create_dir_all(&base).map_err(|e| e.to_string())?;

    if source.is_file() {
        extract_zip(&source, &base)?;
    }
    let scan_root = if source.is_file() { &base } else { &source };
    let home_dir = find_home_with_binary(scan_root, &["nginx.exe"])?;
    ensure_nginx_dirs(&home_dir)?;

    let www = www_root(&install_root);
    fs::create_dir_all(&www).map_err(|e| e.to_string())?;
    let store_for_www = load_store();
    crate::stack::www::sync_site_files(&store_for_www, &install_root)?;

    let install = NginxInstall {
        version_label: version_name.to_string(),
        home_dir: home_dir.to_string_lossy().into_owned(),
        port,
        pid: None,
    };
    store.nginx = Some(install.clone());
    save_store(&store)?;

    let php_port = store.php.as_ref().map(|p| p.port).unwrap_or(9000);
    write_config(&install, php_port, &www)?;
    Ok(install)
}

pub fn write_config(install: &NginxInstall, php_port: u16, www: &Path) -> Result<(), String> {
    let home = Path::new(&install.home_dir);
    let conf_path = nginx_runtime_conf_path(home);
    let www_path = path_forward(www);
    let conf = render_nginx_conf(home, install.port, php_port, &www_path);
    fs::write(&conf_path, conf).map_err(|e| e.to_string())?;
    remove_legacy_conf_if_present();
    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    let mut store = load_store();
    if let Some(nginx) = store.nginx.take() {
        let _ = stop(&nginx);
    }
    save_store(&store)
}

pub fn start() -> Result<NginxInstall, String> {
    let mut store = load_store();
    let install = store.nginx.as_ref().ok_or("Nginx 尚未安装")?.clone();
    if let Some(running_pid) =
        check_port_before_start_with_process(install.port, install.pid, "Nginx", Some("nginx.exe"))?
    {
        if let Some(ref mut n) = store.nginx {
            if n.pid.is_none() {
                n.pid = Some(running_pid);
                save_store(&store)?;
            }
        }
        return Ok(install);
    }
    let root = store.install_root.as_ref().ok_or("安装目录未设置")?;
    let install_root = Path::new(root);
    let php_port = store.php.as_ref().map(|p| p.port).unwrap_or(9000);
    let www = resolve_www_root(install_root, &store.settings);
    write_config(&install, php_port, &www)?;

    let home = Path::new(&install.home_dir);
    ensure_nginx_dirs(home)?;
    let nginx = home.join("nginx.exe");
    if !nginx.exists() {
        return Err(format!("未找到 nginx.exe: {}", nginx.display()));
    }

    let home_str = install.home_dir.clone();
    let test = run_command_in(
        &nginx,
        Some(home),
        &["-p", &home_str, "-c", RUNTIME_CONF_NAME, "-t"],
    )?;
    if !test.status.success() {
        return Err(format!(
            "Nginx 配置检查失败:\n{}\n{}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        ));
    }

    spawn_service_in(
        &nginx,
        Some(home),
        &["-p", &home_str, "-c", RUNTIME_CONF_NAME],
    )?;

    if !wait_for_port(install.port, Duration::from_secs(8)) {
        let log = log_path(&install);
        let hint = tail_file(&log, 4096).unwrap_or_default();
        return Err(format!(
            "Nginx 启动失败，端口 {} 未监听。请查看 {}\n{hint}",
            install.port,
            log.display()
        ));
    }

    let pid = find_pid_by_port(install.port);
    if let Some(ref mut n) = store.nginx {
        n.pid = pid;
        save_store(&store)?;
    }
    store.nginx.clone().ok_or_else(|| "Nginx 状态丢失".into())
}

pub fn stop(install: &NginxInstall) -> Result<(), String> {
    let home = Path::new(&install.home_dir);
    let nginx = home.join("nginx.exe");
    if nginx.exists() {
        let home_str = install.home_dir.clone();
        let _ = run_command_in(
            &nginx,
            Some(home),
            &["-p", &home_str, "-c", RUNTIME_CONF_NAME, "-s", "quit"],
        );
        std::thread::sleep(Duration::from_millis(500));
    }
    if is_port_listening(install.port) {
        if let Some(pid) = install.pid.or_else(|| find_pid_by_port(install.port)) {
            kill_pid(pid)?;
        }
    } else if let Some(pid) = install.pid {
        let _ = kill_pid(pid);
    }
    let mut store = load_store();
    if let Some(ref mut n) = store.nginx {
        n.pid = None;
        save_store(&store)?;
    }
    Ok(())
}

pub fn stop_from_store() -> Result<NginxInstall, String> {
    let install = load_store().nginx.ok_or("Nginx 尚未安装")?;
    stop(&install)?;
    load_store().nginx.ok_or_else(|| "Nginx 状态丢失".into())
}

pub fn status(install: &NginxInstall) -> crate::stack::types::ServiceStatus {
    service_status(install.port, install.pid)
}

pub fn log_path(install: &NginxInstall) -> PathBuf {
    Path::new(&install.home_dir).join("logs").join("error.log")
}

pub fn runtime_conf_path(install: &NginxInstall) -> PathBuf {
    nginx_runtime_conf_path(Path::new(&install.home_dir))
}

fn ensure_nginx_dirs(home: &Path) -> Result<(), String> {
    fs::create_dir_all(home.join("logs")).map_err(|e| e.to_string())?;
    fs::create_dir_all(home.join("temp")).map_err(|e| e.to_string())?;
    Ok(())
}

fn render_nginx_conf(home: &Path, port: u16, php_port: u16, www: &str) -> String {
    let mime_types = path_forward(&home.join("conf").join("mime.types"));
    let fastcgi_params = path_forward(&home.join("conf").join("fastcgi_params"));
    format!(
        r#"# Generated by dev-tools — Nginx + PHP FastCGI
worker_processes  1;
error_log logs/error.log;
pid logs/nginx.pid;

events {{
    worker_connections  1024;
}}

http {{
    include       {mime_types};
    default_type  application/octet-stream;
    sendfile      on;
    keepalive_timeout  65;
    client_max_body_size 128m;

    server {{
        listen       {port};
        server_name  localhost;
        root         {www};
        index        index.php index.html index.htm;

        location / {{
            try_files $uri $uri/ /index.php?$query_string;
        }}

        location ~ \.php$ {{
            fastcgi_pass   127.0.0.1:{php_port};
            fastcgi_index  index.php;
            fastcgi_param  SCRIPT_FILENAME  $document_root$fastcgi_script_name;
            include        {fastcgi_params};
        }}

        location ~ /\.ht {{
            deny all;
        }}
    }}
}}
"#
    )
}

fn remove_legacy_conf_if_present() {
    let store = load_store();
    if let Some(root) = store.install_root {
        let legacy = Path::new(&root).join("nginx").join(RUNTIME_CONF_NAME);
        let _ = fs::remove_file(legacy);
    }
}

fn path_forward(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
