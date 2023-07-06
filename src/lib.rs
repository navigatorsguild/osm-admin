extern crate core;

use std::path::PathBuf;

use benchmark_rs::stopwatch::StopWatch;
use osm_io::osm::apidb_dump::read::reader::Reader;
use osm_io::osm::apidb_dump::write::writer::Writer as ApiDbDumpWriter;
use osm_io::osm::model::bounding_box::BoundingBox;
use osm_io::osm::model::element::Element;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::file_info::FileInfo;
use osm_io::osm::pbf::reader::Reader as PbfReader;
use osm_io::osm::pbf::writer::Writer;

pub(crate) mod db;

pub fn import(
    input_path: PathBuf,
    _input_format: String,
    output_path: PathBuf,
    _compression_level: i8,
    jobs: i16,
    host: String,
    port: String,
    user: String,
    password: Option<String>,
    var_lib_path: &PathBuf,
    var_log_path: &PathBuf,
) -> Result<(), anyhow::Error> {
    let mut stopwatch = StopWatch::new();
    stopwatch.start();

    log::info!("Start apidb dump generation");
    let pbf_reader = PbfReader::new(&input_path)?;
    let mut apidb_dump_writer = ApiDbDumpWriter::new(output_path.clone(), 0)?;
    for element in pbf_reader.elements()? {
        apidb_dump_writer.write_element(element)?;
    }
    apidb_dump_writer.close()?;
    log::info!("Finish apidb dump generation, time (hours): {}", stopwatch);

    stopwatch.reset();
    stopwatch.start();
    log::info!("Start load into OSM DB");
    db::pg::restore(jobs, host, port, user, password, &output_path, var_lib_path, var_log_path)?;
    log::info!("Finish load into OSM DB, time (hours): {}", stopwatch);
    Ok(())
}

pub fn export(
    dump_path: &PathBuf,
    output_path: &PathBuf,
    _output_format: String,
    _compression_level: i8,
    bounding_box: Option<BoundingBox>,
    calc_bounding_box: bool,
    osmosis_replication_timestamp: Option<i64>,
    osmosis_replication_sequence_number: Option<i64>,
    osmosis_replication_base_url: Option<String>,
    jobs: i16,
    host: String,
    port: String,
    user: String,
    password: Option<String>,
    _var_lib_path: &PathBuf,
    var_log_path: &PathBuf,
) -> Result<(), anyhow::Error> {
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    db::pg::dump(
        jobs,
        host,
        port,
        user,
        password,
        &dump_path,
        _var_lib_path,
        var_log_path,
    )?;

    println!("{:?}", bounding_box);

    let reader = Reader::new(dump_path.clone(), dump_path.clone())?;

    let info = FileInfo::new(
        calculate_bounding_box(calc_bounding_box, bounding_box, &reader)?,
        ["OsmSchema-V0.6", "DenseNodes"].map(|s| s.to_string()).to_vec(),
        ["Sort.Type_then_ID"].map(|s| s.to_string()).to_vec(),
        Some(format!("osm-admin-{}", option_env!("CARGO_PKG_VERSION").unwrap())),
        Some("from-apidb-dump".to_string()),
        osmosis_replication_timestamp,
        osmosis_replication_sequence_number,
        osmosis_replication_base_url,
    );

    let mut writer = Writer::from_file_info(
        output_path.clone(),
        info,
        CompressionType::Zlib,
    )?;

    writer.write_header()?;
    for element in reader.elements()? {
        writer.write_element(element)?;
    }
    writer.close()?;

    log::info!("Osm export time: {}", stopwatch);
    Ok(())
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
