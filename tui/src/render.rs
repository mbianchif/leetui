use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
};

use crate::app::App;

pub fn user_profile(f: &mut Frame, rect: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(45)])
        .split(rect);

    let Some(ref user_data) = app.user_data else {
        let span = Span::styled("OFFLINE", Style::default().fg(Color::Rgb(255, 45, 85)));
        let paragraph = Paragraph::new(span).alignment(Alignment::Right);
        f.render_widget(paragraph, chunks[1]);
        return;
    };

    if let Some(ref user) = user_data.matched_user {
        let style = Style::default().fg(Color::White).bold();
        let span = Span::styled(user.username.to_uppercase(), style);
        f.render_widget(Paragraph::new(span), chunks[0]);
    }
}

pub fn search_bar(f: &mut Frame, rect: Rect, _app: &mut App) {
    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::Rgb(100, 100, 100)));

    let search_text = Paragraph::new("  Type '/' to search...")
        .style(Style::default().fg(Color::Gray))
        .block(block);

    f.render_widget(search_text, rect);
}

pub fn problem_list(f: &mut Frame, rect: Rect, app: &mut App) {
    let header_style = Style::default().fg(Color::Rgb(100, 100, 100)).bold();
    let header = Row::new(vec!["ID", "TITLE", "DIFFICULTY", "STATUS"])
        .style(header_style)
        .height(1)
        .bottom_margin(0);

    let rows = app.problems.iter().enumerate().map(|(i, p)| {
        let bg = if i % 2 == 0 {
            Color::Rgb(50, 50, 50)
        } else {
            Color::Reset
        };

        let row_style = Style::default().bg(bg).fg(Color::Gray);

        Row::new(vec![
            format!(" {}", p.frontend_question_id),
            p.title.clone(),
            format!("{:?}", p.difficulty),
            format!("{:.1}%", p.ac_rate),
        ])
        .style(row_style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Min(30),
            Constraint::Length(10),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .highlight_symbol("▎");

    f.render_stateful_widget(table, rect, &mut app.table_state);
}

pub fn home_controls(f: &mut Frame, rect: Rect, _app: &mut App) {
    let keys_style = Style::default().fg(Color::Gray);
    let desc_style = Style::default().fg(Color::DarkGray);

    let current_keys = Line::from(vec![
        Span::styled(" q ", keys_style),
        Span::styled("QUIT  ", desc_style),
        Span::styled(" jk ", keys_style),
        Span::styled("MOVE  ", desc_style),
        Span::styled(" / ", keys_style),
        Span::styled("SEARCH  ", desc_style),
        Span::styled(" enter ", keys_style),
        Span::styled("SELECT ", desc_style),
    ]);

    let help = Paragraph::new(current_keys).alignment(Alignment::Center);

    f.render_widget(help, rect);
}
