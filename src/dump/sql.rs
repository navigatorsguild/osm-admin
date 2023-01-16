use chrono::{NaiveDateTime, ParseError, ParseResult};
use num_format::Locale::fa;
use crate::error::osm_error;
use crate::error::osm_error::GenericError;

pub(crate) fn parse_sql_time(s: &str) -> Result<NaiveDateTime, ParseError>{
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")
}

pub(crate) fn parse_sql_bool(s: &str) -> Result<bool, GenericError> {
    match s {
        "t" => {Ok(true)}
        "f" => {Ok(false)}
        _ => {
            Err(osm_error::GenericError::from(osm_error::OsmError { message: format!("Wrong boolean literal: {}", s) }))
        }
    }
}