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
        Self {
            platform: platform,
        }
    }

    pub fn change_platform(&mut self, platform: Platforms) {
        self.platform = platform;
    }

    pub fn display_type(&self) {
        println!("{:?}", self.platform);
    }
}

pub fn new(platform: Platforms) -> Provider {
    let provider = Provider::new(platform);
    provider.display_type();
    provider
}
