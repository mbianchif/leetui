mod app;

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

use app::{App, handler, picker::PickerApp};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let Ok(session) = env::var("LEETCODE_SESSION") else {
        return Err("LEETCODE_SESSION is not defined".into());
    };

    let Ok(csrf) = env::var("CSRF_TOKEN") else {
        return Err("CSRF_TOKEN is not defined".into());
    };

    let (client_tx, client_rx) = mpsc::channel(10);
    let (action_tx, action_rx) = mpsc::channel(100);
    let throbber_interval = time::interval(Duration::from_millis(30));
    let client = LeetCodeClient::new(session, csrf)?;

    // Initialize the input handlers.
    tokio::spawn(handler::spawn_keyboard(action_tx.clone()));
    tokio::spawn(handler::spawn_ticker(action_tx.clone(), throbber_interval));
    tokio::spawn(handler::spawn_client(action_tx, client_rx, client));
    let mut app = PickerApp::new(client_tx).await;

    // Setup the terminal backend.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    app.event_loop(&mut terminal, action_rx).await?;

    // Cleanup.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
