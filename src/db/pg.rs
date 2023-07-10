use std::fs::File;
use std::fs::Permissions;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use anyhow::anyhow;
use benchmark_rs::stopwatch::StopWatch;
use postgres::{Client, NoTls};

pub(crate) fn count_objects(
    host: String,
    port: String,
    database: String,
    user: String,
    password: Option<String>,
) -> Result<(i64, i64, i64), anyhow::Error> {
    let connection_string = match password {
        None => {
            let pgpass_path = PathBuf::from("/root/.pgpass");
            let pgpass_password_opt = read_password_file(&host, &port, &database, &user, &pgpass_path)?;
            match pgpass_password_opt {
                None => {
                    log::info!("No credentials and no correct PGPASSFILE entry provided. Will succeed on trust connections");
                    format!("host={host} port={port} user={user} dbname=openstreetmap")
                }
                Some(pgpass_password) => {
                    log::info!("Using password from PGPASSFILE");
                    format!("host={host} port={port} user={user} password={pgpass_password} dbname=openstreetmap")
                }
            }
        }
        Some(password) => {
            format!("host={host} port={port} user={user} password={password} dbname=openstreetmap")
        }
    };
    let mut client = Client::connect(
        connection_string.as_str(),
        NoTls,
    )?;
    let rows = client.query(format!("select relname, n_live_tup from pg_stat_user_tables where schemaname = 'public' AND (relname = 'nodes' OR relname = 'ways' OR relname = 'relations');").as_str(), &[])?;
    let mut result = (0, 0, 0);
    for row in rows {
        let relname: String = row.get("relname");
        let n_live_tup: i64 = row.get("n_live_tup");
        match relname.as_str() {
            "nodes" => { result.0 = n_live_tup }
            "ways" => { result.1 = n_live_tup }
            "relations" => { result.2 = n_live_tup }
            _ => {}
        }
    }
    Ok(result)
}

pub(crate) fn restore(
    jobs: i16,
    host: String,
    port: String,
    database: String,
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

pub(crate) fn dump(
    jobs: i16,
    host: String,
    port: String,
    database: String,
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

    let pgpass_path = PathBuf::from("/root/.pgpass");

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
                    log::error!("Failed dumping OSM database, see pg_dump stdout at: {:?}, see stderr at: {:?}, time: {}",
                        stdout_path,
                        stderr_path,
                        stopwatch,
                    );
                    Err(anyhow!("Failed dumping OSM database"))
                }
                Some(0) => {
                    let du = benchmark_rs::disk_usage::disk_usage(&dump_path)?;
                    log::info!("Finished dumping OSM database, disk: {}, time: {}",
                        benchmark_rs::disk_usage::to_human(du),
                        stopwatch
                    );
                    Ok(())
                }
                Some(code) => {
                    log::error!("Failed dumping OSM database, error code: {}, see stdout at: {:?}, see stderr at: {:?}, time: {}",
                        code,
                        stdout_path,
                        stderr_path,
                        stopwatch,
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

fn read_password_file(
    host: &String,
    port: &String,
    database: &String,
    user: &String,
    pgpass_path: &PathBuf,
) -> Result<Option<String>, anyhow::Error> {
    if pgpass_path.exists() {
        let permissions = std::fs::metadata(pgpass_path).unwrap().permissions();
        let mode = permissions.mode() & 0o777_u32;
        if mode == 0o600 {
            log::info!("Found PGPASSFILE at: {:?}, permissions: {:#o}", pgpass_path, mode);
            let pgpass_file = BufReader::new(File::open(pgpass_path)?);

            let server_prefix = format!("{}:{}:{}:{}",
                                        host,
                                        port,
                                        database,
                                        user
            );
            let mut result = None;
            for line_result in pgpass_file.lines() {
                let line = line_result?;
                if line.starts_with(&server_prefix) {
                    let parts: Vec<String> = line.split(":").map(|s| s.to_string()).collect();
                    result = parts.last().cloned();
                }
            }
            Ok(result)
        } else {
            let error_string = format!("Found PGPASSFILE at: {:?}, wrong permissions: {:#o}. Must be 0o600", pgpass_path, mode);
            log::warn!("{}", error_string);
            Err(anyhow!(error_string))
        }
    } else {
        log::info!("No credentials and no PGPASSFILE file provided. Will succeed on trust connections");
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    // use crate::db::pg::count_objects;
    //
    // #[test]
    // fn test_count_objects() -> Result<(), anyhow::Error> {
    //     let host = "localhost".to_string();
    //     let port = "5432".to_string();
    //     let user = "openstreetmap".to_string();
    //     let password = Some("openstreetmap".to_string());
    //     let (nodes, ways, relations) = count_objects(host, port, user, password)?;
    //     println!("{nodes}, {ways}, {relations}");
    //     Ok(())
    // }
    //
    // #[test]
    // fn test_count_objects_no_password() -> Result<(), anyhow::Error> {
    //     let host = "localhost".to_string();
    //     let port = "5432".to_string();
    //     let user = "openstreetmap".to_string();
    //     let password = None;
    //     let (nodes, ways, relations) = count_objects(host, port, user, password)?;
    //     println!("{nodes}, {ways}, {relations}");
    //     Ok(())
    // }
}
