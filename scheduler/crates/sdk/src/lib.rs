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
pub(crate) use base::BaseClient;
pub use base::{APIError, APIErrorVariant, APIResponse};
use calendar::CalendarClient;
pub use calendar::{
    CreateCalendarInput, DeleteCalendarInput, GetCalendarEventsInput, GetCalendarInput,
    StopCalendarSyncInput, SyncCalendarInput, UpdateCalendarInput,
};
use event::CalendarEventClient;
pub use event::{
    CreateEventInput, DeleteEventInput, GetEventInput, GetEventsInstancesInput, UpdateEventInput,
};
pub use nettu_scheduler_api_structs::dtos::*;
pub use nettu_scheduler_api_structs::send_event_reminders::AccountRemindersDTO as AccountReminders;
pub use nettu_scheduler_domain::{
    providers::google::*, providers::outlook::*, scheduling::RoundRobinAlgorithm, BusyCalendar,
    CalendarEventReminder, IntegrationProvider, Metadata, RRuleOptions, ScheduleRule,
    ServiceMultiPersonOptions, SyncedCalendar, TimePlan, ID,
};
use schedule::ScheduleClient;
pub use schedule::{CreateScheduleInput, UpdateScheduleInput};
use service::ServiceClient;
pub use service::{
    AddBusyCalendar, AddServiceUserInput, CreateBookingIntendInput, CreateServiceInput,
    GetServiceBookingSlotsInput, RemoveBookingIntendInput, RemoveBusyCalendar,
    RemoveServiceUserInput, UpdateServiceInput, UpdateServiceUserInput,
};
pub use shared::{KVMetadata, MetadataFindInput};
use status::StatusClient;
use std::sync::Arc;
use user::UserClient;
pub use user::{CreateUserInput, GetUserFreeBusyInput, UpdateUserInput};

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

pub use nettu_scheduler_domain::Tz;
pub use nettu_scheduler_domain::Weekday;

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
        let user = UserClient::new(base);

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
