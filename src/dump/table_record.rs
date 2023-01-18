use chrono::{NaiveDateTime};
use crate::dump::changeset_record::ChangesetRecord;
use crate::dump::node_record::NodeRecord;
use crate::dump::node_tag_record::NodeTagRecord;
use crate::dump::table_record::TableRecord::User;
use crate::dump::user_record::UserRecord;
use crate::error::osm_error;
use crate::error::osm_error::GenericError;

#[derive(Debug)]
pub(crate) enum RelationMemberType {
    Node,
    Way,
    Relation,
}

impl TryFrom<&str> for RelationMemberType{
    type Error = GenericError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "n" => {
                Ok(RelationMemberType::Node)
            }
            "w" => {
                Ok(RelationMemberType::Way)
            }
            "r" => {
                Ok(RelationMemberType::Relation)
            }
            _ => {
                Err(osm_error::GenericError::from(osm_error::OsmError { message: format!("Unknown relation member type: {}", value) }))
            }
        }
    }
}



#[derive(Debug)]
pub(crate) enum TableRecord {
    Node {
        node_record: NodeRecord,
    },
    NodeTag {
        node_tag_record: NodeTagRecord,
    },
    Way {
        way_id: i64,
        changeset_id: i64,
        timestamp: NaiveDateTime,
        version: i64,
        visible: bool,
        redaction_id: i32,
    },
    WayTag {
        way_id: i64,
        k: String,
        v: String,
        version: i64,
    },
    WayNode {
        way_id: i64,
        node_id: i64,
        version: i64,
        sequence_id: i64,
    },
    Relation {
        relation_id: i64,
        changeset_id: i64,
        timestamp: NaiveDateTime,
        version: i64,
        visible: bool,
        redaction_id: i32,
    },
    RelationTag {
        relation_id: i64,
        k: String,
        v: String,
        version: i64,
    },
    RelationMember {
        relation_id: i64,
        member_type: RelationMemberType,
        member_id: i64,
        member_role: String,
        version: i64,
        sequence_id: i32,
    },
    Changeset {
        changeset_record: ChangesetRecord,
    },
    User {
        user_record: UserRecord,
    },
}