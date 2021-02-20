mod account;
mod calendar;
mod context;
mod event;
mod schedule;
mod service;
mod shared;
mod user;

pub mod ctx {
    use super::*;
    pub use account::repos::IAccountRepo;
    pub use calendar::repos::ICalendarRepo;
    pub use context::Context;
    pub use event::repos::{IEventRepo, IReminderRepo};
    pub use schedule::repos::IScheduleRepo;
    pub use service::repos::IServiceRepo;
    pub use user::repos::IUserRepo;
    pub mod results {
        pub use super::shared::repo::DeleteResult;
    }
}

pub mod domain {
    use super::*;
    pub use account::domain::{Account, AccountSettings, AccountWebhookSettings};
    pub use calendar::domain::{Calendar, CalendarSettings, CalendarView};
    pub use event::domain::event::{CalendarEvent, CalendarEventReminder};
    pub use event::domain::Reminder;
    pub use schedule::domain::{Schedule, ScheduleRule};
    pub use service::domain::Service;
    pub use user::domain::User;
}
