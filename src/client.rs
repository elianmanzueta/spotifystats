use chrono::Duration;
use futures::{pin_mut, TryStreamExt};
use rspotify::clients::OAuthClient;
use rspotify::model::{SimplifiedArtist, TimeRange};
use rspotify::scopes;
use rspotify::{AuthCodeSpotify, Credentials, OAuth};

async fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.num_seconds();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;

    // Format the string as "minutes:seconds", ensuring seconds are always two digits
    format!("{}:{:02}", minutes, seconds)
}

pub async fn auth(creds: Credentials) -> AuthCodeSpotify {
    let oauth = OAuth::from_env(scopes!("user-top-read")).unwrap();

    let spotify = AuthCodeSpotify::new(creds, oauth);

    if let Ok(url) = spotify.get_authorize_url(true) {
        spotify.prompt_for_token(&url).await.unwrap();
    } else {
        println!("Couldn't perform OAuth authentication.");
    }

    spotify
}

pub async fn get_user_display_name(client: &AuthCodeSpotify) -> String {
    let user = client.me().await.unwrap();
    user.display_name.unwrap()
}

pub struct TopTrack {
    pub place: u32,
    pub name: String,
    pub duration: String,
    pub artists: Vec<SimplifiedArtist>,
}

pub struct TopArtist {
    pub place: u32,
    pub name: String,
}

pub async fn get_top_tracks(
    client: &AuthCodeSpotify,
    time_range: TimeRange,
    limit: u8,
) -> Vec<TopTrack> {
    let stream = client.current_user_top_tracks(Some(time_range));
    pin_mut!(stream);

    let mut top_tracks: Vec<TopTrack> = Vec::new();

    while let Some(item) = stream.try_next().await.unwrap() {
        let mut artists: Vec<SimplifiedArtist> = Vec::new();

        for artist in item.artists {
            artists.push(artist)
        }

        let place = top_tracks.len() as u32 + 1;

        let duration = format_duration(item.duration);

        top_tracks.push(TopTrack {
            place,
            name: item.name,
            duration: duration.await,
            artists,
        });

        if top_tracks.len() as u8 == limit {
            break;
        }
    }

    top_tracks
}

pub async fn get_top_artists(client: &AuthCodeSpotify, limit: u8) -> Vec<TopArtist> {
    let stream = client.current_user_top_artists(Some(TimeRange::MediumTerm));
    pin_mut!(stream);

    let mut top_artists: Vec<TopArtist> = Vec::new();

    while let Some(item) = stream.try_next().await.unwrap() {
        let place = top_artists.len() as u32 + 1;

        top_artists.push(TopArtist {
            place,
            name: item.name,
        });

        if top_artists.len() as u8 == limit {
            break;
        }
    }
    top_artists
}
