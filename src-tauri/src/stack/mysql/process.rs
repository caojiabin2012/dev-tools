use std::path::Path;

use crate::stack::mysql::config::write_config;
use crate::stack::mysql::install::find_mysqld;
use crate::stack::process_util::{
    check_port_before_start_with_process, find_pid_by_port, is_port_listening, kill_pid,
    path_forward, run_command_in, service_status, spawn_service_in, tail_file, wait_for_port,
    wait_for_port_release,
};
use crate::stack::store::{load_store, mysql_paths, save_store};
use crate::stack::types::MysqlInstall;

pub fn stop(install: &MysqlInstall) -> Result<(), String> {
    if is_port_listening(install.port) {
        if let Ok(admin) = mysqladmin_path(Path::new(&install.home_dir)) {
            let port = install.port.to_string();
            let bin_dir = Path::new(&install.home_dir).join("bin");
            let _ = run_command_in(
                &admin,
                Some(&bin_dir),
                &["-h", "127.0.0.1", "-u", "root", "-P", &port, "shutdown"],
            );
            std::thread::sleep(std::time::Duration::from_millis(800));
        }
        if is_port_listening(install.port) {
            if let Some(pid) = install.pid.or_else(|| find_pid_by_port(install.port)) {
                kill_pid(pid)?;
            }
        }
    } else if let Some(pid) = install.pid {
        let _ = kill_pid(pid);
    }
    let _ = wait_for_port_release(install.port, std::time::Duration::from_secs(10));
    clear_pid()
}

pub fn start() -> Result<MysqlInstall, String> {
    let mut store = load_store();
    let install = store.mysql.as_ref().ok_or("MySQL 尚未安装")?.clone();
    let expected = match install.engine {
        crate::stack::types::DbEngine::MariaDb => Some("mariadbd.exe"),
        crate::stack::types::DbEngine::Mysql => Some("mysqld.exe"),
    };
    if let Some(running_pid) = check_port_before_start_with_process(
        install.port,
        install.pid,
        install.engine.label(),
        expected,
    )? {
        if let Some(ref mut mysql) = store.mysql {
            if mysql.pid.is_none() {
                mysql.pid = Some(running_pid);
                save_store(&store)?;
            }
        }
        let _ = crate::stack::mysql::sample::ensure_test_database(&install);
        return Ok(install);
    }

    let root = store.install_root.as_ref().ok_or("安装目录未设置")?;
    let install_root = Path::new(root);
    write_config(install_root, &install)?;

    let (my_ini_path, data_dir, logs_dir) = mysql_paths(install_root, install.engine);
    std::fs::create_dir_all(&logs_dir).map_err(|e| e.to_string())?;
    remove_stale_pid_files(&data_dir);
    let datadir_ini = data_dir.join("my.ini");
    if datadir_ini.exists() {
        let _ = std::fs::remove_file(datadir_ini);
    }
    let _ = wait_for_port_release(install.port, std::time::Duration::from_secs(5));

    let mysqld = find_mysqld(Path::new(&install.home_dir), install.engine)?;
    let home = Path::new(&install.home_dir);
    let ini = path_forward(&my_ini_path);
    let log = logs_dir.join("error.log");
    let defaults = format!("--defaults-file={ini}");

    for attempt in 0..2 {
        let pid = spawn_service_in(
            &mysqld,
            Some(&home.join("bin")),
            &[&defaults],
        )?;

        if wait_for_port(install.port, std::time::Duration::from_secs(20)) {
            let running_pid = find_pid_by_port(install.port).unwrap_or(pid);
            if let Some(ref mut mysql) = store.mysql {
                mysql.pid = Some(running_pid);
                save_store(&store)?;
            }
            let _ = crate::stack::mysql::sample::ensure_test_database(&install);
            return store.mysql.clone().ok_or_else(|| "MySQL 状态丢失".into());
        }

        let _ = kill_pid(pid);
        if let Some(running_pid) = find_pid_by_port(install.port) {
            if let Some(ref mut mysql) = store.mysql {
                mysql.pid = Some(running_pid);
                save_store(&store)?;
            }
            let _ = crate::stack::mysql::sample::ensure_test_database(&install);
            return store.mysql.clone().ok_or_else(|| "MySQL 状态丢失".into());
        }

        if attempt == 0 {
            if let Some(pid) = find_pid_by_port(install.port) {
                let _ = kill_pid(pid);
            }
            let _ = wait_for_port_release(install.port, std::time::Duration::from_secs(3));
            remove_stale_pid_files(&data_dir);
            continue;
        }

        let hint = tail_file(&log, 4096).unwrap_or_default();
        return Err(format!(
            "{} 启动失败，端口 {} 未就绪。请查看 {}\n{hint}",
            install.engine.label(),
            install.port,
            log.display()
        ));
    }

    unreachable!()
}

pub fn stop_from_store() -> Result<MysqlInstall, String> {
    let store = load_store();
    let install = store.mysql.as_ref().ok_or("MySQL 尚未安装")?.clone();
    stop(&install)?;
    load_store()
        .mysql
        .clone()
        .ok_or_else(|| "MySQL 状态丢失".into())
}

pub fn status(install: &MysqlInstall) -> crate::stack::types::ServiceStatus {
    service_status(install.port, install.pid)
}

fn mysqladmin_path(home: &Path) -> Result<std::path::PathBuf, String> {
    for name in ["mysqladmin.exe", "mariadb-admin.exe"] {
        let path = home.join("bin").join(name);
        if path.exists() {
            return Ok(path);
        }
    }
    Err("mysqladmin.exe 不存在".into())
}

fn remove_stale_pid_files(data_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(data_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("pid") {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn clear_pid() -> Result<(), String> {
    let mut store = load_store();
    if let Some(ref mut mysql) = store.mysql {
        mysql.pid = None;
        save_store(&store)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn port_probe_when_mariadb_running() {
        if !Path::new(r"D:\jiabin\mysql\mariadb-11.4.2-winx64\bin\mariadbd.exe").exists() {
            return;
        }
        if !is_port_listening(3307) {
            return;
        }
        assert!(
            find_pid_by_port(3307).is_some(),
            "netstat should find MariaDB listener on 3307"
        );
    }

    #[test]
    fn mariadb_start_integration() {
        let home = Path::new(r"D:\jiabin\mysql\mariadb-11.4.2-winx64\bin\mariadbd.exe");
        if !home.exists() {
            return;
        }
        let _ = stop_from_store();
        let started = start().expect("MariaDB start should succeed");
        assert!(is_port_listening(started.port));
        let _ = stop(&started);
    }
}
