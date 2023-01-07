use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

use clap::{arg, Command};
use simple_logger::SimpleLogger;
use num_cpus;

use osm_admin::import;

fn command() -> Command {
    Command::new("osm").about("Tools for OSM database administration").subcommand_required(true).arg_required_else_help(true).allow_external_subcommands(true).subcommand(
        Command::new("import").about("Import OSM from file into database`")
            .arg(arg!(--input <INPUT> "Input file path").required(true).value_parser(clap::value_parser!(PathBuf)).num_args(1))
            .arg(arg!(--"input-format" <INPUT_FORMAT> "The input format").value_parser(["pbf"]).default_value("pbf").num_args(1))
            .arg(arg!(--output <OUTPUT> "Output directory path").required(true).value_parser(clap::value_parser!(PathBuf)).num_args(1))
            .arg(arg!(--"compression-level" <COMPRESSION_LEVEL> "Compression level 0..9. Zero means no compression").value_parser(0..10).default_value("0").num_args(1))
            .arg(arg!(--jobs <JOBS> "Number of database load jobs. Zero means autodetect CPU allocation and use all CPUs. When more jobs than CPUs specified, the number is capped on CPU limit").value_parser(0..1024).default_value("0").num_args(1))
            .arg(arg!(--host <HOST> "Database host").required(true).num_args(1))
            .arg(arg!(--port <PORT> "Database port").default_value("5432").num_args(1))
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

fn main() {
    let mut command = command();
    let import_help = command.find_subcommand_mut("import").unwrap().render_long_help();
    let matches = command.get_matches();

    SimpleLogger::new().init().unwrap();
    log::info!("Started OSM Admin.");

    let var_log_path = PathBuf::from_str("/var/log/osm/").unwrap();
    let var_lib_path = PathBuf::from_str("/var/lib/osm/").unwrap();

    match matches.subcommand() {
        Some(("import", sub_matches)) => {
            let input_path = sub_matches.get_one::<PathBuf>("input").unwrap().clone();

            let input_format = sub_matches.get_one::<String>("input-format").unwrap().clone();

            let template_path = var_lib_path.as_path().join("template");

            let template_mapping_path = var_lib_path.as_path().join("template-mapping.json");

            let output_path = sub_matches.get_one::<PathBuf>("output").unwrap().clone();

            let compression_level: i8 = *sub_matches.get_one::<i64>("compression-level").unwrap() as i8;

            let jobs: i16 = adjust_jobs_to_available_cpus(*sub_matches.get_one::<i64>("jobs").unwrap() as i16);

            let host = sub_matches.get_one::<String>("host").unwrap().clone();

            let port = sub_matches.get_one::<String>("port").unwrap().clone();

            let user = sub_matches.get_one::<String>("user").unwrap().clone();

            let prompt_password = sub_matches.get_flag("password");

            let dont_prompt_password = sub_matches.get_flag("no-password");

            let password = if prompt_password {
                Some(rpassword::prompt_password("Please enter password: ").unwrap())
            } else if dont_prompt_password {
                // direct pg client to PGPASSFILE
                None
            } else {
                log::error!("Failed OSM Admin");
                eprint!("{}", import_help);
                exit(1)
            };

            log::info!("Started OSM import");
            let result = import(
                input_path,
                input_format,
                template_path,
                template_mapping_path,
                output_path,
                compression_level,
                jobs,
                host,
                port,
                user,
                password,
                &var_lib_path,
                &var_log_path,
            );
            match result {
                Ok(_) => {
                    log::info!("Finished OSM import")
                }
                Err(e) => {
                    log::error!("Failed OSM import: {}", e);
                    exit(1)
                }
            }
        }
        _ => unreachable!()
    }
    log::info!("Finished OSM Admin.");
}
