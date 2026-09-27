use crate::danmaku::{danmaku_to_ass, parse_bilibili_xml};
use crate::error::{LuxError, Result};
use crate::types::{Stream, VideoData};
use crate::utils::{format_size, sanitize_filename};
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::{HeaderMap, HeaderValue, RANGE, REFERER, USER_AGENT};
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub output_dir: PathBuf,
    pub output_name: Option<String>,
    pub info_only: bool,
    pub stream_id: Option<String>,
    pub concurrency: usize,
    pub retry_times: usize,
    pub referer: Option<String>,
    pub user_agent: Option<String>,
    pub download_captions: bool,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("."),
            output_name: None,
            info_only: false,
            stream_id: None,
            concurrency: 4,
            retry_times: 3,
            referer: None,
            user_agent: None,
            download_captions: true,
        }
    }
}

pub struct Downloader {
    options: DownloadOptions,
    client: reqwest::Client,
}

impl Downloader {
    pub fn new(options: DownloadOptions) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_default();
        Self { options, client }
    }

    /// Prints formatted stream information table matching `lux -i` output.
    pub fn print_info(&self, data: &VideoData) {
        println!();
        println!(" Site:      {}", data.site);
        println!(" Title:     {}", data.title);
        println!(" Type:      {:?}", data.media_type);
        println!(" Streams:   # Available qualities");

        let mut sorted_streams: Vec<(&String, &Stream)> = data.streams.iter().collect();
        sorted_streams.sort_by(|a, b| b.1.size.cmp(&a.1.size));

        for (id, stream) in sorted_streams {
            let size_str = if stream.size > 0 {
                format_size(stream.size)
            } else {
                "Unknown size".to_string()
            };
            println!(
                "    [--stream {}] -------------------",
                id
            );
            println!("    Quality:         {}", stream.quality);
            println!("    Size:            {}", size_str);
            println!("    Extension:       {}", stream.ext);
            println!("    Parts:           {}", stream.parts.len());
            if stream.need_mux {
                println!("    Need Mux:        true");
            }
            println!();
        }
    }

    /// Executes the full download pipeline.
    pub async fn download(&self, data: &VideoData) -> Result<PathBuf> {
        if self.options.info_only {
            self.print_info(data);
            return Ok(self.options.output_dir.clone());
        }

        let stream = data
            .select_stream(self.options.stream_id.as_deref())
            .ok_or_else(|| LuxError::NoStreamsFound(data.url.clone()))?;

        tokio::fs::create_dir_all(&self.options.output_dir).await?;

        let base_name = self
            .options
            .output_name
            .clone()
            .unwrap_or_else(|| sanitize_filename(&data.title));

        let final_filename = format!("{}.{}", base_name, stream.ext);
        let final_path = self.options.output_dir.join(&final_filename);

        println!("Downloading {} [{}]...", final_filename, stream.quality);

        // Download all parts
        let mut part_paths = Vec::new();
        for (idx, part) in stream.parts.iter().enumerate() {
            let part_name = if stream.parts.len() == 1 {
                format!("{}.download", final_filename)
            } else {
                format!("{}.part{}.download", final_filename, idx)
            };
            let part_dest = self.options.output_dir.join(part_name);
            self.download_part(&part.url, &part_dest, part.size, data.site.as_str()).await?;
            part_paths.push(part_dest);
        }

        // Assemble parts into final target file
        let mut out_file = File::create(&final_path).await?;
        for part_p in &part_paths {
            let mut in_file = File::open(part_p).await?;
            tokio::io::copy(&mut in_file, &mut out_file).await?;
            tokio::fs::remove_file(part_p).await.ok();
        }

        println!("✓ Download finished: {}", final_path.display());

        // Download captions/danmaku if available
        if self.options.download_captions {
            for (key, caption) in &data.captions {
                let caption_path = self.options.output_dir.join(format!("{}.{}.ass", base_name, key));
                if let Ok(resp) = self.client.get(&caption.part.url).send().await {
                    if let Ok(xml_str) = resp.text().await {
                        if let Ok(items) = parse_bilibili_xml(&xml_str) {
                            let ass = danmaku_to_ass(&items, &data.title);
                            tokio::fs::write(&caption_path, ass).await.ok();
                            println!("✓ Subtitles saved: {}", caption_path.display());
                        }
                    }
                }
            }
        }

        Ok(final_path)
    }

    async fn download_part(
        &self,
        url: &str,
        dest: &Path,
        expected_size: u64,
        site: &str,
    ) -> Result<()> {
        let mut headers = HeaderMap::new();
        let ua = self.options.user_agent.as_deref().unwrap_or(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
        );
        headers.insert(USER_AGENT, HeaderValue::from_str(ua).unwrap());

        if let Some(ref ref_url) = self.options.referer {
            headers.insert(REFERER, HeaderValue::from_str(ref_url).unwrap());
        } else if site == "bilibili" {
            headers.insert(REFERER, HeaderValue::from_static("https://www.bilibili.com"));
        }

        // Check existing file size for resume
        let existing_len = if dest.exists() {
            tokio::fs::metadata(dest).await.map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        if existing_len > 0 {
            headers.insert(RANGE, HeaderValue::from_str(&format!("bytes={}-", existing_len)).unwrap());
        }

        let resp = self.client.get(url).headers(headers).send().await?;

        let total_size = if existing_len > 0 {
            resp.content_length().map(|l| l + existing_len).unwrap_or(expected_size)
        } else {
            resp.content_length().unwrap_or(expected_size)
        };

        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_position(existing_len);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dest)
            .await?;

        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            pb.inc(chunk.len() as u64);
        }

        pb.finish_with_message("Done");
        Ok(())
    }
}
