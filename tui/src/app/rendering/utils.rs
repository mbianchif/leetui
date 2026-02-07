use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Clear, Paragraph},
};

pub fn markdown_to_text(md: &str) -> Text<'_> {
    md.into()
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn render_empty_background(frame: &mut Frame, area: Rect, text: &str) {
    let style = Style::default().fg(Color::DarkGray).dim().bold();
    let buf = frame.buffer_mut();

    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if (x + y) & 3 == 0 {
                buf[(x, y)].set_symbol("╱").set_style(style);
            }
        }
    }

    let text_len = text.len();

    let h_padding = 1;
    let area_w = text_len as u16 + h_padding;
    let area_h = 3;

    let x = area.x + area.width.saturating_sub(area_w) / 2;
    let y = area.y + area.height.saturating_sub(area_h) / 2;

    let message_area = Rect::new(x, y, area_w, area_h);
    frame.render_widget(Clear, message_area);

    let paragraph = Paragraph::new(text.fg(Color::DarkGray).dim().bold());
    frame.render_widget(paragraph, Rect::new(x + h_padding, y + 1, area_w, 1));
}
