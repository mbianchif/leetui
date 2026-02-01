use std::time::Duration;

use api::LeetCodeClient;
use ratatui::crossterm::event::{self, Event, KeyCode};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::Interval,
};

use crate::app::Action;

pub async fn spawn_keyboard(tx: Sender<Action>) {
    while !tx.is_closed() {
        if event::poll(Duration::from_millis(16)).unwrap_or_default() {
            if let Event::Key(key) = event::read().unwrap() {
                let action = match key.code {
                    KeyCode::Char('q') => Some(Action::Quit),
                    KeyCode::Char('j') => Some(Action::MoveDown),
                    KeyCode::Char('k') => Some(Action::MoveUp),
                    _ => None,
                };

                if let Some(action) = action {
                    let _ = tx.send(action).await;
                }
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
    FetchProfile { username: String },
    FetchUserStatus,
    FetchProblems { skip: usize, limit: usize },
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
            ClientRequest::FetchProblems { skip, limit } => client
                .get_problem_list(skip, limit)
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
