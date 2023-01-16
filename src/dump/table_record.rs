use chrono::{NaiveDateTime};
use crate::dump::table_record::TableRecord::User;
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
pub(crate) enum UserStatus {
    Pending,
    Active,
    Confirmed,
    Suspended,
    Deleted,
}

impl TryFrom<&str> for UserStatus{
    type Error = GenericError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pending" => {
                Ok(UserStatus::Pending)
            }
            "active" => {
                Ok(UserStatus::Active)
            }
            "confirmed" => {
                Ok(UserStatus::Confirmed)
            }
            "suspended" => {
                Ok(UserStatus::Suspended)
            }
            "deleted" => {
                Ok(UserStatus::Deleted)
            }
            _ => {
                Err(osm_error::GenericError::from(osm_error::OsmError { message: format!("Unknown user status: {}", value) }))
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum FormatEnum {
    Html,
    Markdown,
    Text,
}

impl TryFrom<&str> for FormatEnum{
    type Error = GenericError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "html" => {
                Ok(FormatEnum::Html)
            }
            "markdown" => {
                Ok(FormatEnum::Markdown)
            }
            "text" => {
                Ok(FormatEnum::Text)
            }
            _ => {
                Err(osm_error::GenericError::from(osm_error::OsmError { message: format!("Unknown format: {}", value) }))
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum TableRecord {
    Node {
        node_id: i64,
        latitude: i32,
        longitude: i32,
        changeset_id: i64,
        visible: bool,
        timestamp: NaiveDateTime,
        tile: i64,
        version: i64,
        redaction_id: i32,
    },
    NodeTag {
        node_id: i64,
        version: i64,
        k: String,
        v: String,
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
        id: i64,
        user_id: i64,
        created_at: NaiveDateTime,
        min_lat: i32,
        max_lat: i32,
        min_lon: i32,
        max_lon: i32,
        closed_at: NaiveDateTime,
        num_changes: i32,
    },
    User {
        email: String,
        id: i64,
        pass_crypt: String,
        creation_time: NaiveDateTime,
        display_name: String,
        data_public: bool,
        description: String,
        home_lat: f64,
        home_lon: f64,
        home_zoom: i16,
        pass_salt: String,
        email_valid: bool,
        new_email: String,
        creation_ip: String,
        languages: String,
        status: UserStatus,
        terms_agreed: Option<NaiveDateTime>,
        consider_pd: bool,
        auth_uid: String,
        preferred_editor: String,
        terms_seen: bool,
        description_format: FormatEnum,
        changesets_count: i32,
        traces_count: i32,
        diary_entries_count: i32,
        image_use_gravatar: bool,
        auth_provider: String,
        home_tile: Option<i64>,
        tou_agreed: Option<NaiveDateTime>,
    },
}