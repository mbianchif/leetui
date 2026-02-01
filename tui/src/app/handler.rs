use std::time::Duration;

use api::LeetCodeClient;
use ratatui::crossterm::event::{self, Event};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::Interval,
};

use crate::app::Action;

/// Creates the keyboard listener future.
///
/// # Arguments
/// * `tx` - A sending end to send the key events to the application.
pub async fn spawn_keyboard(tx: Sender<Action>) {
    while !tx.is_closed() {
        if event::poll(Duration::from_millis(8)).unwrap_or_default() {
            if let Event::Key(key_event) = event::read().unwrap() {
                let _ = tx.send(Action::Key(key_event)).await;
            }
        }
    }
}

/// Creates the ticker listener future.
///
/// # Arguments
/// * `tx` - A sending end to send the tick events to the application.
/// * `interval` - The interval to sleep in between ticks.
pub async fn spawn_ticker(tx: Sender<Action>, mut interval: Interval) {
    loop {
        interval.tick().await;
        let _ = tx.try_send(Action::Tick);
    }
}

/// The variants of requests the application can make to the client listener.
pub enum ClientRequest {
    FetchProfile {
        username: String,
    },
    FetchUserStatus,
    FetchProblems {
        skip: usize,
        limit: usize,
        search: Option<String>,
    },
    FetchDailyChallenge,
}

/// Creates the client listener future.
///
/// # Arguments
/// * `tx` - A sending end to send the leetcode api responses to the application.
/// * `rx` - A receiving end to receive the application requests.
/// * `client` - The LeetCode api abstraction.
pub async fn spawn_client(
    tx: Sender<Action>,
    mut rx: Receiver<ClientRequest>,
    client: LeetCodeClient,
) {
    while let Some(req) = rx.recv().await {
        let result = match req {
            ClientRequest::FetchUserStatus => {
                client.get_status().await.map(Action::UserStatusLoaded)
            }
            ClientRequest::FetchProfile { username } => client
                .get_profile(&username)
                .await
                .map(Action::UserProfileLoaded),
            ClientRequest::FetchProblems {
                skip,
                limit,
                search: None,
            } => client
                .get_problem_list(skip, limit)
                .await
                .map(|p| Action::ProblemListLoaded(p.questions)),
            ClientRequest::FetchProblems {
                skip,
                limit,
                search: Some(keywords),
            } => client
                .search_problem(skip, limit, &keywords)
                .await
                .map(|p| Action::ProblemListLoaded(p.questions)),
            ClientRequest::FetchDailyChallenge => client
                .get_daily_challenge()
                .await
                .map(|p| Action::DailyChallengeLoaded(p.question)),
        };

        match result {
            Ok(action) => {
                let _ = tx.send(action).await;
            }
            Err(e) => {
                let _ = tx.send(Action::NetworkError(e.to_string())).await;
            }
        }
    }
}
