# TODO: `lux-rs` ⚡️🦀

A fast, lightweight media downloader written in pure Rust, inspired by [iawia002/lux](https://github.com/iawia002/lux) (the popular Go-based video download utility).

---

## 🎯 Mission & Goals

- **Ultra-Fast & Zero Runtime Dependencies**: Single static binary with sub-5ms cold-start time and minimal CPU/memory footprint.
- **First-Class Asian & Global Media Support**: Parity with Bilibili, Douyin/TikTok, YouTube, Weibo, Youku, and iQiyi.
- **Danmaku (Bullet Comments) Engine**: Native download of XML/protobuf danmaku comments and conversion into SubStation Alpha (`.ass`) format for player overlays.
- **Multi-Part Stream Concat**: Seamless download and concatenation of multi-segment FLV/MP4 streams.

---

## 🏗️ Crates Architecture Plan

- [ ] `lux-core`: URL routing, extractor registration, stream selection models (`Stream`, `Part`, `Format`), and download progress hooks.
- [ ] `lux-extractors`: Clean, lightweight site extractors focusing on high-speed JSON APIs.
- [ ] `lux-danmaku`: Parser for Bilibili XML/protobuf danmaku and converter to styled `.ass` subtitles.
- [ ] `lux-net`: High-concurrency chunk downloader with automatic resume, HTTP/2 multiplexing, and custom cookie handling.
- [ ] `lux-cli`: Clean terminal interface with interactive stream format selection and batch downloading.

---

## 📋 Implementation Checklist

### Phase 1: Core Architecture & Extractor Harness
- [ ] Implement `VideoData`, `StreamData`, and `PartData` structures:
  ```rust
  pub struct VideoData {
      pub site: String,
      pub title: String,
      pub streams: HashMap<String, StreamData>,
  }
  ```
- [ ] Design lightweight `Extractor` trait optimized for minimal async overhead.
- [ ] Implement URL router using pattern matching.
- [ ] Cookie parser supporting Netscape format and direct browser cookies.

### Phase 2: Danmaku & Subtitle Engine
- [ ] Bilibili danmaku XML parser (`<d p="time,mode,size,color,...">text</d>`).
- [ ] Protobuf-based modern Bilibili segmented danmaku parser.
- [ ] Real-time converter to SubStation Alpha (`.ass`) subtitle files with collision detection and font styling.

### Phase 3: Priority Site Extractors
- [ ] **Bilibili (`bilibili.com`)**:
  - [ ] Multi-part video resolution (P1, P2, ...).
  - [ ] 4K/1080p60/HDR stream selection via `wbi` signing and session cookies.
  - [ ] Audio/video separate stream fetching.
  - [ ] Danmaku download flag (`-d`).
- [ ] **YouTube (`youtube.com`)**:
  - [ ] Adaptive format extraction and direct MP4/WebM links.
- [ ] **Douyin / TikTok (`douyin.com`, `tiktok.com`)**:
  - [ ] Watermark-free video extraction.
- [ ] **Weibo & Xiaohongshu**:
  - [ ] Short-form video extraction and image set downloading.

### Phase 4: Download Engine & Segment Concat
- [ ] Multi-threaded segment downloader using `tokio` and `reqwest`.
- [ ] Automatic download resuming using HTTP `Range` requests and local `.part` markers.
- [ ] Lossless stream concatenation for multi-part video pieces without full re-encoding.
- [ ] Interactive CLI selection prompt (`inquire` or `dialoguer`) for picking resolution/format.

### Phase 5: CLI & Packaging
- [ ] Fast CLI arguments matching `lux` flags (`-i` info, `-p` playlist, `-c` cookie, `-f` format).
- [ ] JSON output flag (`-j`) for programmatic script integration.
- [ ] Cross-compilation targets: macOS (Apple Silicon / Intel), Linux (x86_64, aarch64, musl), and Windows.
- [ ] WebAssembly / WASI compilation target for running in Cloudflare Workers and browser extensions.

### Phase 6: Benchmarks & Parity Tests
- [ ] Startup latency benchmark comparing `lux-rs` vs Go `lux` vs Python `yt-dlp`.
- [ ] Memory footprint audit during 10-thread parallel downloads.
- [ ] End-to-end integration tests with recorded HTTP replay fixtures.
