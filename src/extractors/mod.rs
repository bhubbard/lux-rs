pub mod bilibili;
pub mod douyin;
pub mod universal;
pub mod youtube;

use crate::error::{LuxError, Result};
use crate::types::VideoData;
use crate::utils::{extract_domain, match_one_of};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct ExtractOptions {
    pub cookie: Option<String>,
    pub playlist: bool,
    pub episode_title_only: bool,
}

#[async_trait]
pub trait Extractor: Send + Sync {
    async fn extract(&self, url: &str, options: &ExtractOptions) -> Result<Vec<VideoData>>;
}

pub struct ExtractorRegistry {
    extractors: HashMap<String, Arc<dyn Extractor>>,
    fallback: Arc<dyn Extractor>,
}

impl Default for ExtractorRegistry {
    fn default() -> Self {
        let mut reg = Self {
            extractors: HashMap::new(),
            fallback: Arc::new(universal::UniversalExtractor::default()),
        };
        reg.register("bilibili", Arc::new(bilibili::BilibiliExtractor::default()));
        reg.register("youtube", Arc::new(youtube::YouTubeExtractor::default()));
        reg.register("youtu", Arc::new(youtube::YouTubeExtractor::default()));
        reg.register("douyin", Arc::new(douyin::DouyinExtractor::default()));
        reg.register("tiktok", Arc::new(douyin::DouyinExtractor::default()));
        reg
    }
}

impl ExtractorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, domain: &str, extractor: Arc<dyn Extractor>) {
        self.extractors.insert(domain.to_lowercase(), extractor);
    }

    pub fn resolve_url(&self, input_url: &str) -> std::result::Result<(String, String), crate::error::LuxError> {
        let trimmed = input_url.trim();

        // 1. Resolve shortlinks like av170001, BV1xx411c7mD, ep198061
        if let Some(m) = match_one_of(trimmed, &[r"^(av|BV|ep)\w+"]) {
            let prefix = &m[1];
            let base = match prefix.as_str() {
                "ep" => "https://www.bilibili.com/bangumi/play/",
                _ => "https://www.bilibili.com/video/",
            };
            Ok((format!("{}{}", base, trimmed), "bilibili".to_string()))
        } else {
            let parsed = url::Url::parse(trimmed)?;
            let host = parsed.host_str().unwrap_or("");
            let domain = extract_domain(host);
            Ok((trimmed.to_string(), domain))
        }
    }

    pub async fn extract(&self, input_url: &str, options: &ExtractOptions) -> Result<Vec<VideoData>> {
        let (resolved_url, target_domain) = self.resolve_url(input_url)?;

        let extractor = self
            .extractors
            .get(&target_domain.to_lowercase())
            .cloned()
            .unwrap_or_else(|| self.fallback.clone());

        let mut data_list = extractor.extract(&resolved_url, options).await?;
        for data in data_list.iter_mut() {
            data.fill_up_streams_data();
        }

        if data_list.is_empty() {
            return Err(LuxError::NoStreamsFound(resolved_url));
        }

        Ok(data_list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortlink_bilibili_resolution() {
        let reg = ExtractorRegistry::new();
        let (url, domain) = reg.resolve_url("BV1xx411c7mD").expect("resolve BV");
        assert_eq!(url, "https://www.bilibili.com/video/BV1xx411c7mD");
        assert_eq!(domain, "bilibili");

        let (url, domain) = reg.resolve_url("ep198061").expect("resolve ep");
        assert_eq!(url, "https://www.bilibili.com/bangumi/play/ep198061");
        assert_eq!(domain, "bilibili");

        let (url, domain) = reg.resolve_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ").expect("resolve yt");
        assert_eq!(domain, "youtube");
        assert_eq!(url, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    }
}
