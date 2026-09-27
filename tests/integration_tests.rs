use lux_rs::{
    danmaku_to_ass, parse_bilibili_xml, DownloadOptions, Downloader, Part, Stream, VideoData,
};
use tempfile::tempdir;

#[tokio::test]
async fn test_video_data_stream_selection_and_fill() {
    let mut data = VideoData::new("https://example.com/video", "test_site", "Sample Video");
    let mut stream1080 = Stream::new("1080", "1080P FHD");
    stream1080.parts.push(Part::new("https://cdn.example.com/part1.flv", 10_000_000, "flv"));
    stream1080.parts.push(Part::new("https://cdn.example.com/part2.flv", 15_000_000, "flv"));

    let mut stream720 = Stream::new("720", "720P HD");
    stream720.parts.push(Part::new("https://cdn.example.com/720p.mp4", 12_000_000, "mp4"));

    data.streams.insert("1080".to_string(), stream1080);
    data.streams.insert("720".to_string(), stream720);

    data.fill_up_streams_data();

    // Verify 1080 stream auto-fill
    let s1080 = data.streams.get("1080").unwrap();
    assert_eq!(s1080.size, 25_000_000);
    assert_eq!(s1080.ext, "mp4"); // flv converted to mp4 container

    // Verify stream selection: highest size chosen when not specified
    let selected = data.select_stream(None).unwrap();
    assert_eq!(selected.id, "1080");

    // Verify explicit selection
    let explicit = data.select_stream(Some("720")).unwrap();
    assert_eq!(explicit.id, "720");
}

#[tokio::test]
async fn test_danmaku_full_conversion_pipeline() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <i>
        <d p="5.2,1,25,16777215,1600000000,0,user1,101">First Comment</d>
        <d p="12.8,4,25,65280,1600000001,0,user2,102">Bottom Sticky Green</d>
        <d p="2.1,5,30,16711680,1600000002,0,user3,103">Top Red Banner</d>
    </i>"#;

    let items = parse_bilibili_xml(xml).expect("must parse XML");
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].text, "Top Red Banner"); // Sorted by time (2.1s)
    assert_eq!(items[1].text, "First Comment");   // 5.2s
    assert_eq!(items[2].text, "Bottom Sticky Green"); // 12.8s

    let ass = danmaku_to_ass(&items, "Danmaku Episode 1");
    assert!(ass.contains("[V4+ Styles]"));
    assert!(ass.contains("Dialogue: 0,0:00:02.10,0:00:10.10"));
    assert!(ass.contains("Top Red Banner"));
    assert!(ass.contains("Bottom Sticky Green"));
}

#[tokio::test]
async fn test_downloader_info_only_mode() {
    let tmp = tempdir().unwrap();
    let mut data = VideoData::new("https://test.com/v", "mock", "Mock Title");
    let mut stream = Stream::new("default", "Standard 1080p");
    stream.parts.push(Part::new("https://test.com/v.mp4", 5_000_000, "mp4"));
    data.streams.insert("default".to_string(), stream);
    data.fill_up_streams_data();

    let dl = Downloader::new(DownloadOptions {
        output_dir: tmp.path().to_path_buf(),
        info_only: true,
        ..Default::default()
    });

    let res = dl.download(&data).await;
    assert!(res.is_ok());
}
