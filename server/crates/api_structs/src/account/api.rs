use serde::{Deserialize, Serialize};

use crate::dtos::AccountDTO;

pub mod create_account {
    use super::*;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub code: String,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct APIResponse {
        pub account: AccountDTO,
        pub secret_api_key: String,
    }
}

pub mod set_account_pub_key {
    use super::*;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub public_key_b64: Option<String>,
    }
}

pub mod set_account_webhook {
    use super::*;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub webhook_url: String,
    }
}
