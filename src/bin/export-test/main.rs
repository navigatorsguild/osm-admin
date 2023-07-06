use std::path::PathBuf;

use benchmark_rs::stopwatch::StopWatch;
use clap::{arg, Command};
use log::LevelFilter;
use osm_io::osm::model::element::Element;
use osm_io::osm::model::relation::Member;
use osm_io::osm::pbf::reader::Reader;
use simple_logger::SimpleLogger;

fn command() -> Command {
    Command::new("osm-import-test").about("Integration tests for OSM export")
        .arg(arg!(--input <INPUT> "Input file path").required(true).value_parser(clap::value_parser!(PathBuf)).num_args(1))
        .arg(arg!(--output <OUTPUT> "Output file path").required(true).value_parser(clap::value_parser!(PathBuf)).num_args(1))
        .arg_required_else_help(true)
}

pub fn main() -> Result<(), anyhow::Error> {
    let matches = command().get_matches();
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();
    let mut stop_watch = StopWatch::new();
    stop_watch.start();
    log::info!("Started OSM export integration test");
    let input_path = matches.get_one::<PathBuf>("input").unwrap().clone();
    let output_path = matches.get_one::<PathBuf>("output").unwrap().clone();

    let input_reader = Reader::new(&input_path)?;
    let output_reader = Reader::new(&output_path)?;

    let mut input_elements = input_reader.elements()?;
    let mut output_elements = output_reader.elements()?;

    let mut input_element = input_elements.next();
    let mut output_element = output_elements.next();
    while input_element.is_some() && output_element.is_some() {
        // println!("input: {:?}", input_element);
        // println!("output: {:?}", output_element);
        // println!("");
        assert!(elements_are_equal(input_element.unwrap(), output_element.unwrap()));
        input_element = input_elements.next();
        output_element = output_elements.next();
    }

    assert!(input_element.is_none());
    assert!(output_element.is_none());
    log::info!("Finished OSM export integration test, time: {}", stop_watch);
    Ok(())
}

fn elements_are_equal(input_element: Element, output_element: Element) -> bool {
    if input_element.is_node() && output_element.is_node() {
        nodes_are_equal(input_element, output_element)
    } else if input_element.is_way() && output_element.is_way() {
        ways_are_equal(input_element, output_element)
    } else if input_element.is_relation() && output_element.is_relation() {
        relations_are_equal(input_element, output_element)
    } else if input_element.is_sentinel() && output_element.is_sentinel() {
        true
    } else {
        false
    }
}

fn nodes_are_equal(input_element: Element, output_element: Element) -> bool {
    let input_node = if let Element::Node { node } = input_element {
        node
    } else {
        panic!("Not a node");
    };

    let output_node = if let Element::Node { node } = output_element {
        node
    } else {
        panic!("Not a node");
    };

    input_node.id() == output_node.id()
        && input_node.version() == output_node.version()
        && input_node.visible() == output_node.visible()
        && input_node.coordinate() == output_node.coordinate()
        && input_node.timestamp() == output_node.timestamp()
        && input_node.uid() == output_node.uid()
        && input_node.user() == output_node.user()
        && input_node.changeset() == output_node.changeset()
        && input_node.tags() == output_node.tags()
}

fn ways_are_equal(input_element: Element, output_element: Element) -> bool {
    let input_way = if let Element::Way { way } = input_element {
        way
    } else {
        panic!("Not a way");
    };

    let output_way = if let Element::Way { way } = output_element {
        way
    } else {
        panic!("Not a way");
    };

    input_way.id() == output_way.id()
        && input_way.version() == output_way.version()
        && input_way.visible() == output_way.visible()
        && input_way.timestamp() == output_way.timestamp()
        && input_way.uid() == output_way.uid()
        && input_way.user() == output_way.user()
        && input_way.changeset() == output_way.changeset()
        && input_way.tags() == output_way.tags()
        && input_way.refs() == output_way.refs()
}

fn compare_relation_members(input: &Vec<Member>, output: &Vec<Member>) -> bool {
    let mut input_members: Vec<String> = input
        .into_iter()
        .map(|member| format!("{:?}", member))
        .collect();
    input_members.sort();

    let mut output_members: Vec<String> = output
        .into_iter()
        .map(|member| format!("{:?}", member))
        .collect();
    output_members.sort();

    input_members == output_members
}

fn relations_are_equal(input_element: Element, output_element: Element) -> bool {
    let input_relation = if let Element::Relation { relation } = input_element {
        relation
    } else {
        panic!("Not a relation");
    };

    let output_relation = if let Element::Relation { relation } = output_element {
        relation
    } else {
        panic!("Not a relation");
    };

    input_relation.id() == output_relation.id()
        && input_relation.version() == output_relation.version()
        && input_relation.visible() == output_relation.visible()
        && input_relation.timestamp() == output_relation.timestamp()
        && input_relation.uid() == output_relation.uid()
        && input_relation.user() == output_relation.user()
        && input_relation.changeset() == output_relation.changeset()
        && input_relation.tags() == output_relation.tags()
        && compare_relation_members(input_relation.members(), output_relation.members())
}
