use crate::shared::entity::Entity;

/// A `Reminder` represents a specific time before the occurence a
/// `CalendarEvent` when the corresponding `Account` should be notified.
#[derive(Debug, Clone)]
pub struct Reminder {
    pub id: String,
    /// The `CalendarEvent` this `Reminder` is associated with
    pub event_id: String,
    /// The `Account` this `Reminder` is associated with and which
    /// should receive a webhook notification at `remind_at`
    pub account_id: String,
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
    fn id(&self) -> String {
        self.id.clone()
    }
}

#[derive(Debug, Clone)]
pub struct EventRemindersExpansionJob {
    pub id: String,
    pub event_id: String,
    pub timestamp: i64,
}

impl Entity for EventRemindersExpansionJob {
    fn id(&self) -> String {
        self.id.clone()
    }
}
