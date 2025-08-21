use actix_web::{
    App, HttpResponse, HttpServer, Responder, dev::ServerHandle, get, middleware, web,
};
use base64::{Engine, prelude::BASE64_STANDARD};
use rand::distr::{Alphanumeric, SampleString};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::{
    env, process,
    sync::{Arc, Mutex},
};

use crate::{
    commands::connect::get_server_info,
    database,
    providers::{
        self, spotify::{CLIENT_ID_ENV, CLIENT_SECRET_ENV}, PlatformParameters
    },
};

const AUTHORIZE_API_LINK: &str = "https://accounts.spotify.com/authorize";
const ACCESS_TOKEN_API_LINK: &str = "https://accounts.spotify.com/api/token";

#[derive(Serialize, Deserialize, Clone)]
pub struct AccessTokenJson {
    access_token: String,
    refresh_token: String,
}

// TODO: not a huge fan of this solution but it works for now lol
#[derive(Serialize, Deserialize, Clone)]
pub struct RefreshTokenJson {
    access_token: String,
}

fn insert_authorization_header(headers: &mut HeaderMap) {
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
}

async fn get_access_token(
    code: String,
    redirect_uri: String,
) -> Result<AccessTokenJson, providers::Error> {
    let mut headers = HeaderMap::new();

    insert_authorization_header(&mut headers);

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
        log::error!(
            "Found status code {} instead of {}",
            status_code,
            reqwest::StatusCode::OK
        );
        return Err(providers::Error {
            error_type: providers::ErrorType::Unknown,
            message: resp.text().await?,
        });
    }

    let json = resp.json::<AccessTokenJson>().await?;
    Ok(json)
}

async fn get_refresh_token(refresh_token: String) -> Result<AccessTokenJson, providers::Error> {
    let mut headers = HeaderMap::new();

    insert_authorization_header(&mut headers);

    log::debug!("Refreshing token");
    let resp = reqwest::Client::new()
        .post(ACCESS_TOKEN_API_LINK)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
        ])
        .headers(headers)
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

    let json = resp.json::<RefreshTokenJson>().await?;
    let creds = AccessTokenJson {
        access_token: json.access_token,
        refresh_token: refresh_token,
    };
    database::spotify::set_creds(creds.clone());
    Ok(creds)
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

fn get_redirect_uri(ip: &String, port: u16) -> String {
    format!("http://{}:{}/callback", ip, port)
}

pub async fn login_server() -> Result<AccessTokenJson, providers::Error> {
    let login_server_info = get_server_info();
    let ip = login_server_info.ip;
    let port = login_server_info.port;

    let redirect_uri = get_redirect_uri(&ip, port);
    let state = Alphanumeric.sample_string(&mut rand::rng(), 16);
    let url = get_authorize_url(&redirect_uri, &state);

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
                .service(web::redirect("/login", url.clone()))
                .service(callback)
                .wrap(middleware::Logger::default())
        }
    })
    .disable_signals()
    .bind((ip.clone(), port))?
    .workers(1)
    .run();

    stop_handle.register(server.handle());
    log::warn!(
        "Go to http://{}:{}/login on your browser",
        ip,
        port
    );

    server.await?;

    if state != *query_state.state.lock().unwrap() {
        return Err(providers::Error {
            error_type: providers::ErrorType::Unknown,
            message: "Different state between authorization URL and callback".to_string(),
        });
    }

    let creds = get_access_token(query_state.code.lock().unwrap().clone(), redirect_uri).await?;
    Ok(creds)
}

pub async fn connect() -> Result<Option<PlatformParameters>, providers::Error> {
    let mut params = PlatformParameters::default();
    let creds: AccessTokenJson = match database::spotify::get_creds() {
        Some(db_creds) => db_creds,
        None => {
            log::error!(
                "Couldn't find Spotify credentials, please use `imaginal connect` and try again."
            );
            process::exit(1);
        }
    };

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
