mod account;
mod base;
mod status;

use account::AccountClient;
use status::StatusClient;
use std::sync::Arc;

pub(crate) use base::{APIResponse, BaseClient};

#[derive(Clone)]
pub struct NettuSDK {
    pub account: AccountClient,
    pub status: StatusClient,
}

impl NettuSDK {
    fn create(base: BaseClient) -> Self {
        let base = Arc::new(base);
        let account = AccountClient::new(base.clone());
        let status = StatusClient::new(base.clone());

        Self { account, status }
    }

    pub fn new(address: String) -> Self {
        let base = BaseClient::new(address);
        Self::create(base)
    }

    pub fn new_admin(address: String, api_key: String) -> Self {
        let mut base = BaseClient::new(address);
        base.set_api_key(api_key);

        Self::create(base)
    }
}
