use chrono::NaiveDateTime;

#[derive(Debug)]
pub(crate) struct ChangesetRecord {
    pub(crate) id: i64,
    pub(crate) user_id: i64,
    pub(crate) created_at: NaiveDateTime,
    pub(crate) min_lat: i32,
    pub(crate) max_lat: i32,
    pub(crate) min_lon: i32,
    pub(crate) max_lon: i32,
    pub(crate) closed_at: NaiveDateTime,
    pub(crate) num_changes: i32,
}
