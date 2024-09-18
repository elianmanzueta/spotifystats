pub mod client;
use crate::client::{get_env_var, get_top_tracks, Client};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use dotenvy::dotenv;
use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Paragraph},
    DefaultTerminal, Frame,
};
use rspotify::{
    model::{SimplifiedArtist, TimeRange},
    AuthCodeSpotify, Credentials,
};

#[derive(Debug)]
pub struct App {
    running: bool,
    output: String, // TODO: Change into vector? Want to make stylized spans. Save whole struct?
    // username TODO: Get username (or user info?)
    // query_type TODO: Show something like "{User}'s top {tracks|artists"
    result_limit: u8,
    time_range: TimeRange,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: false,
            output: "".to_string(),
            result_limit: 10,
            time_range: TimeRange::ShortTerm,
        }
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
        let title = Span::styled("Your Spotify Stats!", Style::default().fg(Color::Green))
            .add_modifier(Modifier::BOLD);
        let output = Paragraph::new(self.output.clone())
            .block(
                Block::bordered()
                    .title(title.clone())
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded),
            )
            .centered();

        let area = centered_widget(
            frame.area(),
            Constraint::Length(100),
            Constraint::Length(10),
        );

        frame.render_widget(output, area)
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

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn centered_widget(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

async fn authenticate() -> Option<AuthCodeSpotify> {
    dotenv().ok();
    let id = get_env_var("RSPOTIFY_CLIENT_ID");
    let secret = get_env_var("RSPOTIFY_CLIENT_SECRET");
    let redirect_uri = get_env_var("RSPOTIFY_REDIRECT_URI");

    let cred = Client {
        creds: Credentials {
            id,
            secret: Some(secret),
        },
        redirect_uri,
    };

    let client = match cred.auth().await {
        Some(client) => client,
        None => {
            panic!("Authentication failed.")
        }
    };

    Some(client)
}

#[derive(Debug)]
struct TopTracksResult {
    index: usize,
    track_name: String,
    artists: Vec<String>,
}

fn get_user(client: &AuthCodeSpotify) {}

fn get_artists(artists: Vec<SimplifiedArtist>) -> Vec<String> {
    let artists: Vec<String> = artists.iter().map(|artist| artist.name.clone()).collect();
    artists
}

async fn run_get_top_tracks(
    client: &AuthCodeSpotify,
    time_range: TimeRange,
    limit: u8,
) -> Result<Vec<TopTracksResult>> {
    let top_tracks = get_top_tracks(client, time_range, limit).await?;
    let mut result: Vec<TopTracksResult> = Vec::new();

    for (index, track) in top_tracks.iter().enumerate() {
        let top_track = TopTracksResult {
            index: index + 1,
            track_name: track.name.clone(),
            artists: get_artists(track.artists.clone()),
        };

        result.push(top_track);
    }

    Ok(result)
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenv().ok();
    color_eyre::install()?;

    let mut app = App::new();

    let time_range = app.time_range;
    let result_limit = app.result_limit;

    let client = if let Some(client) = authenticate().await {
        client
    } else {
        println!("Authentication failed...");
        return Ok(());
    };

    let top_tracks = run_get_top_tracks(&client, time_range, result_limit).await?;

    // TUI
    let terminal = ratatui::init();
    let result = app.run(terminal);
    ratatui::restore();
    result
}
