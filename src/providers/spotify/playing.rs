use reqwest::header::HeaderMap;
use serde::Deserialize;

use crate::providers::{self, PlatformParameters, Song};

const CURRENTLY_PLAYING_API_LINK: &str = "https://api.spotify.com/v1/me/player/currently-playing";

#[derive(Deserialize)]
struct CurrentlyPlayingSchema {
    is_playing: bool,
    item: Item,
}

#[derive(Deserialize)]
struct Item {
    album: Album,
    artists: Vec<Artist>,
    name: String,
}

#[derive(Deserialize)]
struct Album {
    name: String,
}

#[derive(Deserialize)]
struct Artist {
    name: String,
}

pub async fn currently_playing(
    parameters: Option<PlatformParameters>,
) -> Result<Option<Song>, providers::Error> {
    if parameters.is_none() {
        panic!("Unexpected, no parameters found");
    }

    let mut headers = HeaderMap::new();

    headers.insert(
        reqwest::header::AUTHORIZATION,
        format!(
            "Bearer {}",
            parameters.unwrap().spotify_access_token.unwrap()
        )
        .parse()
        .unwrap(),
    );

    let client = reqwest::Client::new();
    let response = client
        .get(CURRENTLY_PLAYING_API_LINK)
        .headers(headers)
        .send()
        .await?;

    match response.status() {
        reqwest::StatusCode::NO_CONTENT => {
            return Ok(None);
        }
        reqwest::StatusCode::UNAUTHORIZED => {
            return Err(providers::Error {
                error_type: providers::ErrorType::ExpiredToken,
                message: "Current token is expired".to_string(),
            });
        }
        reqwest::StatusCode::TOO_MANY_REQUESTS => {
            return Err(providers::Error {
                error_type: providers::ErrorType::Ratelimit,
                message: "Too many requests".to_string(),
            });
        }
        _ => {}
    }

    let results = response.json::<CurrentlyPlayingSchema>().await?;

    let artist_name: String;
    match results.item.artists.into_iter().next() {
        Some(artist) => {
            artist_name = artist.name;
        }
        None => {
            artist_name = String::from("None");
        }
    }

    let currently_playing: Option<Song> = Some(Song {
        playing: results.is_playing,
        title: results.item.name,
        artist: artist_name,
        album: results.item.album.name,
    });

    Ok(currently_playing)
}
