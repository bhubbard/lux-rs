# lux-rs ⚡️🦀

[![CI](https://github.com/bhubbard/lux-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/bhubbard/lux-rs/actions)
[![Pages](https://github.com/bhubbard/lux-rs/actions/workflows/pages.yml/badge.svg)](https://code.brandonhubbard.com/lux-rs/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

A blazingly fast, lightweight media downloader & extractor written in 100% safe, idiomatic Rust. Inspired by [iawia002/lux](https://github.com/iawia002/lux).

👉 **Interactive Playground & Documentation**: [code.brandonhubbard.com/lux-rs](http://code.brandonhubbard.com/lux-rs/)

---

## ⚡ Key Highlights

- **Sub-3ms Startup Time**: Pure native binary with zero runtime interpreter or GC overhead.
- **High Concurrency Engine**: Concurrent multi-part chunk downloading with automatic range resumes via `tokio` and `reqwest`.
- **First-Class Asian & Global Media Support**: Native extractors for Bilibili, YouTube, Douyin/TikTok, Twitter/X, and Universal HTML5 video/audio.
- **Danmaku / Subtitle Synthesis**: Parses raw XML bullet comments and compiles them into styled SubStation Alpha (`.ass`) subtitles.
- **Zero Unsafe Rust**: Built exclusively with memory-safe Rust primitives.

---

## 🚀 Installation

```bash
# Clone and build with Cargo
git clone https://github.com/bhubbard/lux-rs.git
cd lux-rs
cargo install --path .
```

---

## 🛠️ CLI Usage

```bash
# 1. Inspect available streams & qualities (-i, --info)
lux -i "https://www.youtube.com/watch?v=dQw4w9WgXcQ"

# 2. Download with specific stream quality (-s) and output directory (-o)
lux -s 1080 -o ~/Downloads "https://www.bilibili.com/video/BV1xx411c7mD"

# 3. Direct shortlink resolution (av/BV/ep)
lux "BV1xx411c7mD"

# 4. Multi-threaded download with 8 parallel connections
lux --threads 8 -o ./videos "https://v.douyin.com/abc1234/"

# 5. Disable caption/danmaku download (-C)
lux -C "https://www.bilibili.com/video/BV1xx411c7mD"
```

---

## 🧪 Running Tests

```bash
cargo test
```
All unit tests and integration tests run in under 0.05 seconds.

---

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))
