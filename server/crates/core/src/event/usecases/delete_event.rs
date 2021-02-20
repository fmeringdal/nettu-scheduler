use super::sync_event_reminders::{EventOperation, SyncEventRemindersUseCase};
use crate::shared::usecase::{execute, UseCase};
use crate::{
    context::Context,
    shared::{
        auth::Permission,
        usecase::{execute_with_policy, PermissionBoundary, UseCaseErrorContainer},
    },
};

pub struct DeleteEventUseCase {
    pub user_id: String,
    pub event_id: String,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFound,
}

#[async_trait::async_trait(?Send)]
impl UseCase for DeleteEventUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    type Context = Context;

    // TODO: use only one db call
    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let e = ctx.repos.event_repo.find(&self.event_id).await;
        match e {
            Some(event) if event.user_id == self.user_id => {
                ctx.repos.event_repo.delete(&event.id).await;

                let sync_event_reminders = SyncEventRemindersUseCase {
                    event: &event,
                    op: EventOperation::Deleted,
                };
                // TODO: handl err
                execute(sync_event_reminders, ctx).await;

                Ok(())
            }
            _ => Err(UseCaseErrors::NotFound),
        }
    }
}

impl PermissionBoundary for DeleteEventUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::DeleteCalendarEvent]
    }
}
