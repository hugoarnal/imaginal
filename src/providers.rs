use std::{thread, time};

mod lastfm;
mod spotify;

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
        match self.platform {
            Platforms::Spotify => {
                todo!("Spotify playing implementation");
            },
            Platforms::LastFM => {
                // TODO: horrid error handling
                match lastfm::currently_playing().await {
                    Ok(currently_playing) => {
                        println!("{}", currently_playing.unwrap().title);
                    },
                    Err(_) => {
                        println!("Error occured oh no");
                    }
                }
            },
        }
    }

    pub fn wait(&self) {
        thread::sleep(time::Duration::from_secs(self.platform.ratelimit()));
    }
}

pub fn new(platform: Platforms) -> Provider {
    Provider::new(platform)
}
