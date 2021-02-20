use crate::shared::usecase::UseCase;
use crate::{
    context::Context,
    shared::{
        auth::Permission,
        usecase::{execute_with_policy, PermissionBoundary, UseCaseErrorContainer},
    },
};

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFoundError,
    UnableToDelete,
}

pub struct DeleteCalendarUseCase {
    calendar_id: String,
    user_id: String,
}

#[async_trait::async_trait(?Send)]
impl UseCase for DeleteCalendarUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let calendar = ctx.repos.calendar_repo.find(&self.calendar_id).await;
        match calendar {
            Some(calendar) if calendar.user_id == self.user_id => {
                ctx.repos.calendar_repo.delete(&calendar.id).await;
                let repo_res = ctx.repos.event_repo.delete_by_calendar(&calendar.id).await;
                if repo_res.is_err() {
                    return Err(UseCaseErrors::UnableToDelete);
                }
                let repo_res = ctx
                    .repos
                    .service_repo
                    .remove_calendar_from_services(&calendar.id)
                    .await;
                if repo_res.is_err() {
                    return Err(UseCaseErrors::UnableToDelete);
                }

                Ok(())
            }
            _ => Err(UseCaseErrors::NotFoundError),
        }
    }
}

impl PermissionBoundary for DeleteCalendarUseCase {
    fn permissions(&self) -> Vec<crate::shared::auth::Permission> {
        vec![Permission::DeleteCalendar]
    }
}
