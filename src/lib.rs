use std::path::PathBuf;

pub(crate) mod db;
pub(crate) mod import;
pub(crate) mod export;
pub mod error;
pub mod reporting;

use crate::export::pbf;
use crate::import::dump;
use crate::error::osm_error;
use crate::db::pg::{load, dump};
use crate::reporting::stopwatch::StopWatch;

pub fn import(
    input_path: PathBuf,
    input_format: String,
    template_path: PathBuf,
    template_mapping_path: PathBuf,
    output_path: PathBuf,
    compression_level: i8,
    jobs: i16,
    host: String,
    port: String,
    user: String,
    password: Option<String>,
    var_lib_path: &PathBuf,
    var_log_path: &PathBuf
) -> Result<(), osm_error::GenericError> {
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    dump::generate(
        input_path,
        input_format,
        &template_path,
        &template_mapping_path,
        &output_path,
        compression_level)?;
    load(jobs, host, port, user, password, &output_path, var_lib_path, var_log_path)?;
    log::info!("Osm import time: {}", stopwatch);
    Ok(())
}

pub fn export(
    dump_path: PathBuf,
    output_path: PathBuf,
    output_format: String,
    compression_level: i8,
    jobs: i16,
    host: String,
    port: String,
    user: String,
    password: Option<String>,
    var_lib_path: &PathBuf,
    var_log_path: &PathBuf) -> Result<(), osm_error::GenericError> {
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    dump(
        jobs,
        host,
        port,
        user,
        password,
        &dump_path,
        var_lib_path,
        var_log_path
    )?;
    pbf::generate(
        dump_path,
        output_path
    )?;
    log::info!("Osm export time: {}", stopwatch);
    Ok(())
}
