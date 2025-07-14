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

pub struct Provider {
    platform: Platforms,
    // TODO: I don't really think this is a great idea for now,
    // think about moving it later on.
    access_token: String
}

impl Provider {
    pub fn new(platform: Platforms) -> Self {
        match platform {
            Platforms::Spotify => {
                spotify::verify();
            },
            Platforms::LastFM => {
                lastfm::verify();
            }
        }
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
                let _ = lastfm::currently_playing().await;
            },
        }
    }
}

pub fn new(platform: Platforms) -> Provider {
    Provider::new(platform)
}
