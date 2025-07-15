use std::env;

pub fn check_env_existence(var: &str, panic: bool) -> bool {
    match env::var(var) {
        Ok(_) => {
            true
        },
        Err(_) => {
            if panic {
                panic!("Couldn't find {} environment variable.", var);
            }
            false
        }
    }
}
