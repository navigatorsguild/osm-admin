use crate::dump::node_record::NodeRecord;
use crate::dump::node_relations_reader::NodeRelationsReader;
use crate::dump::node_tag_record::NodeTagRecord;

#[derive(Debug)]
pub(crate) struct NodeRelation{
    pub(crate) node: NodeRecord,
    pub(crate) tags: Vec<NodeTagRecord>,
}