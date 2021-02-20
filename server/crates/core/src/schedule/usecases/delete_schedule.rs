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

pub struct DeleteScheduleUseCase {
    schedule_id: String,
    user_id: String,
}

#[async_trait::async_trait(?Send)]
impl UseCase for DeleteScheduleUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let schedule = ctx.repos.schedule_repo.find(&self.schedule_id).await;
        match schedule {
            Some(schedule) if schedule.user_id == self.user_id => {
                let res = ctx.repos.schedule_repo.delete(&schedule.id).await;
                if res.is_none() {
                    return Err(UseCaseErrors::UnableToDelete);
                }
                let res = ctx
                    .repos
                    .service_repo
                    .remove_schedule_from_services(&schedule.id)
                    .await;
                if res.is_err() {
                    return Err(UseCaseErrors::UnableToDelete);
                }

                Ok(())
            }
            _ => Err(UseCaseErrors::NotFoundError),
        }
    }
}

impl PermissionBoundary for DeleteScheduleUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::DeleteSchedule]
    }
}
