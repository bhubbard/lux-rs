use scraper::{Html, Selector};

/// Parses an HTML string into a queryable document.
pub fn get_doc(html: &str) -> Html {
    Html::parse_document(html)
}

/// Extracts title following the precedence: `meta[property="og:title"]` -> `<title>` -> `<h1>`.
pub fn get_title(html: &str) -> String {
    let doc = get_doc(html);

    // 1. Try og:title
    if let Ok(og_selector) = Selector::parse(r#"meta[property="og:title"], meta[name="og:title"]"#) {
        if let Some(el) = doc.select(&og_selector).next() {
            if let Some(content) = el.value().attr("content") {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }

    // 2. Try <title>
    if let Ok(title_selector) = Selector::parse("title") {
        if let Some(el) = doc.select(&title_selector).next() {
            let text = el.text().collect::<Vec<_>>().join("");
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    // 3. Try <h1>
    if let Ok(h1_selector) = Selector::parse("h1") {
        if let Some(el) = doc.select(&h1_selector).next() {
            let text = el.text().collect::<Vec<_>>().join("");
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    "untitled".to_string()
}

/// Extracts image URLs from the document matching an optional CSS class filter.
pub fn get_images(html: &str, img_class: Option<&str>) -> (String, Vec<String>) {
    let doc = get_doc(html);
    let title = get_title(html);
    let mut urls = Vec::new();

    let selector_str = match img_class {
        Some(cls) => format!("img.{}", cls),
        None => "img".to_string(),
    };

    if let Ok(sel) = Selector::parse(&selector_str) {
        for el in doc.select(&sel) {
            if let Some(src) = el.value().attr("src") {
                urls.push(src.to_string());
            } else if let Some(data_src) = el.value().attr("data-src") {
                urls.push(data_src.to_string());
            }
        }
    }

    (title, urls)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_title_priority() {
        let og_html = r#"<html><head><meta property="og:title" content="你的名字。"></head><body>hello</body></html>"#;
        assert_eq!(get_title(og_html), "你的名字。");

        let title_html = "<html><head><title>hello</title></head><body>hello</body></html>";
        assert_eq!(get_title(title_html), "hello");

        let h1_html = "<html><head></head><body><h1>  aa </h1></body></html>";
        assert_eq!(get_title(h1_html), "aa");
    }

    #[test]
    fn test_get_images() {
        let html = r#"<html><head><title>gallery</title></head><body><img class="photo" src="pic1.jpg" /><img class="thumb" src="pic2.jpg" /></body></html>"#;
        let (title, photos) = get_images(html, Some("photo"));
        assert_eq!(title, "gallery");
        assert_eq!(photos, vec!["pic1.jpg"]);
    }
}
