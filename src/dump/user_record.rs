use chrono::NaiveDateTime;
use crate::error::osm_error;
use crate::error::osm_error::GenericError;

#[derive(Debug)]
pub(crate) enum UserStatus {
    Pending,
    Active,
    Confirmed,
    Suspended,
    Deleted,
}

impl TryFrom<&str> for UserStatus {
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

impl TryFrom<&str> for FormatEnum {
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
pub(crate) struct UserRecord {
    pub(crate) email: String,
    pub(crate) id: i64,
    pub(crate) pass_crypt: String,
    pub(crate) creation_time: NaiveDateTime,
    pub(crate) display_name: String,
    pub(crate) data_public: bool,
    pub(crate) description: String,
    pub(crate) home_lat: f64,
    pub(crate) home_lon: f64,
    pub(crate) home_zoom: i16,
    pub(crate) pass_salt: String,
    pub(crate) email_valid: bool,
    pub(crate) new_email: String,
    pub(crate) creation_ip: String,
    pub(crate) languages: String,
    pub(crate) status: UserStatus,
    pub(crate) terms_agreed: Option<NaiveDateTime>,
    pub(crate) consider_pd: bool,
    pub(crate) auth_uid: String,
    pub(crate) preferred_editor: String,
    pub(crate) terms_seen: bool,
    pub(crate) description_format: FormatEnum,
    pub(crate) changesets_count: i32,
    pub(crate) traces_count: i32,
    pub(crate) diary_entries_count: i32,
    pub(crate) image_use_gravatar: bool,
    pub(crate) auth_provider: String,
    pub(crate) home_tile: Option<i64>,
    pub(crate) tou_agreed: Option<NaiveDateTime>,
}
