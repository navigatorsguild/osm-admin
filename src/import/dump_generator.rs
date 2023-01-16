use std::borrow::Borrow;
use std::fs;
use std::io::Write;
use std::ops::{Shl, Shr};
use std::path::PathBuf;

use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};
use escape_string;
use json;
use json::JsonValue;
use num_format::{Buffer, Locale};
use osmpbf::{DenseNode, Element, ElementReader, Node, Relation, RelMemberType, Way};

use crate::error::osm_error;
use crate::error::osm_error::OsmError;
use crate::import::table_data_writers::TableDataWriters;
use crate::reporting::disk_usage::{disk_usage, to_human};
use crate::reporting::stopwatch::StopWatch;

pub fn generate(
    input_path: PathBuf,
    input_format: String,
    template_path: &PathBuf,
    template_mapping_path: &PathBuf,
    output_path: &PathBuf,
    compression_level: i8) -> Result<(), osm_error::GenericError> {
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    log::info!("Generate Postgresql dump, input: {:?}, input format: {}, output: {:?}, compression level: {}",
        input_path,
        input_format,
        output_path,
        compression_level,
    );
    // TODO: use compression level
    create_result_dir(template_path, &output_path)?;
    let template_mapping = load_template_mapping(&template_mapping_path)?;
    let mut writers = TableDataWriters::new(template_mapping, &output_path)?;
    writers.log();
    write_data_files(input_path, &mut writers)?;
    writers.close();
    log::info!("Postgresql dump time (hours): {}", stopwatch);
    log::info!("Postgresql dump size: {}", to_human(disk_usage(output_path).unwrap()));
    Ok(())
}

fn write_data_files(input_path: PathBuf, writers: &mut TableDataWriters) -> Result<(), osm_error::GenericError> {
    let mut nodes = 0_i64;
    let mut dense_nodes = 0_i64;
    let mut ways = 0_i64;
    let mut relations = 0_i64;
    let flush_threshold = 10_000_000;
    let reader = ElementReader::from_path(&input_path).or_else(|e| {
        Err(OsmError::new(format!("{:?}: {}", input_path, e)))
    })?;
    log::info!("Started writing table data files");
    reader.for_each(|element| match element {
        Element::Node(n) => {
            write_node(writers, n).unwrap();
            nodes = nodes + 1;
            if nodes % flush_threshold == 0 {
                writers.flush_buffers().unwrap();
                log::info!("Wrote {} nodes", nodes);
            }
        }
        Element::DenseNode(n) => {
            write_dense_node(writers, n).unwrap();
            dense_nodes = dense_nodes + 1;
            if dense_nodes % flush_threshold == 0 {
                writers.flush_buffers().unwrap();
                log::info!("Wrote {} dense nodes", dense_nodes);
            }
        }
        Element::Way(w) => {
            write_way(writers, w).unwrap();
            ways = ways + 1;
            if ways % flush_threshold == 0 {
                writers.flush_buffers().unwrap();
                log::info!("Wrote {} ways", ways);
            }
        }
        Element::Relation(r) => {
            write_relation(writers, r).unwrap();
            relations = relations + 1;
            if relations % flush_threshold == 0 {
                writers.flush_buffers().unwrap();
                log::info!("Wrote {} relations", relations);
            }
        }
    })?;

    writers.flush_buffers().unwrap();
    write_changesets(writers)?;
    write_users(writers)?;
    let mut formatted_nodes = Buffer::default();
    formatted_nodes.write_formatted(&nodes, &Locale::en);

    let mut formatted_dense_nodes = Buffer::default();
    formatted_dense_nodes.write_formatted(&dense_nodes, &Locale::en);

    let mut formatted_ways = Buffer::default();
    formatted_ways.write_formatted(&ways, &Locale::en);

    let mut formatted_relations = Buffer::default();
    formatted_relations.write_formatted(&relations, &Locale::en);

    let mut formatted_changesets = Buffer::default();
    formatted_changesets.write_formatted(writers.changeset_user_index.len().borrow(), &Locale::en);

    let mut formatted_users = Buffer::default();
    formatted_users.write_formatted(writers.user_index.len().borrow(), &Locale::en);

    log::info!("Processed {:?}, nodes: {}, dense nodes: {}, ways: {}, relations: {}, changesets: {}, users: {}",
        input_path,
        formatted_nodes.as_str(),
        formatted_dense_nodes.as_str(),
        formatted_ways.as_str(),
        formatted_relations.as_str(),
        formatted_changesets.as_str(),
        formatted_users.as_str(),
    );

    log::info!("Finished writing table data files");
    Ok(())
}

fn write_changesets(writers: &mut TableDataWriters) -> Result<(), osm_error::GenericError> {
    for element in writers.changeset_user_index.range(..)? {
        let (changeset_id, user_id) = element?;
        // public.changeset_tags (changeset_id, k, v)
        // template context: 4221.dat
        let line = format!("{}\t{}\t{}\n",
                           changeset_id,
                           "created_by",
                           format!("osm-admin {}", option_env!("CARGO_PKG_VERSION").unwrap()),
        );
        writers.changeset_tags.get_writer().write(line.as_bytes())?;

        let line = format!("{}\t{}\t{}\n",
                           changeset_id,
                           "replication",
                           "true"
        );
        writers.changeset_tags.get_writer().write(line.as_bytes())?;

        // public.changesets (id, user_id, created_at, min_lat, max_lat, min_lon, max_lon, closed_at, num_changes)
        // template context: 4222.dat
        let t = chrono::offset::Utc::now();
        let line = format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                           changeset_id,
                           user_id,
                           to_sql_time_micro(t.timestamp_nanos()),
                           -900000000,
                           900000000,
                           -1800000000,
                           1800000000,
                           to_sql_time_micro(t.timestamp_nanos()),
                           0
        );
        writers.changesets.get_writer().write(line.as_bytes())?;
    }

    Ok(())
}

fn write_users(writers: &mut TableDataWriters) -> Result<(), osm_error::GenericError> {
    // public.users (email, id, pass_crypt, creation_time, display_name, data_public, description, home_lat, home_lon, home_zoom, pass_salt, email_valid, new_email, creation_ip, languages, status, terms_agreed, consider_pd, auth_uid, preferred_editor, terms_seen, description_format, changesets_count, traces_count, diary_entries_count, image_use_gravatar, auth_provider, home_tile, tou_agreed)
    // template context: 4290.dat
    for element in writers.user_index.range(..)? {
        let (user_id, user_name) = element?;

        let t = chrono::offset::Utc::now();
        let line = format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                           format!("osm-admin-user-{}@example.com", user_id),
                           user_id,
                           "00000000000000000000000000000000",
                           to_sql_time_micro(t.timestamp_nanos()),
                           user_name,
                           boolean_prefix(true),
                           user_name,
                           0,
                           0,
                           3,
                           "00000000",
                           boolean_prefix(false),
                           "\\N",
                           "\\N",
                           "\\N",
                           "pending",
                           "\\N",
                           boolean_prefix(false),
                           "\\N",
                           "\\N",
                           boolean_prefix(false),
                           "markdown",
                           0,
                           0,
                           0,
                           boolean_prefix(false),
                           "\\N",
                           "\\N",
                           "\\N",
        );
        writers.users.get_writer().write(line.as_bytes())?;
    }

    Ok(())
}

fn write_way(writers: &mut TableDataWriters, w: Way) -> Result<(), osm_error::GenericError> {
    let info = w.info();
    let user_id = info.uid().unwrap() as i64;
    let user = info.user().unwrap().unwrap_or("unknown-by-osm-admin");
    let changeset = info.changeset().unwrap();

    writers.user_index_buffer.insert(user_id, user.to_string());
    writers.changeset_user_index_buffer.insert(changeset, user_id);

    w.refs().enumerate().for_each(|(sequence_id, node_id)| {
        // public.current_way_nodes (way_id, node_id, sequence_id)
        // template context: 4234.dat
        let line = format!("{}\t{}\t{}\n",
                           w.id(),
                           node_id,
                           sequence_id + 1
        );
        writers.current_way_nodes.get_writer().write(line.as_bytes()).unwrap();

        // public.way_nodes (way_id, node_id, version, sequence_id)
        // template context: 4292.dat
        let line = format!("{}\t{}\t{}\t{}\n",
                           w.id(),
                           node_id,
                           w.info().version().unwrap(),
                           sequence_id + 1
        );
        writers.way_nodes.get_writer().write(line.as_bytes()).unwrap();
    });

    w.tags().for_each(|(k, v)| {
        // public.current_way_tags (way_id, k, v)
        // template context: 4235.dat
        let escaped_tag = escape_string::escape(v);
        let line = format!("{}\t{}\t{}\n",
                           w.id(),
                           k,
                           escaped_tag,
        );
        writers.current_way_tags.get_writer().write(line.as_bytes()).unwrap();

        // public.way_tags (way_id, k, v, version)
        // template context: 4293.dat
        let line = format!("{}\t{}\t{}\t{}\n",
                           w.id(),
                           k,
                           escaped_tag,
                           w.info().version().unwrap()
        );
        writers.way_tags.get_writer().write(line.as_bytes()).unwrap();
    });


    // public.current_ways (id, changeset_id, "timestamp", visible, version)
    // template context: 4236.dat
    let line = format!("{}\t{}\t{}\t{}\t{}\n",
                       w.id(),
                       w.info().changeset().unwrap(),
                       to_sql_time(w.info().milli_timestamp().unwrap()),
                       boolean_prefix(w.info().visible()),
                       w.info().version().unwrap()
    );
    writers.current_ways.get_writer().write(line.as_bytes()).unwrap();

    // public.ways (way_id, changeset_id, "timestamp", version, visible, redaction_id)
    // template context: 4294.dat"
    let line = format!("{}\t{}\t{}\t{}\t{}\t\\N\n",
                       w.id(),
                       w.info().changeset().unwrap(),
                       to_sql_time(w.info().milli_timestamp().unwrap()),
                       w.info().version().unwrap(),
                       boolean_prefix(w.info().visible())
    );
    writers.ways.get_writer().write(line.as_bytes()).unwrap();

    Ok(())
}

fn write_relation(writers: &mut TableDataWriters, r: Relation) -> Result<(), osm_error::GenericError> {
    let info = r.info();
    let user_id = info.uid().unwrap() as i64;
    let user = info.user().unwrap().unwrap_or("unknown-by-osm-admin");
    let changeset = info.changeset().unwrap();

    writers.user_index_buffer.insert(user_id, user.to_string());
    writers.changeset_user_index_buffer.insert(changeset, user_id);
    r.members().enumerate().for_each(|(sequence, member)| {
        let member_type = match member.member_type {
            RelMemberType::Node => { "Node" }
            RelMemberType::Way => { "Way" }
            RelMemberType::Relation => { "Relation" }
        };

        // public.current_relation_members (relation_id, member_type, member_id, member_role, sequence_id)
        // template context: 4230.dat
        let escaped_role = escape_string::escape(member.role().unwrap_or(""));
        let line = format!("{}\t{}\t{}\t{}\t{}\n",
                           r.id(),
                           member_type,
                           member.member_id,
                           escaped_role,
                           sequence + 1,
        );
        writers.current_relation_members.get_writer().write(line.as_bytes()).unwrap();

        // public.relation_members (relation_id, member_type, member_id, member_role, version, sequence_id)
        // template context: 4277.dat
        let line = format!("{}\t{}\t{}\t{}\t{}\t{}\n",
                           r.id(),
                           member_type,
                           member.member_id,
                           escaped_role,
                           r.info().version().unwrap(),
                           sequence + 1,
        );
        writers.relation_members.get_writer().write(line.as_bytes()).unwrap();
    });

    r.tags().for_each(|(k, v)| {
        // public.current_relation_tags (relation_id, k, v)
        // template context: 4231.dat
        let escaped_tag = escape_string::escape(&v);
        let line = format!("{}\t{}\t{}\n",
                           r.id(),
                           k,
                           escaped_tag,
        );
        writers.current_relation_tags.get_writer().write(line.as_bytes()).unwrap();

        // public.relation_tags (relation_id, k, v, version)
        // template context: 4278.dat
        let line = format!("{}\t{}\t{}\t{}\n",
                           r.id(),
                           k,
                           escaped_tag,
                           r.info().version().unwrap()
        );
        writers.relation_tags.get_writer().write(line.as_bytes()).unwrap();
    });


    // public.current_relations (id, changeset_id, "timestamp", visible, version)
    // template context: 4232.dat
    let line = format!("{}\t{}\t{}\t{}\t{}\n",
                       r.id(),
                       r.info().changeset().unwrap(),
                       to_sql_time(r.info().milli_timestamp().unwrap()),
                       boolean_prefix(r.info().visible()),
                       r.info().version().unwrap()
    );
    writers.current_relations.get_writer().write(line.as_bytes()).unwrap();

    // public.relations (relation_id, changeset_id, "timestamp", version, visible, redaction_id)
    // template context: 4279.dat
    let line = format!("{}\t{}\t{}\t{}\t{}\t\\N\n",
                       r.id(),
                       r.info().changeset().unwrap(),
                       to_sql_time(r.info().milli_timestamp().unwrap()),
                       r.info().version().unwrap(),
                       boolean_prefix(r.info().visible())
    );
    writers.relations.get_writer().write(line.as_bytes()).unwrap();

    Ok(())
}

fn boolean_prefix(v: bool) -> char {
    match v {
        true => { 't' }
        false => { 'f' }
    }
}

fn to_sql_time(t: i64) -> String {
    let datetime = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(t / 1000, 0).unwrap(), Utc);
    let sql_time: String = datetime.to_rfc3339_opts(SecondsFormat::Secs, true);
    sql_time.replace("T", " ").replace("Z", "")
}

fn to_sql_time_micro(t: i64) -> String {
    let datetime = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(t / (1e9 as i64), (t % (1e9 as i64)) as u32).unwrap(), Utc);
    let sql_time: String = datetime.to_rfc3339_opts(SecondsFormat::Micros, true);
    sql_time.replace("T", " ").replace("Z", "")
}


fn calculate_tile(lat: f64, lon: f64) -> u64 {
    let x = ((lon + 180.0) * 65535.0 / 360.0).round() as u64;
    let y = ((lat + 90.0) * 65535.0 / 180.0).round() as u64;


    let mut tile = 0_u64;

    for i in (0..16).rev() {
        tile = tile.shl(1) | (x.shr(i) & 1_u64);
        tile = tile.shl(1) | (y.shr(i) & 1_u64);
    }
    tile
}

fn write_node(writers: &mut TableDataWriters, n: Node) -> Result<(), osm_error::GenericError> {
    let info = n.info();
    let user_id = info.uid().unwrap() as i64;
    let user = info.user().unwrap().unwrap_or("unknown-by-osm-admin");
    let changeset = info.changeset().unwrap();

    writers.user_index_buffer.insert(user_id, user.to_string());
    writers.changeset_user_index_buffer.insert(changeset, user_id);

    // public.current_nodes (id, latitude, longitude, changeset_id, visible, "timestamp", tile, version)
    // template context: 4228.dat
    let line = format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                       n.id(),
                       n.decimicro_lat(),
                       n.decimicro_lon(),
                       changeset,
                       boolean_prefix(info.visible()),
                       to_sql_time(info.milli_timestamp().unwrap()),
                       calculate_tile(n.lat(), n.lon()),
                       info.version().unwrap()
    );

    writers.current_nodes.get_writer().write(line.as_bytes())?;
    writers.current_nodes.get_writer().write("\n".as_bytes())?;

    // public.nodes (node_id, latitude, longitude, changeset_id, visible, "timestamp", tile, version, redaction_id)
    // template context: 4260.dat
    writers.nodes.get_writer().write(line.as_bytes())?;
    writers.nodes.get_writer().write("\t\\N\n".as_bytes())?;

    n.tags().for_each(|(k, v)| {
        // public.node_tags (node_id, version, k, v)
        // template context: 4259.dat
        let escaped_tag = escape_string::escape(&v);
        let line = format!("{}\t{}\t{}\t{}\n",
                           n.id(),
                           info.version().unwrap(),
                           k,
                           escaped_tag,
        );
        writers.node_tags.get_writer().write(line.as_bytes()).unwrap();

        // public.current_node_tags (node_id, k, v)
        // template context: 4227.dat
        let line = format!("{}\t{}\t{}\n",
                           n.id(),
                           k,
                           escaped_tag,
        );
        writers.current_node_tags.get_writer().write(line.as_bytes()).unwrap();
    });
    Ok(())
}

fn write_dense_node(writers: &mut TableDataWriters, n: DenseNode) -> Result<(), osm_error::GenericError> {
    let info = n.info().unwrap();
    let user_id = info.uid() as i64;
    let user = info.user().unwrap_or("unknown-by-osm-admin");
    let changeset = info.changeset();
    writers.user_index_buffer.insert(user_id, user.to_string());
    writers.changeset_user_index_buffer.insert(changeset, user_id);

    // public.current_nodes (id, latitude, longitude, changeset_id, visible, "timestamp", tile, version)
    // template context: 4228.dat
    let line = format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                       n.id,
                       n.decimicro_lat(),
                       n.decimicro_lon(),
                       changeset,
                       boolean_prefix(info.visible()),
                       to_sql_time(info.milli_timestamp()),
                       calculate_tile(n.lat(), n.lon()),
                       info.version()
    );

    writers.current_nodes.get_writer().write(line.as_bytes())?;
    writers.current_nodes.get_writer().write("\n".as_bytes())?;


    // public.nodes (node_id, latitude, longitude, changeset_id, visible, "timestamp", tile, version, redaction_id)
    // template context: 4260.dat

    writers.nodes.get_writer().write(line.as_bytes())?;
    writers.nodes.get_writer().write("\t\\N\n".as_bytes())?;

    n.tags().for_each(|(k, v)| {
        // public.node_tags (node_id, version, k, v)
        // template context: 4259.dat
        let escaped_tag = escape_string::escape(&v);
        let line = format!("{}\t{}\t{}\t{}\n",
                           n.id,
                           info.version(),
                           k,
                           escaped_tag,
        );
        writers.node_tags.get_writer().write(line.as_bytes()).unwrap();

        // public.current_node_tags (node_id, k, v)
        // template context: 4227.dat
        let line = format!("{}\t{}\t{}\n",
                           n.id,
                           k,
                           escaped_tag,
        );
        writers.current_node_tags.get_writer().write(line.as_bytes()).unwrap();
    });
    Ok(())
}

fn load_template_mapping(template_mapping_path: &PathBuf) -> Result<JsonValue, osm_error::GenericError> {
    let template_mapping_string = fs::read_to_string(template_mapping_path).or_else(|e| {
        Err(OsmError::new(format!("{:?}: {}", template_mapping_path, e)))
    })?;

    let template_mapping = json::parse(template_mapping_string.as_str())?;
    if template_mapping.is_object() {
        Ok(template_mapping)
    } else {
        Err(osm_error::GenericError::from(osm_error::OsmError { message: "Template mapping must be a JSON object".to_string()}))
    }
}

fn create_result_dir(template_path: &PathBuf, output_path: &PathBuf) -> Result<(), osm_error::GenericError> {
    fs::create_dir_all(&output_path).or_else(|e| {
        Err(OsmError::new(format!("{:?}: {}", output_path, e)))
    })?;
    let paths = std::fs::read_dir(template_path).or_else(|e| {
        Err(OsmError::new(format!("{:?}: {}", template_path, e)))
    })?;
    for path_result in paths {
        let path = path_result.unwrap().path();
        let dest = output_path.join(path.file_name().unwrap());
        fs::copy(&path, &dest).or_else(|e| {
            Err(OsmError::new(format!("copy {:?} -> {:?}: {}", path, dest, e)))
        })?;
    }
    Ok(())
}
