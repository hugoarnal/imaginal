mod providers;
mod utils;

use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let mut provider = providers::new(providers::Platforms::Spotify);
    provider.connect().await;
    loop {
        provider.currently_playing().await;
        provider.wait()
    }
    // Ok(())
}
