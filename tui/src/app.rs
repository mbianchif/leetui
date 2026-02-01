use std::collections::HashSet;

use api::{MatchedUser, ProblemSummary, UserStatus};
use ratatui::{
    Frame,
    crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, TableState, Wrap},
};
use tokio::sync::mpsc::Sender;

use crate::{handler::ClientRequest, render};

#[derive(Default)]
pub enum View {
    #[default]
    Home,
}

pub enum Action {
    Key(event::KeyEvent),

    Tick,

    UserStatusLoaded(UserStatus),
    UserProfileLoaded(MatchedUser),
    ProblemListLoaded(Vec<ProblemSummary>),

    NetworkError(String),
}

pub enum SearchInputMode {
    Normal,
    Editing,
}

pub struct App {
    pub view: View,
    pub user_status: Option<UserStatus>,
    pub user_data: Option<MatchedUser>,
    pub error_message: Option<String>,
    pub client_tx: Sender<ClientRequest>,

    // throbber
    pub is_loading: bool,
    pub spinner_index: usize,

    // search bar
    pub input: String,
    pub input_mode: SearchInputMode,

    // problem list
    pub problems: Vec<ProblemSummary>,
    pub table_state: TableState,
    pub known_ids: HashSet<String>,
    pub has_more: bool,
}

impl App {
    pub async fn new(client_tx: Sender<ClientRequest>) -> Self {
        let app = Self {
            view: View::default(),
            problems: Vec::new(),
            table_state: TableState::default(),
            user_status: None,
            user_data: None,
            is_loading: true,
            spinner_index: 0,
            error_message: None,
            client_tx,
            input: String::new(),
            input_mode: SearchInputMode::Normal,
            known_ids: HashSet::new(),
            has_more: true,
        };

        app.send_request(ClientRequest::FetchUserStatus);
        app.send_request(ClientRequest::FetchProblems {
            skip: 0,
            limit: 100,
            search: None,
        });
        app
    }

    pub fn render(&mut self, f: &mut Frame) {
        match self.view {
            View::Home => self.render_home_view(f),
        }
    }

    fn handle_home_normal_keys(&mut self, key: KeyEvent) -> bool {
        match (key.code, key.modifiers) {
            (KeyCode::Char('j'), _) => self.move_down(1),
            (KeyCode::Char('k'), _) => self.move_up(1),
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => self.move_down(20),
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => self.move_up(20),
            (KeyCode::Char('/'), _) => {
                self.input.clear();
                self.input_mode = SearchInputMode::Editing;
            }
            (KeyCode::Esc, _) => return false,
            _ => {}
        };

        true
    }

    fn handle_home_editing_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(ch) => {
                self.input.push(ch);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Esc => {
                self.input_mode = SearchInputMode::Normal;
            }
            KeyCode::Enter => {
                self.input_mode = SearchInputMode::Normal;
                self.problems.clear();
                self.known_ids.clear();
                self.is_loading = true;
                self.has_more = true;

                self.send_request(ClientRequest::FetchProblems {
                    skip: 0,
                    limit: 50,
                    search: Some(self.input.clone()),
                })
            }
            _ => {}
        }
    }

    pub fn update(&mut self, action: Action) -> bool {
        match action {
            Action::Key(key) => match self.input_mode {
                SearchInputMode::Normal => return self.handle_home_normal_keys(key),
                SearchInputMode::Editing => self.handle_home_editing_keys(key),
            },
            Action::UserStatusLoaded(status) => {
                let username = status.username.clone();
                self.is_loading = true;
                self.send_request(ClientRequest::FetchProfile { username });
                self.user_status = Some(status);
            }
            Action::UserProfileLoaded(profile) => {
                self.user_data = Some(profile);
                self.is_loading = false;
            }
            Action::ProblemListLoaded(problems) => {
                self.has_more = problems.len() == 50;

                for p in problems {
                    if !self.known_ids.contains(&p.frontend_question_id) {
                        self.known_ids.insert(p.frontend_question_id.clone());
                        self.problems.push(p);
                    }
                }

                if self.table_state.selected().is_none() {
                    self.table_state.select(Some(0));
                }

                self.is_loading = false;
            }
            Action::Tick if self.is_loading => {
                self.spinner_index = self.spinner_index.wrapping_add(1);
            }
            Action::NetworkError(e) => {
                self.is_loading = false;
                self.error_message = Some(e);
            }
            _ => {}
        }

        true
    }

    fn send_request(&self, req: ClientRequest) {
        let tx = self.client_tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(req).await;
        });
    }

    fn move_down(&mut self, amount: usize) {
        let i = self
            .table_state
            .selected()
            .map(|i| (i + amount).min(self.problems.len().saturating_sub(1)))
            .unwrap_or_default();

        self.table_state.select(Some(i));

        let threshold = 25;
        if i + threshold >= self.problems.len() && !self.is_loading && self.has_more {
            self.is_loading = true;

            self.send_request(ClientRequest::FetchProblems {
                skip: self.problems.len(),
                limit: 50,
                search: (!self.input.is_empty()).then_some(self.input.clone()),
            });
        }
    }

    fn move_up(&mut self, amount: usize) {
        let i = self
            .table_state
            .selected()
            .map(|i| i.saturating_sub(amount).max(0))
            .unwrap_or_default();

        self.table_state.select(Some(i));
    }

    fn render_home_view(&mut self, f: &mut Frame) {
        let outer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(4),
                Constraint::Min(0),
                Constraint::Length(4),
            ])
            .split(f.area());

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // padding
                Constraint::Length(1), // profile
                Constraint::Length(1), // padding
                Constraint::Length(1), // search bar
                Constraint::Length(1), // padding
                Constraint::Min(0),    // problem list
                Constraint::Length(1), // padding
                Constraint::Length(1), // controls
            ])
            .split(outer_layout[1]);

        render::user_profile(f, main_chunks[1], self);
        render::search_bar(f, main_chunks[3], self);
        render::problem_list(f, main_chunks[5], self);
        render::home_controls(f, main_chunks[7], self);

        if let Some(ref err) = self.error_message {
            let error_display = Paragraph::new(err.as_str())
                .block(
                    Block::default()
                        .title(" NETWORK ERROR ")
                        .borders(Borders::ALL),
                )
                .style(Style::default().fg(Color::Red))
                .wrap(Wrap { trim: true });

            f.render_widget(error_display, main_chunks[5]);
        }
    }
}
