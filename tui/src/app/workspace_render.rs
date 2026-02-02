use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
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
    } else if app.new_file_input.is_empty() {
        Span::from(" Enter new file name (with extension)".fg(Color::Gray))
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

pub fn controls(f: &mut Frame, rect: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(0)])
        .split(rect);

    if let Some(..) = app.error_message {
        let err_text = format!("! ERROR");
        let style = Style::default().fg(Color::Rgb(255, 45, 85));
        let span = Span::styled(err_text, style);
        let paragraph = Paragraph::new(span);
        f.render_widget(paragraph, chunks[0]);
    } else if app.is_loading {
        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let frame = spinner[app.spinner_index % spinner.len()];
        let loading_text = format!(" {frame} FETCHING");
        let style = Style::default().fg(Color::Rgb(0, 255, 150));
        let span = Span::styled(loading_text, style);
        let paragraph = Paragraph::new(span);
        f.render_widget(paragraph, chunks[0]);
    }

    let keys_style = Style::default().fg(Color::Gray);
    let desc_style = Style::default().fg(Color::DarkGray);

    let current_keys = Line::from(vec![
        Span::styled("esc ", keys_style),
        Span::styled("EXIT   ", desc_style),
        Span::styled("j/k ", keys_style),
        Span::styled("MOVE   ", desc_style),
        Span::styled("enter ", keys_style),
        Span::styled("CONFIRM   ", desc_style),
    ]);

    let help = Paragraph::new(current_keys).alignment(Alignment::Center);
    f.render_widget(help, rect);
}
