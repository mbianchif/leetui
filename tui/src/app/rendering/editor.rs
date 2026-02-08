use std::collections::HashSet;

use api::{Language, Question};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, HorizontalAlignment, Layout, Rect},
    style::{Color, Style, Styled, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Padding, Paragraph, Row, Table, Tabs, Wrap},
};

use super::utils;
use crate::app::{App, app::EditorState};

pub fn language_selector(f: &mut Frame, rect: Rect, app: &mut App) {
    let constraints: &[_] = match app.editor_state {
        EditorState::Description | EditorState::TestCases => &[
            Constraint::Length(3), // language selector
        ],
        _ => &[
            Constraint::Min(0),    // languages
            Constraint::Length(3), // language selector
        ],
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(rect);

    if matches!(app.editor_state, EditorState::SelectingLanguage) {
        render_language_grid(f, chunks[0], app);
        render_selected_language(f, chunks[1], app);
    } else {
        render_selected_language(f, chunks[0], app);
    }
}

fn render_language_grid(f: &mut Frame, rect: Rect, app: &mut App) {
    let question = app.question.as_ref().unwrap();

    let languages: Vec<_> = question
        .code_snippets
        .iter()
        .map(|snippet| &snippet.lang)
        .collect();

    let columns = 3;
    let rows = (languages.len() as f32 / columns as f32).ceil() as usize;

    let mut table_rows = Vec::new();
    for r in 0..rows {
        let mut row_cells = Vec::new();
        for c in 0..columns {
            let index = c * rows + r;

            if let Some(lang) = languages.get(index) {
                let is_selected = index == app.language_selection_index;

                let style = if is_selected {
                    Style::default().fg(Color::Rgb(0, 255, 150)).bold()
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let text = format!("  {lang}");
                let cell = Cell::from(text.set_style(style));
                row_cells.push(cell);
            } else {
                row_cells.push(Cell::from(""));
            }
        }

        table_rows.push(Row::new(row_cells).height(1));
    }

    let widths = vec![Constraint::Percentage(100 / columns as u16); columns];
    let table = Table::new(table_rows, widths).column_spacing(2);

    f.render_widget(table, rect);
}

fn render_selected_language(f: &mut Frame, rect: Rect, app: &mut App) {
    let color = if matches!(app.editor_state, EditorState::SelectingLanguage) {
        Color::Rgb(255, 160, 80)
    } else {
        Color::DarkGray
    };

    let block = Block::bordered()
        .title(" SELECTED LANGUAGE ")
        .border_style(Style::default().fg(color));

    let inner = block.inner(rect);
    f.render_widget(block, rect);

    let text = match app.selected_language {
        Some(ref lang) => format!(" {lang}").fg(color).bold(),
        None => " none".fg(Color::DarkGray),
    };

    f.render_widget(text, inner);
}

pub fn description(f: &mut Frame, rect: Rect, app: &mut App) {
    let question = app.question.as_ref().unwrap();
    let language_selector_offset = if matches!(app.editor_state, EditorState::SelectingLanguage) {
        (question.code_snippets.len() as f32 / 3.0).ceil() as u16
    } else {
        0
    };

    let block = Block::bordered()
        .title(format!(" {}. {} ", question.question_id, question.title))
        .title_alignment(HorizontalAlignment::Center);

    let md = utils::markdown_to_text(&question.content);
    let paragraph = Paragraph::new(md)
        .block(block)
        .wrap(Wrap { trim: true })
        .scroll((app.description_offset as u16 + language_selector_offset, 0));

    f.render_widget(paragraph, rect);
}

pub fn test_cases(frame: &mut Frame, area: Rect, app: &mut App) {
    let Some(question) = &app.question else {
        return;
    };

    let constraints: &[_] = match app.editor_state {
        EditorState::Description | EditorState::SelectingLanguage => &[
            Constraint::Length(3), // case selector
        ],
        _ => &[
            Constraint::Length(3), // case selector
            Constraint::Length(1), // padding
            Constraint::Min(0),    // case parameters
        ],
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    render_tabs(frame, chunks[0], app);

    if matches!(
        app.editor_state,
        EditorState::TestCases | EditorState::EditingTestCaseField
    ) {
        app.last_test_case_viewport_height = chunks[2].height;
        render_test_case_fields(frame, chunks[2], app, question);
    }
}

fn render_tabs(frame: &mut Frame, area: Rect, app: &App) {
    let selected_color = match app.editor_state {
        EditorState::Description | EditorState::SelectingLanguage => Color::DarkGray,
        _ => Color::Rgb(255, 160, 80),
    };

    let unselected_style = Style::default().fg(Color::DarkGray);
    let selected_style = Style::default().bg(Color::Reset).fg(selected_color).bold();

    let titles: Vec<Line> = app
        .test_cases
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let style = if i == app.selected_test_case {
                selected_style
            } else {
                unselected_style
            };

            Line::from(format!(" Case {} ", i + 1)).style(style)
        })
        .collect();

    let border_color = match app.editor_state {
        EditorState::Description | EditorState::SelectingLanguage => Color::DarkGray,
        _ => Color::Rgb(255, 160, 80),
    };

    let border_style = Style::default().fg(border_color);
    let outer_block = Block::bordered()
        .border_style(border_style)
        .title_alignment(HorizontalAlignment::Right)
        .title(" TEST CASES ");

    let tabs = Tabs::new(titles).divider("").select(app.selected_test_case);
    let inner = outer_block.inner(area);

    frame.render_widget(tabs, inner);
    frame.render_widget(outer_block, area);
}

fn render_test_case_fields(frame: &mut Frame, area: Rect, app: &App, question: &Question) {
    if app.test_cases.is_empty() {
        utils::render_empty_background(frame, area, " NO TEST CASES DEFINED ");
        return;
    }

    let container_block = Block::default()
        .borders(Borders::NONE)
        .padding(Padding::horizontal(1));

    let inner_area = container_block.inner(area);
    frame.render_widget(container_block, area);

    let case = &app.test_cases[app.selected_test_case];
    let param_names = &question.meta_data.params;

    let mut constraints = Vec::new();
    for _ in 0..param_names.len() {
        constraints.push(Constraint::Length(1));
        constraints.push(Constraint::Length(3));
        constraints.push(Constraint::Length(1));
    }

    let total_height = (param_names.len() * 5 + 10) as u16;
    let virtual_area = Rect {
        x: inner_area.x,
        y: inner_area
            .y
            .saturating_sub(app.test_cases_scroll_offset as u16),
        width: inner_area.width,
        height: total_height,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(virtual_area);

    for i in 0..param_names.len() {
        let chunk_base = i * 3;
        let label_chunk = chunks[chunk_base];
        let box_chunk = chunks[chunk_base + 1];

        let clipped_label = label_chunk.intersection(inner_area);
        let clipped_box = box_chunk.intersection(inner_area);

        if inner_area.height > 0 || inner_area.height > 0 {
            render_parameter_block(
                frame,
                clipped_label,
                clipped_box,
                &param_names[i].name,
                &case.input[i],
                i == app.selected_case_text,
            );
        }
    }

    if let EditorState::EditingTestCaseField = app.editor_state {
        let box_chunk = chunks[app.selected_case_text * 3 + 1];
        if box_chunk.y >= area.y && box_chunk.y < area.bottom() {
            let val_len = case.input[app.selected_case_text].len() as u16;
            frame.set_cursor_position((box_chunk.x + 2 + val_len, box_chunk.y + 1));
        }
    }
}

fn render_parameter_block(
    frame: &mut Frame,
    label_area: Rect,
    box_area: Rect,
    label: &str,
    value: &str,
    is_selected: bool,
) {
    let fg = if is_selected {
        Color::Rgb(0, 255, 150)
    } else {
        Color::DarkGray
    };

    let paragraph = Paragraph::new(format!("{label} =").fg(fg).bold());
    frame.render_widget(paragraph, label_area);

    let color = Color::Rgb(50, 50, 50);
    let border_style = Style::default().fg(color);
    let block = Block::default()
        .bg(color)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner_text = Paragraph::new(format!(" {}", value)).block(block);
    frame.render_widget(inner_text, box_area);
}

pub fn editor_controls(frame: &mut Frame, rect: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(0)])
        .split(rect);

    if let Some(..) = app.error_message {
        let err_text = format!("! ERROR");
        let style = Style::default().fg(Color::Rgb(255, 45, 85));
        let span = Span::styled(err_text, style);
        let paragraph = Paragraph::new(span);
        frame.render_widget(paragraph, chunks[0]);
    } else if app.is_loading {
        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let spinner_frame = spinner[app.spinner_index % spinner.len()];
        let loading_text = format!(" {spinner_frame} FETCHING");
        let style = Style::default().fg(Color::Rgb(0, 255, 150));
        let span = Span::styled(loading_text, style);
        let paragraph = Paragraph::new(span);
        frame.render_widget(paragraph, chunks[0]);
    }

    let keys_style = Style::default().fg(Color::Gray);
    let desc_style = Style::default().fg(Color::DarkGray);

    let current_keys = match app.editor_state {
        EditorState::SelectingLanguage => Line::from(vec![
            Span::styled("esc ", keys_style),
            Span::styled("BACK  ", desc_style),
            Span::styled("jk ", keys_style),
            Span::styled("MOVE  ", desc_style),
            Span::styled("enter ", keys_style),
            Span::styled("SELECT  ", desc_style),
        ]),
        EditorState::Description => Line::from(vec![
            Span::styled("esc ", keys_style),
            Span::styled("BACK  ", desc_style),
            Span::styled("jk ", keys_style),
            Span::styled("MOVE  ", desc_style),
            Span::styled("e ", keys_style),
            Span::styled("EDITOR  ", desc_style),
            Span::styled("r ", keys_style),
            Span::styled("TEST  ", desc_style),
            Span::styled("s ", keys_style),
            Span::styled("SUBMIT  ", desc_style),
        ]),
        EditorState::TestCases => Line::from(vec![
            Span::styled("esc ", keys_style),
            Span::styled("BACK  ", desc_style),
            Span::styled("hjkl ", keys_style),
            Span::styled("MOVE  ", desc_style),
            Span::styled("a ", keys_style),
            Span::styled("ADD  ", desc_style),
            Span::styled("d ", keys_style),
            Span::styled("DELETE  ", desc_style),
            Span::styled("enter ", keys_style),
            Span::styled("SELECT  ", desc_style),
        ]),
        EditorState::EditingTestCaseField => Line::from(vec![
            Span::styled("esc ", keys_style),
            Span::styled("BACK  ", desc_style),
            Span::styled("hjkl ", keys_style),
            Span::styled("MOVE  ", desc_style),
            Span::styled("enter ", keys_style),
            Span::styled("SELECT  ", desc_style),
        ]),
    };

    let help = Paragraph::new(current_keys).alignment(Alignment::Center);
    frame.render_widget(help, rect);
}
