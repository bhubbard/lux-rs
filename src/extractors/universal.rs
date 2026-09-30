use crate::error::{LuxError, Result};
use crate::extractors::{ExtractOptions, Extractor};
use crate::parser::{get_doc, get_title};
use crate::types::{DataType, Part, Stream, VideoData};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use scraper::Selector;

#[derive(Default, Debug, Clone, Copy)]
pub struct UniversalExtractor;

#[async_trait]
impl Extractor for UniversalExtractor {
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

        let resp = client.get(url).headers(headers).send().await?;
        let html = resp.text().await?;
        let doc = get_doc(&html);
        let title = get_title(&html);

        let mut data = VideoData::new(url, "universal", title);
        let mut streams = std::collections::HashMap::new();

        // 1. Look for og:video
        if let Ok(sel) = Selector::parse(r#"meta[property="og:video"], meta[name="og:video"]"#) {
            if let Some(el) = doc.select(&sel).next() {
                if let Some(video_url) = el.value().attr("content") {
                    let mut stream = Stream::new("default", "Standard");
                    stream.parts.push(Part::new(video_url, 0, "mp4"));
                    streams.insert("default".to_string(), stream);
                }
            }
        }

        // 2. Look for <video src="..."> or <video><source src="...">
        if streams.is_empty() {
            if let Ok(sel) = Selector::parse("video source, video") {
                for el in doc.select(&sel) {
                    if let Some(src) = el.value().attr("src") {
                        if !src.is_empty() {
                            let mut stream = Stream::new("video", "Standard");
                            let ext = if src.contains(".webm") { "webm" } else { "mp4" };
                            stream.parts.push(Part::new(src, 0, ext));
                            streams.insert("video".to_string(), stream);
                            break;
                        }
                    }
                }
            }
        }

        // 3. Look for <audio src="...">
        if streams.is_empty() {
            if let Ok(sel) = Selector::parse("audio source, audio") {
                for el in doc.select(&sel) {
                    if let Some(src) = el.value().attr("src") {
                        if !src.is_empty() {
                            let mut stream = Stream::new("audio", "Standard Audio");
                            stream.parts.push(Part::new(src, 0, "mp3"));
                            data.media_type = DataType::Audio;
                            streams.insert("audio".to_string(), stream);
                            break;
                        }
                    }
                }
            }
        }

        if streams.is_empty() {
            return Err(LuxError::NoStreamsFound(url.to_string()));
        }

        data.streams = streams;
        Ok(vec![data])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_universal_extractor_parse() {
        let html = r#"<html><head><title>Universal Test</title></head><body><video src="https://example.com/video.mp4"></video></body></html>"#;
        let doc = get_doc(html);
        let title = get_title(html);
        assert_eq!(title, "Universal Test");

        let sel = Selector::parse("video").unwrap();
        let src = doc.select(&sel).next().unwrap().value().attr("src").unwrap();
        assert_eq!(src, "https://example.com/video.mp4");
    }
}
