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
                todo!("LastFM verification implementation");
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
                match spotify::get_access_token().await {
                    Ok(access_token) => {
                        self.access_token = access_token;
                    }
                    Err(_) => {
                        panic!("Couldn't initialize connection with Spotify");
                    }
                }
            },
            Platforms::LastFM => {
                todo!("LastFM connection implementation");
            },
        }
    }
}

pub fn new(platform: Platforms) -> Provider {
    Provider::new(platform)
}
