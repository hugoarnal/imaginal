use std::{env, fmt::Display, thread, time};

use crate::utils::check_env_existence;

mod lastfm;
mod spotify;

const PRIORITY_PLATFORM: &str = "PRIORITY_PLATFORM";

#[allow(dead_code)]
pub struct Song {
    playing: bool,
    title: String,
    artist: String,
    album: String,
}

#[derive(Debug)]
pub enum ErrorType {
    ExpiredToken,
    RequestError,
    WebServerError,
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
            error_type: ErrorType::RequestError,
            message: error.to_string(),
        }
    }
}

impl From<actix_web::Error> for Error {
    fn from(error: actix_web::Error) -> Self {
        Error {
            error_type: ErrorType::WebServerError,
            message: error.to_string(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error {
            error_type: ErrorType::Unknown,
            message: error.to_string(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.error_type, self.message)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Platforms {
    Spotify,
    LastFM,
}

pub trait PlatformParameters {
    fn get_spotify_access_token(&self) -> String;
}

// this is for future platforms implementation, might remove?
#[allow(unreachable_patterns)]
impl Platforms {
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
        parameters: Option<impl PlatformParameters>,
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
    spotify_params: Option<spotify::Parameters>,
}

impl Provider {
    pub fn new(platform: Platforms) -> Self {
        platform.verify();
        log::info!("Using provider {:?}", platform);
        Self {
            platform: platform,
            spotify_params: None,
        }
    }

    pub async fn connect(&mut self) {
        let mut success = false;

        match self.platform {
            Platforms::Spotify => {
                match spotify::connect().await {
                    Ok(access_token) => {
                        self.spotify_params = Some(spotify::Parameters { access_token });
                        success = true;
                    }
                    Err(_) => {
                        panic!("Error occured during Spotify connection");
                    }
                };
            }
            _ => {
                log::warn!("No login implementation detected for {:?}", self.platform);
            }
        }
        if success {
            log::debug!("Successfully connected to {:?}", self.platform);
        }
    }

    fn retrieve_params(&self) -> Option<impl PlatformParameters> {
        match self.platform {
            Platforms::Spotify => self.spotify_params.clone(),
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
