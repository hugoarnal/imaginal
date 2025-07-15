mod providers;
mod utils;

use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let platform = providers::detect_platform();

    // TODO: this is probably a bad way to handle this
    match platform {
        Some(_) => {}
        None => panic!("No platforms detected."),
    }

    let mut provider = providers::new(platform.unwrap());
    provider.connect().await;
    loop {
        provider.currently_playing().await;
        provider.wait()
    }
    // Ok(())
}
