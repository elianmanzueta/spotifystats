use crate::client::{get_env_var, get_top_tracks, Client};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use dotenvy::dotenv;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::{Block, Paragraph},
    DefaultTerminal, Frame,
};
use rspotify::{AuthCodeSpotify, Credentials};

async fn get_client() -> AuthCodeSpotify {
    dotenv().ok();
    let id = get_env_var("RSPOTIFY_CLIENT_ID");
    let secret = get_env_var("RSPOTIFY_CLIENT_SECRET");
    let redirect_uri = get_env_var("RSPOTIFY_REDIRECT_URI");

    let client = Client {
        creds: Credentials {
            id,
            secret: Some(secret),
        },
        redirect_uri,
    };

    client
        .auth()
        .await
        .unwrap_or_else(|| panic!("Client not authenticated."))
}

#[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    running: bool,
    auth: bool,
    input: String,
    tracks: Vec<String>,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/master/examples>
    fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(4),
            Constraint::Length(2),
            Constraint::Min(1),
        ]);

        let [help_area, input_area, output_area] = vertical.areas(frame.area());

        let help_text = Paragraph::new(
            "Welcome to Spotify Stats!\nPress enter to start the authentication process.",
        )
        .block(Block::bordered())
        .centered();

        let input = Paragraph::new("Input here").block(Block::bordered());
        frame.render_widget(help_text, help_area);
        frame.render_widget(input, input_area);
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }
    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
