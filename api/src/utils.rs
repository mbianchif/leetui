use htmd::HtmlToMarkdown;

use crate::error::{LeetCodeErr, Result};

pub fn convert_to_markdown(html: &str) -> Result<String> {
    let converter = HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style"])
        .build();

    converter
        .convert(html)
        .map_err(|_| LeetCodeErr::Api("Received invalid HTML".into()))
}
