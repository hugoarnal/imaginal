use serde::{Deserialize, Serialize};
use std::env;

use crate::providers::Song;
use crate::utils::check_env_existence;

const API_URL: &str = "http://ws.audioscrobbler.com/2.0/";
const API_KEY_ENV: &str = "LASTFM_API_KEY";
const SHARED_SECRET_ENV: &str = "LASTFM_SHARED_SECRET";
const USERNAME_ENV: &str = "LASTFM_USERNAME";

// Generated using Hoppscotch data schema, very useful
#[derive(Debug, Serialize, Deserialize)]
struct CurrentlyPlayingSchema {
    recenttracks: RecentTracks,
}

#[derive(Debug, Serialize, Deserialize)]
struct RecentTracks {
    track: Vec<Track>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Track {
    artist: TextFields,
    album: TextFields,
    name: String,
    #[serde(rename = "@attr")]
    attr: Option<TrackAttr>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TextFields {
    #[serde(rename = "#text")]
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrackAttr {
    nowplaying: String,
}

pub fn verify() -> bool {
    check_env_existence(API_KEY_ENV, true);
    check_env_existence(SHARED_SECRET_ENV, true);
    check_env_existence(USERNAME_ENV, true)
}

pub async fn currently_playing() -> Result<Option<Song>, reqwest::Error> {
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
            println!("No tracks detected at all, returning None");
        }
    }
    Ok(currently_playing)
}
