pub mod client;
use std::time::Duration;

use crate::client::{get_env_var, get_top_tracks, Client};
use app2::{update, view, Message, Model, RunningState};
use client::{get_top_artists, get_user_display_name, TopArtist, TopArtists, TopTrack, TopTracks};

pub mod app2;
use crossterm::event::{self, Event, KeyCode};
use dotenvy::dotenv;
use rspotify::{model::TimeRange, AuthCodeSpotify, ClientError, Credentials};

async fn authenticate() -> AuthCodeSpotify {
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

    match cred.auth().await {
        Some(client) => {
            client
        },
        None => {
            panic!("Authentication failed.")
        }
    }
}

struct UserResults {
    tracks: Vec<TopTracks>,
    artists: Vec<TopArtists>,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenv().ok();
    color_eyre::install()?;

    let mut model = Model {
        time_range: TimeRange::ShortTerm,
        limit: 10,
        running_state: RunningState::Running,
        ..Default::default()
    };

    let client = authenticate().await;

    let display_name = get_user_display_name(&client).await;

    let top_tracks = get_top_tracks(&client, model.time_range, model.limit as u8).await?;

    let top_artists = get_top_artists(&client, model.time_range, model.limit as u8).await?;

    tui::install_panic_hook();

    let mut terminal = tui::init_terminal()?;

    while model.running_state != RunningState::Done {
        terminal.draw(|f| view(&mut model, f))?;

        let mut current_msg = handle_event(&model)?;

        while current_msg.is_some() {
            current_msg = update(&mut model, current_msg.unwrap());
        }
    }

    tui::restore_terminal()?;
    Ok(())
}

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
        KeyCode::Char('m') => Some(Message::ChangeTimeRange),
        KeyCode::Char('q') => Some(Message::Quit),

        _ => None,
    }
}

mod tui {
    use ratatui::{
        backend::{Backend, CrosstermBackend},
        crossterm::{
            terminal::{
                disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
            },
            ExecutableCommand,
        },
        Terminal,
    };
    use std::{io::stdout, panic};

    pub fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Ok(terminal)
    }

    pub fn restore_terminal() -> color_eyre::Result<()> {
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn install_panic_hook() {
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            stdout().execute(LeaveAlternateScreen).unwrap();
            disable_raw_mode().unwrap();
            original_hook(panic_info);
        }));
    }
}
