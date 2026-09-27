use crate::error::{LuxError, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

#[derive(Debug, Clone, PartialEq)]
pub struct DanmakuItem {
    pub time_secs: f64,
    pub mode: u8,
    pub font_size: u32,
    pub color_rgb: u32,
    pub timestamp: u64,
    pub pool: u8,
    pub user_hash: String,
    pub text: String,
}

impl DanmakuItem {
    pub fn parse_p_attribute(p_attr: &str, text: String) -> Option<Self> {
        let parts: Vec<&str> = p_attr.split(',').collect();
        if parts.is_empty() {
            return None;
        }

        let time_secs = parts.first()?.parse::<f64>().unwrap_or(0.0);
        let mode = parts.get(1).and_then(|s| s.parse::<u8>().ok()).unwrap_or(1);
        let font_size = parts.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(25);
        let color_rgb = parts.get(3).and_then(|s| s.parse::<u32>().ok()).unwrap_or(16777215);
        let timestamp = parts.get(4).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        let pool = parts.get(5).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
        let user_hash = parts.get(6).unwrap_or(&"").to_string();

        Some(Self {
            time_secs,
            mode,
            font_size,
            color_rgb,
            timestamp,
            pool,
            user_hash,
            text,
        })
    }
}

/// Parses raw Bilibili XML danmaku buffer into structured DanmakuItems.
pub fn parse_bilibili_xml(xml: &str) -> Result<Vec<DanmakuItem>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut items = Vec::new();
    let mut current_p = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"d" {
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            if attr.key.as_ref() == b"p" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_p = Some(val.to_string());
                                }
                            }
                        }
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                if let Some(p) = current_p.take() {
                    if let Ok(text) = e.unescape() {
                        if let Some(item) = DanmakuItem::parse_p_attribute(&p, text.to_string()) {
                            items.push(item);
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(LuxError::Other(format!("XML error: {:?}", e))),
            _ => {}
        }
        buf.clear();
    }

    items.sort_by(|a, b| a.time_secs.partial_cmp(&b.time_secs).unwrap());
    Ok(items)
}

/// Converts danmaku list into SubStation Alpha (.ass) format.
pub fn danmaku_to_ass(items: &[DanmakuItem], title: &str) -> String {
    let mut out = String::new();
    out.push_str("[Script Info]\n");
    out.push_str(&format!("Title: {}\n", title));
    out.push_str("ScriptType: v4.00+\n");
    out.push_str("Collisions: Normal\n");
    out.push_str("PlayResX: 1920\n");
    out.push_str("PlayResY: 1080\n\n");

    out.push_str("[V4+ Styles]\n");
    out.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
    out.push_str("Style: Default,Arial,36,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,1,0,2,20,20,20,1\n\n");

    out.push_str("[Events]\n");
    out.push_str("Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    for item in items {
        let start_time = format_ass_time(item.time_secs);
        let end_time = format_ass_time(item.time_secs + 8.0); // 8 second scroll duration
        // Mode 1: scroll R2L; Mode 4: bottom; Mode 5: top
        let bgr_color = ((item.color_rgb & 0xFF) << 16) | (item.color_rgb & 0x00FF00) | ((item.color_rgb >> 16) & 0xFF);
        let color_tag = format!("\\c&H{:06X}&", bgr_color);

        let effect_tag = match item.mode {
            4 => format!("{{\\an2{}}}", color_tag),
            5 => format!("{{\\an8{}}}", color_tag),
            _ => format!("{{\\move(1920, 100, -500, 100){}}}", color_tag),
        };

        out.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}{}\n",
            start_time, end_time, effect_tag, item.text
        ));
    }

    out
}

fn format_ass_time(seconds: f64) -> String {
    let total_secs = seconds.max(0.0);
    let hrs = (total_secs / 3600.0) as u32;
    let mins = ((total_secs % 3600.0) / 60.0) as u32;
    let secs = (total_secs % 60.0) as u32;
    let csecs = (total_secs.fract() * 100.0).round() as u32;
    format!("{}:{:02}:{:02}.{:02}", hrs, mins, secs, csecs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bilibili_xml_danmaku() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><i><d p="23.89,1,25,16777215,1482062772,0,a1b2c3d4,123456">Hello Danmaku!</d><d p="10.5,5,30,16711680,1482062773,0,e5f6,123457">Top Banner</d></i>"#;
        let items = parse_bilibili_xml(xml).expect("parse xml");
        assert_eq!(items.len(), 2);
        // Should be sorted by time
        assert_eq!(items[0].text, "Top Banner");
        assert_eq!(items[0].time_secs, 10.5);
        assert_eq!(items[1].text, "Hello Danmaku!");
        assert_eq!(items[1].time_secs, 23.89);

        let ass = danmaku_to_ass(&items, "Test Video");
        assert!(ass.contains("[Script Info]"));
        assert!(ass.contains("Dialogue: 0,0:00:10.50,0:00:18.50"));
        assert!(ass.contains("Top Banner"));
    }
}
