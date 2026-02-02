mod app;

use std::{env, error::Error, time::Duration};

use api::LeetCodeClient;
use ratatui::{Terminal, prelude::Backend};
use tokio::{sync::mpsc, time};

use app::{App, handler};

use crate::app::UpdateResult;

/// Retrieves the needed LeetCode variables to create the `LeetCodeClient` api.
///
/// # Returns
/// The variables or an error if the variables are undefined.
fn retrieve_leetcode_vars() -> Result<(String, String), &'static str> {
    let session = env::var("LEETCODE_SESSION").map_err(|_| "LEETCODE_SESSION is not defined")?;
    let csrf = env::var("CSRF_TOKEN").map_err(|_| "CSRF_TOKEN is not defined")?;
    Ok((session, csrf))
}

/// Setups and runs the entire application.
///
/// # Arguments
/// * `terminal` - The terminal instance to use.
///
/// # Returns
/// Any generic error the application has.
async fn run_app<B: Backend + 'static>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
    let (client_tx, client_rx) = mpsc::channel(100);
    let (action_tx, mut action_rx) = mpsc::channel(100);
    let throbber_interval = time::interval(Duration::from_millis(30));

    let (session, csrf) = retrieve_leetcode_vars()?;
    let client = LeetCodeClient::new(session.clone(), csrf.clone())?;
    let mut app = App::new(client_tx);

    // Initialize the input listeners.
    tokio::spawn(handler::spawn_keyboard(action_tx.clone()));
    tokio::spawn(handler::spawn_ticker(action_tx.clone(), throbber_interval));
    tokio::spawn(handler::spawn_client(action_tx, client_rx, client));

    while let Some(action) = action_rx.recv().await {
        match app.update(action) {
            UpdateResult::Continue => {
                terminal.draw(|f| app.render(f))?;
            }
            UpdateResult::SkipRendering => {}
            UpdateResult::Exit => break,
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();
    let res = run_app(&mut terminal).await;
    ratatui::restore();
    res
}
