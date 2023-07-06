use std::fs::File;
use std::fs::Permissions;
use std::io::{BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use anyhow::anyhow;
use benchmark_rs::stopwatch::StopWatch;

pub fn restore(
    jobs: i16,
    host: String,
    port: String,
    user: String,
    password: Option<String>,
    dump_path: &PathBuf,
    _var_lib_path: &PathBuf,
    var_log_path: &PathBuf,
) -> Result<(), anyhow::Error> {
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

    let stdout_path = var_log_path.join("pg_restore.log");
    let stderr_path = var_log_path.join("pg_restore.error.log");

    let (stdout, stderr) = create_redirects(&stdout_path, &stderr_path)?;

    let p = Command::new("pg_restore")
        .arg("-h").arg(host)
        .arg("-p").arg(port)
        .arg("-U").arg(user)
        .arg("-j").arg(jobs.to_string())
        .arg("-d").arg(database)
        .arg("--no-password")
        .arg(dump_path)
        .stdout(std::process::Stdio::from(stdout))
        .stderr(std::process::Stdio::from(stderr))
        .spawn()?;

    let result = p.wait_with_output();
    match result {
        Ok(output) => {
            match output.status.code() {
                None => {
                    log::error!("Failed loading OSM database, see pg_restore stdout at: {:?}, see stderr at: {:?}",
                        stdout_path,
                        stderr_path
                    );
                    Err(anyhow!("Failed loading OSM database"))
                }
                Some(0) => {
                    Ok(())
                }
                Some(code) => {
                    log::error!("Failed loading OSM database, error code: {}, see stdout at: {:?}, see stderr at: {:?}",
                        code,
                        stdout_path,
                        stderr_path
                    );
                    Err(anyhow!("Failed loading OSM database"))
                }
            }
        }
        Err(_) => {
            log::error!("Failed loading OSM database");
            Err(anyhow!("Failed loading OSM database"))
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
    _var_lib_path: &PathBuf,
    var_log_path: &PathBuf,
) -> Result<(), anyhow::Error> {
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    log::info!("Dump OSM, host: {}:{}, user: {:?}, password provided: {}, jobs: {}, dump path: {:?}",
        host,
        port,
        user,
        match password {Some(_) => "Yes", None => "No"},
        jobs,
        dump_path
    );

    let database = "openstreetmap".to_string();
    let pgpass_path = PathBuf::from("~/.pgpass");

    write_password_file(&host, &port, &database, &user, &password, &pgpass_path)?;

    let stdout_path = var_log_path.join("pg_dump.log");
    let stderr_path = var_log_path.join("pg_dump.error.log");

    let (stdout, stderr) = create_redirects(&stdout_path, &stderr_path)?;

    let p = Command::new("pg_dump")
        .arg("-h").arg(host)
        .arg("-p").arg(port)
        .arg("-U").arg(user)
        .arg("-j").arg(jobs.to_string())
        .arg("-d").arg(database)
        .arg("--no-password")
        .arg("--file").arg(dump_path)
        .arg("--format").arg("d")
        .arg("--compress").arg("0")
        .arg("--table").arg("public.nodes")
        .arg("--table").arg("public.node_tags")
        .arg("--table").arg("public.ways")
        .arg("--table").arg("public.way_tags")
        .arg("--table").arg("public.way_nodes")
        .arg("--table").arg("public.relations")
        .arg("--table").arg("public.relation_tags")
        .arg("--table").arg("public.relation_members")
        .arg("--table").arg("public.users")
        .arg("--table").arg("public.changesets")
        .stdout(std::process::Stdio::from(stdout))
        .stderr(std::process::Stdio::from(stderr))
        .spawn()?;

    let result = p.wait_with_output();
    match result {
        Ok(output) => {
            match output.status.code() {
                None => {
                    log::error!("Failed dumping OSM database, see pg_dump stdout at: {:?}, see stderr at: {:?}",
                        stdout_path,
                        stderr_path
                    );
                    Err(anyhow!("Failed dumping OSM database"))
                }
                Some(0) => {
                    log::info!("Finished dumping OSM database. Time: {}", stopwatch);
                    Ok(())
                }
                Some(code) => {
                    log::error!("Failed dumping OSM database, error code: {}, see stdout at: {:?}, see stderr at: {:?}",
                        code,
                        stdout_path,
                        stderr_path
                    );
                    Err(anyhow!("Failed dumping OSM database"))
                }
            }
        }
        Err(_) => {
            log::error!("Failed dumping OSM database");
            Err(anyhow!("Failed dumping OSM database"))
        }
    }
}

fn create_redirects(
    stdout_path: &PathBuf,
    stderr_path: &PathBuf,
) -> Result<(File, File), anyhow::Error> {
    let stdout = File::create(stdout_path).or_else(|e| {
        Err(anyhow!("{:?}: {}", stdout_path, e))
    })?;
    let stderr = File::create(stderr_path).or_else(|e| {
        Err(anyhow!("{:?}: {}", stderr_path, e))
    })?;
    Ok((stdout, stderr))
}

fn write_password_file(
    host: &String,
    port: &String,
    database: &String,
    user: &String,
    password: &Option<String>,
    pgpass_path: &PathBuf,
) -> Result<(), anyhow::Error> {
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
                Err(anyhow!("{:?}: {}", pgpass_path, e))
            })?;
            pgpass_file.set_permissions(permissions).or_else(|e| {
                Err(anyhow!("{:?}: {}", pgpass_path, e))
            })?;
            let mut writer = BufWriter::new(pgpass_file);
            writer.write(credentials.as_bytes()).or_else(|e| {
                Err(anyhow!("{:?}: {}", pgpass_path, e))
            })?;
            writer.flush()?;
        }
    }
    Ok(())
}

