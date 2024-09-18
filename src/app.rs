use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Paragraph},
    DefaultTerminal, Frame,
};

use rspotify::model::TimeRange;

use crate::client::TopTrackResult;

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub top_tracks: Vec<TopTrackResult>,
    pub username: String,
    pub result_limit: u8,
    pub time_range: TimeRange,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: false,
            top_tracks: Vec::new(),
            username: String::new(),
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
        let layout = Layout::new(
            Direction::Vertical,
            vec![Constraint::Length(10), Constraint::Length(50)],
        )
        .split(frame.area());

        let intro = Span::styled(
            format!("Hello {}!", &self.username),
            Style::default().fg(Color::Green),
        );

        let title = Span::styled("Spotify Stats!", Style::default().fg(Color::Green));

        let title_widget = Paragraph::new(intro)
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .title(title)
                    .fg(Color::Green)
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded),
            )
            .centered();

        let output = self.parse_output();
        let output = Paragraph::new(output)
            .block(Block::bordered().border_type(BorderType::Rounded))
            .centered();

        frame.render_widget(title_widget, layout[0]);
        frame.render_widget(output, layout[1]);
    }

    pub fn parse_output(&mut self) -> Text {
        let mut lines = Text::default();

        for track in &self.top_tracks {
            let index = track.index;
            let track_name = track.track_name.clone();
            let artists = track.artists.join(", ");
            let duration = track.duration.clone();

            let result = vec![
                Span::styled(
                    index.to_string(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::ITALIC),
                ),
                Span::styled(" - ", Style::default()),
                Span::styled(track_name, Style::default()),
                Span::styled(" by ", Style::default()),
                Span::styled(artists, Style::default()),
                Span::styled(
                    format!(" ({})", duration),
                    Style::default().fg(Color::White),
                ),
            ];

            let text: Vec<Line<'_>> = vec![result.into()];
            lines.extend(text)
        }
        lines
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
