// use std::collections::HashMap;
//
// use osmpbf::Node;
//
// use crate::dump::node_relation::NodeRelation;
// use crate::dump::table_def::TableDef;
// use crate::dump::table_reader::{TableIterator, TableReader};
// use crate::dump::table_record::TableRecord;
// use crate::error::osm_error::GenericError;
//
// #[derive(Clone)]
// pub(crate) struct NodeRelationsReader {
//     nodes_reader: TableReader,
//     node_tags_reader: TableReader,
// }
//
// impl NodeRelationsReader {
//     pub(crate) fn new(nodes_def: &TableDef, node_tags_def: &TableDef) -> Result<Self, GenericError> {
//         let nodes_reader = TableReader::new(nodes_def)?;
//         let node_tags_reader = TableReader::new(node_tags_def)?;
//         Ok(
//             NodeRelationsReader {
//                 nodes_reader,
//                 node_tags_reader,
//             }
//         )
//     }
// }
//
// impl IntoIterator for NodeRelationsReader {
//     type Item = NodeRelation;
//     type IntoIter = NodeRelationsIterator;
//
//     fn into_iter(self) -> Self::IntoIter {
//         NodeRelationsIterator::new(&self).unwrap()
//     }
// }
//
// pub(crate) struct NodeRelationsIterator {
//     reader: NodeRelationsReader,
//     nodes_iterator: TableIterator,
//     node_tags_iterator: TableIterator,
//     node_tags: HashMap<i64, Vec<TableRecord>>,
// }
//
//
// impl NodeRelationsIterator {
//     pub(crate) fn new(node_relations_reader_: &NodeRelationsReader) -> Result<NodeRelationsIterator, GenericError> {
//         log::info!("Create node relation iterator");
//         let reader = node_relations_reader_.clone();
//         let nodes = reader.nodes_reader.clone().into_iter();
//         let node_tags = reader.node_tags_reader.clone().into_iter();
//         Ok(
//             NodeRelationsIterator {
//                 reader,
//                 nodes_iterator: nodes,
//                 node_tags_iterator: node_tags,
//                 node_tags: Default::default(),
//             }
//         )
//     }
//
//     pub(crate) fn fetch_tags_for_node(&mut self, id: &i64, tags: &mut Vec<TableRecord>){
//
//     }
// }
//
// impl Iterator for NodeRelationsIterator {
//     type Item = NodeRelation;
//
//
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if let Some(node) = self.nodes_iterator.next() {
//             if let TableRecord::Node { node_id, latitude, longitude, changeset_id, visible, timestamp, tile, version, redaction_id } = &node {
//                 for node_tag in self.node_tags_iterator.next() {
//                     if let TableRecord::NodeTag { node_id, version, k, v } = &node_tag {
//                         self.node_tags.insert(*node_id, *node_tag);
//                     }
//
//                 }
//                 Some(
//                     NodeRelation {}
//                 )
//             } else {
//                 None
//             }
//         } else {
//             None
//         }
//     }
// }
