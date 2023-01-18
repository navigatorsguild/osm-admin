use chrono::NaiveDateTime;

#[derive(Debug)]
pub(crate) struct NodeRecord {
    pub(crate) node_id: i64,
    pub(crate) latitude: i32,
    pub(crate) longitude: i32,
    pub(crate) changeset_id: i64,
    pub(crate) visible: bool,
    pub(crate) timestamp: NaiveDateTime,
    pub(crate) tile: i64,
    pub(crate) version: i64,
    pub(crate) redaction_id: Option<i32>,
}
