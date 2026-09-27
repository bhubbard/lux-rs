use crate::error::{LuxError, Result};
use crate::extractors::{ExtractOptions, Extractor};
use crate::types::{Part, Stream, VideoData};
use crate::utils::match_one_of;
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Default)]
pub struct YouTubeExtractor;

#[async_trait]
impl Extractor for YouTubeExtractor {
    async fn extract(&self, url: &str, options: &ExtractOptions) -> Result<Vec<VideoData>> {
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
            ),
        );

        if let Some(ref cookie) = options.cookie {
            if let Ok(val) = HeaderValue::from_str(cookie) {
                headers.insert(reqwest::header::COOKIE, val);
            }
        }

        let vid_id = match_one_of(url, &[r"(?:v=|\/embed\/|\/watch\?v=|youtu\.be\/)([a-zA-Z0-9_-]{11})"])
            .and_then(|m| m.get(1).cloned())
            .ok_or_else(|| LuxError::Extractor("Unable to extract YouTube video ID".to_string()))?;

        let watch_url = format!("https://www.youtube.com/watch?v={}&bpctr=9999999999&has_verified=1", vid_id);
        let resp = client.get(&watch_url).headers(headers).send().await?;
        let html = resp.text().await?;

        // Locate ytInitialPlayerResponse
        let player_json_str = match_one_of(&html, &[r#"ytInitialPlayerResponse\s*=\s*(\{.+?\});"#])
            .and_then(|m| m.get(1).cloned())
            .ok_or_else(|| LuxError::Extractor("ytInitialPlayerResponse not found in page".to_string()))?;

        let player_data: Value = serde_json::from_str(&player_json_str)?;

        let title = player_data["videoDetails"]["title"]
            .as_str()
            .unwrap_or("YouTube Video")
            .to_string();

        let mut video_data = VideoData::new(url, "youtube", title);
        let mut streams = HashMap::new();

        if let Some(formats) = player_data["streamingData"]["formats"].as_array() {
            for f in formats {
                if let Some(stream_url) = f["url"].as_str() {
                    let itag = f["itag"].to_string();
                    let quality = f["qualityLabel"].as_str().unwrap_or("Standard").to_string();
                    let mime = f["mimeType"].as_str().unwrap_or("");
                    let ext = if mime.contains("webm") { "webm" } else { "mp4" };
                    let size = f["contentLength"].as_str().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);

                    let mut stream = Stream::new(&itag, &quality);
                    stream.parts.push(Part::new(stream_url, size, ext));
                    stream.ext = ext.to_string();
                    streams.insert(itag, stream);
                }
            }
        }

        if let Some(adaptive) = player_data["streamingData"]["adaptiveFormats"].as_array() {
            for f in adaptive {
                if let Some(stream_url) = f["url"].as_str() {
                    let itag = f["itag"].to_string();
                    let quality = f["qualityLabel"]
                        .as_str()
                        .or_else(|| f["audioQuality"].as_str())
                        .unwrap_or("Adaptive")
                        .to_string();
                    let mime = f["mimeType"].as_str().unwrap_or("");
                    let ext = if mime.contains("audio") {
                        if mime.contains("webm") { "opus" } else { "m4a" }
                    } else if mime.contains("webm") {
                        "webm"
                    } else {
                        "mp4"
                    };
                    let size = f["contentLength"].as_str().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);

                    let mut stream = Stream::new(&itag, &quality);
                    stream.parts.push(Part::new(stream_url, size, ext));
                    stream.ext = ext.to_string();
                    streams.insert(itag, stream);
                }
            }
        }

        if streams.is_empty() {
            return Err(LuxError::NoStreamsFound(url.to_string()));
        }

        video_data.streams = streams;
        Ok(vec![video_data])
    }
}
