use serde::Deserialize;
use std::env;

use crate::providers::{self, Song};
use crate::utils::check_env_existence;

const API_URL: &str = "http://ws.audioscrobbler.com/2.0/";
const API_KEY_ENV: &str = "LASTFM_API_KEY";
const SHARED_SECRET_ENV: &str = "LASTFM_SHARED_SECRET";
const USERNAME_ENV: &str = "LASTFM_USERNAME";

// Generated using Hoppscotch data schema, very useful
#[derive(Deserialize)]
struct CurrentlyPlayingSchema {
    recenttracks: RecentTracks,
}

#[derive(Deserialize)]
struct RecentTracks {
    track: Vec<Track>,
}

#[derive(Deserialize)]
struct Track {
    artist: TextFields,
    album: TextFields,
    name: String,
    #[serde(rename = "@attr")]
    attr: Option<TrackAttr>,
}

#[derive(Deserialize)]
struct TextFields {
    #[serde(rename = "#text")]
    text: String,
}

#[derive(Deserialize)]
struct TrackAttr {
    nowplaying: String,
}

#[derive(Deserialize)]
struct Error {
    error: u16,
}

pub fn verify(panic: bool) -> bool {
    check_env_existence(API_KEY_ENV, panic);
    check_env_existence(SHARED_SECRET_ENV, panic);
    check_env_existence(USERNAME_ENV, panic)
}

pub async fn currently_playing() -> Result<Option<Song>, providers::Error> {
    let username = env::var(USERNAME_ENV).unwrap();
    let api_key = env::var(API_KEY_ENV).unwrap();

    let query = [
        ("method", "user.getrecenttracks"),
        ("user", username.as_str()),
        ("api_key", api_key.as_str()),
        ("format", "json"),
        ("limit", "1"),
    ];

    let client = reqwest::Client::new();
    let response = client.get(API_URL).query(&query).send().await?;

    if response.status() != 200 {
        let results = response.json::<Error>().await?;
        let mut error_type: providers::ErrorType = providers::ErrorType::Unknown;
        let mut message = "Unhandled request error coming from LastFM";

        match results.error {
            6 => {
                panic!("Unknown user")
            }
            10 => {
                panic!("Incorrect API Key")
            }
            29 => {
                error_type = providers::ErrorType::Ratelimit;
                message = "Too many requests";
            }
            _ => {}
        }

        return Err(providers::Error {
            error_type: error_type,
            message: message.to_string(),
        });
    }

    let results = response.json::<CurrentlyPlayingSchema>().await?;

    let mut currently_playing: Option<Song> = None;

    let first_element = results.recenttracks.track.into_iter().next();
    match first_element {
        Some(track) => {
            let mut playing = false;

            match track.attr {
                Some(track_attr) => {
                    if track_attr.nowplaying == "true" {
                        playing = true;
                    }
                }
                _ => {}
            }

            currently_playing = Some(Song {
                album: track.album.text,
                playing: playing,
                title: track.name,
                artist: track.artist.text,
            });
        }
        None => {
            log::debug!("No tracks detected at all");
        }
    }
    Ok(currently_playing)
}
