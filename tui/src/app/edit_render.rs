use api::Question;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, Padding, Paragraph, Wrap},
};

use crate::app::{App, Multiplexer};

/// The primary color used for highlights (Mint Green)
const MINT: Color = Color::Rgb(0, 255, 150);
const DARK_GRAY: Color = Color::Rgb(100, 100, 100);
const BG_ALT: Color = Color::Rgb(50, 50, 50);

pub fn loading<M: Multiplexer>(f: &mut Frame, rect: Rect, _app: &App<M>) {
    let text = "⠋ FETCHING PROBLEM DETAILS...".fg(MINT);
    f.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::new().padding(ratatui::widgets::Padding::vertical(2))),
        rect,
    );
}

pub fn description(f: &mut Frame, rect: Rect, q: &Question) {
    let block = Block::default()
        .borders(Borders::all())
        .title(format!(" {} ", q.title))
        .border_style(Style::default().fg(DARK_GRAY))
        .padding(Padding::horizontal(1));

    let paragraph = Paragraph::new(q.content.to_string())
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, rect);
}

pub fn test_cases(f: &mut Frame, rect: Rect, q: &Question) {
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .title(" EXAMPLE TEST CASES ")
        .title_style(Style::default().fg(DARK_GRAY))
        .border_style(Style::default().fg(DARK_GRAY));

    let content = q
        .example_testcases
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("\n");

    f.render_widget(
        Paragraph::new(content).block(block).fg(Color::DarkGray),
        rect,
    );
}

pub fn language_selector<M: Multiplexer>(f: &mut Frame, rect: Rect, app: &mut App<M>) {
    let q = app.question.as_ref().unwrap();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" SELECT LANGUAGE ")
        .border_style(Style::default().fg(MINT));

    let items: Vec<ListItem> = q
        .code_snippets
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i % 2 == 0 {
                Style::default().bg(BG_ALT)
            } else {
                Style::default()
            };
            ListItem::new(format!("  {}", s.lang)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_spacing(HighlightSpacing::Always)
        .highlight_symbol("▎".fg(MINT))
        .highlight_style(Style::default().bg(Color::Rgb(60, 60, 60)).fg(MINT).bold());

    f.render_stateful_widget(list, rect, &mut app.list_state);
}

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
