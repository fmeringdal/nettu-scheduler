mod account;
mod base;
mod calendar;
mod schedule;
mod status;
mod user;

use account::AccountClient;
use schedule::ScheduleClient;
use status::StatusClient;
use std::sync::Arc;
use user::UserClient;

pub(crate) use base::{APIResponse, BaseClient};

pub use schedule::CreateScheduleInput;

/// Nettu Scheduler Server SDK
///
/// The SDK contains methods for interacting with the Nettu Scheduler server
/// API.
#[derive(Clone)]
pub struct NettuSDK {
    pub account: AccountClient,
    pub schedule: ScheduleClient,
    pub status: StatusClient,
    pub user: UserClient,
}

impl NettuSDK {
    pub fn new<T: Into<String>>(address: String, api_key: T) -> Self {
        let mut base = BaseClient::new(address);
        base.set_api_key(api_key.into());
        let base = Arc::new(base);
        let account = AccountClient::new(base.clone());
        let schedule = ScheduleClient::new(base.clone());
        let status = StatusClient::new(base.clone());
        let user = UserClient::new(base.clone());

        Self {
            account,
            schedule,
            status,
            user,
        }
    }
}
