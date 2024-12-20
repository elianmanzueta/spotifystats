use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{block::Title, Block, BorderType, Paragraph, ScrollbarState, Wrap},
    Frame,
};

use rspotify::model::TimeRange;

use crate::client::{TopArtistResult, TopTrackResult};

pub struct Model {
    pub running_state: RunningState,
    pub time_range: TimeRange,
    pub username: String,
    pub limit: usize,
    pub top_tracks: Vec<TopTrackResult>,
    pub top_artists: Vec<TopArtistResult>,
    pub scrollbar_state: ScrollbarState,
    pub scroll_position: usize,
}

impl Model {
    pub fn new() -> Model {
        Model {
            running_state: RunningState::Done,
            username: "None".to_string(),
            limit: 10,
            scrollbar_state: ScrollbarState::default(),
            scroll_position: 0,
            top_tracks: Vec::new(),
            top_artists: Vec::new(),
            time_range: TimeRange::ShortTerm,
        }
    }

    pub fn top_artists_widget(&mut self) -> Paragraph {
        let output = self.parse_top_artists_output();
        let style = Style::new().green();
        let widget = Paragraph::new(output)
            .block(
                Block::bordered()
                    .border_type(BorderType::QuadrantInside)
                    .border_style(style)
                    .title_alignment(Alignment::Center),
            )
            .wrap(Wrap { trim: true })
            .centered();
        widget
    }

    pub fn parse_top_tracks_output(&mut self) -> Text {
        let mut lines = Text::default();

        for result in &self.top_artists {
            let index = track.index;
            let track_name = &track.track_name;
            let artists = track.artists.join(", ");
            let duration = &track.duration;

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
    pub fn parse_top_artists_output(&mut self) -> Text {
        let mut lines = Text::default();

        for track in &self.top_artists {
            let index = track.index;
            let artist_name = track.artist_name.clone();
            let artist_genres = track.genres.clone();

            let result = vec![
                Span::styled(
                    index.to_string(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::ITALIC),
                ),
                Span::styled(" - ", Style::default()),
                Span::styled(artist_name, Style::default()),
                Span::styled(" ", Style::default()),
                Span::styled(format!("({})", artist_genres), Style::default()),
            ];

            let text: Vec<Line<'_>> = vec![result.into()];
            lines.extend(text)
        }
        lines
    }

    fn show_time_range(time_range: &TimeRange) -> String {
        match time_range {
            TimeRange::ShortTerm => "Short Term".to_string(),
            TimeRange::MediumTerm => "Medium Term".to_string(),
            TimeRange::LongTerm => "Long Term".to_string(),
        }
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(Eq, PartialEq)]
pub enum Message {
    ScrollUp,
    ScrollDown,
    Quit,
}

pub fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::ScrollDown => {
            model.scroll_position = model.scroll_position.saturating_add(1);
            model.scrollbar_state = model.scrollbar_state.position(model.scroll_position);
        }
        Message::ScrollUp => {
            model.scroll_position = model.scroll_position.saturating_sub(1);
            model.scrollbar_state = model.scrollbar_state.position(model.scroll_position)
        }
        Message::Quit => model.running_state = RunningState::Done,
    };
    None
}

pub fn render_top_tracks(model: &mut Model, frame: &mut Frame, area: Rect) {
    let style = Style::new().green();

    let output = model.parse_top_tracks_output();

    let widget = Paragraph::new(output)
        .scroll((0, 0))
        .block(
            Block::bordered()
                .border_type(BorderType::QuadrantInside)
                .border_style(style)
                .title_alignment(Alignment::Center),
        )
        .wrap(Wrap { trim: true })
        .centered();

    frame.render_widget(widget, area);
}

pub fn draw(model: &mut Model, frame: &mut Frame) {
    let layout = Layout::new(
        Direction::Vertical,
        vec![Constraint::Fill(1), Constraint::Fill(1)],
    )
    .split(frame.area());

    render_top_tracks(model, frame, layout[0]);
}
/// Convert Event to Message
///
/// We don't need to pass in a `model` to this function in this example
/// but you might need it as your project evolves
fn handle_event(_: &Model) -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key));
            }
        }
    }
    Ok(None)
}

fn handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('j') => Some(Message::ScrollDown),
        KeyCode::Char('k') => Some(Message::ScrollUp),
        KeyCode::Char('q') => Some(Message::Quit),
        _ => None,
    }
}
