use std::env;

use crate::{database, providers::Platform, utils::check_env_existence};

pub const IP_ENV: &str = "LOGIN_SERVER_IP";
pub const PORT_ENV: &str = "LOGIN_SERVER_PORT";
pub const DEFAULT_IP: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 9761;

#[derive(Clone)]
pub struct LoginServerInfo {
    pub ip: String,
    pub port: u16
}

pub fn get_server_info() -> LoginServerInfo {
    // TODO: better generic implementation for ip / port
    // something like `get_server_env`
    let ip: String = if check_env_existence(IP_ENV, false) {
        env::var(IP_ENV).unwrap()
    } else {
        DEFAULT_IP.to_string()
    };

    let port: u16 = if check_env_existence(PORT_ENV, false) {
        env::var(PORT_ENV).unwrap().parse().unwrap()
    } else {
        DEFAULT_PORT
    };


    LoginServerInfo {
        ip: ip,
        port: port
    }
}

pub async fn connect(platform: Platform) {
    log::warn!("Trying to connect to {} platform.", platform);
    match platform.login_server().await {
        Ok(ok) => match ok {
            Some(c) => {
                log::debug!("Saving credentials to database");
                database::spotify::set_creds(c);
                log::info!("Done! You can now use `imaginal` for {}.", platform);
            }
            None => log::info!("Connection to platform not needed"),
        },
        Err(err) => {
            log::error!("Error occured: {}", err);
        }
    };
}
