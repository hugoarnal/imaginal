mod commands;
mod database;
mod providers;
mod utils;

use std::process;

use clap::{Command, command};
use dotenv::dotenv;
use log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = command!()
        .subcommand(Command::new("connect").about("Connect to OAuth provider platforms"))
        .get_matches();

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

    match matches.subcommand() {
        Some(("connect", _)) => {
            commands::connect::connect();
            Ok(())
        }
        _ => {
            let mut provider = providers::new(platform.unwrap());
            provider.connect().await;
            loop {
                provider.currently_playing().await;
                provider.wait();
            }
        }
    }
}
