use crate::dtos::AccountDTO;
use nettu_scheduler_domain::Account;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountResponse {
    pub account: AccountDTO,
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

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub public_jwt_key: Option<String>,
    }

    pub type APIResponse = AccountResponse;
}

pub mod set_account_webhook {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
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

pub mod add_account_integration {
    use nettu_scheduler_domain::IntegrationProvider;

    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub client_id: String,
        pub client_secret: String,
        pub redirect_uri: String,
        pub provider: IntegrationProvider,
    }

    pub type APIResponse = String;
}

pub mod remove_account_integration {
    use super::*;
    use nettu_scheduler_domain::IntegrationProvider;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub provider: IntegrationProvider,
    }

    pub type APIResponse = String;
}
