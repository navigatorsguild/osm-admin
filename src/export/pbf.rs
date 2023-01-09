use std::path::PathBuf;

use crate::error::osm_error;
use crate::reporting::stopwatch::StopWatch;

pub fn generate(
    dump_path: PathBuf,
    output_path: PathBuf,
) -> Result<(), osm_error::GenericError> {
    let mut stopwatch = StopWatch::new();
    Ok(())
}