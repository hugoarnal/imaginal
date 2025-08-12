mod database;
mod providers;
mod utils;

use std::process;

use dotenv::dotenv;
use log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    dotenv().ok();
    let platform = providers::detect_platform();

    match platform {
        Some(p) => {
            log::debug!("Found platform {:?}", p.clone())
        }
        None => {
            log::error!("No platforms detected");
            process::exit(1);
        }
    }

    let mut provider = providers::new(platform.unwrap());
    provider.connect().await;
    loop {
        provider.currently_playing().await;
        provider.wait();
    }
}
