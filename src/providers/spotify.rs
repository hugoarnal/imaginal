use actix_web::{
    App, HttpResponse, HttpServer, Responder, dev::ServerHandle, get, middleware, web,
};
use parking_lot::Mutex;
use rand::distr::{Alphanumeric, SampleString};
use serde::Deserialize;
use std::{env, sync::Arc};

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
const IP: &str = "127.0.0.1";
const PORT: u16 = 9761;

pub fn verify() {
    check_env_existence(CLIENT_ID_ENV, true);
    check_env_existence(CLIENT_SECRET_ENV, true);
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
        *self.code.lock() = info.code.clone();
        *self.state.lock() = info.state.clone();
    }
}

#[get("/callback")]
async fn callback(
    info: web::Query<QueryInfo>,
    query_state: web::Data<QueryState>,
    stop_handle: web::Data<StopHandle>,
) -> impl Responder {
    query_state.update(info);
    stop_handle.stop(false);
    HttpResponse::NoContent().finish()
}

pub async fn connect() -> Result<(), std::io::Error> {
    // TODO: host a /login endpoint like in the official post so that a DE is not needed
    // https://developer.spotify.com/documentation/web-api/tutorials/code-flow

    let redirect_uri = format!("http://{}:{}/callback", IP, PORT);
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

    // https://github.com/actix/examples/blob/49ea95e9e69e64f5c14f4c43692e4e7916218d6d/shutdown-server/src/main.rs
    let stop_handle = web::Data::new(StopHandle::default());
    let query_state = web::Data::new(QueryState::default());

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
    .bind((IP, PORT))?
    .workers(1)
    .run();

    stop_handle.register(server.handle());

    server.await?;
    println!("Code: {}", query_state.code.lock());
    println!("State: {}", query_state.state.lock());
    Ok(())
}

#[derive(Default)]
struct StopHandle {
    inner: Mutex<Option<ServerHandle>>,
}

impl StopHandle {
    /// Sets the server handle to stop.
    pub(crate) fn register(&self, handle: ServerHandle) {
        *self.inner.lock() = Some(handle);
    }

    /// Sends stop signal through contained server handle.
    pub(crate) fn stop(&self, graceful: bool) {
        #[allow(clippy::let_underscore_future)]
        let _ = self.inner.lock().as_ref().unwrap().stop(graceful);
    }
}
