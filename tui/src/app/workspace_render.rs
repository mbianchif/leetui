use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::{App, app::WorkspaceState};

pub fn file_selector(f: &mut Frame, rect: Rect, app: &mut App) {
    let border_color = match app.workspace_state {
        WorkspaceState::FileSelector => Color::Rgb(0, 255, 150),
        WorkspaceState::NewFileMenu => Color::DarkGray,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" WORKSPACE FILES ")
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(rect);
    f.render_widget(&block, rect);

    if app.local_files.is_empty() {
        let style = Style::default().fg(Color::DarkGray).dim().bold();
        let buf = f.buffer_mut();

        for y in inner_area.top()..inner_area.bottom() {
            for x in inner_area.left()..inner_area.right() {
                if (x + y) & 7 == 0 {
                    buf[(x, y)].set_symbol("╱").set_style(style);
                }
            }
        }

        let text = " EMPTY WORKSPACE ";
        let text_len = text.len();

        let area_h = 3;
        let area_w = text_len as u16 + 2;

        let x = inner_area.x + inner_area.width.saturating_sub(area_w) / 2 - 1;
        let y = inner_area.y + inner_area.height.saturating_sub(area_h) / 2;

        let message_area = Rect::new(x, y, area_w, area_h);
        f.render_widget(Clear, message_area);

        let paragraph = Paragraph::new(text.fg(Color::DarkGray).dim().bold());
        f.render_widget(paragraph, Rect::new(x, y + 1, area_w, 1));
        return;
    }

    let items = app
        .local_files
        .iter()
        .map(|name| ListItem::new(format!(" {name}")));

    let fg_color = match app.workspace_state {
        WorkspaceState::FileSelector => Color::Rgb(0, 255, 150),
        WorkspaceState::NewFileMenu => Color::DarkGray,
    };

    let highlight_style = Style::default()
        .fg(fg_color)
        .bg(Color::Rgb(50, 50, 50))
        .bold();

    let list = List::new(items)
        .block(block)
        .highlight_style(highlight_style);

    f.render_stateful_widget(list, rect, &mut app.file_list_state);
}

pub fn file_creator(f: &mut Frame, rect: Rect, app: &mut App) {
    let is_active = matches!(app.workspace_state, WorkspaceState::NewFileMenu);

    let border_color = if is_active {
        Color::Rgb(0, 255, 150)
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .borders(Borders::all())
        .title(" CREATE FILE ")
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(rect);
    f.render_widget(block, rect);

    let input_content = if !is_active {
        Span::from(" Type 'n' to create a new file...".fg(Color::Gray))
    } else {
        Span::from(format!(" {}", app.new_file_input).fg(Color::Rgb(0, 255, 150)))
    };

    f.render_widget(Paragraph::new(input_content), inner_area);

    let lang_infer = match app.detected_language {
        Some(ref lang) if is_active => {
            let name = format!("{} ", lang.to_string().to_lowercase());
            name.fg(Color::Rgb(0, 255, 150)).bold()
        }
        None if is_active => ".? ".fg(Color::DarkGray),
        _ => "".into(),
    };

    f.render_widget(lang_infer.into_right_aligned_line(), inner_area);

    if is_active {
        f.set_cursor_position((
            inner_area.x + app.new_file_input.len() as u16 + 1,
            inner_area.y,
        ));
    }
}
