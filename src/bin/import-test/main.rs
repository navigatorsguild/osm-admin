use std::path::PathBuf;
use std::process::exit;
use benchmark_rs::stopwatch::StopWatch;

use clap::{arg, Command};
use log::LevelFilter;
use osmpbf::{Element, ElementReader};
use postgres::{Client, NoTls};
use simple_logger::SimpleLogger;

fn command() -> Command {
    Command::new("osm-import-test").about("Integration tests for OSM import")
        .arg(arg!(--input <INPUT> "Input file path").required(true).value_parser(clap::value_parser!(PathBuf)).num_args(1))
        .arg(arg!(--host <HOST> "Database host").required(true).num_args(1))
        .arg(arg!(--port <PORT> "Database port").default_value("5432").num_args(1))
        .arg(arg!(--user <USER> "Database administrator user name").value_parser(clap::value_parser!(String)).required(true).num_args(1))
        .arg(arg!(--password <PASSWORD> "Database password. Used for the internal test database only").value_parser(clap::value_parser!(String)).required(true).num_args(1))
        .arg_required_else_help(true)
}

pub fn main() {
    let matches = command().get_matches();
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();
    log::info!("Started OSM import integration test");
    let mut stop_watch = StopWatch::new();
    stop_watch.start();
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
                let rows = client.query(format!("select node_id from public.nodes where node_id = {}", n.id()).as_str(), &[]).unwrap();
                assert!(rows.len() >= 1);
            }
            Element::DenseNode(n) => {
                let rows = client.query(format!("select node_id from public.nodes where node_id = {}", n.id).as_str(), &[]).unwrap();
                assert!(rows.len() >= 1);
            }
            Element::Way(w) => {
                let rows = client.query(format!("select way_id from public.ways where way_id = {}", w.id()).as_str(), &[]).unwrap();
                assert!(rows.len() >= 1);
            }
            Element::Relation(r) => {
                let rows = client.query(format!("select relation_id from public.relations where relation_id = {}", r.id()).as_str(), &[]).unwrap();
                assert!(rows.len() >= 1);
            }
        }
    });
    match result {
        Ok(_) => {
            log::info!("Finished OSM import integration test, time: {}", stop_watch)
        }
        Err(e) => {
            log::error!("Failed OSM import integration test, error: {}, time: {}", e, stop_watch);
            exit(1)
        }
    }
}