use std::fs;
use std::io::Write;
use std::ops::{Add, AddAssign, SubAssign};
use std::path::PathBuf;

use num_format::Locale::{el, ta, to};

use crate::error::osm_error;
use crate::reporting::stopwatch::StopWatch;

pub fn generate(
    dump_path: &PathBuf,
    output_path: &PathBuf,
) -> Result<(), osm_error::GenericError> {
    let mut stopwatch = StopWatch::new();
    // let toc_path = dump_path.join("toc.dat");
    // let toc = fs::read(toc_dat_path)?;


    // let toc = Toc::new(toc_path);
    // let table_defs = toc.parse();
    // let users = index_users(table_defs.users);
    // let changesets = index_changesets(table_defs.changesets);
    // let writer: OsmWriter = PbfWriter::new(users, changesets);
    // writer.write_nodes(table_defs.nodes, table_defs.node_tags);
    // writer.write_ways(table_defs.ways, table_defs.way_nodes, table_defs.way_tags);
    // writer.write_relations(table_defs.relations, table_defs.relation_members, table_defs.relation_tags);

    // let raw_table_defs = get_table_def_strings(&toc);

    // println!("{:?}", raw_table_defs);
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
