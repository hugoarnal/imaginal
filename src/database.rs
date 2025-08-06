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

pub mod spotify {
    use std::{
        fs::{self, File},
        io::Write,
        path::Path,
    };

    use crate::{
        database::{DATABASE_FOLDER, init_folder},
        providers::spotify::AccessTokenJson,
    };

    const ACCESS_TOKEN_FILE: &str = "spotify_access_token.json";

    pub fn get_creds() -> Option<AccessTokenJson> {
        if !init_folder() {
            log::error!("Couldn't create or enter `database` folder");
            return None;
        }

        let full_path = format!("{}/{}", DATABASE_FOLDER, ACCESS_TOKEN_FILE);
        let path = Path::new(&full_path);

        if path.exists() {
            let content = fs::read_to_string(path);
            if content.is_err() {
                log::error!("Couldn't read {}", full_path);
                return None;
            }
            match serde_json::from_str::<AccessTokenJson>(content.unwrap().as_str()) {
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
        let content: String;
        match serde_json::to_string(&creds) {
            Ok(string) => content = string,
            Err(_) => {
                log::error!("Couldn't serialize credentials");
                return false;
            }
        };

        // TODO: simplify this redundancy
        let full_path = format!("{}/{}", DATABASE_FOLDER, ACCESS_TOKEN_FILE);
        let path = Path::new(&full_path);

        let file = File::create(path);
        let mut output: File;
        if file.is_ok() {
            output = file.unwrap();
        } else {
            log::error!("Couldn't open {}", full_path);
            return false;
        }
        match write!(output, "{}", content) {
            Ok(_) => true,
            Err(_) => {
                log::error!("Couldn't write to {}", full_path);
                false
            },
        }
    }
}
