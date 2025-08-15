use crate::utils::check_env_existence;

pub mod connection;
pub mod playing;

const CLIENT_ID_ENV: &str = "SPOTIFY_CLIENT_ID";
const CLIENT_SECRET_ENV: &str = "SPOTIFY_CLIENT_SECRET";

pub fn verify(panic: bool) -> bool {
    check_env_existence(CLIENT_ID_ENV, panic);
    check_env_existence(CLIENT_SECRET_ENV, panic)
}
