use std::{
    env,
    fmt::{self, Display},
    io, thread, time,
};

use crate::utils::check_env_existence;

mod lastfm;
pub mod spotify;

const PRIORITY_PLATFORM: &str = "PRIORITY_PLATFORM";
const RATELIMIT_WAIT_SECS: u64 = 60;

#[allow(dead_code)]
pub struct Song {
    playing: bool,
    title: String,
    artist: String,
    album: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorType {
    ExpiredToken,
    Request,
    WebServer,
    Ratelimit,
    Unknown,
}

#[derive(Debug)]
pub struct Error {
    error_type: ErrorType,
    message: String,
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error {
            error_type: ErrorType::Request,
            message: error.to_string(),
        }
    }
}

impl From<actix_web::Error> for Error {
    fn from(error: actix_web::Error) -> Self {
        Error {
            error_type: ErrorType::WebServer,
            message: error.to_string(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error {
            error_type: ErrorType::Unknown,
            message: error.to_string(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.error_type, self.message)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    Spotify,
    LastFM,
}

#[derive(Clone, Default)]
pub struct PlatformParameters {
    spotify_access_token: Option<String>,
    spotify_refresh_token: Option<String>,
}

impl Platform {
    async fn connect(&self) -> Result<Option<PlatformParameters>, Error> {
        match *self {
            Platform::Spotify => spotify::connection::connect().await,
            _ => {
                log::warn!("No login implementation detected for {:?}", self);
                Ok(None)
            }
        }
    }

    async fn refresh(
        &self,
        parameters: Option<PlatformParameters>,
    ) -> Result<Option<PlatformParameters>, Error> {
        match *self {
            Platform::Spotify => spotify::connection::refresh(parameters.clone()).await,
            _ => {
                log::warn!("No refresh implementation detected for {:?}", self);
                Ok(None)
            }
        }
    }

    fn verify(&self) -> bool {
        match *self {
            Platform::Spotify => spotify::verify(true),
            Platform::LastFM => lastfm::verify(true),
        }
    }

    fn ratelimit(&self) -> u64 {
        match *self {
            Platform::Spotify => 2,
            Platform::LastFM => 2,
        }
    }

    async fn currently_playing(
        &self,
        parameters: Option<PlatformParameters>,
    ) -> Result<Option<Song>, Error> {
        match *self {
            Platform::Spotify => spotify::playing::currently_playing(parameters).await,
            Platform::LastFM => lastfm::currently_playing().await,
        }
    }

    pub async fn login_server(
        &self,
    ) -> Result<Option<spotify::connection::AccessTokenJson>, Error> {
        match *self {
            Platform::Spotify => Ok(Some(spotify::connection::login_server().await?)),
            _ => Ok(None)
        }
    }
}

pub struct Provider {
    platform: Platform,
    params: Option<PlatformParameters>,
}

impl Provider {
    pub fn new(platform: Platform) -> Self {
        platform.verify();
        log::info!("Using provider {:?}", platform);
        Self {
            platform: platform,
            params: None,
        }
    }

    pub async fn connect(&mut self) {
        match self.platform.connect().await {
            Ok(params) => {
                self.params = params;
                log::debug!("Successfully connected to {:?}", self.platform);
            }
            Err(err) => {
                log::error!("Error occured during {:?} connection", self.platform);
                panic!("{}", err);
            }
        }
    }

    pub async fn refresh(&mut self) {
        match self.platform.refresh(self.params.clone()).await {
            Ok(params) => {
                self.params = params;
                log::info!("Successfully connected to {:?}", self.platform);
            }
            Err(err) => {
                log::error!("Couldn't refresh access_token using refresh_token");
                log::debug!("{}", err.message);
            }
        };
    }

    fn retrieve_params(&self) -> Option<PlatformParameters> {
        match self.platform {
            Platform::Spotify => self.params.clone(),
            Platform::LastFM => None,
        }
    }

    pub async fn currently_playing(&mut self) {
        let params = self.retrieve_params();

        match self.platform.currently_playing(params).await {
            Ok(currently_playing) => {
                match currently_playing {
                    Some(song) => {
                        println!("{} - {}", song.title, song.artist);
                        println!("Album: {}", song.album);
                    }
                    None => {
                        println!("No song detected");
                    }
                };
            }
            Err(err) => {
                match err.error_type {
                    ErrorType::ExpiredToken => {
                        log::info!("{}", err)
                    }
                    _ => {
                        log::error!("{}", err)
                    }
                };
                match err.error_type {
                    ErrorType::ExpiredToken => {
                        self.refresh().await;
                    }
                    ErrorType::Ratelimit => {
                        let duration = time::Duration::from_secs(RATELIMIT_WAIT_SECS);
                        log::warn!("Waiting {:?} before retrying", duration);
                        thread::sleep(duration);
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn wait(&self) {
        let duration = time::Duration::from_secs(self.platform.ratelimit());
        log::debug!("Waiting {:?} until next request", duration);
        thread::sleep(duration);
    }
}

pub fn new(platform: Platform) -> Provider {
    Provider::new(platform)
}

fn get_platform_from_env() -> Option<Platform> {
    let platform_env = env::var(PRIORITY_PLATFORM).unwrap().to_lowercase();

    if platform_env.contains("lastfm") {
        return Some(Platform::LastFM);
    }
    if platform_env.contains("spotify") {
        return Some(Platform::Spotify);
    }
    None
}

pub fn detect_platform() -> Option<Platform> {
    log::debug!("Trying to detect platform using env");
    if check_env_existence(PRIORITY_PLATFORM, false) {
        return get_platform_from_env();
    }
    if lastfm::verify(false) {
        return Some(Platform::LastFM);
    }
    if spotify::verify(false) {
        return Some(Platform::Spotify);
    }
    None
}
