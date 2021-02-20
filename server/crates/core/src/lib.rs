mod account;
mod calendar;
mod event;
mod schedule;
mod service;
mod shared;
mod user;

pub mod domain {
    use super::*;
    pub use account::domain::{Account, AccountSettings, AccountWebhookSettings};
    pub use calendar::domain::{Calendar, CalendarSettings, CalendarView};
    pub mod booking_slots {
        pub use super::event::domain::booking_slots::*;
    }
    pub mod event_instance {
        pub use super::event::domain::event_instance::*;
    }
    pub use event::domain::event::{
        CalendarEvent, CalendarEventReminder, RRuleFrequenzy, RRuleOptions,
    };
    pub use event::domain::event_instance::EventInstance;
    pub use event::domain::Reminder;
    pub use schedule::domain::{Schedule, ScheduleRule};
    pub use service::domain::{Service, ServiceResource};
    pub use user::domain::User;
}

pub use shared::entity::Entity;
