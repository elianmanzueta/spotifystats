use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{block::Title, Block, BorderType, Paragraph, ScrollbarState, Wrap},
    Frame,
};

use rspotify::{model::TimeRange, AuthCodeSpotify};

use crate::client::{TopArtists, TopTracks};

pub struct Model {
    pub running_state: RunningState,
    pub time_range: TimeRange,
    pub display_name: String,
    pub limit: usize,
    pub client: AuthCodeSpotify,
    pub top_tracks: TopTracks,
    pub top_artists: TopArtists,
    pub scrollbar_state: ScrollbarState,
    pub scroll_position: usize,
}

impl Model {
    pub fn new() -> Model {
        Model {
            running_state: RunningState::Running,
            display_name: "None".to_string(),
            limit: 10,
            scrollbar_state: ScrollbarState::default(),
            scroll_position: 0,
            top_tracks: TopTracks {
                time_range: TimeRange::ShortTerm,
                tracks: Vec::new(),
            },
            top_artists: TopArtists {
                time_range: TimeRange::ShortTerm,
                artists: Vec::new(),
            },
            time_range: TimeRange::ShortTerm,
            client: AuthCodeSpotify::default(),
        }
    }

    pub fn parse_top_tracks_output(&self) -> Text {
        let mut lines = Text::default();

        for result in self.top_tracks.iter() {
            let index = result.index;
            let track_name = result.track_name;
            let artists = result.artists.join(", ");
            let duration = result.duration;

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

    pub fn parse_top_artists_output(&self) -> Text {
        let mut lines = Text::default();

        for track in self.top_artists.iter() {
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

    fn show_time_range(&self) -> Title<'_> {
        match self.time_range {
            TimeRange::ShortTerm => Title::from("Short Term"),
            TimeRange::MediumTerm => Title::from("Medium Term"),
            TimeRange::LongTerm => Title::from("Long Term"),
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
    ChangeTimeRange,
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
        Message::ChangeTimeRange => match model.time_range {
            TimeRange::LongTerm => model.time_range = TimeRange::ShortTerm,
            TimeRange::MediumTerm => model.time_range = TimeRange::LongTerm,
            TimeRange::ShortTerm => model.time_range = TimeRange::MediumTerm,
        },
    };
    None
}

pub fn render_top_tracks_widget(model: &mut Model, frame: &mut Frame, area: Rect) {
    let style = Style::new().green();

    let title = model.show_time_range();
    let output = model.parse_top_tracks_output();

    let widget = Paragraph::new(output)
        .scroll((0, 0))
        .block(
            Block::bordered()
                .border_type(BorderType::QuadrantInside)
                .title(title)
                .border_style(style)
                .title_alignment(Alignment::Center),
        )
        .wrap(Wrap { trim: true })
        .centered();

    frame.render_widget(widget, area);
}

pub fn render_top_artists_widget(model: &mut Model, frame: &mut Frame, area: Rect) {
    let style = Style::new().green();

    let title = model.show_time_range();
    let output = model.parse_top_artists_output();

    let widget = Paragraph::new(output)
        .scroll((0, 0))
        .block(
            Block::bordered()
                .border_type(BorderType::QuadrantInside)
                .title(title)
                .border_style(style)
                .title_alignment(Alignment::Center),
        )
        .wrap(Wrap { trim: true })
        .centered();

    frame.render_widget(widget, area);
}

pub fn view(model: &mut Model, frame: &mut Frame) {
    let layout = Layout::new(
        Direction::Vertical,
        vec![Constraint::Fill(1), Constraint::Fill(1)],
    )
    .split(frame.area());

    render_top_tracks_widget(model, frame, layout[0]);
    render_top_artists_widget(model, frame, layout[1])
}
