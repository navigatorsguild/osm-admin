pub type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(thiserror::Error, Debug)]
#[error("{message:}")]
pub struct OsmError {
    pub message: String,
}

impl OsmError {
    pub fn new(message: String) -> OsmError {
        OsmError {
            message
        }
    }
}
