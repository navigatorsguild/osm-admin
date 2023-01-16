use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fs;
use std::ops::{AddAssign, Deref, SubAssign};
use std::path::PathBuf;

use regex::Regex;
use transient_btree_index::{BtreeConfig, BtreeIndex};
// use crate::dump::node_relations_reader::NodeRelationsReader;

use crate::dump::table_def::TableDef;
use crate::dump::table_reader::TableReader;
use crate::dump::table_record::TableRecord;
use crate::error::osm_error;
use crate::error::osm_error::GenericError;
use crate::reporting::stopwatch::StopWatch;

pub mod table_def;
pub mod table_fields;
mod table_record;
mod table_reader;
mod sql;
mod node_relations_reader;
mod node_relation;

pub(crate) struct Dump {
    path: PathBuf,
    tables: HashMap<String, TableDef>,
    user_index: BtreeIndex<i64, String>,
    changeset_user_index: BtreeIndex<i64, i64>,
}

impl Dump {
    pub(crate) fn new(dump_path: &PathBuf) -> Result<Self, GenericError> {
        let user_index = BtreeIndex::<i64, String>::with_capacity(BtreeConfig::default(), 0)?;
        let changeset_user_index = BtreeIndex::<i64, i64>::with_capacity(BtreeConfig::default(), 0)?;
        let mut dump = Dump {
            path: dump_path.clone(),
            tables: HashMap::new(),
            user_index,
            changeset_user_index,
        };

        match dump.init() {
            Ok(_) => {
                Ok(dump)
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    fn init(&mut self) -> Result<(), GenericError> {
        let toc_path = self.path.join("toc.dat");
        let toc = fs::read(toc_path)?;
        let raw_table_defs = Self::get_table_def_strings(&toc);
        let re = Regex::new("^([^ ]+) \\((.+)\\)$").unwrap();
        for raw_table_def in raw_table_defs {
            let path_ = self.path.join(&raw_table_def.1);

            let captures = re.captures(&raw_table_def.0).unwrap();
            let name = captures.get(1).unwrap().as_str();
            let fields: Vec<&str> = captures.get(2).unwrap().as_str().split(", ").collect();
            self.tables.insert(
                name.to_string(),
                TableDef::new(
                    name.to_string(),
                    path_,
                    fields.iter().map(|e| {
                        e.to_string()
                    }
                    ).collect(),
                )?,
            );
        }
        Ok(())
    }

    fn get_table_def_strings(toc: &Vec<u8>) -> Vec<(String, String)> {
        let mut result: Vec<(String, String)> = Vec::new();
        let copy = "COPY ".as_bytes();
        let from_stdin = " FROM stdin".as_bytes();
        let dotdat = ".dat".as_bytes();
        let mut i: usize = 0;
        let mut start_table_def;
        let mut end_table_def;
        let mut start_file_name;
        let mut end_file_name;
        while i < toc.len() {
            if toc[i..].starts_with(copy) {
                i.add_assign(copy.len());
                start_table_def = i;
                while i < toc.len() {
                    if toc[i..].starts_with(from_stdin) {
                        end_table_def = i;
                        i.add_assign(from_stdin.len());
                        while i < toc.len() {
                            if toc[i..].starts_with(dotdat) {
                                start_file_name = i - 1;
                                i.add_assign(dotdat.len());
                                end_file_name = i;
                                while start_file_name > 0 && toc[start_file_name].is_ascii_digit() {
                                    start_file_name.sub_assign(1);
                                }
                                start_file_name.add_assign(1);
                                result.push(
                                    (
                                        String::from_utf8(toc[start_table_def..end_table_def].to_vec()).unwrap(),
                                        String::from_utf8(toc[start_file_name..end_file_name].to_vec()).unwrap(),
                                    )
                                );
                                break;
                            }
                            i.add_assign(1);
                        }
                        break;
                    }
                    i.add_assign(1);
                }
            }
            i.add_assign(1);
        }
        result
    }

    fn index_changesets(&mut self) -> Result<(), GenericError> {
        let reader = TableReader::new(self.tables.get("public.changesets").unwrap())?;
        for record in reader {
            if let TableRecord::Changeset { id, user_id, ..} = record {
                self.changeset_user_index.insert(id, user_id).unwrap();
            } else {
                return Err(osm_error::GenericError::from(osm_error::OsmError { message: format!("Not a changeset record") }))
            }
        }
        Ok(())
    }

    fn index_users(&mut self) -> Result<(), GenericError> {
        let reader = TableReader::new(self.tables.get("public.users").unwrap())?;
        for record in reader {
            if let TableRecord::User { id, display_name, .. } = record{
                self.user_index.insert(id, display_name).unwrap();
            } else {
                return Err(osm_error::GenericError::from(osm_error::OsmError { message: format!("Not a user record") }))
            }
        }
        Ok(())
    }

    // fn write_nodes(&mut self) -> Result<(), GenericError> {
    //     let nodes_def = self.tables.get("public.nodes").unwrap();
    //     let node_tags_def = self.tables.get("public.node_tags").unwrap();
    //     let reader = NodeRelationsReader::new(nodes_def, node_tags_def)?;
    //     for node_relation in reader {
    //         println!("{:?}", node_relation)
    //     }
    //
    //     // for record in reader {
    //     //     if let TableRecord::Node { .. } = record{
    //     //     } else {
    //     //         return Err(osm_error::GenericError::from(osm_error::OsmError { message: format!("Not a node record") }))
    //     //     }
    //     // }
    //     //
    //     // let reader = TableReader::new(self.tables.get("public.node_tags").unwrap())?;
    //     // for record in reader {
    //     //     if let TableRecord::NodeTag { .. } = record{
    //     //     } else {
    //     //         return Err(osm_error::GenericError::from(osm_error::OsmError { message: format!("Not a node record") }))
    //     //     }
    //     // }
    //     Ok(())
    // }

    pub(crate) fn to_pbf(&mut self, output_path: &PathBuf) -> Result<(), GenericError> {
        log::info!("Start generating PBF: {}", output_path.to_str().unwrap());
        let mut stopwatch = StopWatch::new();
        stopwatch.start();
        self.index_changesets()?;
        self.index_users()?;

        // self.write_nodes()?;

        // write nodes
        // write ways
        // write relations
        log::info!("osm.pbf generation time: {}", stopwatch);
        Ok(())
    }

}
