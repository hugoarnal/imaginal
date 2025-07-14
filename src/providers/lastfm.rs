use std::env;
use serde::{Serialize, Deserialize};

use crate::providers::CurrentlyPlaying;

const API_URL: &str = "http://ws.audioscrobbler.com/2.0/";
const API_KEY_ENV: &str = "LASTFM_API_KEY";
const SHARED_SECRET_ENV: &str = "LASTFM_SHARED_SECRET";
const USERNAME_ENV: &str = "LASTFM_USERNAME";

// todo comment already in spotify.rs
fn check_env_existance(var: &str, panic: bool) -> bool {
    match env::var(var) {
        Ok(_) => {
            true
        },
        Err(_) => {
            if panic {
                panic!("Couldn't find {} environment variable.", var);
            }
            false
        }
    }
}

pub fn verify() {
    check_env_existance(API_KEY_ENV, true);
    check_env_existance(SHARED_SECRET_ENV, true);
    check_env_existance(USERNAME_ENV, true);
}

// Generated using Hoppscotch data schema, very useful
#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentlyPlayingSchema {
    recenttracks: Recenttracks,
}

#[derive(Debug, Serialize, Deserialize)]
struct Recenttracks {
    track: Vec<Track>,
    #[serde(rename = "@attr")]
    attr: Attr,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Attr {
    user: String,
    total_pages: String,
    page: String,
    per_page: String,
    total: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Track {
    artist: Album,
    streamable: String,
    image: Vec<Image>,
    mbid: String,
    album: Album,
    name: String,
    #[serde(rename = "@attr")]
    attr: Option<TrackAttr>,
    url: String,
    date: Option<Date>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Album {
    mbid: String,
    #[serde(rename = "#text")]
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrackAttr {
    nowplaying: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Date {
    uts: String,
    #[serde(rename = "#text")]
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Image {
    size: String,
    #[serde(rename = "#text")]
    text: String,
}

pub async fn currently_playing() -> Result<(), reqwest::Error> {
    let username = env::var(USERNAME_ENV).unwrap();
    let api_key = env::var(API_KEY_ENV).unwrap();

    let querystring = [
        ("method", "user.getrecenttracks"),
        ("user", username.as_str()),
        ("api_key", api_key.as_str()),
        ("format", "json"),
        ("limit", "1"),
    ];

    let client = reqwest::Client::new();
    let response = client.get(API_URL)
        .query(&querystring)
        .send()
        .await?;

    let results = response
        .json::<CurrentlyPlayingSchema>()
        .await
        .unwrap();

    dbg!(results);
    Ok(())
}
