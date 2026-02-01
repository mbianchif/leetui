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

/// Retrieves the needed LeetCode variables to create the `LeetCodeClient` api.
///
/// # Returns
/// The variables or an error if the variables are undefined.
fn retrieve_leetcode_vars() -> Result<(String, String), &'static str> {
    let session = env::var("LEETCODE_SESSION").map_err(|_| "LEETCODE_SESSION is not defined")?;
    let csrf = env::var("CSRF_TOKEN").map_err(|_| "CSRF_TOKEN is not defined")?;
    Ok((session, csrf))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (session, csrf) = retrieve_leetcode_vars()?;

    let (client_tx, client_rx) = mpsc::channel(10);
    let (action_tx, action_rx) = mpsc::channel(100);
    let throbber_interval = time::interval(Duration::from_millis(30));
    let client = LeetCodeClient::new(session, csrf)?;

    // Initialize the input listeners.
    tokio::spawn(handler::spawn_keyboard(action_tx.clone()));
    tokio::spawn(handler::spawn_ticker(action_tx.clone(), throbber_interval));
    tokio::spawn(handler::spawn_client(action_tx, client_rx, client));

    // Setup the terminal backend.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    if env::args().find(|arg| arg == "--editor").is_some() {
        // do nothing yet...
    } else {
        let mut app = PickerApp::new(client_tx).await;
        app.event_loop(&mut terminal, action_rx).await?;
    }

    // Cleanup.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
