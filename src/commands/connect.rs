use crate::{database, providers::Platform};

pub async fn connect(platform: Platform) {
    match platform.login_server().await {
        Ok(ok) => {
            match ok {
                Some(c) => {
                    log::debug!("Saving credentials to database");
                    database::spotify::set_creds(c);
                    log::info!("Done! You can now use `imaginal`.");
                }
                None => log::info!("Connection to platform not needed")
            }
        }
        Err(err) => {
            log::error!("Error occured: {}", err);
        }
    };
}
