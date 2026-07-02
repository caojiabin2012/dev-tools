use std::fs;
use std::path::{Path, PathBuf};

use crate::stack::extract::{extract_zip, find_mysql_home};
use crate::stack::mysql::config::render_my_ini;
use crate::stack::process_util::run_command_in;
use crate::stack::store::{load_store, mysql_paths, require_install_root, save_store};
use crate::stack::types::{DbEngine, MysqlInstall};

pub fn install(
    source_path: Option<&str>,
    port: u16,
    engine: DbEngine,
    version_name: &str,
    version_id: Option<&str>,
) -> Result<MysqlInstall, String> {
    let mut store = load_store();
    let install_root = require_install_root()?;
    fs::create_dir_all(&install_root).map_err(|e| e.to_string())?;

    let source = crate::stack::download::resolve_source("mysql", source_path, version_id)?;
    let mysql_base = install_root.join("mysql");
    fs::create_dir_all(&mysql_base).map_err(|e| e.to_string())?;

    let scan_root = if source.is_file() {
        extract_zip(&source, &mysql_base)?;
        &mysql_base
    } else {
        &source
    };
    let home_dir = find_mysql_home(scan_root)?;

    let mysqld = resolve_mysqld_binary(&home_dir, engine)?;
    let version_label = format!("{} · {}", engine.label(), version_name);

    let (my_ini_path, data_dir, logs_dir) = mysql_paths(&install_root, engine);
    fs::create_dir_all(&logs_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    let ini = render_my_ini(engine, &home_dir, &data_dir, &logs_dir, port);
    fs::write(&my_ini_path, ini).map_err(|e| e.to_string())?;

    if !is_data_initialized(&data_dir) {
        if has_partial_data(&data_dir) {
            reset_data_dir(&data_dir)?;
        }
        initialize_database(engine, &home_dir, &mysqld, &my_ini_path, &data_dir, port)?;
    }

    let install = MysqlInstall {
        engine,
        version_label,
        home_dir: home_dir.to_string_lossy().into_owned(),
        port,
        initialized: true,
        pid: None,
    };
    store.mysql = Some(install.clone());
    save_store(&store)?;
    Ok(install)
}

pub fn uninstall() -> Result<(), String> {
    let mut store = load_store();
    if let Some(mysql) = store.mysql.take() {
        let _ = crate::stack::mysql::process::stop(&mysql);
    }
    save_store(&store)
}

pub fn find_mysqld(home: &Path, engine: crate::stack::types::DbEngine) -> Result<PathBuf, String> {
    resolve_mysqld_binary(home, engine)
}

fn resolve_mysqld_binary(home: &Path, engine: crate::stack::types::DbEngine) -> Result<PathBuf, String> {
    let order = match engine {
        crate::stack::types::DbEngine::MariaDb => ["mariadbd.exe", "mysqld.exe"],
        crate::stack::types::DbEngine::Mysql => ["mysqld.exe", "mariadbd.exe"],
    };
    for name in order {
        let path = home.join("bin").join(name);
        if path.exists() {
            return Ok(path);
        }
    }
    Err(format!("未找到 mysqld.exe / mariadbd.exe: {}", home.display()))
}

fn resolve_install_db_binary(home: &Path) -> Result<PathBuf, String> {
    for name in ["mariadb-install-db.exe", "mysql_install_db.exe"] {
        let path = home.join("bin").join(name);
        if path.exists() {
            return Ok(path);
        }
    }
    Err(format!(
        "未找到 mariadb-install-db.exe: {}",
        home.join("bin").display()
    ))
}

fn is_data_initialized(data_dir: &Path) -> bool {
    data_dir.join("mysql").is_dir()
}

fn has_partial_data(data_dir: &Path) -> bool {
    data_dir
        .read_dir()
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false)
}

fn reset_data_dir(data_dir: &Path) -> Result<(), String> {
    if data_dir.exists() {
        fs::remove_dir_all(data_dir).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(data_dir).map_err(|e| e.to_string())
}

fn initialize_database(
    engine: DbEngine,
    home_dir: &Path,
    mysqld: &Path,
    my_ini_path: &Path,
    data_dir: &Path,
    port: u16,
) -> Result<(), String> {
    let bin_dir = home_dir.join("bin");
    let ini = path_forward(my_ini_path);
    let datadir = path_forward(data_dir);

    let output = match engine {
        DbEngine::Mysql => run_command_in(
            mysqld,
            Some(home_dir),
            &[
                &format!("--defaults-file={ini}"),
                "--initialize-insecure",
            ],
        )?,
        DbEngine::MariaDb => {
            let install_db = resolve_install_db_binary(home_dir)?;
            run_command_in(
                &install_db,
                Some(&bin_dir),
                &[
                    &format!("--datadir={datadir}"),
                    &format!("--config={ini}"),
                    &format!("--port={port}"),
                    "--default-user",
                    "--silent",
                ],
            )?
        }
    };

    if !output.status.success() {
        return Err(format!(
            "{} 初始化失败:\n{}\n{}",
            engine.label(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let datadir_ini = data_dir.join("my.ini");
    if datadir_ini.exists() {
        let _ = fs::remove_file(datadir_ini);
    }

    Ok(())
}

fn path_forward(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
