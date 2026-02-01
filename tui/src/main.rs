mod app;
mod handler;
mod render;

use std::{env, error::Error, io, time::Duration};

use api::LeetCodeClient;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};
use tokio::{sync::mpsc, time};

use app::App;
use handler::{spawn_client, spawn_keyboard, spawn_ticker};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let Ok(session) = env::var("LEETCODE_SESSION") else {
        return Err("LEETCODE_SESSION is not defined".into());
    };

    let Ok(csrf) = env::var("CSRF_TOKEN") else {
        return Err("CSRF_TOKEN is not defined".into());
    };

    let (client_tx, client_rx) = mpsc::channel(10);
    let (action_tx, mut action_rx) = mpsc::channel(100);
    let throbber_interval = time::interval(Duration::from_millis(60));
    let client = LeetCodeClient::new(session, csrf)?;

    // Initialize the input handlers.
    tokio::spawn(spawn_keyboard(action_tx.clone()));
    tokio::spawn(spawn_ticker(action_tx.clone(), throbber_interval));
    tokio::spawn(spawn_client(action_tx, client, client_rx));
    let mut app = App::new(client_tx).await;

    // Setup the terminal backend.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run event loop.
    loop {
        terminal.draw(|f| app.render(f))?;
        if let Some(action) = action_rx.recv().await {
            if !app.update(action) {
                break;
            }
        }
    }

    // Cleanup.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
