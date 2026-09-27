use crate::error::{LuxError, Result};
use crate::extractors::{ExtractOptions, Extractor};
use crate::parser::get_title;
use crate::types::{CaptionPart, Part, Stream, VideoData};
use crate::utils::match_one_of;
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, REFERER, USER_AGENT};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Default)]
pub struct BilibiliExtractor;

#[derive(Deserialize)]
struct PlayUrlResponse {
    code: i32,
    message: String,
    data: Option<PlayUrlData>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct PlayUrlData {
    dash: Option<DashData>,
    durl: Option<Vec<DurlData>>,
    accept_description: Option<Vec<String>>,
    accept_quality: Option<Vec<u32>>,
}

#[derive(Deserialize)]
struct DashData {
    video: Vec<DashStreamItem>,
    audio: Option<Vec<DashStreamItem>>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct DashStreamItem {
    id: u32,
    base_url: Option<String>,
    backup_url: Option<Vec<String>>,
    bandwidth: Option<u64>,
    codecs: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct DurlData {
    order: u32,
    length: u64,
    size: u64,
    url: String,
}

#[async_trait]
impl Extractor for BilibiliExtractor {
    async fn extract(&self, url: &str, options: &ExtractOptions) -> Result<Vec<VideoData>> {
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
            ),
        );
        headers.insert(REFERER, HeaderValue::from_static("https://www.bilibili.com"));

        if let Some(ref cookie) = options.cookie {
            if let Ok(val) = HeaderValue::from_str(cookie) {
                headers.insert(reqwest::header::COOKIE, val);
            }
        }

        // Fetch page HTML to extract initial state (bvid, cid, title)
        let page_resp = client.get(url).headers(headers.clone()).send().await?;
        let page_html = page_resp.text().await?;
        let page_title = get_title(&page_html);

        // Extract bvid and cid using regex
        let bvid = match_one_of(url, &[r"video/(BV\w+)"])
            .map(|m| m[1].clone())
            .or_else(|| match_one_of(&page_html, &[r#""bvid":"(BV\w+)""#]).map(|m| m[1].clone()))
            .ok_or_else(|| LuxError::Extractor("Unable to locate Bilibili BVID".to_string()))?;

        let cid = match_one_of(&page_html, &[r#""cid":(\d+)"#])
            .map(|m| m[1].clone())
            .ok_or_else(|| LuxError::Extractor("Unable to locate Bilibili CID".to_string()))?;

        // Request playurl API
        let api_url = format!(
            "https://api.bilibili.com/x/player/playurl?bvid={}&cid={}&qn=120&fnval=4048&fnver=0&fourk=1",
            bvid, cid
        );

        let play_resp: PlayUrlResponse = client
            .get(&api_url)
            .headers(headers)
            .send()
            .await?
            .json()
            .await?;

        if play_resp.code != 0 {
            return Err(LuxError::Extractor(format!(
                "Bilibili API error: code {}, msg: {}",
                play_resp.code, play_resp.message
            )));
        }

        let mut video_data = VideoData::new(url, "bilibili", page_title);
        let mut streams = HashMap::new();

        if let Some(data) = play_resp.data {
            if let Some(dash) = data.dash {
                // Process DASH streams
                for v in dash.video {
                    let stream_id = format!("{}", v.id);
                    let quality_label = match v.id {
                        127 => "8K Ultra HD",
                        120 => "4K Ultra HD",
                        116 => "1080P 60fps",
                        80 => "1080P High Definition",
                        64 => "720P",
                        32 => "480P",
                        16 => "360P",
                        _ => "Standard",
                    };

                    let video_url = v.base_url.or_else(|| v.backup_url.and_then(|b| b.into_iter().next()));
                    if let Some(v_url) = video_url {
                        let mut stream = Stream::new(&stream_id, quality_label);
                        stream.parts.push(Part::new(v_url, v.bandwidth.unwrap_or(0), "mp4"));

                        // If audio exists, attach audio part for muxing
                        if let Some(ref audios) = dash.audio {
                            if let Some(best_audio) = audios.first() {
                                if let Some(ref a_url) = best_audio.base_url {
                                    stream.parts.push(Part::new(a_url.clone(), best_audio.bandwidth.unwrap_or(0), "m4a"));
                                    stream.need_mux = true;
                                }
                            }
                        }

                        stream.ext = "mp4".to_string();
                        streams.insert(stream_id, stream);
                    }
                }
            } else if let Some(durls) = data.durl {
                // Process legacy durl parts
                let mut stream = Stream::new("default", "Standard");
                for d in durls {
                    stream.parts.push(Part::new(d.url, d.size, "mp4"));
                }
                stream.ext = "mp4".to_string();
                streams.insert("default".to_string(), stream);
            }
        }

        // Attach danmaku caption
        let danmaku_url = format!("https://api.bilibili.com/x/v1/dm/list.so?oid={}", cid);
        video_data.captions.insert(
            "danmaku".to_string(),
            CaptionPart {
                part: Part::new(danmaku_url, 0, "xml"),
                lang: Some("zh-CN".to_string()),
            },
        );

        video_data.streams = streams;
        Ok(vec![video_data])
    }
}
