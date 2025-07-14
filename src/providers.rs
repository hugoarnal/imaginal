mod spotify;

#[derive(Debug)]
pub enum Platforms {
    Spotify,
    LastFM,
}

pub struct Provider {
    platform: Platforms
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
        }
    }

    pub async fn connect(&mut self) {
        match self.platform {
            Platforms::Spotify => {
                let access_token = spotify::get_access_token().await;
            },
            Platforms::LastFM => {
                let _ = "";
            },
        }
    }
}

pub fn new(platform: Platforms) -> Provider {
    Provider::new(platform)
}
