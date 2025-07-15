use serde::Deserialize;
use std::env;

use crate::utils::check_env_existence;

#[allow(dead_code)]
#[derive(Deserialize)]
struct AccessTokenJson {
    access_token: String,
    token_type: String,
    expires_in: i32,
}

const AUTHORIZE_API_LINK: &str = "https://accounts.spotify.com/authorize";
const CURRENTLY_PLAYING_API_LINK: &str = "https://accounts.spotify.com/me/player/currently-playing";
const CLIENT_ID_ENV: &str = "SPOTIFY_CLIENT_ID";
const CLIENT_SECRET_ENV: &str = "SPOTIFY_CLIENT_SECRET";

pub fn verify() {
    check_env_existence(CLIENT_ID_ENV, true);
    check_env_existence(CLIENT_SECRET_ENV, true);
}

use rand::distr::{Alphanumeric, SampleString};

pub fn connect() {
    // TODO: host a /login endpoint like in the official post so that a DE is not needed
    // https://developer.spotify.com/documentation/web-api/tutorials/code-flow

    let redirect_uri = String::from("http://127.0.0.1:9761/callback");
    let mut url = String::from(AUTHORIZE_API_LINK);
    let state = Alphanumeric.sample_string(&mut rand::rng(), 16);

    url.push_str("?response_type=code");
    url.push_str(format!("&client_id={}", env::var(CLIENT_ID_ENV).unwrap()).as_str());
    url.push_str("&scope=user-read-currently-playing");
    url.push_str(format!("&redirect_uri={}", redirect_uri).as_str());
    url.push_str(format!("&state={}", state).as_str());

    match open::that(url) {
        Ok(_) => {}
        Err(_) => {
            panic!("Couldn't open Spotify connection link");
        }
    };
}
