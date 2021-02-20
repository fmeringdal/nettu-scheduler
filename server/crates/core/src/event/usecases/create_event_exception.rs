use crate::shared::usecase::UseCase;
use crate::{
    context::Context,
    shared::{
        auth::Permission,
        usecase::{execute_with_policy, PermissionBoundary, UseCaseErrorContainer},
    },
};

pub struct CreateEventExceptionUseCase {
    event_id: String,
    exception_ts: i64,
    user_id: String,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFoundError,
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateEventExceptionUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let mut event = match ctx.repos.event_repo.find(&self.event_id).await {
            Some(event) if event.user_id == self.user_id => event,
            _ => return Err(UseCaseErrors::NotFoundError),
        };

        event.exdates.push(self.exception_ts);

        let repo_res = ctx.repos.event_repo.save(&event).await;
        if repo_res.is_err() {
            return Err(UseCaseErrors::StorageError);
        }

        Ok(())
    }
}

impl PermissionBoundary for CreateEventExceptionUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::UpdateCalendarEvent]
    }
}
