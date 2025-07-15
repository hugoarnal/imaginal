use std::{thread, time};

mod lastfm;
mod spotify;

#[allow(dead_code)]
pub struct CurrentlyPlaying {
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
    fn verify(&self) {
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
    fn currently_playing(&self) -> impl Future<Output = Result<std::option::Option<CurrentlyPlaying>, reqwest::Error>> {
        match *self {
            Platforms::Spotify => {
                todo!("Spotify playing implementation")
            }
            Platforms::LastFM => {
                lastfm::currently_playing()
            }
        }
    }
}

pub struct Provider {
    platform: Platforms,
    // TODO: I don't really think this is a great idea for now,
    // think about moving it later on.
    access_token: String
}

impl Provider {
    pub fn new(platform: Platforms) -> Self {
        platform.verify();
        Self {
            platform: platform,
            access_token: String::new()
        }
    }

    pub async fn connect(&mut self) {
        match self.platform {
            Platforms::Spotify => {
                todo!("Spotify login implementation");
            },
            _ => {
                println!("No login implementation detected for this platform");
            },
        }
    }

    pub async fn currently_playing(&mut self) {
        match self.platform.currently_playing().await {
            Ok(currently_playing) => {
                let song = currently_playing.unwrap();
                println!("{} - {}", song.title, song.artist);
                println!("Album: {}", song.album);
            },
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
