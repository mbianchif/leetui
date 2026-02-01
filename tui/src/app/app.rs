use api::{MatchedUser, ProblemSummary, UserStatus};
use ratatui::{Frame, Terminal, crossterm::event, prelude::Backend};
use tokio::{sync::mpsc::Receiver, task};

/// The types of events that exist in both apps.
pub enum Action {
    Key(event::KeyEvent),

    Tick,

    UserStatusLoaded(UserStatus),
    UserProfileLoaded(MatchedUser),
    ProblemListLoaded(Vec<ProblemSummary>),
    DailyChallengeLoaded(ProblemSummary),

    NetworkError(String),
}

/// The main application trait that both the picker and the editor view application must implement.
pub trait App {
    fn render(&mut self, frame: &mut Frame);

    fn update(&mut self, action: Action) -> bool;

    /// Implements the main event loop for both the `Picker` and the `Editor`.
    ///
    /// # Arguments
    /// * `terminal` - The ratatui `Terminal` to use.
    /// * `rx` - An events listener.
    ///
    /// # Returns
    /// An error if there is an issue drawing to the screen.
    async fn event_loop<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        mut rx: Receiver<Action>,
    ) -> Result<(), B::Error> {
        loop {
            terminal.draw(|f| self.render(f))?;
            if let Some(action) = rx.recv().await {
                if !task::block_in_place(|| self.update(action)) {
                    break;
                }
            }
        }

        Ok(())
    }
}
