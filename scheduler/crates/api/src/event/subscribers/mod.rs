use super::{
    create_event::CreateEventUseCase,
    delete_event::DeleteEventUseCase,
    sync_event_reminders::{EventOperation, SyncEventRemindersTrigger, SyncEventRemindersUseCase},
    update_event::UpdateEventUseCase,
};
use crate::shared::usecase::{execute, Subscriber};
use nettu_scheduler_domain::CalendarEvent;

pub struct CreateRemindersOnEventCreated;

#[async_trait::async_trait(?Send)]
impl Subscriber<CreateEventUseCase> for CreateRemindersOnEventCreated {
    async fn notify(&self, e: &CalendarEvent, ctx: &nettu_scheduler_infra::NettuContext) {
        let sync_event_reminders = SyncEventRemindersUseCase {
            request: SyncEventRemindersTrigger::EventModified(&e, EventOperation::Created),
        };

        // Sideeffect, ignore result
        let _ = execute(sync_event_reminders, ctx).await;
    }
}

pub struct DeleteRemindersOnEventDeleted;

#[async_trait::async_trait(?Send)]
impl Subscriber<DeleteEventUseCase> for DeleteRemindersOnEventDeleted {
    async fn notify(&self, e: &CalendarEvent, ctx: &nettu_scheduler_infra::NettuContext) {
        let sync_event_reminders = SyncEventRemindersUseCase {
            request: SyncEventRemindersTrigger::EventModified(&e, EventOperation::Deleted),
        };

        // Sideeffect, ignore result
        let _ = execute(sync_event_reminders, ctx).await;
    }
}

pub struct SyncRemindersOnEventUpdated;

#[async_trait::async_trait(?Send)]
impl Subscriber<UpdateEventUseCase> for SyncRemindersOnEventUpdated {
    async fn notify(&self, e: &CalendarEvent, ctx: &nettu_scheduler_infra::NettuContext) {
        let sync_event_reminders = SyncEventRemindersUseCase {
            request: SyncEventRemindersTrigger::EventModified(&e, EventOperation::Updated),
        };

        // Sideeffect, ignore result
        let _ = execute(sync_event_reminders, ctx).await;
    }
}
