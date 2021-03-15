use crate::shared::entity::{Entity, ID};

/// A `Reminder` represents a specific time before the occurence a
/// `CalendarEvent` at which the owner `Account` should be notified.
#[derive(Debug, Clone)]
pub struct Reminder {
    pub id: ID,
    /// The `CalendarEvent` this `Reminder` is associated with
    pub event_id: ID,
    /// The `Account` this `Reminder` is associated with and which
    /// should receive a webhook notification at `remind_at`
    pub account_id: ID,
    /// The timestamp at which the `Account` should be notified.
    /// This is usually some minutes before a `CalendarEvent`
    pub remind_at: i64,
    /// This field is needed to avoid sending duplicate `Reminder`s to the `Account`.
    /// There are 2 proccesses which produces `Reminder`s, one is triggered when a `CalendarEvent`
    /// is created or modified and the other is a job scheduler generating
    /// `Reminder`s on a schedule. These two could possibly interfere and generate
    /// duplicate `Reminder`s for the same `CalendarEvent`. The former has a higher priority
    /// than the latter.
    /// The job sending out the `Reminder`s is going to detect duplicate `Reminder`s
    /// and filter away the one with the lowest priority.
    pub priority: i64,
}

impl Entity for Reminder {
    fn id(&self) -> &ID {
        &self.id
    }
}

#[derive(Debug, Clone)]
pub struct EventRemindersExpansionJob {
    pub id: ID,
    pub event_id: ID,
    pub timestamp: i64,
}

impl Entity for EventRemindersExpansionJob {
    fn id(&self) -> &ID {
        &self.id
    }
}
