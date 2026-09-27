use crate::error::{LuxError, Result};
use crate::extractors::{ExtractOptions, Extractor};
use crate::types::{Part, Stream, VideoData};
use crate::utils::match_one_of;
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Default)]
pub struct DouyinExtractor;

#[async_trait]
impl Extractor for DouyinExtractor {
    async fn extract(&self, url: &str, _options: &ExtractOptions) -> Result<Vec<VideoData>> {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()?;

        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (iPhone; CPU iPhone OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1",
            ),
        );

        let resp = client.get(url).headers(headers.clone()).send().await?;
        let final_url = resp.url().to_string();
        let html = resp.text().await?;

        // Extract video ID from redirected URL or HTML
        let item_id = match_one_of(&final_url, &[r"video/(\d+)"])
            .or_else(|| match_one_of(&html, &[r#"item_id=(\d+)"#, r#""itemId":"(\d+)""#]))
            .and_then(|m| m.get(1).cloned())
            .ok_or_else(|| LuxError::Extractor("Unable to locate Douyin/TikTok video ID".to_string()))?;

        // Look for _ROUTER_DATA or RENDER_DATA in page
        let mut title = "Douyin Video".to_string();
        let mut video_url = None;

        if let Some(caps) = match_one_of(&html, &[r#"id="_ROUTER_DATA"\s*type="application/json">(.+?)</script>"#]) {
            if let Ok(data) = serde_json::from_str::<Value>(&caps[1]) {
                if let Some(desc) = data["loaderData"]["video_(id)/page"]["videoInfoRes"]["item_list"][0]["desc"].as_str() {
                    title = desc.to_string();
                }
                if let Some(urls) = data["loaderData"]["video_(id)/page"]["videoInfoRes"]["item_list"][0]["video"]["play_addr"]["url_list"].as_array() {
                    if let Some(first) = urls.first().and_then(|u| u.as_str()) {
                        video_url = Some(first.replace("playwm", "play"));
                    }
                }
            }
        }

        if video_url.is_none() {
            // Fallback: extract directly via regex
            if let Some(caps) = match_one_of(&html, &[r#"playAddr:\s*\[\{"src":"([^"]+)""#]) {
                video_url = Some(caps[1].replace(r"\u002F", "/"));
            }
        }

        let chosen_url = video_url.ok_or_else(|| LuxError::Extractor(format!("No video URL extracted for Douyin item {}", item_id)))?;

        let mut video_data = VideoData::new(url, "douyin", title);
        let mut streams = HashMap::new();
        let mut stream = Stream::new("default", "1080P HD (No Watermark)");
        stream.parts.push(Part::new(chosen_url, 0, "mp4"));
        stream.ext = "mp4".to_string();
        streams.insert("default".to_string(), stream);

        video_data.streams = streams;
        Ok(vec![video_data])
    }
}
