use nettu_scheduler_domain::Account;
use serde::{Deserialize, Serialize};

use crate::dtos::AccountDTO;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountResponse {
    account: AccountDTO,
}

impl AccountResponse {
    pub fn new(account: Account) -> Self {
        Self {
            account: AccountDTO::new(&account),
        }
    }
}

pub mod create_account {
    use super::*;

    #[derive(Deserialize, Serialize)]
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

    impl APIResponse {
        pub fn new(account: Account) -> Self {
            Self {
                account: AccountDTO::new(&account),
                secret_api_key: account.secret_api_key,
            }
        }
    }
}

pub mod get_account {
    use super::*;

    pub type APIResponse = AccountResponse;
}

pub mod set_account_pub_key {
    use super::*;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub public_key_b64: Option<String>,
    }

    pub type APIResponse = AccountResponse;
}

pub mod set_account_webhook {
    use super::*;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub webhook_url: String,
    }

    pub type APIResponse = AccountResponse;
}

pub mod delete_account_webhook {
    use super::*;

    pub type APIResponse = AccountResponse;
}
