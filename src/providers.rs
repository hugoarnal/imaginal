use std::{
    env,
    fmt::{self, Display},
    io, thread, time,
};

use crate::utils::check_env_existence;

mod lastfm;
mod spotify;

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
pub enum Platforms {
    Spotify,
    LastFM,
}

#[derive(Clone, Default)]
pub struct PlatformParameters {
    spotify_access_token: Option<String>,
    spotify_refresh_token: Option<String>,
}

// this is for future platforms implementation, might remove?
#[allow(unreachable_patterns)]
impl Platforms {
    async fn connect(&self) -> Result<Option<PlatformParameters>, Error> {
        match *self {
            Platforms::Spotify => spotify::connect().await,
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
            Platforms::Spotify => spotify::refresh(parameters.clone()).await,
            _ => {
                log::warn!("No refresh implementation detected for {:?}", self);
                Ok(None)
            }
        }
    }

    fn verify(&self) -> bool {
        match *self {
            Platforms::Spotify => spotify::verify(true),
            Platforms::LastFM => lastfm::verify(true),
            _ => {
                todo!("This platform hasn't been implemented")
            }
        }
    }

    fn ratelimit(&self) -> u64 {
        match *self {
            Platforms::Spotify => 2,
            Platforms::LastFM => 2,
            _ => 2,
        }
    }

    async fn currently_playing(
        &self,
        parameters: Option<PlatformParameters>,
    ) -> Result<Option<Song>, Error> {
        match *self {
            Platforms::Spotify => spotify::currently_playing(parameters).await,
            Platforms::LastFM => lastfm::currently_playing().await,
            _ => {
                todo!("This platform hasn't been implemented")
            }
        }
    }
}

pub struct Provider {
    platform: Platforms,
    params: Option<PlatformParameters>,
}

impl Provider {
    pub fn new(platform: Platforms) -> Self {
        platform.verify();
        log::info!("Using provider {:?}", platform);
        Self {
            platform: platform,
            params: None,
        }
    }

    pub async fn connect(&mut self) {
        let success: bool;

        match self.platform.connect().await {
            Ok(params) => {
                self.params = params;
                success = true;
            }
            Err(_) => {
                panic!("Error occured during Spotify connection");
            }
        }
        if success {
            log::debug!("Successfully connected to {:?}", self.platform);
        }
    }

    pub async fn refresh(&mut self) {
        let success: bool;

        match self.platform.refresh(self.params.clone()).await {
            Ok(params) => {
                self.params = params;
                success = true;
            }
            Err(_) => {
                panic!("Error occured during Spotify connection");
            }
        }
        if success {
            log::debug!("Successfully connected to {:?}", self.platform);
        }
    }

    fn retrieve_params(&self) -> Option<PlatformParameters> {
        match self.platform {
            Platforms::Spotify => self.params.clone(),
            Platforms::LastFM => None,
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
                log::error!("{}", err);
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

pub fn new(platform: Platforms) -> Provider {
    Provider::new(platform)
}

fn get_platform_from_env() -> Option<Platforms> {
    let platform_env = env::var(PRIORITY_PLATFORM).unwrap().to_lowercase();

    if platform_env.contains("lastfm") {
        return Some(Platforms::LastFM);
    }
    if platform_env.contains("spotify") {
        return Some(Platforms::Spotify);
    }
    None
}

pub fn detect_platform() -> Option<Platforms> {
    log::debug!("Trying to detect platform using env");
    if check_env_existence(PRIORITY_PLATFORM, false) {
        return get_platform_from_env();
    }
    if lastfm::verify(false) {
        return Some(Platforms::LastFM);
    }
    if spotify::verify(false) {
        return Some(Platforms::Spotify);
    }
    None
}
