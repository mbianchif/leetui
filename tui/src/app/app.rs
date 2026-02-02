use std::{collections::HashSet, fs};

use api::{Language, MatchedUser, ProblemSummary, Question, UserStatus};
use ratatui::{
    Frame,
    crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Clear, ListState, Paragraph, TableState},
};
use tokio::sync::mpsc::Sender;

use super::{handler::ClientRequest, home_render};
use crate::app::{editor, utils_render, workspace_render};

/// The types of events that exist in both apps.
#[derive(Debug)]
pub enum Action {
    Key(event::KeyEvent),

    Tick,

    UserStatusLoaded(UserStatus),
    UserProfileLoaded(MatchedUser),
    ProblemListLoaded(Vec<ProblemSummary>),
    DailyChallengeLoaded(ProblemSummary),
    QuestionLoaded(Question),

    NetworkError(String),
}

pub enum UpdateResult {
    Continue,
    SkipRendering,
    Exit,
}

pub enum HomeInputState {
    Normal,
    Searching,
}

pub enum WorkspaceState {
    NewFileMenu,
    FileSelector,
}

enum AppState {
    Home,
    Workspace,
    Editor,
}

pub struct App {
    // Main Fields
    pub error_message: Option<String>,
    client_tx: Sender<ClientRequest>,
    state: AppState,

    // User Profile
    pub user_status: Option<UserStatus>,
    pub user_data: Option<MatchedUser>,

    // Throbber
    pub is_loading: bool,
    pub spinner_index: usize,

    // Search Bar
    pub search_bar_input: String,
    pub home_input_state: HomeInputState,

    // Problem List
    pub daily_challenge: Option<ProblemSummary>,
    pub problems: Vec<ProblemSummary>,
    pub problem_table_state: TableState,
    pub known_ids: HashSet<String>,
    pub has_more: bool,

    // Workspace State
    pub local_files: Vec<String>,
    pub file_list_state: ListState,
    pub workspace_state: WorkspaceState,
    pub question: Option<Question>,
    pub new_file_input: String,
    pub detected_language: Option<Language>,
}

impl App {
    /// Creates a new `PickerApp`.
    ///
    /// # Arguments
    /// * `client_tx` - A sender to tell the client handler to make a request to the LeetCode api.
    ///
    /// # Returns
    /// A new instance of `Self`.
    pub fn new(client_tx: Sender<ClientRequest>) -> Self {
        let app = Self {
            problems: Vec::new(),
            problem_table_state: TableState::default().with_selected(0),
            user_status: None,
            user_data: None,
            is_loading: true,
            spinner_index: 0,
            error_message: None,
            client_tx,
            search_bar_input: String::new(),
            home_input_state: HomeInputState::Normal,
            known_ids: HashSet::new(),
            has_more: true,
            daily_challenge: None,
            state: AppState::Home,
            workspace_state: WorkspaceState::FileSelector,
            question: None,
            local_files: Vec::new(),
            file_list_state: ListState::default().with_selected(Some(0)),
            new_file_input: String::new(),
            detected_language: None,
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

    pub fn render(&mut self, frame: &mut Frame) {
        match self.state {
            AppState::Home => self.render_home(frame),
            AppState::Workspace => {
                self.render_home(frame);
                self.render_workspace(frame);
            }
            AppState::Editor => self.render_editor(frame),
        }
    }

    fn render_home(&mut self, frame: &mut Frame) {
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

        home_render::user_profile(frame, main_chunks[1], self);
        home_render::search_bar(frame, main_chunks[3], self);
        home_render::daily_challenge(frame, main_chunks[5], self);
        home_render::problem_list(frame, main_chunks[7], self);
        home_render::controls(frame, main_chunks[9], self);

        if let Some(ref err) = self.error_message {
            let err_line = Paragraph::new(format!(" ERROR: {}", err))
                .style(Style::default().fg(Color::Red).bg(Color::Black));
            frame.render_widget(err_line, main_chunks[9]);
        }
    }

    fn render_workspace(&mut self, frame: &mut Frame) {
        if self.question.is_some() {
            let main_chunks = Layout::default()
                .constraints([Constraint::Min(0), Constraint::Length(1)])
                .split(frame.area());

            frame.render_widget(Clear, main_chunks[1]);

            let floating_pane = utils_render::centered_rect(70, 70, main_chunks[0]);
            frame.render_widget(Clear, floating_pane);

            let floating_chunks = Layout::default()
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(floating_pane);

            workspace_render::file_creator(frame, floating_chunks[0], self);
            workspace_render::file_selector(frame, floating_chunks[1], self);
            workspace_render::controls(frame, main_chunks[1], self);
        }
    }

    pub fn render_editor(&mut self, frame: &mut Frame) {}

    pub fn update(&mut self, action: Action) -> UpdateResult {
        match self.state {
            AppState::Home => self.update_home(action),
            AppState::Workspace => self.update_workspace(action),
            AppState::Editor => self.update_editor(action),
        }
    }

    fn update_home(&mut self, action: Action) -> UpdateResult {
        match action {
            Action::Key(key) => match self.home_input_state {
                HomeInputState::Normal => return self.handle_home_normal_key(key),
                HomeInputState::Searching => return self.handle_home_searching_key(key),
            },
            Action::ProblemListLoaded(problems) => {
                self.has_more = problems.len() >= 50;

                for p in problems {
                    if !self.known_ids.contains(&p.frontend_question_id) {
                        self.known_ids.insert(p.frontend_question_id.clone());
                        self.problems.push(p);
                    }
                }

                if self.problem_table_state.selected().is_none() {
                    self.problem_table_state.select(Some(0));
                }

                self.is_loading = false;
            }
            Action::QuestionLoaded(question) => {
                self.is_loading = false;
                self.question = Some(question);
                self.state = AppState::Workspace;
                self.refresh_local_files();
            }
            Action::Tick => {
                if !self.is_loading {
                    return UpdateResult::SkipRendering;
                }

                self.spinner_index = self.spinner_index.wrapping_add(1);
            }
            Action::NetworkError(e) => {
                self.is_loading = false;
                self.error_message = Some(e);
            }
            Action::UserStatusLoaded(status) => {
                let username = status.username.clone();
                self.send_request(ClientRequest::FetchProfile { username });
                self.user_status = Some(status);
            }
            Action::UserProfileLoaded(profile) => {
                self.user_data = Some(profile);
                self.is_loading = false;
            }
            Action::DailyChallengeLoaded(problem) => {
                self.daily_challenge = Some(problem);
                self.is_loading = false;
            }
        };

        UpdateResult::Continue
    }

    fn update_workspace(&mut self, action: Action) -> UpdateResult {
        match action {
            Action::Key(key_event) => match self.workspace_state {
                WorkspaceState::FileSelector => {
                    self.handle_workspace_file_selector_key(key_event);
                }
                WorkspaceState::NewFileMenu => {
                    self.handle_workspace_new_file_key(key_event);
                }
            },
            Action::Tick => {
                if !self.is_loading {
                    return UpdateResult::SkipRendering;
                }

                self.spinner_index = self.spinner_index.wrapping_add(1);
            }
            _ => {}
        };

        UpdateResult::Continue
    }

    fn update_editor(&mut self, action: Action) -> UpdateResult {
        match action {
            Action::Key(key_event) => self.handle_editor_key(key_event),
            Action::Tick => {
                if !self.is_loading {
                    return UpdateResult::SkipRendering;
                }

                self.spinner_index = self.spinner_index.wrapping_add(1);
            }
            _ => {}
        };

        UpdateResult::Continue
    }

    /// Handles an incoming key event for the normal mode.
    ///
    /// # Arguments
    /// * `key` - The incoming key event.
    ///
    /// # Returns
    /// The result of an update.
    fn handle_home_normal_key(&mut self, key: KeyEvent) -> UpdateResult {
        match (key.code, key.modifiers) {
            (KeyCode::Char('j'), _) => self.scroll_down_problem_list(1),
            (KeyCode::Char('k'), _) => self.scroll_up_problem_list(1),
            (KeyCode::Char('/'), _) => {
                self.search_bar_input.clear();
                self.home_input_state = HomeInputState::Searching;
            }
            (KeyCode::Enter, _) => {
                let slug = self
                    .problem_table_state
                    .selected()
                    .map(|i| self.problems[i].title_slug.clone());

                if let Some(slug) = slug {
                    self.is_loading = true;
                    self.send_request(ClientRequest::FetchQuestion { slug });
                }
            }
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                let slug = self
                    .daily_challenge
                    .as_ref()
                    .map(|dc| dc.title_slug.clone());

                if let Some(slug) = slug {
                    self.is_loading = true;
                    self.send_request(ClientRequest::FetchQuestion { slug });
                }
            }
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => self.scroll_down_problem_list(20),
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => self.scroll_up_problem_list(20),
            (KeyCode::Char('g'), _) => {
                self.problem_table_state.select(Some(0));
            }
            (KeyCode::Char('G'), _) => {
                let last = self.problems.len().saturating_sub(1);
                self.problem_table_state.select(Some(last));
            }
            (KeyCode::Esc, _) => return UpdateResult::Exit,
            _ => {}
        };

        UpdateResult::Continue
    }

    /// Handles an incoming key event for the searching mode.
    ///
    /// # Arguments
    /// * `key` - The incoming key event.
    fn handle_home_searching_key(&mut self, key: KeyEvent) -> UpdateResult {
        match key.code {
            KeyCode::Char(ch) => {
                self.search_bar_input.push(ch);
            }
            KeyCode::Backspace => {
                self.search_bar_input.pop();
            }
            KeyCode::Esc => {
                self.home_input_state = HomeInputState::Normal;
            }
            KeyCode::Enter => {
                self.home_input_state = HomeInputState::Normal;
                self.problems.clear();
                self.known_ids.clear();
                self.is_loading = true;
                self.has_more = true;

                self.send_request(ClientRequest::FetchProblems {
                    skip: 0,
                    limit: 50,
                    search: Some(self.search_bar_input.clone()),
                })
            }
            _ => {}
        }

        UpdateResult::Continue
    }

    fn handle_workspace_file_selector_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') => self.file_list_state.scroll_down_by(1),
            KeyCode::Char('k') => self.file_list_state.scroll_up_by(1),
            KeyCode::Esc => match self.workspace_state {
                WorkspaceState::FileSelector => {
                    self.state = AppState::Home;
                }
                WorkspaceState::NewFileMenu => {
                    self.workspace_state = WorkspaceState::FileSelector;
                }
            },
            KeyCode::Char('n') => {
                self.new_file_input.clear();
                self.workspace_state = WorkspaceState::NewFileMenu;
            }
            KeyCode::Enter => {}
            _ => {}
        }
    }

    fn handle_workspace_new_file_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(ch) => {
                self.new_file_input.push(ch);
                let ext = self.new_file_input.rsplit('.').next().unwrap_or_default();
                self.detected_language = Language::from_ext(ext);
            }
            KeyCode::Backspace => {
                self.new_file_input.pop();
                let ext = self.new_file_input.rsplit('.').next().unwrap_or_default();
                self.detected_language = Language::from_ext(ext);
            }
            KeyCode::Enter if self.detected_language.is_some() => {
                if let Err(e) = editor::create_file(self) {
                    self.error_message = Some(e.to_string());
                } else {
                    self.refresh_local_files();
                    self.new_file_input.clear();
                    self.workspace_state = WorkspaceState::FileSelector;
                }
            }
            KeyCode::Enter => {
                self.error_message = Some("failed to infer the language".to_string());
            }
            KeyCode::Esc => {
                self.workspace_state = WorkspaceState::FileSelector;
            }
            _ => {}
        }
    }

    fn handle_editor_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.state = AppState::Workspace;
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
    fn scroll_down_problem_list(&mut self, amount: usize) {
        let last_index = self.problems.len().saturating_sub(1);
        let current = self.problem_table_state.selected().unwrap_or_default();
        let next = (current + amount).min(last_index);

        self.problem_table_state.select(Some(next));

        let threshold = 25;
        if next + threshold >= self.problems.len() && !self.is_loading && self.has_more {
            self.is_loading = true;
            self.send_request(ClientRequest::FetchProblems {
                skip: self.problems.len(),
                limit: 50,
                search: (!self.search_bar_input.is_empty())
                    .then_some(self.search_bar_input.clone()),
            });
        }
    }

    /// Moves the problem list up by a fixed amount.
    ///
    /// # Arguments
    /// * `amount` - The amount of problems to move the cursor by.
    fn scroll_up_problem_list(&mut self, amount: usize) {
        let i = self
            .problem_table_state
            .selected()
            .map(|i| i.saturating_sub(amount).max(0))
            .unwrap_or_default();

        self.problem_table_state.select(Some(i));
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

    fn refresh_local_files(&mut self) {
        let Some(ref q) = self.question else { return };

        let dir_path = std::env::home_dir()
            .unwrap_or_default()
            .join(".leetui")
            .join(&q.title_slug);

        let _ = fs::create_dir_all(&dir_path);

        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                        let ext = name.rsplit('.').next().unwrap_or_default();

                        if Language::from_ext(ext).is_some() {
                            files.push(name.to_string());
                        }
                    }
                }
            }
        }

        files.sort();
        self.local_files = files;
    }
}
