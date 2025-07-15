use std::{env, thread, time};

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
pub enum Platforms {
    Spotify,
    LastFM,
}

impl Platforms {
    fn verify(&self) -> bool {
        match *self {
            Platforms::Spotify => spotify::verify(),
            Platforms::LastFM => lastfm::verify(),
        }
    }

    fn ratelimit(&self) -> u64 {
        match *self {
            Platforms::Spotify => 2,
            Platforms::LastFM => 2,
        }
    }

    // TODO: can't this be type aliased?
    fn currently_playing(&self) -> impl Future<Output = Result<Option<Song>, reqwest::Error>> {
        match *self {
            Platforms::Spotify => {
                todo!("Spotify playing implementation")
            }
            Platforms::LastFM => lastfm::currently_playing(),
        }
    }
}

pub struct Provider {
    platform: Platforms,
    spotify_access_token: Option<String>,
}

impl Provider {
    pub fn new(platform: Platforms) -> Self {
        platform.verify();
        println!("Provider {:?}", platform);
        Self {
            platform: platform,
            spotify_access_token: None,
        }
    }

    pub async fn connect(&mut self) {
        let mut success = false;

        match self.platform {
            Platforms::Spotify => {
                match spotify::connect().await {
                    Ok(access_token) => {
                        self.spotify_access_token = Some(access_token);
                        success = true;
                    }
                    Err(_) => {
                        panic!("Error occured during Spotify connection");
                    }
                };
            }
            _ => {
                println!("No login implementation detected for {:?}", self.platform);
            }
        }
        if success {
            println!("Successfully connected to {:?}", self.platform);
        }
    }

    pub async fn currently_playing(&mut self) {
        match self.platform.currently_playing().await {
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
            Err(_) => {
                println!("Error occured oh no");
            }
        }
    }

    pub fn wait(&self) {
        thread::sleep(time::Duration::from_secs(self.platform.ratelimit()));
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
    // Prioritize LastFM over other platforms (due to simplicity etc...)
    // TODO: explain in docs
    if check_env_existence(PRIORITY_PLATFORM, false) {
        return get_platform_from_env();
    }
    if lastfm::verify() {
        return Some(Platforms::LastFM);
    }
    if spotify::verify() {
        return Some(Platforms::Spotify);
    }
    None
}
