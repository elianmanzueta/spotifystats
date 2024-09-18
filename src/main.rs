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
use rspotify::{model::TimeRange, Credentials};

#[derive(Debug, Default)]
pub struct App {
    running: bool,
    output: String,
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

        let area = center(
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

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenv().ok();
    color_eyre::install()?;

    let id = get_env_var("RSPOTIFY_CLIENT_ID");
    let secret = get_env_var("RSPOTIFY_CLIENT_SECRET");
    let redirect_uri = get_env_var("RSPOTIFY_REDIRECT_URI");
    let time_range = TimeRange::ShortTerm;

    let mut app = App::new();

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
            panic!("!!!")
        }
    };

    let top_tracks = get_top_tracks(&client, time_range, 10).await?;
    let mut tracks = String::new();

    for (index, track) in top_tracks.iter().enumerate() {
        let artist_names = track
            .artists
            .iter() // Iterate over the artists
            .take(2)
            .map(|artist| artist.name.as_str()) // Map each artist to their name
            .collect::<Vec<&str>>() // Collect into a Vec<&String>
            .join(", ");

        tracks.push_str(&format!("{} - {} by {}\n", index, track.name, artist_names));
    }

    app.output = tracks;

    let terminal = ratatui::init();
    let result = app.run(terminal);
    ratatui::restore();
    result
}
