use crate::shared::auth::Permission;
use crate::{calendar::domain::Calendar, shared::usecase::UseCase};
use crate::{context::Context, shared::usecase::PermissionBoundary};
use serde::Serialize;

struct CreateCalendarUseCase {
    pub user_id: String,
}

#[derive(Debug)]
enum UseCaseErrors {
    UserNotFoundError,
    StorageError,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UseCaseRes {
    pub calendar_id: String,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateCalendarUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let user = ctx.repos.user_repo.find(&self.user_id).await;
        if user.is_none() {
            return Err(UseCaseErrors::UserNotFoundError);
        }

        let calendar = Calendar::new(&self.user_id);

        let res = ctx.repos.calendar_repo.insert(&calendar).await;
        match res {
            Ok(_) => Ok(UseCaseRes {
                calendar_id: calendar.id.clone(),
            }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}

impl PermissionBoundary for CreateCalendarUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::CreateCalendar]
    }
}
