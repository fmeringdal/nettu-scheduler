mod account;
mod base;
mod calendar;
mod event;
mod schedule;
mod service;
mod status;
mod user;

use account::AccountClient;
use calendar::CalendarClient;
use event::CalendarEventClient;
use schedule::ScheduleClient;
use service::ServiceClient;
use status::StatusClient;
use std::sync::Arc;
use user::UserClient;

pub(crate) use base::{APIResponse, BaseClient};

pub use calendar::{
    CreateCalendarInput, DeleteCalendarInput, GetCalendarEventsInput, GetCalendarInput,
    UpdateCalendarSettingsInput,
};
pub use event::{
    CreateEventInput, DeleteEventInput, GetEventInput, GetEventsInstancesInput, UpdateEventInput,
};
pub use nettu_scheduler_domain::{CalendarEventReminder, RRuleOptions, ScheduleRule, TimePlan, ID};
pub use schedule::{CreateScheduleInput, UpdateScheduleInput};
pub use service::{
    AddServiceUserInput, GetSerivceBookingSlotsInput, RemoveServiceUserInput,
    UpdateServiceUserInput,
};

/// Nettu Scheduler Server SDK
///
/// The SDK contains methods for interacting with the Nettu Scheduler server
/// API.
#[derive(Clone)]
pub struct NettuSDK {
    pub account: AccountClient,
    pub calendar: CalendarClient,
    pub event: CalendarEventClient,
    pub schedule: ScheduleClient,
    pub service: ServiceClient,
    pub status: StatusClient,
    pub user: UserClient,
}

impl NettuSDK {
    pub fn new<T: Into<String>>(address: String, api_key: T) -> Self {
        let mut base = BaseClient::new(address);
        base.set_api_key(api_key.into());
        let base = Arc::new(base);
        let account = AccountClient::new(base.clone());
        let calendar = CalendarClient::new(base.clone());
        let event = CalendarEventClient::new(base.clone());
        let schedule = ScheduleClient::new(base.clone());
        let service = ServiceClient::new(base.clone());
        let status = StatusClient::new(base.clone());
        let user = UserClient::new(base.clone());

        Self {
            account,
            calendar,
            event,
            schedule,
            service,
            status,
            user,
        }
    }
}
