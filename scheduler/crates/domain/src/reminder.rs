use crate::shared::entity::ID;

/// A `Reminder` represents a specific time before the occurrence a
/// `CalendarEvent` at which the owner `Account` should be notified.
#[derive(Debug, Clone, PartialEq)]
pub struct Reminder {
    /// The `CalendarEvent` this `Reminder` is associated with
    pub event_id: ID,
    /// The `Account` this `Reminder` is associated with and which
    /// should receive a webhook notification at `remind_at`
    pub account_id: ID,
    /// The timestamp at which the `Account` should be notified.
    /// This is usually some minutes before a `CalendarEvent`
    pub remind_at: i64,
    /// This field is needed to avoid sending duplicate `Reminder`s to the `Account`.
    /// For more info see the db schema comments
    pub version: i64,
    /// User defined identifier to be able to separate reminders at same timestamp for the same
    /// event.
    /// For example: "ask_for_booking_review" or "send_invoice"
    pub identifier: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventRemindersExpansionJob {
    pub event_id: ID,
    pub timestamp: i64,
    pub version: i64,
}
