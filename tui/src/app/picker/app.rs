use std::collections::HashSet;

use api::{MatchedUser, ProblemSummary, UserStatus};
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, TableState, Wrap},
};
use tokio::sync::mpsc::Sender;

use super::render;
use crate::app::{Action, App, handler::ClientRequest};

pub enum InputMode {
    Normal,
    Searching,
}

/// The initial `leetui` app, it lets the user search problems and select them
/// to start proposing a solution by initiating the secondary editor application.
pub struct PickerApp {
    // main fields
    pub client_tx: Sender<ClientRequest>,
    pub error_message: Option<String>,

    // user profile
    pub user_status: Option<UserStatus>,
    pub user_data: Option<MatchedUser>,

    // throbber
    pub is_loading: bool,
    pub spinner_index: usize,

    // search bar
    pub input: String,
    pub input_mode: InputMode,

    // problem list
    pub daily_challenge: Option<ProblemSummary>,
    pub problems: Vec<ProblemSummary>,
    pub table_state: TableState,
    pub known_ids: HashSet<String>,
    pub has_more: bool,
}

impl PickerApp {
    /// Creates a new `PickerApp`.
    ///
    /// # Arguments
    /// * `client_tx` - A sender to tell the client handler to make a request to the LeetCode api.
    /// # Returns
    /// A new instance of `Self`.
    pub async fn new(client_tx: Sender<ClientRequest>) -> Self {
        let app = Self {
            problems: Vec::new(),
            table_state: TableState::default(),
            user_status: None,
            user_data: None,
            is_loading: true,
            spinner_index: 0,
            error_message: None,
            client_tx,
            input: String::new(),
            input_mode: InputMode::Normal,
            known_ids: HashSet::new(),
            has_more: true,
            daily_challenge: None,
        };

        app.send_request(ClientRequest::FetchUserStatus);
        app.send_request(ClientRequest::FetchDailyChallenge);
        app.send_request(ClientRequest::FetchProblems {
            skip: 0,
            limit: 100,
            search: None,
        });
        app
    }

    /// Sends a client request to the client handler
    ///
    /// # Arguments
    /// * `req` - The request to send.
    fn send_request(&self, req: ClientRequest) {
        let tx = self.client_tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(req).await;
        });
    }

    /// Handles an incoming key event for the normal mode.
    ///
    /// # Arguments
    /// * `key` - The incoming key event.
    ///
    /// # Returns
    /// A boolean to tell the upstream caller if the app should keep running.
    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        match (key.code, key.modifiers) {
            (KeyCode::Char('j'), _) => self.move_down(1),
            (KeyCode::Char('k'), _) => self.move_up(1),
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => self.move_down(20),
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => self.move_up(20),
            (KeyCode::Char('/'), _) => {
                self.input.clear();
                self.input_mode = InputMode::Searching;
            }
            (KeyCode::Char('g'), _) => {
                self.table_state.select(Some(0));
            }
            (KeyCode::Char('G'), _) => {
                let last = self.problems.len().saturating_sub(1);
                self.table_state.select(Some(last));
            }
            (KeyCode::Char('q'), _) => return false,
            _ => {}
        };

        true
    }

    /// Handles an incoming key event for the searching mode.
    ///
    /// # Arguments
    /// * `key` - The incoming key event.
    fn handle_searching_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(ch) => {
                self.input.push(ch);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
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

    /// Moves the problem list down by a fixed amount.
    ///
    /// This method will also issue a request if nearing the end of the list.
    ///
    /// # Arguments
    /// * `amount` - The amount of problems to move the cursor by.
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

    /// Moves the problem list up by a fixed amount.
    ///
    /// # Arguments
    /// * `amount` - The amount of problems to move the cursor by.
    fn move_up(&mut self, amount: usize) {
        let i = self
            .table_state
            .selected()
            .map(|i| i.saturating_sub(amount).max(0))
            .unwrap_or_default();

        self.table_state.select(Some(i));
    }
}

impl App for PickerApp {
    fn render(&mut self, frame: &mut Frame) {
        let outer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(4),
                Constraint::Min(0),
                Constraint::Length(4),
            ])
            .split(frame.area());

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // padding
                Constraint::Length(1), // profile
                Constraint::Length(1), // padding
                Constraint::Length(1), // search bar
                Constraint::Length(1), // padding
                Constraint::Length(3), // daily
                Constraint::Length(1), // padding
                Constraint::Min(0),    // problem list
                Constraint::Length(1), // padding
                Constraint::Length(1), // controls
            ])
            .split(outer_layout[1]);

        render::user_profile(frame, main_chunks[1], self);
        render::search_bar(frame, main_chunks[3], self);
        render::daily_challenge(frame, main_chunks[5], self);
        render::problem_list(frame, main_chunks[7], self);
        render::controls(frame, main_chunks[9], self);

        if let Some(ref err) = self.error_message {
            let error_display = Paragraph::new(err.as_str())
                .block(
                    Block::default()
                        .title(" NETWORK ERROR ")
                        .borders(Borders::ALL),
                )
                .style(Style::default().fg(Color::Red))
                .wrap(Wrap { trim: true });

            frame.render_widget(error_display, main_chunks[5]);
        }
    }

    fn update(&mut self, action: Action) -> bool {
        match action {
            Action::Key(key) => match self.input_mode {
                InputMode::Normal => return self.handle_normal_key(key),
                InputMode::Searching => self.handle_searching_key(key),
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
                self.has_more = problems.len() >= 50;

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
            Action::DailyChallengeLoaded(problem) => {
                self.daily_challenge = Some(problem);
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
}
