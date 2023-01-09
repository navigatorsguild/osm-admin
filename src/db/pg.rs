use std::fs::{File};
use std::fs::Permissions;
use std::io::{BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use crate::error::osm_error::{GenericError, OsmError};
use crate::reporting::stopwatch::StopWatch;

pub fn load(
    jobs: i16,
    host: String,
    port: String,
    user: String,
    password: Option<String>,
    dump_path: &PathBuf,
    _var_lib_path: &PathBuf,
    var_log_path: &PathBuf,
) -> Result<(), GenericError> {
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    log::info!("Load OSM, host: {}:{}, user: {:?}, password provided: {}, jobs: {}, dump path: {:?}",
        host,
        port,
        user,
        match password {Some(_) => "Yes", None => "No"},
        jobs,
        dump_path
    );

    // TODO: get database as parameter
    let database = "openstreetmap".to_string();
    let pgpass_path = PathBuf::from("/root/.pgpass");

    write_password_file(&host, &port, &database, &user, &password, &pgpass_path)?;

    let pg_restore_stdout_path = var_log_path.join("pg_restore.log");
    let pg_restore_stderr_path = var_log_path.join("pg_restore.error.log");


    let pg_restore_stdout = File::create(&pg_restore_stdout_path).or_else(|e| {
        Err(OsmError::new(format!("{:?}: {}", pg_restore_stdout_path, e)))
    })?;
    let pg_restore_stderr = File::create(&pg_restore_stderr_path).or_else(|e| {
        Err(OsmError::new(format!("{:?}: {}", pg_restore_stderr_path, e)))
    })?;


    let p = Command::new("pg_restore")
        .arg("-h").arg(host)
        .arg("-p").arg(port)
        .arg("-U").arg(user)
        .arg("-j").arg(jobs.to_string())
        .arg("-d").arg(database)
        .arg("--no-password")
        .arg(dump_path)
        .stdout(std::process::Stdio::from(pg_restore_stdout))
        .stderr(std::process::Stdio::from(pg_restore_stderr))
        .spawn()?;

    let result = p.wait_with_output();
    match result {
        Ok(output) => {
            match output.status.code() {
                None => {
                    log::error!("Failed loading OSM database, see pg_restore stdout at: {:?}, see stderr at: {:?}",
                        pg_restore_stdout_path,
                        pg_restore_stderr_path
                    );
                    Err(Box::new(OsmError::new("Failed loading OSM database".to_string())))
                }
                Some(0) => {
                    log::info!("Finished loading OSM database. Time: {}", stopwatch);
                    Ok(())
                }
                Some(code) => {
                    log::error!("Failed loading OSM database, error code: {}, see stdout at: {:?}, see stderr at: {:?}",
                        code,
                        pg_restore_stdout_path,
                        pg_restore_stderr_path
                    );
                    Err(Box::new(OsmError::new("Failed loading OSM database".to_string())))
                }
            }
        }
        Err(_) => {
            log::error!("Failed loading OSM database");
            Err(Box::new(OsmError::new("Failed loading OSM database".to_string())))
        }
    }
}

pub fn dump(
    jobs: i16,
    host: String,
    port: String,
    user: String,
    password: Option<String>,
    dump_path: &PathBuf,
    var_lib_path: &PathBuf,
    var_log_path: &PathBuf,
) -> Result<(), GenericError> {
    Ok(())
}

fn write_password_file(host: &String, port: &String, database: &String, user: &String, password: &Option<String>, pgpass_path: &PathBuf) -> Result<(), GenericError> {
    match password {
        None => {
            if std::path::Path::new(pgpass_path).exists() {
                let permissions = std::fs::metadata(pgpass_path).unwrap().permissions();
                let mode = permissions.mode() & 0o777_u32;
                if mode == 0o600 {
                    log::info!("Found PGPASSFILE at: {:?}, permissions: {:#o}", pgpass_path, mode);
                } else {
                    log::warn!("Found PGPASSFILE at: {:?}, wrong permissions: {:#o}. Must be 0o600", pgpass_path, mode);
                }

            } else {
                log::info!("No credentials and no PGPASSFILE file provided. Will succeed on trust connections");
            }
        }
        Some(password) => {
            if std::path::Path::new(pgpass_path).exists() {
                log::warn!("Overwriting the PGPASSFILE at {:?} with provided credentials", pgpass_path);
            } else {
                log::info!("Writing provided credentials to PGPASSFILE at: {:?}", pgpass_path);
            }
            let credentials = format!("{}:{}:{}:{}:{}",
                                      host,
                                      port,
                                      database,
                                      user,
                                      password
            );
            let permissions = Permissions::from_mode(0o600);
            let pgpass_file = File::create(pgpass_path).or_else(|e| {
                Err(OsmError::new(format!("{:?}: {}", pgpass_path, e)))
            })?;
            pgpass_file.set_permissions(permissions).or_else(|e| {
                Err(OsmError::new(format!("{:?}: {}", pgpass_path, e)))
            })?;
            let mut writer = BufWriter::new(pgpass_file);
            writer.write(credentials.as_bytes()).or_else(|e| {
                Err(OsmError::new(format!("{:?}: {}", pgpass_path, e)))
            })?;
            writer.flush()?;
        }
    }
    Ok(())
}

