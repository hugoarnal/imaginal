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
    use std::{fs, path::Path};

    use crate::{
        database::{DATABASE_FOLDER, init_folder},
        providers::spotify::AccessTokenJson,
    };

    const ACCESS_TOKEN_FILE: &str = "spotify_access_token.json";

    pub fn get_creds() -> Option<AccessTokenJson> {
        if !init_folder() {
            return None;
        }

        let full_path = format!("{}/{}", DATABASE_FOLDER, ACCESS_TOKEN_FILE);
        let path = Path::new(&full_path);

        if path.exists() {
            // TODO: add file contents
            let content = "";
            match serde_json::from_str::<AccessTokenJson>(content) {
                Ok(content) => {
                    return Some(content);
                }
                Err(_) => {
                    return None;
                }
            }
        } else {
            return None;
        }
    }

    pub fn set_creds(creds: AccessTokenJson) -> bool {
        serde_json::to_string(&creds).unwrap();
        false
    }
}
