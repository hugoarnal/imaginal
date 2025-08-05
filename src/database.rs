pub mod spotify {
    use crate::{providers::spotify::AccessTokenJson};

    pub fn get_creds() -> Option<AccessTokenJson> {
        None
    }

    pub fn set_creds(creds: AccessTokenJson) -> bool {
        serde_json::to_string(&creds).unwrap();
        false
    }
}
