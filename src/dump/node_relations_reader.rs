use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;

use osmpbf::Node;

use crate::dump::node_record::NodeRecord;
use crate::dump::node_relation::NodeRelation;
use crate::dump::node_tag_record::NodeTagRecord;
use crate::dump::table_def::TableDef;
use crate::dump::table_reader::{TableIterator, TableReader};
use crate::dump::table_record::TableRecord;
use crate::error::osm_error::GenericError;

#[derive(Clone)]
pub(crate) struct NodeRelationsReader {
    nodes_reader: TableReader,
    node_tags_reader: TableReader,
}

impl NodeRelationsReader {
    pub(crate) fn new(nodes_def: &TableDef, node_tags_def: &TableDef) -> Result<Self, GenericError> {
        let nodes_reader = TableReader::new(nodes_def)?;
        let node_tags_reader = TableReader::new(node_tags_def)?;
        Ok(
            NodeRelationsReader {
                nodes_reader,
                node_tags_reader,
            }
        )
    }
}

impl IntoIterator for NodeRelationsReader {
    type Item = NodeRelation;
    type IntoIter = NodeRelationsIterator;

    fn into_iter(self) -> Self::IntoIter {
        NodeRelationsIterator::new(&self).unwrap()
    }
}

pub(crate) struct NodeRelationsIterator {
    reader: NodeRelationsReader,
    nodes_iterator: TableIterator,
    node_tags_iterator: TableIterator,
    node_tags: HashMap<i64, Vec<NodeTagRecord>>,
    next_node_tag_record: Option<NodeTagRecord>,
}


impl NodeRelationsIterator {
    pub(crate) fn new(node_relations_reader_: &NodeRelationsReader) -> Result<NodeRelationsIterator, GenericError> {
        log::info!("Create node relation iterator");
        let reader = node_relations_reader_.clone();
        let nodes = reader.nodes_reader.clone().into_iter();
        let node_tags = reader.node_tags_reader.clone().into_iter();
        Ok(
            NodeRelationsIterator {
                reader,
                nodes_iterator: nodes,
                node_tags_iterator: node_tags,
                node_tags: Default::default(),
                next_node_tag_record: None,
            }
        )
    }

    pub(crate) fn fetch_tags_for_node(&mut self, id: &i64, tags: &mut Vec<TableRecord>) {}
}

impl Iterator for NodeRelationsIterator {
    type Item = NodeRelation;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.nodes_iterator.next() {
            if let TableRecord::Node { node_record } = node {
                let mut current_node_tags = Vec::<NodeTagRecord>::new();
                if let Some(node_tag_record) = self.next_node_tag_record.take() {
                    if node_tag_record.node_id == node_record.node_id {
                        current_node_tags.push(node_tag_record);
                        while let Some(node_tag) = self.node_tags_iterator.next() {
                            if let TableRecord::NodeTag { node_tag_record } = node_tag {
                                if node_tag_record.node_id == node_record.node_id {
                                    current_node_tags.push(node_tag_record)
                                } else {
                                    self.next_node_tag_record = Some(node_tag_record);
                                    break;
                                }
                            } else {
                                log::warn!("Found incorrect record type, not a TableRecord:NodeTag");
                            }
                        }
                    } else {
                        self.next_node_tag_record = Some(node_tag_record);
                    }
                } else {
                    for node_tag in self.node_tags_iterator.next() {
                        if let TableRecord::NodeTag { node_tag_record } = node_tag {
                            if node_tag_record.node_id == node_record.node_id {
                                current_node_tags.push(node_tag_record)
                            } else {
                                self.next_node_tag_record = Some(node_tag_record);
                                break;
                            }
                        } else {
                            log::warn!("Found incorrect record type, not a TableRecord:NodeTag");
                        }
                    }
                }

                Some(
                    NodeRelation {
                        node: node_record,
                        tags: current_node_tags,
                    }
                )
            } else {
                log::warn!("Found incorrect record type, not a TableRecord:Node");
                None
            }
        } else {
            None
        }
    }
}

// for node_tag in self.node_tags_iterator.next() {
// if let TableRecord::NodeTag { node_tag_record } = node_tag {
// if node_tag_record.node_id == node_record.node_id {
// current_node_tags.push(node_tag_record)
// } else {
// self.next_node_tag_record = Some(node_tag_record);
// break;
// }
// } else {
// log::warn!("Found incorrect record type, not a TableRecord:NodeTag");
// }
// }
//
// }
// } else {
// for node_tag in self.node_tags_iterator.next() {
// if let TableRecord::NodeTag { node_tag_record } = node_tag {
// if node_tag_record.node_id == node_record.node_id {
// current_node_tags.push(node_tag_record)
// } else {
// self.next_node_tag_record = Some(node_tag_record);
// break;
// }
// } else {
// log::warn!("Found incorrect record type, not a TableRecord:NodeTag");
// }
// }