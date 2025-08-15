use std::{env, process};

pub fn check_env_existence(var: &str, exit: bool) -> bool {
    log::debug!("Checking for {} existence", var);
    match env::var(var) {
        Ok(_) => true,
        Err(_) => {
            if exit {
                log::error!("Couldn't find {} environment variable.", var);
                process::exit(1);
            }
            false
        }
    }
}
