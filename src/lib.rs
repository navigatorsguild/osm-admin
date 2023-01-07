use std::path::PathBuf;

pub(crate) mod import;
pub mod error;
pub mod reporting;

use import::dump::generate;
use error::osm_error;
use crate::import::db::load;
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
    generate(
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

