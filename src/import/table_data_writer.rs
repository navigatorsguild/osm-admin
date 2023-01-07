use std::borrow::{Borrow, BorrowMut};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use crate::error::osm_error::GenericError;

pub(crate) struct TableDataWriter {
    table_name: String,
    file_name: String,
    file_path: PathBuf,
    writer: BufWriter<File>,
}

impl TableDataWriter {
    pub(crate) fn new(table_name: String, file_name: String, output_path: &PathBuf) -> Result<TableDataWriter, GenericError> {
        let file_path = output_path.join(&file_name);
        let writer = BufWriter::new(File::create(&file_path).unwrap());
        Ok(TableDataWriter {
            table_name,
            file_name,
            file_path,
            writer,
        })
    }

    pub(crate) fn close(&mut self) {
        self.writer.write_all("\\.\n".as_bytes()).expect(format!("Problem writing table data footer: {:?}", self.file_path).as_str());
        self.writer.flush().expect(format!("Problem flushing table data file {:?}", self.file_path).as_str());
    }

    pub(crate) fn get_writer(&mut self) -> &mut BufWriter<File> {
        self.writer.borrow_mut()
    }
    pub(crate) fn get_table_name(&self) -> &str {
        self.table_name.borrow()
    }

    pub(crate) fn get_file_name(&self) -> &str {
        self.file_name.borrow()
    }
}
