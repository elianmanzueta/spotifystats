use chrono::Duration;
use futures::{pin_mut, TryStreamExt};
use rspotify::clients::OAuthClient;
use rspotify::model::{SimplifiedArtist, TimeRange};
use rspotify::{scopes, ClientError};
use rspotify::{AuthCodeSpotify, Credentials, OAuth};

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.num_seconds();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;

    // Format the string as "minutes:seconds", ensuring seconds are always two digits
    format!("{}:{:02}", minutes, seconds)
}

pub struct Client {
    pub creds: Credentials,
    pub redirect_uri: String,
}

impl Client {
    pub async fn auth(&self) -> Option<AuthCodeSpotify> {
        let oauth = match OAuth::from_env(scopes!("user-top-read")) {
            Some(oauth) => oauth,
            None => {
                println!("Failed to retrieve OAuth from environment.");
                return None;
            }
        };

        let spotify = AuthCodeSpotify::new(self.creds.clone(), oauth);

        if let Ok(url) = spotify.get_authorize_url(true) {
            if let Err(e) = spotify.prompt_for_token(&url).await {
                println!("Failed to prompt for token: {:?}", e);
            }
        } else {
            println!("Couldn't perform OAuth authentication.");
        }

        Some(spotify)
    }
}

pub async fn get_user_display_name(client: &AuthCodeSpotify) -> String {
    match client.me().await {
        Ok(user) => match user.display_name {
            Some(username) => username,
            None => "Unknown User".to_string(),
        },
        Err(e) => {
            println!("Error fetching user: {:?}", e);
            "Unknown User".to_string()
        }
    }
}

pub struct TopTrack {
    pub place: u32,
    pub name: String,
    pub duration: String,
    pub artists: Vec<SimplifiedArtist>,
}

pub async fn get_top_tracks(
    client: &AuthCodeSpotify,
    time_range: TimeRange,
    limit: u8,
) -> Result<Vec<TopTrack>, ClientError> {
    let stream = client.current_user_top_tracks(Some(time_range));
    pin_mut!(stream);

    let mut top_tracks: Vec<TopTrack> = Vec::new();

    while let Some(item) = stream.try_next().await? {
        let place = top_tracks.len() as u32 + 1;
        let duration = format_duration(item.duration);

        top_tracks.push(TopTrack {
            place,
            name: item.name,
            duration,
            artists: item.artists,
        });

        if top_tracks.len() as u8 == limit {
            break;
        }
    }

    Ok(top_tracks)
}

pub struct TopArtist {
    pub place: u32,
    pub name: String,
}

pub async fn get_top_artists(
    client: &AuthCodeSpotify,
    limit: u8,
) -> Result<Vec<TopArtist>, ClientError> {
    let stream = client.current_user_top_artists(Some(TimeRange::MediumTerm));
    pin_mut!(stream);

    let mut top_artists: Vec<TopArtist> = Vec::new();

    while let Some(item) = stream.try_next().await? {
        let place = top_artists.len() as u32 + 1;

        top_artists.push(TopArtist {
            place,
            name: item.name,
        });

        if top_artists.len() as u8 == limit {
            break;
        }
    }
    Ok(top_artists)
}
