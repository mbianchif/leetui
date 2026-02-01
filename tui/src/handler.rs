use std::time::Duration;

use api::LeetCodeClient;
use ratatui::crossterm::event::{self, Event};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::Interval,
};

use crate::app::Action;

pub async fn spawn_keyboard(tx: Sender<Action>) {
    while !tx.is_closed() {
        if event::poll(Duration::from_millis(8)).unwrap_or_default() {
            if let Event::Key(key_event) = event::read().unwrap() {
                let _ = tx.send(Action::Key(key_event)).await;
            }
        }
    }
}

pub async fn spawn_ticker(tx: Sender<Action>, mut interval: Interval) {
    loop {
        interval.tick().await;
        let _ = tx.send(Action::Tick).await;
    }
}

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
}

pub async fn spawn_client(
    tx: Sender<Action>,
    client: LeetCodeClient,
    mut rx: Receiver<ClientRequest>,
) -> api::Result<()> {
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

    Ok(())
}
