pub mod providers;

use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let mut test = providers::new(providers::Platforms::Spotify);
    test.change_platform(providers::Platforms::LastFM);
    test.display_type();
    Ok(())
}
