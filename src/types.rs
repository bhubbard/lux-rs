use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Indicates the kind of media extracted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    #[default]
    Video,
    Audio,
    Image,
}

/// A single segment or chunk of a media stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Part {
    pub url: String,
    pub size: u64,
    pub ext: String,
}

impl Part {
    pub fn new(url: impl Into<String>, size: u64, ext: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            size,
            ext: ext.into(),
        }
    }
}

/// A subtitle, closed caption, or danmaku item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptionPart {
    #[serde(flatten)]
    pub part: Part,
    pub lang: Option<String>,
}

/// A specific quality stream of a media item (e.g. 1080P, 720P, audio track).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stream {
    pub id: String,
    pub quality: String,
    pub parts: Vec<Part>,
    pub size: u64,
    pub ext: String,
    pub need_mux: bool,
}

impl Stream {
    pub fn new(id: impl Into<String>, quality: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            quality: quality.into(),
            parts: Vec::new(),
            size: 0,
            ext: String::new(),
            need_mux: false,
        }
    }
}

/// Root data structure representing an extracted media target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoData {
    pub url: String,
    pub site: String,
    pub title: String,
    pub media_type: DataType,
    pub streams: HashMap<String, Stream>,
    #[serde(default)]
    pub captions: HashMap<String, CaptionPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl VideoData {
    pub fn new(url: impl Into<String>, site: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            site: site.into(),
            title: title.into(),
            media_type: DataType::Video,
            streams: HashMap::new(),
            captions: HashMap::new(),
            error: None,
        }
    }

    /// Automatically fills in default IDs, qualities, sizes, and merged extensions.
    pub fn fill_up_streams_data(&mut self) {
        for (id, stream) in self.streams.iter_mut() {
            if stream.id.is_empty() {
                stream.id = id.clone();
            }
            if stream.quality.is_empty() {
                stream.quality = id.clone();
            }

            if self.media_type == DataType::Video && stream.ext.is_empty() && !stream.parts.is_empty() {
                let mut ext = stream.parts[0].ext.clone();
                match ext.to_lowercase().as_str() {
                    "ts" | "flv" | "f4v" => ext = "mp4".to_string(),
                    _ => {}
                }
                stream.ext = ext;
            }

            if stream.size == 0 {
                stream.size = stream.parts.iter().map(|p| p.size).sum();
            }
        }
    }

    /// Returns the stream sorted by highest quality/size, or a specific stream by ID.
    pub fn select_stream(&self, requested_id: Option<&str>) -> Option<&Stream> {
        if let Some(req) = requested_id {
            if let Some(s) = self.streams.get(req) {
                return Some(s);
            }
        }
        // Return stream with largest size or first available
        self.streams.values().max_by_key(|s| s.size)
    }
}
