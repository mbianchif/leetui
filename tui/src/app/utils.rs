pub fn html_to_markdown(html: &str) -> String {
    let cleaned_html = html
        .replace("<sup>", "^")
        .replace("</sup>", "")
        .replace("<sub>", "_")
        .replace("</sub>", "")
        .replace("&nbsp;", " ");

    html2md::parse_html(&cleaned_html)
}
