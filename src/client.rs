use chrono::Duration;
use futures::{pin_mut, TryStreamExt};
use rspotify::clients::OAuthClient;
use rspotify::model::{SimplifiedArtist, TimeRange};
use rspotify::{scopes, ClientError};
use rspotify::{AuthCodeSpotify, Credentials, OAuth};

pub fn get_env_var(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("Variable not found: {}", key))
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.num_seconds();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;

    format!("{}:{:02}", minutes, seconds)
}

pub struct Client {
    pub creds: Credentials,
    pub redirect_uri: String,
}

impl Client {
    pub fn new() -> Client {
        Client {
            creds: Credentials {
                id: String::from(""),
                secret: Some(String::from("")),
            },
            redirect_uri: String::from(""),
        }
    }
    pub async fn auth(&self) -> Option<AuthCodeSpotify> {
        let oauth = match OAuth::from_env(scopes!("user-top-read")) {
            Some(oauth) => oauth,
            None => {
                println!("Failed to retrieve OAuth from environment.");
                return None;
            }
        };

        let spotify = AuthCodeSpotify::new(self.creds.clone(), oauth);

        if let Ok(url) = spotify.get_authorize_url(false) {
            if let Err(e) = spotify.prompt_for_token(&url).await {
                println!("{:?}", e);
            }
        } else {
            println!("Couldn't perform OAuth authentication.");
        }

        Some(spotify)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
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

pub fn get_artists(artists: Vec<SimplifiedArtist>) -> Vec<String> {
    let artists: Vec<String> = artists.iter().map(|artist| artist.name.clone()).collect();
    artists
}

#[derive(Debug, Clone)]
pub struct TopTrack {
    pub index: usize,
    pub track_name: String,
    pub duration: String,
    pub artists: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TopTracks {
    pub time_range: TimeRange,
    pub tracks: Vec<TopTrack>,
}

#[derive(Debug, Clone)]
pub struct TopTracksIterator {
    top_tracks: TopTracks,
    current: usize,
}

impl Iterator for TopTracksIterator {
    type Item = TopTrack;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.top_tracks.tracks.len() {
            let track = self.top_tracks.tracks[self.current].clone();
            self.current += 1;
            Some(track)
        } else {
            None
        }
    }
}

impl TopTracks {
    pub fn iter(&self) -> TopTracksIterator {
        TopTracksIterator {
            top_tracks: self.clone(),
            current: 0,
        }
    }
}

pub async fn get_top_tracks(
    client: &AuthCodeSpotify,
    time_range: TimeRange,
    limit: u8,
) -> Result<TopTracks, ClientError> {
    let stream = client.current_user_top_tracks(Some(time_range));
    pin_mut!(stream);

    let mut tracks: Vec<TopTrack> = Vec::new();

    while let Some(item) = stream.try_next().await? {
        let top_track = TopTrack {
            index: tracks.len() + 1,
            track_name: item.name.clone(),
            duration: format_duration(item.duration),
            artists: get_artists(item.artists.clone()),
        };

        if top_track.index as u8 == limit + 1 {
            break;
        }
        tracks.push(top_track);
    }

    let result = TopTracks { time_range, tracks };

    Ok(result)
}

#[derive(Debug, Clone)]
pub struct TopArtist {
    pub index: usize,
    pub artist_name: String,
    pub genres: String,
}

#[derive(Debug, Clone)]
pub struct TopArtists {
    pub time_range: TimeRange,
    pub artists: Vec<TopArtist>,
}
pub async fn get_top_artists(
    client: &AuthCodeSpotify,
    time_range: TimeRange,
    limit: u8,
) -> Result<Vec<TopArtist>, ClientError> {
    let stream = client.current_user_top_artists(Some(time_range));
    pin_mut!(stream);

    let mut result: Vec<TopArtist> = Vec::new();

    while let Some(item) = stream.try_next().await? {
        let top_artist = TopArtist {
            index: result.len() + 1,
            artist_name: item.name.clone(),
            genres: item
                .genres
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<String>>()
                .join(", "),
        };

        if top_artist.index as u8 == limit + 1 {
            break;
        }
        result.push(top_artist);
    }
    Ok(result)
}
