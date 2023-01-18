#[derive(Debug)]
pub(crate) struct NodeTagRecord {
    pub(crate) node_id: i64,
    pub(crate) version: i64,
    pub(crate) k: String,
    pub(crate) v: String,
}
