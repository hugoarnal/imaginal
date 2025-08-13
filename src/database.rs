use std::fs;
use std::path::Path;

const DATABASE_FOLDER: &str = "database";

fn init_folder() -> bool {
    let path = Path::new(DATABASE_FOLDER);
    if path.exists() && path.is_dir() {
        return true;
    } else if !path.exists() {
        let dir = fs::create_dir(DATABASE_FOLDER);
        if dir.is_err() {
            return false;
        }
        return true;
    } else {
        return false;
    }
}

fn get_full_path(file_name: &str) -> String {
    format!("{}/{}", DATABASE_FOLDER, file_name)
}

pub mod spotify {
    use std::{
        fs::{self, File},
        io::Write,
        os::unix::fs::PermissionsExt,
        path::Path,
    };

    use crate::{
        database::{get_full_path, init_folder},
        providers::spotify::connection::AccessTokenJson,
    };

    const ACCESS_TOKEN_FILE: &str = "spotify_access_token.json";

    pub fn get_creds() -> Option<AccessTokenJson> {
        if !init_folder() {
            log::error!("Couldn't create or enter `database` folder");
            return None;
        }

        let full_path = get_full_path(ACCESS_TOKEN_FILE);
        let path = Path::new(&full_path);

        if path.exists() {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => {
                    log::error!("Couldn't read {}", full_path);
                    return None;
                }
            };
            match serde_json::from_str::<AccessTokenJson>(content.as_str()) {
                Ok(content) => {
                    return Some(content);
                }
                Err(_) => {
                    log::error!("Couldn't deserialize {}", full_path);
                    return None;
                }
            }
        }
        None
    }

    pub fn set_creds(creds: AccessTokenJson) -> bool {
        let content: String = match serde_json::to_string(&creds) {
            Ok(string) => string,
            Err(_) => {
                log::error!("Couldn't serialize credentials");
                return false;
            }
        };

        let full_path = get_full_path(ACCESS_TOKEN_FILE);
        let path = Path::new(&full_path);

        let file = File::create(path);
        let mut output: File = match file {
            Ok(c) => c,
            Err(_) => {
                log::error!("Couldn't open {}", full_path);
                return false;
            }
        };
        let mut permissions = output.metadata().unwrap().permissions();
        permissions.set_mode(0o666);

        match std::fs::set_permissions(full_path.clone(), permissions) {
            Ok(_) => {}
            Err(_) => {
                log::error!("Couldn't change permissions of {}", full_path);
                return false;
            }
        };

        match write!(output, "{}", content) {
            Ok(_) => true,
            Err(_) => {
                log::error!("Couldn't write to {}", full_path);
                false
            }
        }
    }
}
