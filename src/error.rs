use thiserror::Error;

#[derive(Error, Debug)]
pub enum LuxError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Extractor error: {0}")]
    Extractor(String),

    #[error("No streams found for media: {0}")]
    NoStreamsFound(String),

    #[error("Invalid stream ID '{selected}' requested. Available: {available}")]
    InvalidStreamId {
        selected: String,
        available: String,
    },

    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("General error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, LuxError>;
