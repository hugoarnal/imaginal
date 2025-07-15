use std::env;
use serde::{Deserialize};

use crate::utils::check_env_existence;

#[allow(dead_code)]
#[derive(Deserialize)]
struct AccessTokenJson {
    access_token: String,
    token_type: String,
    expires_in: i32
}

#[derive(Debug)]
pub enum CurrentlyPlayingError {
    ExpiredToken,
    UnknownError
}

const ACCESS_TOKEN_API_LINK: &str = "https://accounts.spotify.com/api/token";
const CURRENTLY_PLAYING_API_LINK: &str = "https://accounts.spotify.com/me/player/currently-playing";
const CLIENT_ID_ENV: &str = "SPOTIFY_CLIENT_ID";
const CLIENT_SECRET_ENV: &str = "SPOTIFY_CLIENT_SECRET";

pub fn verify() {
    check_env_existence(CLIENT_ID_ENV, true);
    check_env_existence(CLIENT_SECRET_ENV, true);
}

// TODO: can delete like all of this because we need to use the OAuth flow instead of this
pub async fn get_access_token() -> Result<String, reqwest::Error> {
    let resp = reqwest::Client::new()
        .post(ACCESS_TOKEN_API_LINK)
        .form(&[
            ("grant_type", "client_credentials"),
            ("client_id", env::var(CLIENT_ID_ENV).unwrap().as_str()),
            ("client_secret", env::var(CLIENT_SECRET_ENV).unwrap().as_str())
        ])
        .send()
        .await
        .expect("send");
    if resp.status() != 200 {
        // TODO: idk, should return error or panic (?)
    }
    // TODO: add proper logging
    println!("Response status {}", resp.status());

    let json = resp.json::<AccessTokenJson>().await?;
    Ok(json.access_token)
}
