mod account;
mod base;
mod calendar;
mod event;
mod schedule;
mod service;
mod shared;
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
    UpdateCalendarInput,
};
pub use event::{
    CreateEventInput, DeleteEventInput, GetEventInput, GetEventsInstancesInput, UpdateEventInput,
};
pub use nettu_scheduler_domain::{
    providers::google::*, BusyCalendar, CalendarEventReminder, RRuleOptions, ScheduleRule,
    SyncedCalendar, SyncedCalendarProvider, TimePlan, ID,
};
pub use schedule::{CreateScheduleInput, UpdateScheduleInput};
pub use service::{
    AddServiceUserInput, GetServiceBookingSlotsInput, RemoveServiceUserInput,
    UpdateServiceUserInput,
};
pub use shared::{KVMetadata, MetadataFindInput};
pub use user::CreateUserInput;

// Domain
pub use nettu_scheduler_api_structs::dtos::AccountDTO as Account;
pub use nettu_scheduler_api_structs::dtos::AccountSettingsDTO as AccountSettings;
pub use nettu_scheduler_api_structs::dtos::AccountWebhookSettingsDTO as AccountWebhookSettings;
pub use nettu_scheduler_api_structs::dtos::CalendarDTO as Calendar;
pub use nettu_scheduler_api_structs::dtos::CalendarEventDTO as CalendarEvent;
pub use nettu_scheduler_api_structs::dtos::CalendarSettingsDTO as CalendarSettings;
pub use nettu_scheduler_api_structs::dtos::EventWithInstancesDTO as EventWithIInstances;
pub use nettu_scheduler_api_structs::dtos::ScheduleDTO as Schedule;
pub use nettu_scheduler_api_structs::dtos::ServiceResourceDTO as ServiceResource;
pub use nettu_scheduler_api_structs::dtos::ServiceWithUsersDTO as Service;
pub use nettu_scheduler_api_structs::dtos::UserDTO as User;

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
