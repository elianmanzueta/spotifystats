pub mod client;
use client::{auth, get_top_artists, get_top_tracks, get_user_display_name};
use dotenvy::dotenv;
use rspotify::{model::TimeRange, Credentials};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let id = get_env_var("RSPOTIFY_CLIENT_ID");
    let secret = get_env_var("RSPOTIFY_CLIENT_SECRET");

    let _redirect_uri = String::from("http://localhost:8080/callback");

    let creds = Credentials {
        id,
        secret: Some(secret),
    };

    let client = auth(creds).await;

    let display_name = get_user_display_name(&client).await;

    println!("{}'s top tracks:\n", display_name);

    let limit: u8 = 10;
    let time_range = TimeRange::ShortTerm;
    let user_top_tracks = get_top_tracks(&client, time_range, limit).await;

    for track in user_top_tracks {
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

    let user_top_artists = get_top_artists(&client, limit).await;

    for artist in user_top_artists {
        println!("{}. {}", artist.place, artist.name);
    }
}

fn get_env_var(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("Variable not found: {}", key))
}
