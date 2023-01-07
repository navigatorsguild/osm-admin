use std::path::PathBuf;
use std::process::exit;
use clap::{arg, Command};
use log::LevelFilter;
use osmpbf::{Element, ElementReader};
use simple_logger::SimpleLogger;
use postgres::{Client, NoTls};


fn command() -> Command {
    Command::new("osm-import-test").about("Integration tests for OSM import")
        .arg(arg!(--input <INPUT> "Input file path").required(true).value_parser(clap::value_parser!(PathBuf)).num_args(1))
        .arg(arg!(--host <HOST> "Database host").required(true).num_args(1))
        .arg(arg!(--port <PORT> "Database port").default_value("5432").num_args(1))
        .arg(arg!(--user <USER> "Database administrator user name").value_parser(clap::value_parser!(String)).required(true).num_args(1))
        .arg(arg!(--password <PASSWORD> "Database password. Used for the internal test database only").value_parser(clap::value_parser!(String)).required(true).num_args(1))
        .arg_required_else_help(true)
}

pub fn main(){
    let matches = command().get_matches();
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();
    log::info!("Started OSM import integration test");
    let input_path = matches.get_one::<PathBuf>("input").unwrap().clone();
    let host = matches.get_one::<String>("host").unwrap().clone();
    let port = matches.get_one::<String>("port").unwrap().clone();
    let user = matches.get_one::<String>("user").unwrap().clone();
    let password = matches.get_one::<String>("password").unwrap().clone();
    let mut client = Client::connect(format!("host={host} port={port} user={user} password={password} dbname=openstreetmap").as_str(), NoTls)
        .expect("Failed to connect to test database");

    let reader = ElementReader::from_path(&input_path).expect("Failed to create a reader");
    let result = reader.for_each(|element| {
        match element {
            Element::Node(n) => {
                let rows = client.query(format!("select id from public.current_nodes where id = {}", n.id()).as_str(), &[]).unwrap();
                assert_eq!(rows.len(), 1);
            }
            Element::DenseNode(n) => {
                let rows = client.query(format!("select id from public.current_nodes where id = {}", n.id).as_str(), &[]).unwrap();
                assert_eq!(rows.len(), 1);
            }
            Element::Way(w) => {
                let rows = client.query(format!("select id from public.current_ways where id = {}", w.id()).as_str(), &[]).unwrap();
                assert_eq!(rows.len(), 1);
            }
            Element::Relation(r) => {
                let rows = client.query(format!("select id from public.current_relations where id = {}", r.id()).as_str(), &[]).unwrap();
                assert_eq!(rows.len(), 1);
            }
        }
    });
    match result {
        Ok(_) => {
            log::info!("Finished OSM import integration test")
        }
        Err(e) => {
            log::error!("Failed OSM import integration test, error: {}", e);
            exit(1)
        }
    }
}