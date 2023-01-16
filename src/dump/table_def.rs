use std::path::PathBuf;

use crate::dump::table_fields::TableFields;
use crate::error::osm_error::GenericError;

#[derive(Debug, Clone)]
pub(crate) struct TableDef {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) fields: TableFields,
}

impl TableDef {
    pub(crate) fn new(name: String, path: PathBuf, fields: Vec<String>) -> Result<TableDef, GenericError> {
        let table_def = TableDef {
            name: name.clone(),
            path,
            fields: TableFields::new(name.clone(), fields)?,
        };
        Ok(table_def)
    }

}
