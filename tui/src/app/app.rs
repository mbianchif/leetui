use std::{
    collections::HashSet,
    env,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
};

use api::{MatchedUser, ProblemSummary, Question, UserStatus};
use ratatui::{
    Frame,
    crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{ListState, Paragraph, TableState},
};
use tokio::sync::mpsc::Sender;

use super::{Multiplexer, edit_render, handler::ClientRequest, home_render};

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

pub enum HomeInputState {
    Normal,
    Searching,
}

enum AppState {
    Home,
    LanguageSelection,
    Editor,
}

pub struct App<M: Multiplexer> {
    // Main Fields
    pub error_message: Option<String>,
    client_tx: Sender<ClientRequest>,
    state: AppState,
    multiplexer: M,

    // User Profile
    pub user_status: Option<UserStatus>,
    pub user_data: Option<MatchedUser>,

    // Throbber
    pub is_loading: bool,
    pub spinner_index: usize,

    // Search Bar
    pub input: String,
    pub input_mode: HomeInputState,

    // Problem List
    pub daily_challenge: Option<ProblemSummary>,
    pub problems: Vec<ProblemSummary>,
    pub table_state: TableState,
    pub known_ids: HashSet<String>,
    pub has_more: bool,

    // Question
    pub list_state: ListState,
    pub question: Option<Question>,
}

impl<M: Multiplexer> App<M> {
    /// Creates a new `PickerApp`.
    ///
    /// # Arguments
    /// * `client_tx` - A sender to tell the client handler to make a request to the LeetCode api.
    /// * `multiplexer` - The multiplexer strategy to use when editing.
    ///
    /// # Returns
    /// A new instance of `Self`.
    pub fn new(client_tx: Sender<ClientRequest>, multiplexer: M) -> Self {
        let app = Self {
            problems: Vec::new(),
            table_state: TableState::default().with_selected(0),
            user_status: None,
            user_data: None,
            is_loading: true,
            spinner_index: 0,
            error_message: None,
            client_tx,
            input: String::new(),
            input_mode: HomeInputState::Normal,
            known_ids: HashSet::new(),
            has_more: true,
            daily_challenge: None,
            multiplexer,
            state: AppState::Home,
            list_state: ListState::default().with_selected(Some(0)),
            question: None,
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
            AppState::LanguageSelection => self.render_language_selection(frame),
            AppState::Editor => todo!(),
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

    fn render_language_selection(&mut self, frame: &mut Frame) {
        if self.is_loading {
            edit_render::loading(frame, frame.area(), self);
            return;
        }

        if let Some(ref q) = self.question {
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(1)])
                .split(frame.area());

            let upper_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(main_chunks[0]);

            edit_render::description(frame, upper_chunks[0], q);
            edit_render::language_selector(frame, upper_chunks[1], self);
            edit_render::controls(frame, main_chunks[1], self)
        }
    }

    pub fn update(&mut self, action: Action) -> bool {
        match self.state {
            AppState::Home => self.update_home(action),
            AppState::LanguageSelection => self.update_language_selection(action),
            AppState::Editor => self.update_editor(action),
        }
    }

    fn update_home(&mut self, action: Action) -> bool {
        match action {
            Action::Key(key) => match self.input_mode {
                HomeInputState::Normal => return self.handle_home_normal_key(key),
                HomeInputState::Searching => self.handle_home_searching_key(key),
            },
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
            Action::QuestionLoaded(question) => {
                self.question = Some(question);
                self.state = AppState::LanguageSelection;
                self.is_loading = false;
            }
            Action::Tick if self.is_loading => {
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
            _ => {}
        }

        true
    }

    fn update_language_selection(&mut self, action: Action) -> bool {
        match action {
            Action::Key(key_event) => self.handle_language_selection_key(key_event),
            Action::Tick if self.is_loading => {
                self.spinner_index = self.spinner_index.wrapping_add(1);
            }
            _ => {}
        };

        true
    }

    fn update_editor(&mut self, action: Action) -> bool {
        match action {
            Action::Key(key_event) => todo!(),
            Action::Tick if self.is_loading => {
                self.spinner_index = self.spinner_index.wrapping_add(1);
            }
            _ => {}
        };

        true
    }

    /// Handles an incoming key event for the normal mode.
    ///
    /// # Arguments
    /// * `key` - The incoming key event.
    ///
    /// # Returns
    /// A boolean to tell the upstream caller if the app should keep running.
    fn handle_home_normal_key(&mut self, key: KeyEvent) -> bool {
        match (key.code, key.modifiers) {
            (KeyCode::Char('j'), _) => self.scroll_down_problem_list(1),
            (KeyCode::Char('k'), _) => self.scroll_up_problem_list(1),
            (KeyCode::Char('/'), _) => {
                self.input.clear();
                self.input_mode = HomeInputState::Searching;
            }
            (KeyCode::Enter, _) => {
                let slug = self
                    .table_state
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
                self.table_state.select(Some(0));
            }
            (KeyCode::Char('G'), _) => {
                let last = self.problems.len().saturating_sub(1);
                self.table_state.select(Some(last));
            }
            (KeyCode::Esc, _) => return false,
            _ => {}
        };

        true
    }

    /// Handles an incoming key event for the searching mode.
    ///
    /// # Arguments
    /// * `key` - The incoming key event.
    fn handle_home_searching_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(ch) => {
                self.input.push(ch);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Esc => {
                self.input_mode = HomeInputState::Normal;
            }
            KeyCode::Enter => {
                self.input_mode = HomeInputState::Normal;
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

    fn handle_language_selection_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.state = AppState::Home;
            }
            KeyCode::Char('j') => self.list_state.scroll_down_by(1),
            KeyCode::Char('k') => self.list_state.scroll_up_by(1),
            KeyCode::Enter => {
                let path = match self.prepare_env_for_editor() {
                    Ok(path) => path,
                    Err(e) => {
                        self.error_message = Some(e.to_string());
                        return;
                    }
                };

                self.state = AppState::Editor;
                if let Err(e) = self.multiplexer.open(&path) {
                    self.error_message = Some(e.to_string());
                }
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
        let current = self.table_state.selected().unwrap_or_default();
        let next = (current + amount).min(last_index);

        self.table_state.select(Some(next));

        let threshold = 25;
        if next + threshold >= self.problems.len() && !self.is_loading && self.has_more {
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
    fn scroll_up_problem_list(&mut self, amount: usize) {
        let i = self
            .table_state
            .selected()
            .map(|i| i.saturating_sub(amount).max(0))
            .unwrap_or_default();

        self.table_state.select(Some(i));
    }

    fn prepare_env_for_editor(&mut self) -> io::Result<PathBuf> {
        let Some(ref q) = self.question else {
            unreachable!();
        };

        let index = self.list_state.selected().unwrap_or_default();
        let snippet = &q.code_snippets[index];
        let slug = &q.title_slug;
        let ext = snippet.lang.ext();

        let mut path = env::home_dir().unwrap_or_default();
        path.push(".leetui");
        path.push(slug);
        fs::create_dir_all(&path)?;
        path.push(format!("{slug}.{ext}"));

        let mut file = File::create(&path)?;
        file.write_all(snippet.code.as_bytes())?;
        Ok(path)
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
}
