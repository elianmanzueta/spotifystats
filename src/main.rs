pub mod client;
use core::panic;

use client::{get_top_tracks, Client};
use dotenvy::dotenv;
use rspotify::{model::TimeRange, Credentials};

fn get_env_var(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("Variable not found: {}", key))
}

#[tokio::main]
async fn main() {
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

    let authenticated_client = client.auth().await.unwrap_or_else(|| panic!("Client not authenticated."));

    match get_top_tracks(&authenticated_client, TimeRange::LongTerm, 8).await {
        Ok(top_tracks) => {
            for track in top_tracks {
                let song_name = track.name;
                let artists: Vec<String> = track
                    .artists
                    .iter()
                    .map(|artist| artist.name.clone())
                    .collect();

                let artists = artists.join(", ");
                println!(
                    "{}. {} - {} ({})",
                    track.place, song_name, artists, track.duration
                );
            }
        }
        Err(e) => {
            panic!("Failed to retrieve user's top tracks: {:?}", e);
        }
    }
}
