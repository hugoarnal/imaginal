use actix_web::{
    App, HttpResponse, HttpServer, Responder, dev::ServerHandle, get, middleware, web,
};
use base64::{Engine, prelude::BASE64_STANDARD};
use rand::distr::{Alphanumeric, SampleString};
use reqwest::header::{AUTHORIZATION, HeaderMap};
use serde::{Deserialize, Serialize};
use std::{
    env,
    sync::{Arc, Mutex},
};

use crate::{
    providers::{self, PlatformParameters, Song},
    utils::check_env_existence,
};

const AUTHORIZE_API_LINK: &str = "https://accounts.spotify.com/authorize";
const ACCESS_TOKEN_API_LINK: &str = "https://accounts.spotify.com/api/token";
const CURRENTLY_PLAYING_API_LINK: &str = "https://api.spotify.com/v1/me/player/currently-playing";
const CLIENT_ID_ENV: &str = "SPOTIFY_CLIENT_ID";
const CLIENT_SECRET_ENV: &str = "SPOTIFY_CLIENT_SECRET";
const PORT_ENV: &str = "SPOTIFY_PORT";
const IP: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 9761;

#[derive(Clone)]
pub struct Parameters {
    pub access_token: String,
}

impl PlatformParameters for Parameters {
    fn get_spotify_access_token(&self) -> String {
        self.access_token.clone()
    }
}

#[derive(Deserialize)]
struct AccessTokenJson {
    access_token: String,
}

pub fn verify(panic: bool) -> bool {
    check_env_existence(CLIENT_ID_ENV, panic);
    check_env_existence(CLIENT_SECRET_ENV, panic)
}

async fn get_access_token(code: String, redirect_uri: String) -> Result<String, providers::Error> {
    let mut headers = HeaderMap::new();

    let client_id = env::var(CLIENT_ID_ENV).unwrap();
    let client_secret = env::var(CLIENT_SECRET_ENV).unwrap();
    let encrypted_client_settings = format!("{}:{}", client_id, client_secret);

    headers.insert(
        AUTHORIZATION,
        format!(
            "Basic {}",
            BASE64_STANDARD.encode(encrypted_client_settings)
        )
        .parse()
        .unwrap(),
    );

    log::debug!("Obtaining access token");
    let resp = reqwest::Client::new()
        .post(ACCESS_TOKEN_API_LINK)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code.as_ref()),
            ("redirect_uri", redirect_uri.as_ref()),
        ])
        .headers(headers)
        .send()
        .await
        .expect("send");

    let status_code = resp.status();
    if status_code != 200 {
        // TODO: Transform this panic into log::error once `/login` is implemented
        panic!("Found status code {} instead of 200", status_code)
    }

    let json = resp.json::<AccessTokenJson>().await?;
    Ok(json.access_token)
}

#[derive(Deserialize)]
struct QueryInfo {
    code: String,
    state: String,
}

#[derive(Default, Clone)]
struct QueryState {
    code: Arc<Mutex<String>>,
    state: Arc<Mutex<String>>,
}

impl QueryState {
    fn update(&self, info: web::Query<QueryInfo>) {
        *self.code.lock().unwrap() = info.code.clone();
        *self.state.lock().unwrap() = info.state.clone();
    }
}

#[get("/callback")]
async fn callback(
    info: web::Query<QueryInfo>,
    query_state: web::Data<QueryState>,
    stop_handle: web::Data<StopHandle>,
) -> impl Responder {
    query_state.update(info);

    log::debug!("Response received, killing callback server");
    stop_handle.stop(false).await;
    HttpResponse::NoContent().finish()
}

fn get_authorize_url(redirect_uri: &String, state: &String) -> String {
    let mut url = String::from(AUTHORIZE_API_LINK);

    url.push_str("?response_type=code");
    url.push_str(format!("&client_id={}", env::var(CLIENT_ID_ENV).unwrap()).as_str());
    url.push_str("&scope=user-read-currently-playing");
    url.push_str(format!("&redirect_uri={}", redirect_uri).as_str());
    url.push_str(format!("&state={}", state).as_str());

    url
}

pub async fn connect() -> Result<String, providers::Error> {
    // TODO: host a /login endpoint like in the official post so that a DE is not needed
    // https://developer.spotify.com/documentation/web-api/tutorials/code-flow

    let mut port = DEFAULT_PORT;

    if check_env_existence(PORT_ENV, false) {
        port = env::var(PORT_ENV).unwrap().parse().unwrap();
    }

    let redirect_uri = format!("http://{}:{}/callback", IP, port);
    let state = Alphanumeric.sample_string(&mut rand::rng(), 16);
    let url = get_authorize_url(&redirect_uri, &state);

    log::debug!("Opening {} in client's default browser", url);
    match open::that(url) {
        Ok(_) => {}
        Err(_) => {
            panic!("Couldn't open Spotify connection link");
        }
    };

    // https://github.com/actix/examples/blob/49ea95e9e69e64f5c14f4c43692e4e7916218d6d/shutdown-server/src/main.rs
    let stop_handle = web::Data::new(StopHandle::default());
    let query_state = web::Data::new(QueryState::default());

    log::debug!("Starting callback server");
    let server = HttpServer::new({
        let stop_handle = stop_handle.clone();
        let query_state = query_state.clone();

        move || {
            App::new()
                .app_data(query_state.clone())
                .app_data(stop_handle.clone())
                .service(callback)
                .wrap(middleware::Logger::default())
        }
    })
    .disable_signals()
    .bind((IP, port))?
    .workers(1)
    .run();

    stop_handle.register(server.handle());

    server.await?;

    // TODO: correct error handling
    if state != *query_state.state.lock().unwrap() {
        panic!("Incorrect given state");
    }
    Ok(get_access_token(query_state.code.lock().unwrap().clone(), redirect_uri).await?)
}

#[derive(Default)]
struct StopHandle {
    inner: Mutex<Option<ServerHandle>>,
}

impl StopHandle {
    /// Sets the server handle to stop.
    pub(crate) fn register(&self, handle: ServerHandle) {
        *self.inner.lock().unwrap() = Some(handle);
    }

    /// Sends stop signal through contained server handle.
    pub(crate) async fn stop(&self, graceful: bool) {
        self.inner
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .stop(graceful)
            .await
    }
}

#[derive(Serialize, Deserialize)]
struct CurrentlyPlayingSchema {
    is_playing: bool,
    item: Item,
}

#[derive(Serialize, Deserialize)]
struct Item {
    album: Album,
    artists: Vec<Artist>,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct Album {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct Artist {
    name: String,
}

pub async fn currently_playing(
    parameters: Option<impl PlatformParameters>,
) -> Result<Option<Song>, providers::Error> {
    match parameters {
        Some(_) => {}
        None => panic!("No access_token given"),
    }

    let mut headers = HeaderMap::new();

    headers.insert(
        AUTHORIZATION,
        format!("Bearer {}", parameters.unwrap().get_spotify_access_token())
            .parse()
            .unwrap(),
    );

    let client = reqwest::Client::new();
    let response = client
        .get(CURRENTLY_PLAYING_API_LINK)
        .headers(headers)
        .send()
        .await?;

    let status_code = response.status();
    if status_code == 204 {
        return Ok(None);
    } else if status_code == 401 {
        return Err(providers::Error {
            error_type: providers::ErrorType::ExpiredToken,
            message: "Current token is expired".to_string(),
        });
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
