use crate::shared::usecase::UseCase;

use nettu_scheduler_domain::{Calendar, CalendarEvent, EventRemindersExpansionJob, Reminder};
use nettu_scheduler_infra::NettuContext;
use std::iter::Iterator;

use tracing::error;

#[derive(Debug)]
pub enum EventOperation {
    Created,
    Updated,
    Deleted,
}

/// Synchronizes the upcoming `Reminders` for a `CalendarEvent`
#[derive(Debug)]
pub struct SyncEventRemindersUseCase<'a> {
    pub request: SyncEventRemindersTrigger<'a>,
}

#[derive(Debug)]
pub enum SyncEventRemindersTrigger<'a> {
    /// A `CalendarEvent` has been modified, e.g. deleted, updated og created.
    EventModified(&'a CalendarEvent, EventOperation),
    /// Periodic Job Scheduler that triggers this use case to perform
    /// `EventRemindersExpansionJob`s.
    JobScheduler,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    StorageError,
    CalendarNotFound,
}

async fn create_event_reminders(
    event: &CalendarEvent,
    calendar: &Calendar,
    priority: i64,
    ctx: &NettuContext,
) -> Result<(), UseCaseErrors> {
    let event_reminder_settings = match &event.reminder {
        None => return Ok(()), // Nothing more to do
        Some(settings) => settings,
    };
    let millis_before = event_reminder_settings.minutes_before * 60 * 1000;

    let rrule_set = event.get_rrule_set(&calendar.settings);
    let reminders = match rrule_set {
        Some(rrule_set) => {
            let rrule_set_iter = rrule_set.into_iter();
            let dates = rrule_set_iter
                // Ignore old dates
                .skip_while(|d| d.timestamp_millis() < ctx.sys.get_timestamp_millis())
                // Take the future 100 dates
                .take(100)
                .collect::<Vec<_>>();

            if dates.len() == 100 {
                // There are more reminders to generate, store a job to expand them later
                let job = EventRemindersExpansionJob {
                    id: Default::default(),
                    event_id: event.id.clone(),
                    timestamp: dates[90].timestamp_millis(),
                };
                if ctx
                    .repos
                    .event_reminders_expansion_jobs
                    .bulk_insert(&[job])
                    .await
                    .is_err()
                {
                    error!(
                        "Unable to store event reminders expansion job for event: {}",
                        event.id
                    );
                }
            }

            dates
                .iter()
                .map(|d| Reminder {
                    id: Default::default(),
                    event_id: event.id.to_owned(),
                    account_id: event.account_id.to_owned(),
                    remind_at: d.timestamp_millis() - millis_before,
                    priority,
                })
                .collect()
        }
        None => vec![Reminder {
            id: Default::default(),
            event_id: event.id.to_owned(),
            account_id: event.account_id.to_owned(),
            remind_at: event.start_ts - millis_before,
            priority,
        }],
    };

    // create reminders for the next `self.expansion_interval`
    if ctx.repos.reminders.bulk_insert(&reminders).await.is_err() {
        return Err(UseCaseErrors::StorageError);
    }

    Ok(())
}

#[async_trait::async_trait(?Send)]
impl<'a> UseCase for SyncEventRemindersUseCase<'a> {
    type Response = ();

    type Errors = UseCaseErrors;

    const NAME: &'static str = "SyncEventReminders";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        match &self.request {
            SyncEventRemindersTrigger::EventModified(calendar_event, op) => {
                // Delete event reminder expansion job if it exists
                if ctx
                    .repos
                    .event_reminders_expansion_jobs
                    .delete_by_event(&calendar_event.id)
                    .await
                    .is_err()
                {
                    error!(
                        "Unable to delete event reminder expansion job for event: {}",
                        calendar_event.id
                    );
                    return Err(UseCaseErrors::StorageError);
                }

                // Delete existing reminders
                match op {
                    // There are no reminders if `CalendarEvent` was just created
                    EventOperation::Created => (),
                    _ => {
                        let delete_result = ctx
                            .repos
                            .reminders
                            .delete_by_events(&[calendar_event.id.clone()])
                            .await;
                        if delete_result.is_err() {
                            return Err(UseCaseErrors::StorageError);
                        }
                    }
                }

                // Create new ones if op != delete
                let calendar = match op {
                    EventOperation::Deleted => return Ok(()),
                    EventOperation::Created | EventOperation::Updated => {
                        let calendar = ctx.repos.calendars.find(&calendar_event.calendar_id).await;
                        match calendar {
                            Some(calendar) => calendar,
                            None => return Err(UseCaseErrors::CalendarNotFound),
                        }
                    }
                };

                create_event_reminders(calendar_event, &calendar, 1, ctx).await
            }
            SyncEventRemindersTrigger::JobScheduler => {
                let jobs = ctx
                    .repos
                    .event_reminders_expansion_jobs
                    .delete_all_before(ctx.sys.get_timestamp_millis())
                    .await;

                let event_ids = jobs
                    .iter()
                    .map(|job| job.event_id.to_owned())
                    .collect::<Vec<_>>();

                if ctx
                    .repos
                    .reminders
                    .delete_by_events(&event_ids)
                    .await
                    .is_err()
                {
                    return Err(UseCaseErrors::StorageError);
                }

                let events = match ctx.repos.events.find_many(&event_ids).await {
                    Ok(events) => events,
                    Err(_) => return Err(UseCaseErrors::StorageError),
                };

                for event in events {
                    let calendar = match ctx.repos.calendars.find(&event.calendar_id).await {
                        Some(cal) => cal,
                        None => continue,
                    };
                    if let Err(err) = create_event_reminders(&event, &calendar, 0, ctx).await {
                        error!(
                            "Unable to create event reminders for event {}, Error: {:?}",
                            event.id, err
                        );
                    }
                }

                Ok(())
            }
        }
    }
}
