pub mod client;
use std::time::Duration;

use crate::client::{get_env_var, get_top_tracks, Client};
use app2::{draw, update, Message, Model, RunningState};
use client::{get_top_artists, get_user_display_name};

pub mod app2;
use crossterm::event::{self, Event, KeyCode};
use dotenvy::dotenv;
use rspotify::{model::TimeRange, AuthCodeSpotify, ClientError, Credentials};

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

struct UserResults {
    tracks: Vec<TopTracks>,
    artists: Vec<TopArtists>,
}

async fn get_results(client: &AuthCodeSpotify) -> Result<UserResults, ClientError> {
    let result: UserResults;
    let short_term = get_top_tracks(&client, TimeRange::ShortTerm, 50).await?;
    let medium_term = get_top_tracks(&client, TimeRange::MediumTerm, 50).await?;
    let long_term = get_top_tracks(&client, TimeRange::LongTerm, 50).await?;

    result.tracks.append(short_term)
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenv().ok();
    color_eyre::install()?;

    let mut model = Model {
        time_range: TimeRange::ShortTerm,
        limit: 10,
        ..Default::default()
    };

    let client = if let Some(client) = authenticate().await {
        client
    } else {
        println!("Authentication failed...");
        return Ok(());
    };

    let top_tracks = get_top_tracks(&client, model.time_range, model.limit as u8).await?;
    model.top_tracks = top_tracks;

    let top_artists = get_top_artists(&client, model.time_range, model.limit as u8).await?;
    model.top_artists = top_artists;

    println!("Hello {}!", model.username);
    tui::install_panic_hook();

    let mut terminal = tui::init_terminal()?;

    while model.running_state != RunningState::Done {
        terminal.draw(|f| draw(&mut model, f))?;

        let current_msg = handle_event(&model)?;

        while current_msg.is_some() {
            current_msg = update(&mut model, current_msg.unwrap());
        }
    }

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
