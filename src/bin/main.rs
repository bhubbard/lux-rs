use clap::Parser;
use lux_rs::{DownloadOptions, Downloader, ExtractOptions, ExtractorRegistry, Result};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "lux")]
#[command(author = "Brandon Hubbard <brandon@brandonhubbard.com>")]
#[command(version = "0.0.1")]
#[command(about = "Blazingly fast, lightweight media downloader written in pure safe Rust", long_about = None)]
struct Cli {
    /// URLs to download or inspect
    #[arg(required = true)]
    urls: Vec<String>,

    /// Show stream information only without downloading
    #[arg(short = 'i', long = "info")]
    info: bool,

    /// Specify the stream ID to download
    #[arg(short = 's', long = "stream")]
    stream: Option<String>,

    /// Output directory for downloaded files
    #[arg(short = 'o', long = "output-path", default_value = ".")]
    output_path: PathBuf,

    /// Specify custom output file name (excluding extension)
    #[arg(short = 'O', long = "output-name")]
    output_name: Option<String>,

    /// Custom HTTP cookies string
    #[arg(short = 'c', long = "cookie")]
    cookie: Option<String>,

    /// Download entire playlist if applicable
    #[arg(short = 'p', long = "playlist")]
    playlist: bool,

    /// Do not download captions or danmaku subtitles
    #[arg(short = 'C', long = "no-caption")]
    no_caption: bool,

    /// Number of concurrent download connections
    #[arg(long = "threads", default_value = "4")]
    threads: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let registry = ExtractorRegistry::new();

    let extract_opts = ExtractOptions {
        cookie: cli.cookie.clone(),
        playlist: cli.playlist,
        episode_title_only: false,
    };

    let download_opts = DownloadOptions {
        output_dir: cli.output_path,
        output_name: cli.output_name,
        info_only: cli.info,
        stream_id: cli.stream,
        concurrency: cli.threads,
        retry_times: 3,
        referer: None,
        user_agent: None,
        download_captions: !cli.no_caption,
    };

    let downloader = Downloader::new(download_opts);

    for url in &cli.urls {
        println!("==> Extracting: {}", url);
        match registry.extract(url, &extract_opts).await {
            Ok(videos) => {
                for video in videos {
                    if let Err(e) = downloader.download(&video).await {
                        eprintln!("Error downloading {}: {}", video.title, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to extract {}: {}", url, e);
            }
        }
    }

    Ok(())
}
