use serde::Deserialize;
use std::{env, process};

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

pub fn verify(exit: bool) -> bool {
    check_env_existence(API_KEY_ENV, exit);
    check_env_existence(SHARED_SECRET_ENV, exit);
    check_env_existence(USERNAME_ENV, exit)
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
                log::error!("Unknown user");
                process::exit(1);
            }
            10 => {
                log::error!("Incorrect API Key");
                process::exit(1);
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

    let currently_playing = match results.recenttracks.track.into_iter().next() {
        Some(track) => {
            let playing = match track.attr {
                Some(track_attr) => {
                    if track_attr.nowplaying == "true" {
                        true
                    } else {
                        false
                    }
                }
                _ => {
                    false
                }
            };

            Some(Song {
                album: track.album.text,
                playing: playing,
                title: track.name,
                artist: track.artist.text,
            })
        }
        None => {
            log::debug!("No tracks detected at all");
            None
        }
    };
    Ok(currently_playing)
}
