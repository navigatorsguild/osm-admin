use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use clap::builder::Str;
use num_format::Locale::se;
use postgres::types::IsNull::No;

use crate::dump::changeset_record::ChangesetRecord;
use crate::dump::node_record::NodeRecord;
use crate::dump::node_tag_record::NodeTagRecord;
use crate::dump::sql::{parse_sql_bool, parse_sql_time};
use crate::dump::table_def::TableDef;
use crate::dump::table_fields::TableFields;
use crate::dump::table_record::{RelationMemberType, TableRecord};
use crate::dump::user_record::{FormatEnum, UserRecord, UserStatus};
use crate::error::osm_error;
use crate::error::osm_error::GenericError;

struct RecordBuilder {
    f: fn(&String, &TableFields) -> Option<TableRecord>,
    fields_variant: TableFields,
}

impl RecordBuilder {
    fn build(&self, line: &String) -> Option<TableRecord> {
        (self.f)(line, &self.fields_variant)
    }
}

#[derive(Clone)]
pub(crate) struct TableReader {
    table_def: TableDef,
}

impl TableReader {
    pub(crate) fn new(table_def: &TableDef) -> Result<TableReader, GenericError> {
        Ok(
            TableReader {
                table_def: table_def.clone(),
            }
        )
    }

    fn create_record_builder(&self) -> Result<RecordBuilder, GenericError> {
        match self.table_def.name.as_str() {
            "public.nodes" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_node,
                        fields_variant: self.table_def.fields.clone(),
                    }
                )
            }
            "public.node_tags" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_node_tag,
                        fields_variant: self.table_def.fields.clone(),
                    }
                )
            }
            "public.ways" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_way,
                        fields_variant: self.table_def.fields.clone(),
                    }
                )
            }
            "public.way_nodes" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_way_node,
                        fields_variant: self.table_def.fields.clone(),
                    }
                )
            }
            "public.way_tags" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_way_tag,
                        fields_variant: self.table_def.fields.clone(),
                    }
                )
            }
            "public.relations" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_relation,
                        fields_variant: self.table_def.fields.clone(),
                    }
                )
            }
            "public.relation_members" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_relation_member,
                        fields_variant: self.table_def.fields.clone(),
                    }
                )
            }
            "public.relation_tags" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_relation_tag,
                        fields_variant: self.table_def.fields.clone(),
                    }
                )
            }
            "public.changesets" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_changeset,
                        fields_variant: self.table_def.fields.clone(),
                    }
                )
            }
            "public.users" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_user,
                        fields_variant: self.table_def.fields.clone(),
                    }
                )
            }
            _ => {
                Err(osm_error::GenericError::from(osm_error::OsmError { message: format!("Unknown record type: {}", self.table_def.name) }))
            }
        }
    }

    fn create_node(line: &String, fields: &TableFields) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match fields {
            TableFields::Nodes { node_id, latitude, longitude, changeset_id, visible, timestamp, tile, version, redaction_id } => {
                Some(
                    TableRecord::Node {
                        node_record: NodeRecord {
                            node_id: i64::from_str(columns[*node_id]).unwrap(),
                            latitude: i32::from_str(columns[*latitude]).unwrap(),
                            longitude: i32::from_str(columns[*longitude]).unwrap(),
                            changeset_id: i64::from_str(columns[*changeset_id]).unwrap(),
                            visible: parse_sql_bool(columns[*visible]).unwrap(),
                            timestamp: parse_sql_time(columns[*timestamp]).unwrap(),
                            tile: i64::from_str(columns[*tile]).unwrap(),
                            version: i64::from_str(columns[*version]).unwrap(),
                            redaction_id: i32::from_str(columns[*redaction_id]).ok(),
                        }
                    }
                )
            }
            _ => {
                None
            }
        }
    }

    fn create_node_tag(line: &String, fields: &TableFields) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match fields {
            TableFields::NodeTags { node_id, version, k, v } => {
                Some(
                    TableRecord::NodeTag {
                        node_tag_record: NodeTagRecord {
                            node_id: i64::from_str(columns[*node_id]).unwrap(),
                            version: i64::from_str(columns[*version]).unwrap(),
                            k: columns[*k].to_string(),
                            v: columns[*v].to_string(),
                        }
                    }
                )
            }
            _ => {
                None
            }
        }
    }

    fn create_way(line: &String, fields: &TableFields) -> Option<TableRecord> {
        println!("[{}]", line);
        Some(
            TableRecord::Way {
                way_id: 0,
                changeset_id: 0,
                timestamp: Default::default(),
                version: 0,
                visible: false,
                redaction_id: 0,
            }
        )
    }

    fn create_way_node(line: &String, fields: &TableFields) -> Option<TableRecord> {
        println!("[{}]", line);
        Some(
            TableRecord::WayNode {
                way_id: 0,
                node_id: 0,
                version: 0,
                sequence_id: 0,
            }
        )
    }

    fn create_way_tag(line: &String, fields: &TableFields) -> Option<TableRecord> {
        println!("[{}]", line);
        Some(
            TableRecord::WayTag {
                way_id: 0,
                k: "".to_string(),
                v: "".to_string(),
                version: 0,
            }
        )
    }

    fn create_relation(line: &String, fields: &TableFields) -> Option<TableRecord> {
        println!("[{}]", line);
        Some(
            TableRecord::Relation {
                relation_id: 0,
                changeset_id: 0,
                timestamp: Default::default(),
                version: 0,
                visible: false,
                redaction_id: 0,
            }
        )
    }

    fn create_relation_member(line: &String, fields: &TableFields) -> Option<TableRecord> {
        println!("[{}]", line);
        Some(
            TableRecord::RelationMember {
                relation_id: 0,
                member_type: RelationMemberType::Node,
                member_id: 0,
                member_role: "".to_string(),
                version: 0,
                sequence_id: 0,
            }
        )
    }

    fn create_relation_tag(line: &String, fields: &TableFields) -> Option<TableRecord> {
        println!("[{}]", line);
        Some(
            TableRecord::RelationTag {
                relation_id: 0,
                k: "".to_string(),
                v: "".to_string(),
                version: 0,
            }
        )
    }

    fn create_changeset(line: &String, fields: &TableFields) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match fields {
            TableFields::Changesets { id, user_id, created_at, min_lat, max_lat, min_lon, max_lon, closed_at, num_changes } => {
                Some(
                    TableRecord::Changeset {
                        changeset_record: ChangesetRecord {
                            id: i64::from_str(columns[*id]).unwrap(),
                            user_id: i64::from_str(columns[*user_id]).unwrap(),
                            created_at: parse_sql_time(columns[*created_at]).unwrap(),
                            min_lat: i32::from_str(columns[*min_lat]).unwrap(),
                            max_lat: i32::from_str(columns[*max_lat]).unwrap(),
                            min_lon: i32::from_str(columns[*min_lon]).unwrap(),
                            max_lon: i32::from_str(columns[*max_lon]).unwrap(),
                            closed_at: parse_sql_time(columns[*closed_at]).unwrap(),
                            num_changes: i32::from_str(columns[*num_changes]).unwrap(),
                        }
                    }
                )
            }
            _ => {
                None
            }
        }
    }

    fn create_user(line: &String, fields: &TableFields) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match fields {
            TableFields::Users { email, id, pass_crypt, creation_time, display_name, data_public, description, home_lat, home_lon, home_zoom, pass_salt, email_valid, new_email, creation_ip, languages, status, terms_agreed, consider_pd, auth_uid, preferred_editor, terms_seen, description_format, changesets_count, traces_count, diary_entries_count, image_use_gravatar, auth_provider, home_tile, tou_agreed, } => {
                Some(
                    TableRecord::User {
                        user_record: UserRecord {
                            email: columns[*email].to_string(),
                            id: i64::from_str(columns[*id]).unwrap(),
                            pass_crypt: columns[*pass_crypt].to_string(),
                            creation_time: parse_sql_time(columns[*creation_time]).unwrap(),
                            display_name: columns[*display_name].to_string(),
                            data_public: parse_sql_bool(columns[*data_public]).unwrap(),
                            description: columns[*description].to_string(),
                            home_lat: f64::from_str(columns[*home_lat]).unwrap(),
                            home_lon: f64::from_str(columns[*home_lon]).unwrap(),
                            home_zoom: i16::from_str(columns[*home_zoom]).unwrap(),
                            pass_salt: columns[*pass_salt].to_string(),
                            email_valid: parse_sql_bool(columns[*email_valid]).unwrap(),
                            new_email: columns[*new_email].to_string(),
                            creation_ip: columns[*creation_ip].to_string(),
                            languages: columns[*languages].to_string(),
                            status: UserStatus::try_from(columns[*status]).unwrap(),
                            terms_agreed: parse_sql_time(columns[*terms_agreed]).ok(),
                            consider_pd: parse_sql_bool(columns[*consider_pd]).unwrap(),
                            auth_uid: columns[*auth_uid].to_string(),
                            preferred_editor: columns[*preferred_editor].to_string(),
                            terms_seen: parse_sql_bool(columns[*terms_seen]).unwrap(),
                            description_format: FormatEnum::try_from(columns[*description_format]).unwrap(),
                            changesets_count: i32::from_str(columns[*changesets_count]).unwrap(),
                            traces_count: i32::from_str(columns[*traces_count]).unwrap(),
                            diary_entries_count: i32::from_str(columns[*diary_entries_count]).unwrap(),
                            image_use_gravatar: parse_sql_bool(columns[*image_use_gravatar]).unwrap(),
                            auth_provider: columns[*auth_provider].to_string(),
                            home_tile: i64::from_str(columns[*home_tile]).ok(),
                            tou_agreed: parse_sql_time(columns[*tou_agreed]).ok(),
                        }
                    }
                )
            }
            _ => {
                None
            }
        }
    }
}

impl IntoIterator for TableReader {
    type Item = TableRecord;
    type IntoIter = TableIterator;

    fn into_iter(self) -> Self::IntoIter {
        TableIterator::new(&self).unwrap()
    }
}

pub(crate) struct TableIterator {
    reader: BufReader<File>,
    record_builder: RecordBuilder,
}

impl TableIterator {
    pub(crate) fn new(table_reader: &TableReader) -> Result<TableIterator, GenericError> {
        log::info!("Create iterator for {} from {:?}", table_reader.table_def.name, table_reader.table_def.path);
        let f = File::open(&table_reader.table_def.path)?;
        let mut reader = BufReader::new(f);
        let record_builder = table_reader.create_record_builder()?;
        Ok(
            TableIterator {
                reader,
                record_builder,
            }
        )
    }
}

impl Iterator for TableIterator {
    type Item = TableRecord;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::with_capacity(2048);
        match self.reader.read_line(&mut line) {
            Ok(0) => {
                None
            }
            Ok(l) => {
                match line.starts_with("\\.") || line.is_empty() || line.starts_with("\n"){
                    false => {
                        self.record_builder.build(&line)
                    }
                    true => {
                        None
                    }
                }
            }
            Err(_) => {
                None
            }
        }
    }
}