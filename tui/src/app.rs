use api::{ProblemSummary, UserProfile};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::TableState,
};

use crate::render;

#[derive(Default)]
pub enum View {
    #[default]
    Home,
}

pub struct App {
    pub view: View,
    pub problems: Vec<ProblemSummary>,
    pub table_state: TableState,
    pub user_data: Option<UserProfile>,
}

impl App {
    pub fn new() -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));

        let mock_problems = vec![
            ProblemSummary {
                ac_rate: 51.2,
                difficulty: api::Difficulty::Easy,
                frontend_question_id: "1".to_string(),
                is_favor: false,
                paid_only: false,
                status: Some("ac".to_string()),
                title: "Two Sum".to_string(),
                title_slug: "two-sum".to_string(),
                topic_tags: vec![],
            },
            ProblemSummary {
                ac_rate: 35.4,
                difficulty: api::Difficulty::Medium,
                frontend_question_id: "2".to_string(),
                is_favor: true,
                paid_only: false,
                status: None,
                title: "Add Two Numbers".to_string(),
                title_slug: "add-two-numbers".to_string(),
                topic_tags: vec![],
            },
            ProblemSummary {
                ac_rate: 15.1,
                difficulty: api::Difficulty::Hard,
                frontend_question_id: "4".to_string(),
                is_favor: false,
                paid_only: false,
                status: None,
                title: "Median of Two Sorted Arrays".to_string(),
                title_slug: "median-of-two-sorted-arrays".to_string(),
                topic_tags: vec![],
            },
        ];

        let user_data = UserProfile {
            matched_user: Some(api::MatchedUser {
                username: "LeetCoderPro".to_string(),
                github_url: Some("username".to_string()),
                twitter_url: None,
                linkedin_url: Some("linkedin.com/user".to_string()),
                profile: api::Profile {
                    ranking: 154320,
                    reputation: 42,
                    user_avatar: "https://assets.leetcode.com/users/avatar.jpg".to_string(),
                },
            }),
        };

        Self {
            view: View::default(),
            problems: mock_problems,
            table_state,
            user_data: Some(user_data),
        }
    }

    pub fn render(&mut self, f: &mut Frame) {
        match self.view {
            View::Home => self.render_home_view(f),
        }
    }

    pub fn next(&mut self) {
        let i = self
            .table_state
            .selected()
            .map(|i| {
                if i >= self.problems.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            })
            .unwrap_or_default();

        self.table_state.select(Some(i));
    }

    pub fn prev(&mut self) {
        let i = self
            .table_state
            .selected()
            .map(|i| {
                if i == 0 {
                    self.problems.len().saturating_sub(1)
                } else {
                    i - 1
                }
            })
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
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(outer_layout[1]);

        render::user_profile(f, main_chunks[1], self);
        render::search_bar(f, main_chunks[3], self);
        render::problem_list(f, main_chunks[5], self);
        render::home_controls(f, main_chunks[6], self);
    }
}
