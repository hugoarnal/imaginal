use actix_web::{
    App, HttpResponse, HttpServer, Responder, dev::ServerHandle, get, middleware, web,
};
use base64::{Engine, prelude::BASE64_STANDARD};
use rand::distr::{Alphanumeric, SampleString};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::{
    env,
    sync::{Arc, Mutex},
};

use crate::{
    database,
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

#[derive(Serialize, Deserialize)]
pub struct AccessTokenJson {
    access_token: String,
    refresh_token: String,
}

pub fn verify(panic: bool) -> bool {
    check_env_existence(CLIENT_ID_ENV, panic);
    check_env_existence(CLIENT_SECRET_ENV, panic)
}

async fn get_access_token(
    code: String,
    redirect_uri: String,
) -> Result<AccessTokenJson, providers::Error> {
    let mut headers = HeaderMap::new();

    let client_id = env::var(CLIENT_ID_ENV).unwrap();
    let client_secret = env::var(CLIENT_SECRET_ENV).unwrap();
    let encrypted_client_settings = format!("{}:{}", client_id, client_secret);

    headers.insert(
        reqwest::header::AUTHORIZATION,
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
    if status_code != reqwest::StatusCode::OK {
        // TODO: Transform this panic into log::error once `/login` is implemented
        panic!("Found status code {} instead of 200", status_code)
    }

    let json = resp.json::<AccessTokenJson>().await?;
    Ok(json)
}

async fn get_refresh_token(refresh_token: String) -> Result<AccessTokenJson, providers::Error> {
    let client_id = env::var(CLIENT_ID_ENV).unwrap();

    log::debug!("Refreshing token");
    let resp = reqwest::Client::new()
        .post(ACCESS_TOKEN_API_LINK)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
            ("client_id", client_id.as_str()),
        ])
        .send()
        .await
        .expect("send");

    let status_code = resp.status();
    if status_code != reqwest::StatusCode::OK {
        log::error!("Couldn't refresh Spotify token");
        return Err(providers::Error {
            error_type: providers::ErrorType::Request,
            message: "Couldn't refresh Spotify token".to_string(),
        });
    }

    let json = resp.json::<AccessTokenJson>().await?;
    Ok(json)
}

pub async fn refresh(
    parameters: Option<PlatformParameters>,
) -> Result<Option<PlatformParameters>, providers::Error> {
    if parameters.is_none() {
        return Err(providers::Error {
            error_type: providers::ErrorType::Unknown,
            message: "No parameters provided".to_string(),
        });
    }

    let mut new_params = parameters.clone().unwrap();
    let json = get_refresh_token(new_params.spotify_refresh_token.unwrap()).await?;

    new_params.spotify_access_token = Some(json.access_token);
    new_params.spotify_refresh_token = Some(json.refresh_token);

    Ok(Some(new_params))
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

fn get_server_port() -> u16 {
    let mut port = DEFAULT_PORT;

    if check_env_existence(PORT_ENV, false) {
        port = env::var(PORT_ENV).unwrap().parse().unwrap();
    }
    port
}

async fn login_server(
    redirect_uri: String,
) -> Result<actix_web::web::Data<QueryState>, providers::Error> {
    // TODO: host a /login endpoint like in the official post so that a DE is not needed
    // https://developer.spotify.com/documentation/web-api/tutorials/code-flow

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
    .bind((IP, get_server_port()))?
    .workers(1)
    .run();

    stop_handle.register(server.handle());

    server.await?;

    if state != *query_state.state.lock().unwrap() {
        return Err(providers::Error {
            error_type: providers::ErrorType::Unknown,
            message: "Different state between authorization URL and callback".to_string(),
        });
    }
    Ok(query_state)
}

pub async fn connect() -> Result<Option<PlatformParameters>, providers::Error> {
    let creds: AccessTokenJson;
    let mut params = PlatformParameters::default();

    match database::spotify::get_creds() {
        Some(db_creds) => {
            creds = db_creds;
        }
        None => {
            let redirect_uri = format!("http://{}:{}/callback", IP, get_server_port());
            let query_state = login_server(redirect_uri.clone()).await?;
            creds =
                get_access_token(query_state.code.lock().unwrap().clone(), redirect_uri).await?;
        }
    }
    params.spotify_access_token = Some(creds.access_token);
    params.spotify_refresh_token = Some(creds.refresh_token);
    Ok(Some(params))
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
        panic!("No access_token given");
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
