use api::{Difficulty, ProblemStatus};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Styled, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, HighlightSpacing, Paragraph, Row, Table},
};

use crate::app::{App, HomeInputState, Multiplexer};

/// Renders the user's profile into the given frame.
///
/// # Arguments
/// * `f` - The frame to render the widgets.
/// * `rect` - A rectangle to insert widgets.
/// * `app` - The main application.
pub fn user_profile<M: Multiplexer>(f: &mut Frame, rect: Rect, app: &mut App<M>) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(45)])
        .split(rect);

    match app.user_data {
        Some(ref user) => {
            let style = Style::default().fg(Color::White).bold();
            let span = Span::styled(user.username.to_uppercase(), style);
            f.render_widget(Paragraph::new(span), chunks[0]);
        }
        None => {
            let span = Span::styled("OFFLINE", Style::default().fg(Color::Rgb(255, 45, 85)));
            let paragraph = Paragraph::new(span).alignment(Alignment::Right);
            f.render_widget(paragraph, chunks[1]);
        }
    };
}

/// Renders the search bar into the given frame.
///
/// # Arguments
/// * `f` - The frame to render the widgets.
/// * `rect` - A rectangle to insert widgets.
/// * `app` - The main application.
pub fn search_bar<M: Multiplexer>(f: &mut Frame, rect: Rect, app: &mut App<M>) {
    let color = if matches!(app.input_mode, HomeInputState::Searching) {
        Color::Rgb(0, 255, 150)
    } else {
        Color::Rgb(100, 100, 100)
    };

    let border_style = Style::default().fg(color);
    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(border_style);

    let display_text = if app.input.is_empty() && matches!(app.input_mode, HomeInputState::Normal) {
        "  Type '/' to search...".fg(Color::Gray)
    } else {
        format!("  {}", app.input)
            .fg(Color::Rgb(0, 255, 150))
            .bold()
    };

    f.render_widget(Paragraph::new(display_text).block(block), rect);

    if matches!(app.input_mode, HomeInputState::Searching) {
        f.set_cursor_position((rect.x + app.input.len() as u16 + 3, rect.y));
    }
}

/// Renders the daily challenge section into the given frame.
///
/// # Arguments
/// * `f` - The frame to render the widgets.
/// * `rect` - A rectangle to insert widgets.
/// * `app` - The main application.
pub fn daily_challenge<M: Multiplexer>(f: &mut Frame, rect: Rect, app: &App<M>) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" DAILY CHALLENGE ")
        .border_style(Style::default().fg(Color::Rgb(255, 160, 80)));

    match &app.daily_challenge {
        Some(p) => {
            let style = Style::default();
            let title_style = match p.status {
                Some(ProblemStatus::Accepted) => style.fg(Color::DarkGray).italic(),
                Some(ProblemStatus::Attempted) => style.fg(Color::Rgb(255, 160, 80)),
                _ => style.fg(Color::White),
            };

            let diff_style = match p.difficulty {
                Difficulty::Easy => Style::default().fg(Color::White),
                Difficulty::Medium => Style::default().fg(Color::Gray),
                Difficulty::Hard => Style::default().fg(Color::DarkGray),
            };

            let row = Row::new(vec![
                Cell::from(format!(" {}", p.frontend_question_id)).fg(Color::DarkGray),
                Cell::from(p.title.clone()).style(title_style),
                Cell::from(format!("{:?}", p.difficulty)).style(diff_style),
                Cell::from(format!("{:.1}%", p.ac_rate)).fg(Color::DarkGray),
            ])
            .height(1);

            let table = Table::new(
                vec![row],
                [
                    Constraint::Length(6),
                    Constraint::Min(30),
                    Constraint::Length(10),
                    Constraint::Length(6),
                ],
            )
            .block(block);

            f.render_widget(table, rect);
        }
        None => {
            f.render_widget(Paragraph::new(" Loading...").block(block), rect);
        }
    }
}

/// Renders the problem list into the given frame.
///
/// # Arguments
/// * `f` - The frame to render the widgets.
/// * `rect` - A rectangle to insert widgets.
/// * `app` - The main application.
pub fn problem_list<M: Multiplexer>(f: &mut Frame, rect: Rect, app: &mut App<M>) {
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

        let is_premium_user = app
            .user_status
            .as_ref()
            .map(|user| user.is_premium)
            .unwrap_or_default();

        let is_locked = !is_premium_user && p.paid_only;

        let style = Style::default();
        let title_style = match p.status {
            Some(ProblemStatus::Accepted) => style.fg(Color::DarkGray).italic(),
            _ if is_locked => style.fg(Color::Rgb(80, 80, 80)),
            Some(ProblemStatus::Attempted) => style.fg(Color::Rgb(255, 160, 80)),
            _ => style.fg(Color::White),
        };

        let title_content = if p.paid_only {
            format!("{} ", p.title)
        } else {
            p.title.clone()
        };

        let style = Style::default();
        let diff_style = match p.difficulty {
            Difficulty::Easy => style.fg(Color::White),
            Difficulty::Medium => style.fg(Color::Gray),
            Difficulty::Hard => style.fg(Color::DarkGray),
        };

        let row_style = Style::default().bg(bg);

        Row::new(vec![
            Cell::from(format!(" {}", p.frontend_question_id).fg(Color::DarkGray)),
            Cell::from(title_content).style(title_style),
            Cell::from(format!("{:?}", p.difficulty)).style(diff_style),
            Cell::from(format!("{:.1}%", p.ac_rate)).fg(Color::DarkGray),
        ])
        .style(row_style)
    });

    let highligh_style = Style::default()
        .bg(Color::Rgb(60, 60, 60))
        .fg(Color::Rgb(0, 255, 150))
        .bold();

    let mut table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Min(30),
            Constraint::Length(10),
            Constraint::Length(7),
        ],
    )
    .header(header)
    .highlight_spacing(HighlightSpacing::Always)
    .highlight_symbol("▎".set_style(Color::Rgb(100, 100, 100)));

    if matches!(app.input_mode, HomeInputState::Normal) {
        table = table.row_highlight_style(highligh_style);
    }

    f.render_stateful_widget(table, rect, &mut app.table_state);
}

/// Renders the controls into the given frame.
///
/// # Arguments
/// * `f` - The frame to render the widgets.
/// * `rect` - A rectangle to insert widgets.
/// * `app` - The main application.
pub fn controls<M: Multiplexer>(f: &mut Frame, rect: Rect, app: &mut App<M>) {
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

    let current_keys = match app.input_mode {
        HomeInputState::Normal => Line::from(vec![
            Span::styled("esc ", keys_style),
            Span::styled("QUIT   ", desc_style),
            Span::styled("d ", keys_style),
            Span::styled("DAILY  ", desc_style),
            Span::styled("jk ", keys_style),
            Span::styled("MOVE   ", desc_style),
            Span::styled("enter ", keys_style),
            Span::styled("SELECT  ", desc_style),
        ]),
        HomeInputState::Searching => Line::from(vec![
            Span::styled("esc ", keys_style),
            Span::styled("CANCEL   ", desc_style),
            Span::styled("enter ", keys_style),
            Span::styled("CONFIRM   ", desc_style),
        ]),
    };

    let help = Paragraph::new(current_keys).alignment(Alignment::Center);
    f.render_widget(help, rect);
}
