use regex::Regex;

/// Matches the text against an array of regex patterns, returning captures of the first pattern that matches.
pub fn match_one_of(text: &str, patterns: &[&str]) -> Option<Vec<String>> {
    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(text) {
                let matches: Vec<String> = caps
                    .iter()
                    .filter_map(|m| m.map(|m| m.as_str().to_string()))
                    .collect();
                return Some(matches);
            }
        }
    }
    None
}

/// Matches all occurrences of a regex pattern in the text, returning all captured groups.
pub fn match_all(text: &str, pattern: &str) -> Vec<Vec<String>> {
    let mut results = Vec::new();
    if let Ok(re) = Regex::new(pattern) {
        for caps in re.captures_iter(text) {
            let row: Vec<String> = caps
                .iter()
                .filter_map(|m| m.map(|m| m.as_str().to_string()))
                .collect();
            results.push(row);
        }
    }
    results
}

/// Extracts the secondary domain name from a hostname (e.g. "www.bilibili.com" -> "bilibili").
pub fn extract_domain(host: &str) -> String {
    let clean_host = host.split(':').next().unwrap_or(host);
    let parts: Vec<&str> = clean_host.split('.').collect();
    if parts.len() >= 2 {
        parts[parts.len() - 2].to_string()
    } else {
        clean_host.to_string()
    }
}

/// Formats a byte size into human-readable format (KiB, MiB, GiB).
pub fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GiB", b / GB)
    } else if b >= MB {
        format!("{:.2} MiB", b / MB)
    } else if b >= KB {
        format!("{:.2} KiB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

/// Sanitizes a file name for cross-platform filesystem compatibility.
pub fn sanitize_filename(name: &str) -> String {
    let invalid_chars = ['/', '\\', '?', '%', '*', ':', '|', '"', '<', '>', '.', '\r', '\n'];
    let mut res: String = name
        .chars()
        .map(|c| if invalid_chars.contains(&c) { '_' } else { c })
        .collect();
    res = res.trim().to_string();
    if res.is_empty() {
        "lux_download".to_string()
    } else {
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_one_of() {
        let text = "hello12345";
        let patterns = vec![r"aaa(\d+)", r"hello(\d+)"];
        let matched = match_one_of(text, &patterns).expect("should match");
        assert_eq!(matched[0], "hello12345");
        assert_eq!(matched[1], "12345");

        let no_match = match_one_of(text, &[r"aaa(\d+)", r"bbb(\d+)"]);
        assert!(no_match.is_none());
    }

    #[test]
    fn test_match_all() {
        let text = "hello12345hello123";
        let matches = match_all(text, r"hello(\d+)");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], vec!["hello12345", "12345"]);
        assert_eq!(matches[1], vec!["hello123", "123"]);
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("www.bilibili.com"), "bilibili");
        assert_eq!(extract_domain("youtube.com:443"), "youtube");
        assert_eq!(extract_domain("v.douyin.com"), "douyin");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(2048), "2.00 KiB");
        assert_eq!(format_size(1048576 * 5), "5.00 MiB");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("a/b:c*d?e"), "a_b_c_d_e");
        assert_eq!(sanitize_filename("   "), "lux_download");
    }
}
