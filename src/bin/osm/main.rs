use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

use anyhow::{anyhow, Error};
use benchmark_rs::stopwatch::StopWatch;
use clap::{arg, ArgMatches, Command};
use num_cpus;
use osm_io::osm::model::bounding_box::BoundingBox;
use simple_logger::SimpleLogger;
use tikv_jemallocator::Jemalloc;

use osm_admin::{export, import};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn command() -> Command {
    Command::new("osm").about("Tools for OSM database administration").subcommand_required(true).arg_required_else_help(true).allow_external_subcommands(true)
        .arg(arg!(--verbose "Print progress information").required(false).num_args(0))
        .subcommand(
            Command::new("import").about("Import OSM from file into database")
                .arg(arg!(--input <INPUT> "Input file path").required(true).value_parser(clap::value_parser!(PathBuf)).num_args(1))
                .arg(arg!(--"input-format" <INPUT_FORMAT> "The input format. Currently, only pbf is supported").value_parser(["pbf"]).default_value("pbf").num_args(1))
                .arg(arg!(--output <OUTPUT> "Output directory path").required(true).value_parser(clap::value_parser!(PathBuf)).num_args(1))
                .arg(arg!(--jobs <JOBS> "Number of database load jobs. Zero means autodetect CPU allocation and use all CPUs. When more jobs than CPUs specified, the number is capped on CPU limit").value_parser(0..1024).default_value("0").num_args(1))
                .arg(arg!(--host <HOST> "Database host").required(true).num_args(1))
                .arg(arg!(--port <PORT> "Database port").default_value("5432").num_args(1))
                .arg(arg!(--database <DATABASE> "Database name").default_value("openstreetmap").num_args(1))
                .arg(arg!(--user <USER> "Database administrator user name").value_parser(clap::value_parser!(String)).required(true).num_args(1))
                .arg(arg!(--password "Prompt for password. Either --password or --no-password must be present.").required(false).num_args(0))
                .arg(arg!(--"no-password" "Don't prompt for password. Use PGPASSFILE if available").required(false).num_args(0))
                .arg(arg!(--verbose "Print progress information").required(false).num_args(0))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("export").about("Export OSM data into a file")
                .arg(arg!(--dump <DUMP> "Dump directory path").required(true).value_parser(clap::value_parser!(PathBuf)).num_args(1))
                .arg(arg!(--output <OUTPUT> "Output file path").required(true).value_parser(clap::value_parser!(PathBuf)).num_args(1))
                .arg(arg!(--"output-format" <OUTPUT_FORMAT> "The output format, currently only pbf is supported").value_parser(["pbf"]).default_value("pbf").num_args(1))
                .arg(arg!(--"bounding-box" <BOUNDING_BOX> "The precomputed bounding box in the form 'left,bottom,right,top' as in 5.8663153,47.2701114,15.0419309,55.099161").value_parser(clap::value_parser!(String)).num_args(1))
                .arg(arg!(--"calc-bounding-box" "Calculate the bounding box. Will incur an iteration over all the node elements. When present --bounding-box is ignored").required(false).num_args(0))
                .arg(arg!(--"osmosis-replication-timestamp" <OSMOSIS_REPLICATION_TIMESTAMP> "Osmosis replication timestamp").value_parser(clap::value_parser!(i64)).num_args(1))
                .arg(arg!(--"osmosis-replication-sequence-number" <OSMOSIS_REPLICATION_SEQUENCE_NUMBER> "Osmosis replication sequence number").value_parser(clap::value_parser!(i64)).num_args(1))
                .arg(arg!(--"osmosis-replication-base-url" <OSMOSIS_REPLICATION_BASE_URL> "Osmosis replication base url").value_parser(clap::value_parser!(String)).num_args(1))
                .arg(arg!(--jobs <JOBS> "Number of database dump jobs. Zero means autodetect CPU allocation and use all CPUs. When more jobs than CPUs specified, the number is capped on CPU limit").value_parser(0..1024).default_value("0").num_args(1))
                .arg(arg!(--host <HOST> "Database host").required(true).num_args(1))
                .arg(arg!(--port <PORT> "Database port").default_value("5432").num_args(1))
                .arg(arg!(--database <DATABASE> "Database name").default_value("openstreetmap").num_args(1))
                .arg(arg!(--user <USER> "Database administrator user name").value_parser(clap::value_parser!(String)).required(true).num_args(1))
                .arg(arg!(--password "Prompt for password. Either --password or --no-password must be present.").required(false).num_args(0))
                .arg(arg!(--"no-password" "Don't prompt for password. Use PGPASSFILE if available").required(false).num_args(0))
                .arg_required_else_help(true),
        )
}

fn adjust_jobs_to_available_cpus(jobs: i16) -> i16 {
    let adjusted_jobs: i16;
    let available_cpus = num_cpus::get() as i16;
    if jobs <= 0 || jobs > available_cpus {
        adjusted_jobs = available_cpus;
    } else {
        adjusted_jobs = jobs;
    }
    adjusted_jobs
}

fn main() -> Result<(), Error> {
    let mut stopwatch = StopWatch::new();
    stopwatch.start();

    let command = command();
    let mut command_clone = command.clone();
    let matches = command.get_matches();
    let verbose = matches.get_flag("verbose");
    if verbose {
        SimpleLogger::new().init()?;
    }

    log::info!("Started OSM Admin.");
    let var_log_path = PathBuf::from("/var/log/osm/");
    let var_lib_path = PathBuf::from("/var/lib/osm/");

    let result = match matches.subcommand() {
        Some(("import", sub_matches)) => {
            handle_import(&var_log_path, &var_lib_path, sub_matches, verbose)
        }
        Some(("export", sub_matches)) => {
            handle_export(&var_log_path, &var_lib_path, sub_matches, verbose)
        }
        Some((_, _)) => {
            command_clone.print_help()?;
            exit(1);
        }
        None => {
            command_clone.print_help()?;
            exit(1);
        }
    };
    match result {
        Ok(_) => {
            log::info!("Finished OSM Admin. Total time: {}", stopwatch);
            exit(0);
        }
        Err(e) => {
            log::info!("Failed OSM Admin with error: {}. Total time: {}", e, stopwatch);
            exit(1);
        }
    }
}

fn handle_import(
    var_log_path: &PathBuf,
    var_lib_path: &PathBuf,
    sub_matches: &ArgMatches,
    verbose: bool,
) -> Result<(), Error> {
    log::info!("Started OSM import");
    let input_path = sub_matches.get_one::<PathBuf>("input")
        .unwrap()
        .clone();
    let input_format = sub_matches.get_one::<String>("input-format")
        .unwrap()
        .clone();
    let output_path = sub_matches.get_one::<PathBuf>("output")
        .unwrap()
        .clone();
    let jobs: i16 = adjust_jobs_to_available_cpus(
        *sub_matches.get_one::<i64>("jobs").unwrap() as i16
    );
    let host = sub_matches.get_one::<String>("host")
        .unwrap()
        .clone();
    let port = sub_matches.get_one::<String>("port")
        .unwrap()
        .clone();
    let database = sub_matches.get_one::<String>("database")
        .unwrap()
        .clone();
    let user = sub_matches.get_one::<String>("user")
        .unwrap()
        .clone();
    let prompt_password = sub_matches.get_flag("password");
    let dont_prompt_password = sub_matches.get_flag("no-password");
    let password = get_password(prompt_password, dont_prompt_password)?;

    import(
        input_path,
        input_format,
        output_path,
        jobs,
        host,
        port,
        database,
        user,
        password,
        &var_lib_path,
        &var_log_path,
        verbose,
    )
}

fn handle_export(
    var_log_path: &PathBuf,
    var_lib_path: &PathBuf,
    sub_matches: &ArgMatches,
    verbose: bool,
) -> Result<(), Error> {
    let dump_path = sub_matches.get_one::<PathBuf>("dump")
        .unwrap()
        .clone();
    let output_path = sub_matches.get_one::<PathBuf>("output")
        .unwrap()
        .clone();
    let output_format = sub_matches.get_one::<String>("output-format")
        .unwrap()
        .clone();
    let bounding_box_opt = sub_matches.get_one::<String>("bounding-box")
        .clone();
    let bounding_box = match bounding_box_opt {
        None => { None }
        Some(s) => {
            Some(BoundingBox::from_str(s)?)
        }
    };
    let calc_bounding_box = sub_matches.get_flag("calc-bounding-box");
    let osmosis_replication_timestamp = sub_matches.get_one::<i64>("osmosis-replication-timestamp").copied();
    let osmosis_replication_sequence_number = sub_matches.get_one::<i64>("osmosis-replication-sequence-number").copied();
    let osmosis_replication_base_url = sub_matches.get_one::<String>("osmosis-replication-base-url").cloned();
    let jobs: i16 = adjust_jobs_to_available_cpus(
        *sub_matches.get_one::<i64>("jobs").unwrap() as i16
    );
    let host = sub_matches.get_one::<String>("host")
        .unwrap()
        .clone();
    let port = sub_matches.get_one::<String>("port")
        .unwrap()
        .clone();
    let database = sub_matches.get_one::<String>("database")
        .unwrap()
        .clone();
    let user = sub_matches.get_one::<String>("user")
        .unwrap()
        .clone();
    let prompt_password = sub_matches.get_flag("password");
    let dont_prompt_password = sub_matches.get_flag("no-password");
    let password = get_password(prompt_password, dont_prompt_password)?;

    log::info!("Started OSM export");
    let result = export(
        &dump_path,
        &output_path,
        output_format,
        bounding_box,
        calc_bounding_box,
        osmosis_replication_timestamp,
        osmosis_replication_sequence_number,
        osmosis_replication_base_url,
        jobs,
        host,
        port,
        database,
        user,
        password,
        &var_lib_path,
        &var_log_path,
        verbose,
    );
    match &result {
        Ok(_) => {
            log::info!("Finished OSM export")
        }
        Err(e) => {
            log::error!("Failed OSM export: {}", e);
        }
    }
    result
}

fn get_password(prompt_password: bool, dont_prompt_password: bool) -> Result<Option<String>, anyhow::Error> {
    let password = if prompt_password {
        Ok(Some(rpassword::prompt_password("Please enter password: ")?))
    } else if dont_prompt_password {
        // direct pg client to PGPASSFILE
        Ok(None)
    } else {
        Err(anyhow!("Either --password or --no-password must be specified"))
    };
    password
}
