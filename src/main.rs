mod providers;

use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let mut provider = providers::new(providers::Platforms::LastFM);
    provider.connect().await;
    provider.currently_playing().await;
    Ok(())
}
