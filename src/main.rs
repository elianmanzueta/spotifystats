use dotenvy::dotenv;
use futures::{pin_mut, TryStreamExt};
use rspotify::clients::OAuthClient;
use rspotify::model::{SimplifiedArtist, TimeRange};
use rspotify::scopes;
use rspotify::{AuthCodeSpotify, Credentials, OAuth};

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
    let user_top_tracks = get_top_tracks(&client, limit).await;

    for track in user_top_tracks {
        let song_name = track.name;
        let artists: Vec<String> = track
            .artists
            .iter()
            .map(|artist| artist.name.clone())
            .collect();
        let artists = artists.join(", ");
        println!("{}. {} - {}", track.place, song_name, artists);
    }
}

fn get_env_var(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("Variable not found: {}", key))
}

async fn auth(creds: Credentials) -> AuthCodeSpotify {
    let oauth = OAuth::from_env(scopes!("user-top-read")).unwrap();

    let spotify = AuthCodeSpotify::new(creds, oauth);

    if let Ok(url) = spotify.get_authorize_url(true) {
        spotify.prompt_for_token(&url).await.unwrap();
    } else {
        println!("Couldn't perform OAuth authentication.");
    }

    spotify
}

async fn get_user_display_name(client: &AuthCodeSpotify) -> String {
    let user = client.me().await.unwrap();
    user.display_name.unwrap()
}

struct TopTrack {
    place: u32,
    name: String,
    artists: Vec<SimplifiedArtist>,
}

async fn get_top_tracks(client: &AuthCodeSpotify, limit: u8) -> Vec<TopTrack> {
    let stream = client.current_user_top_tracks(Some(TimeRange::ShortTerm));
    pin_mut!(stream);

    let mut top_tracks: Vec<TopTrack> = Vec::new();


    while let Some(item) = stream.try_next().await.unwrap() {
        let mut artists: Vec<SimplifiedArtist> = Vec::new();
        for artist in item.artists {
            artists.push(artist)
        }

        let place = top_tracks.len() as u32 + 1;

        top_tracks.push(TopTrack {
            place,
            name: item.name,
            artists,
        });

        if top_tracks.len() as u8 == limit {
            break;
        }
    }

    top_tracks
}
