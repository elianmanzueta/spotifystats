pub mod client;
use crate::client::{get_env_var, get_top_tracks, Client};
use app::App;
use client::{get_top_artists, get_user_display_name};

pub mod app;
use dotenvy::dotenv;
use rspotify::{model::TimeRange, AuthCodeSpotify, Credentials};

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

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenv().ok();
    color_eyre::install()?;

    let mut app = App {
        time_range: TimeRange::ShortTerm,
        result_limit: 10,
        ..Default::default()
    };

    let client = if let Some(client) = authenticate().await {
        client
    } else {
        println!("Authentication failed...");
        return Ok(());
    };

    let username = get_user_display_name(&client).await;
    app.username = username;

    let top_tracks = get_top_tracks(&client, app.time_range, app.result_limit).await?;
    app.top_tracks = top_tracks;

    let top_artists = get_top_artists(&client, app.time_range, app.result_limit).await?;
    app.top_artists = top_artists;

    println!("Hello {}!", app.username);

    // TUI
    let terminal = ratatui::init();
    let result = app.run(terminal);
    ratatui::restore();
    result
}
