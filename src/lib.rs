extern crate core;

use std::path::PathBuf;

use benchmark_rs::stopwatch::StopWatch;
use chrono::{DateTime, SecondsFormat, Utc};
use num_format::{Locale, ToFormattedString};
use osm_io::osm::apidb_dump::read::reader::Reader;
use osm_io::osm::apidb_dump::write::writer::Writer as ApiDbDumpWriter;
use osm_io::osm::model::bounding_box::BoundingBox;
use osm_io::osm::model::element::Element;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::file_info::FileInfo;
use osm_io::osm::pbf::reader::Reader as PbfReader;
use osm_io::osm::pbf::writer::Writer;

use crate::db::pg::count_objects;

pub(crate) mod db;

pub fn import(
    input_path: PathBuf,
    _input_format: String,
    output_path: PathBuf,
    jobs: i16,
    host: String,
    port: String,
    database: String,
    user: String,
    password: Option<String>,
    var_lib_path: &PathBuf,
    var_log_path: &PathBuf,
    verbose: bool,
) -> Result<(), anyhow::Error> {
    let mut stopwatch = StopWatch::new();
    stopwatch.start();

    log::info!("Start apidb dump generation");
    let pbf_reader = PbfReader::new(&input_path)?;
    let mut objects = 0 as i64;
    if verbose {
        log::info!("Verbose flag set, counting objects");
        let (nodes, ways, relations) = pbf_reader.count_objects()?;
        log::info!("Finished counting objects, time: {}", stopwatch);
        stopwatch.reset();
        stopwatch.start();
        objects = nodes + ways + relations;
        print_verbose_info(&input_path, &pbf_reader, objects, nodes, ways, relations)
    }
    let mut apidb_dump_writer = ApiDbDumpWriter::new(output_path.clone(), 0)?;
    for (i, element) in pbf_reader.elements()?.enumerate() {
        if verbose && i % 10000000 == 0 && i != 0 {
            print_progress(&output_path, &stopwatch, objects, i)?;
        }
        apidb_dump_writer.write_element(element)?;
    }
    apidb_dump_writer.close()?;
    print_progress(&output_path, &stopwatch, objects, objects as usize)?;

    log::info!("Finish apidb dump generation, time (hours): {}", stopwatch);

    stopwatch.reset();
    stopwatch.start();
    log::info!("Start load into OSM DB");
    db::pg::restore(jobs, host, port, database, user, password, &output_path, var_lib_path, var_log_path)?;
    log::info!("Finish load into OSM DB, time (hours): {}", stopwatch);
    Ok(())
}

fn print_progress(output_path: &PathBuf, stopwatch: &StopWatch, objects: i64, i: usize) -> Result<(), anyhow::Error> {
    let du = benchmark_rs::disk_usage::disk_usage(&output_path);
    log::info!("Processed {} objects, {:.2}%, disk: {}, time: {}",
                    i.to_formatted_string(&Locale::en),
                    i as f64/ objects as f64 * 100 as f64,
                    benchmark_rs::disk_usage::to_human(du?),
                    stopwatch
                );
    Ok(())
}

fn print_verbose_info(input_path: &PathBuf, pbf_reader: &PbfReader, objects: i64, nodes: i64, ways: i64, relations: i64) {
    let info = pbf_reader.info();
    log::info!("Processing: {}", input_path.display());
    for feature in info.required_features() {
        log::info!("Required feature: {}", feature);
    }
    for feature in info.optional_features() {
        log::info!("Optional feature: {}", feature);
    }
    log::info!("Input generated by: {}", info.writingprogram().as_ref().unwrap_or(&"unknown".to_string()));
    log::info!("Nodes: {}", nodes.to_formatted_string(&Locale::en));
    log::info!("Ways: {}", ways.to_formatted_string(&Locale::en));
    log::info!("Relations: {}", relations.to_formatted_string(&Locale::en));
    log::info!("Total OSM objects: {}", objects.to_formatted_string(&Locale::en));
}

pub fn export(
    dump_path: &PathBuf,
    output_path: &PathBuf,
    _output_format: String,
    bounding_box: Option<BoundingBox>,
    calc_bounding_box: bool,
    osmosis_replication_timestamp: Option<i64>,
    osmosis_replication_sequence_number: Option<i64>,
    osmosis_replication_base_url: Option<String>,
    jobs: i16,
    host: String,
    port: String,
    database: String,
    user: String,
    password: Option<String>,
    _var_lib_path: &PathBuf,
    var_log_path: &PathBuf,
    verbose: bool,
) -> Result<(), anyhow::Error> {
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    let (dump_transaction_id, dump_timestamp) = db::pg::dump(
        jobs,
        host.clone(),
        port.clone(),
        database.clone(),
        user.clone(),
        password.clone(),
        &dump_path,
        _var_lib_path,
        var_log_path,
    )?;

    let mut create_reader_stopwatch = StopWatch::new();
    create_reader_stopwatch.start();
    log::info!("Create apidb reader. Will sort tables");
    let reader = Reader::new(dump_path.clone(), dump_path.clone())?;
    log::info!("Finished creating apidb reader, time: {}", create_reader_stopwatch);

    let (selected_osmosis_replication_timestamp, selected_osmosis_replication_sequence_number) = select_replication_params(
        osmosis_replication_timestamp,
        osmosis_replication_sequence_number,
        dump_timestamp,
        dump_transaction_id,
    );

    let info = FileInfo::new(
        calculate_bounding_box(calc_bounding_box, bounding_box, &reader)?,
        ["OsmSchema-V0.6", "DenseNodes", "HistoricalInformation"].map(|s| s.to_string()).to_vec(),
        ["Sort.Type_then_ID"].map(|s| s.to_string()).to_vec(),
        Some(format!("osm-admin-{}", option_env!("CARGO_PKG_VERSION").unwrap())),
        Some("from-apidb-dump".to_string()),
        selected_osmosis_replication_timestamp,
        selected_osmosis_replication_sequence_number,
        osmosis_replication_base_url,
    );

    let mut objects = 0;
    if verbose {
        let (nodes, ways, relations) = count_objects(host, port, database, user, password)?;
        objects = nodes + ways + relations;
        let du = benchmark_rs::disk_usage::disk_usage(&dump_path)?;
        log::info!("Sorted dump disk usage: {}", benchmark_rs::disk_usage::to_human(du));
        log::info!("Nodes: {}", nodes.to_formatted_string(&Locale::en));
        log::info!("Ways: {}", ways.to_formatted_string(&Locale::en));
        log::info!("Relations: {}", relations.to_formatted_string(&Locale::en));
        log::info!("Total OSM objects: {}", objects.to_formatted_string(&Locale::en));
    }

    let mut writer = Writer::from_file_info(
        output_path.clone(),
        info,
        CompressionType::Zlib,
    )?;

    let mut generate_pbf_stopwatch = StopWatch::new();
    generate_pbf_stopwatch.start();
    writer.write_header()?;
    for (i, element) in reader.elements()?.enumerate() {
        writer.write_element(element)?;
        if verbose && i % 10000000 == 0 && i != 0 {
            print_progress(&output_path, &generate_pbf_stopwatch, objects, i)?;
        }
    }
    writer.close()?;
    print_progress(&output_path, &generate_pbf_stopwatch, objects, objects as usize)?;

    log::info!("Osm export time: {}", stopwatch);
    Ok(())
}

fn select_replication_params(
    osmosis_replication_timestamp: Option<i64>,
    osmosis_replication_sequence_number: Option<i64>,
    dump_timestamp: DateTime<Utc>,
    dump_transaction_id: u64
) -> (Option<i64>, Option<i64>){
    let timestamp = match osmosis_replication_timestamp {
        None => {
            log::info!(
                "No osmosis_replication_timestamp provided, using dump timestamp: {}",
                dump_timestamp.to_rfc3339_opts(SecondsFormat::Secs, true)
            );
            Some(dump_timestamp.timestamp_millis() / 1000)
        }
        Some(timestamp) => {
            Some(timestamp)
        }
    };
    let sequence_number = match osmosis_replication_sequence_number{
        None => {
            log::info!(
                "No osmosis_replication_sequence_number provided, using dump transaction id: {}",
                dump_transaction_id
            );
            Some(dump_transaction_id as i64)
        }
        Some(sequence_number) => {
            Some(sequence_number)
        }
    };
    (timestamp, sequence_number)
}

fn calculate_bounding_box(calc_bounding_box: bool, bounding_box_opt: Option<BoundingBox>, reader: &Reader) -> Result<Option<BoundingBox>, anyhow::Error> {
    if calc_bounding_box {
        let mut calculated_bounding_box = None;
        for element in reader.elements()? {
            match element {
                Element::Node { node } => {
                    if node.visible() {
                        if calculated_bounding_box.is_none() {
                            calculated_bounding_box = Some(
                                BoundingBox::new(
                                    node.coordinate().lon(),
                                    node.coordinate().lat(),
                                    node.coordinate().lon(),
                                    node.coordinate().lat(),
                                )
                            )
                        } else {
                            calculated_bounding_box.as_mut().unwrap().merge_point(node.coordinate());
                        }
                    }
                }
                Element::Way { .. } => {
                    break;
                }
                Element::Relation { .. } => {
                    break;
                }
                Element::Sentinel => {
                    break;
                }
            }
        }
        Ok(calculated_bounding_box)
    } else {
        Ok(bounding_box_opt)
    }
}

