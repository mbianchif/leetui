use std::{collections::HashSet, env, fs, io, path::PathBuf};

use api::{Language, MatchedUser, ProblemSummary, Question, UserStatus};
use ratatui::{
    Frame,
    crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Paragraph, TableState},
};
use tokio::sync::mpsc::Sender;

use super::{handler::ClientRequest, rendering, utils};

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
    Other,
}

pub enum UpdateResult {
    Continue,
    SkipRendering,
    Exit,
    OpenEditor,
}

pub enum HomeInputState {
    Normal,
    Searching,
}

#[derive(Clone, Copy)]
pub enum EditorState {
    SelectingLanguage,
    Description,
    TestCases,
    EditingTestCaseField,
}

enum AppState {
    Home,
    Editor,
}

pub struct TestCase {
    pub input: Vec<String>,
    pub output: Option<String>,
    pub expected: Option<String>,
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

    // Editor panel
    pub editor_state: EditorState,
    pub question: Option<Question>,

    // language selector
    pub selected_language: Option<Language>,
    pub solution_paths: Vec<PathBuf>,
    pub language_selection_index: usize,

    // description
    pub description_offset: usize,

    // test cases
    pub test_cases: Vec<TestCase>,
    pub selected_test_case: usize,
    pub selected_case_text: usize,
    pub test_cases_scroll_offset: usize,
    pub last_test_case_viewport_height: u16,
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
            question: None,
            selected_language: None,
            solution_paths: Vec::new(),
            description_offset: 0,
            test_cases: Vec::new(),
            selected_test_case: 0,
            selected_case_text: 0,
            editor_state: EditorState::Description,
            test_cases_scroll_offset: 0,
            last_test_case_viewport_height: 0,
            language_selection_index: 0,
        };

        app.send_request(ClientRequest::FetchUserStatus);
        app.send_request(ClientRequest::FetchDailyChallenge);
        app.send_request(ClientRequest::FetchProblems {
            skip: 0,
            limit: 50,
            search: None,
        });
        app
    }

    pub fn render(&mut self, frame: &mut Frame) {
        match self.state {
            AppState::Home => self.render_home(frame),
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

        rendering::user_profile(frame, main_chunks[1], self);
        rendering::search_bar(frame, main_chunks[3], self);
        rendering::daily_challenge(frame, main_chunks[5], self);
        rendering::problem_list(frame, main_chunks[7], self);
        rendering::home_controls(frame, main_chunks[9], self);

        if let Some(ref err) = self.error_message {
            let err_line = Paragraph::new(format!(" ERROR: {}", err))
                .style(Style::default().fg(Color::Red).bg(Color::Black));
            frame.render_widget(err_line, main_chunks[9]);
        }
    }

    pub fn render_editor(&mut self, frame: &mut Frame) {
        let outer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(4),
                Constraint::Min(0),
                Constraint::Length(4),
            ])
            .split(frame.area());

        let constraints: &[_] = match self.editor_state {
            EditorState::SelectingLanguage => {
                let rows =
                    (self.question.as_ref().unwrap().code_snippets.len() as f32 / 3.0).ceil();

                &[
                    Constraint::Length(1),               // padding
                    Constraint::Min(0),                  // description
                    Constraint::Length(3 + rows as u16), // test cases + language selector + language grid
                    Constraint::Length(1),               // padding
                    Constraint::Length(1),               // controls
                ]
            }
            EditorState::Description => &[
                Constraint::Length(1), // padding
                Constraint::Min(0),    // description
                Constraint::Length(3), // test cases + language selector
                Constraint::Length(1), // padding
                Constraint::Length(1), // controls
            ],
            _ => {
                let inputs = self.question.as_ref().unwrap().meta_data.params.len();
                let exact_size = 2 + 5 * (inputs);

                &[
                    Constraint::Length(1),              // padding
                    Constraint::Min(0),                 // description
                    Constraint::Max(exact_size as u16), // test cases + language selector
                    Constraint::Length(1),              // padding
                    Constraint::Length(1),              // controls
                ]
            }
        };

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(outer_layout[1]);

        rendering::description(frame, main_chunks[1], self);
        rendering::test_cases_languages_pane(frame, main_chunks[2], self);
        rendering::editor_controls(frame, main_chunks[4], self);
    }

    pub fn update(&mut self, action: Action) -> UpdateResult {
        match self.state {
            AppState::Home => self.update_home(action),
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
            Action::QuestionLoaded(mut question) => {
                if let Err(e) = self.load_local_file_paths(&question) {
                    self.error_message = Some(e.to_string());
                    return UpdateResult::Continue;
                }

                question.content = utils::html_to_markdown(&question.content);

                let param_count = question.meta_data.params.len();
                let lines: Vec<_> = question.example_testcases.lines().collect();

                self.test_cases = lines
                    .chunks(param_count)
                    .map(|chunk| TestCase {
                        input: chunk.iter().map(|s| s.to_string()).collect(),
                        output: None,
                        expected: None,
                    })
                    .collect();

                self.is_loading = false;
                self.selected_test_case = 0;
                self.question = Some(question);
                self.state = AppState::Editor;
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
            _ => {}
        };

        UpdateResult::Continue
    }

    fn update_editor(&mut self, action: Action) -> UpdateResult {
        match action {
            Action::Key(key_event) => match self.editor_state {
                EditorState::SelectingLanguage => {
                    self.handle_editor_selecting_language_key(key_event)
                }
                EditorState::Description => self.handle_editor_description_key(key_event),
                EditorState::TestCases => self.handle_editor_test_cases_key(key_event),
                EditorState::EditingTestCaseField => {
                    self.handle_editor_editing_test_case_key(key_event)
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

    /// Handles an incoming key event for the normal mode.
    ///
    /// # Arguments
    /// * `key` - The incoming key event.
    ///
    /// # Returns
    /// The result of an update.
    fn handle_home_normal_key(&mut self, key: KeyEvent) -> UpdateResult {
        match (key.code, key.modifiers) {
            (KeyCode::Char('j'), KeyModifiers::NONE) => self.scroll_down_problem_list(1),
            (KeyCode::Char('k'), KeyModifiers::NONE) => self.scroll_up_problem_list(1),
            (KeyCode::Char('/'), KeyModifiers::NONE) => {
                self.search_bar_input.clear();
                self.home_input_state = HomeInputState::Searching;
            }
            (KeyCode::Enter, _) => {
                let Some(problem) = self
                    .problem_table_state
                    .selected()
                    .map(|i| &self.problems[i])
                else {
                    return UpdateResult::Continue;
                };

                let user_is_premium = self.user_status.as_ref().unwrap().is_premium;
                if !(problem.paid_only && !user_is_premium) {
                    let slug = problem.title_slug.to_string();
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
                    search: (!self.search_bar_input.is_empty())
                        .then_some(self.search_bar_input.clone()),
                })
            }
            _ => {}
        }

        UpdateResult::Continue
    }

    fn handle_editor_selecting_language_key(&mut self, key: KeyEvent) {
        let question = self.question.as_ref().unwrap();
        let snippets = &question.code_snippets;
        let nlangs = snippets.len();
        let rows = (nlangs as f32 / 3.0).ceil() as usize;

        match key.code {
            KeyCode::Esc | KeyCode::Char('c') => {
                self.editor_state = EditorState::Description;
            }
            KeyCode::Char('h') => {
                if self.language_selection_index >= rows {
                    self.language_selection_index -= rows;
                }
            }
            KeyCode::Char('j') => {
                if self.language_selection_index % rows < rows - 1 {
                    self.language_selection_index =
                        (self.language_selection_index + 1).min(nlangs - 1);
                }
            }
            KeyCode::Char('k') => {
                if self.language_selection_index % rows > 0 {
                    self.language_selection_index = self.language_selection_index.saturating_sub(1);
                }
            }
            KeyCode::Char('l') => {
                if self.language_selection_index + rows < nlangs {
                    self.language_selection_index += rows;
                }
            }
            KeyCode::Enter => {
                let lang = snippets[self.language_selection_index].lang;
                self.selected_language = Some(lang);
            }
            KeyCode::Char('t') => {
                self.editor_state = EditorState::TestCases;
            }
            _ => {}
        }
    }

    fn handle_editor_description_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.state = AppState::Home;
            }
            KeyCode::Char('j') => {
                self.description_offset = self.description_offset.saturating_add(1);
            }
            KeyCode::Char('k') => {
                self.description_offset = self.description_offset.saturating_sub(1);
            }
            KeyCode::Char('t') => {
                self.editor_state = EditorState::TestCases;
            }
            KeyCode::Char('c') => {
                self.editor_state = EditorState::SelectingLanguage;
            }
            KeyCode::Char('e') => {} // open editor
            KeyCode::Char('s') => {} // submit code
            KeyCode::Char('r') => {} // run tests
            _ => {}
        }
    }
    fn handle_editor_test_cases_key(&mut self, key: KeyEvent) {
        let Some(ref question) = self.question else {
            unreachable!()
        };

        match key.code {
            KeyCode::Char('h') => {
                if self.selected_test_case == 0 {
                    self.selected_test_case = self.test_cases.len() - 1;
                } else {
                    self.selected_test_case -= 1;
                }
            }
            KeyCode::Char('j') => {
                if self.selected_case_text < question.meta_data.params.len() - 1 {
                    self.selected_case_text += 1;
                    self.adjust_scroll_for_selection();
                }
            }
            KeyCode::Char('k') => {
                if self.selected_case_text > 0 {
                    self.selected_case_text -= 1;
                    self.adjust_scroll_for_selection();
                }
            }
            KeyCode::Char('l') => {
                self.selected_test_case = (self.selected_test_case + 1) % self.test_cases.len();
            }
            KeyCode::Enter => {
                if !self.test_cases.is_empty() {
                    self.editor_state = EditorState::EditingTestCaseField;
                }
            }
            KeyCode::Char('d') => {
                self.test_cases.remove(self.selected_test_case);
                self.selected_test_case = self
                    .selected_test_case
                    .min(self.test_cases.len().saturating_sub(1));
            }
            KeyCode::Char('a') => {
                self.test_cases.push(TestCase {
                    input: vec!["".into(); question.meta_data.params.len()],
                    output: None,
                    expected: None,
                });
            }
            KeyCode::Char('c') => {
                self.editor_state = EditorState::SelectingLanguage;
            }
            KeyCode::Esc | KeyCode::Char('t') => self.editor_state = EditorState::Description,
            KeyCode::Char('s') => {} // submit code
            KeyCode::Char('r') => {} // run tests
            _ => {}
        }
    }

    fn handle_editor_editing_test_case_key(&mut self, key: KeyEvent) {
        let Some(case) = self.test_cases.get_mut(self.selected_test_case) else {
            unreachable!();
        };

        let text = &mut case.input[self.selected_case_text];

        match key.code {
            KeyCode::Char(c) => text.push(c),
            KeyCode::Backspace => {
                text.pop();
            }
            KeyCode::Enter => {
                *text = text.trim().to_string();
                self.editor_state = EditorState::Description;
            }
            KeyCode::Esc => {
                *text = text.trim().to_string();
                self.editor_state = EditorState::TestCases;
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

    pub fn adjust_scroll_for_selection(&mut self) {
        let param_count = self
            .question
            .as_ref()
            .map(|q| q.meta_data.params.len())
            .unwrap_or_default();

        let total_content_height = (5 * param_count) as u16;
        let viewport_height = self.last_test_case_viewport_height;

        let max_scroll = total_content_height.saturating_sub(viewport_height);
        let selection_top = (5 * self.selected_case_text) as u16;

        let selection_bottom = selection_top + 2;
        let mut new_offset = self.test_cases_scroll_offset as u16;

        if selection_top < new_offset {
            new_offset = selection_top;
        } else if selection_bottom >= new_offset + viewport_height.saturating_sub(2) {
            new_offset = (selection_bottom + 2).saturating_sub(viewport_height);
        }

        self.test_cases_scroll_offset = new_offset.min(max_scroll) as usize;
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

    fn load_local_file_paths(&mut self, question: &Question) -> io::Result<()> {
        let slug = &question.title_slug;
        let dir_path = env::home_dir()
            .unwrap_or_default()
            .join(".leetui")
            .join(slug);

        fs::create_dir_all(&dir_path)?;

        self.solution_paths = fs::read_dir(&dir_path)?
            .flatten()
            .map(|file| file.path())
            .collect();

        Ok(())
    }
}
