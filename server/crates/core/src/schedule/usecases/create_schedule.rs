use crate::user::domain::User;
use crate::{
    context::Context,
    shared::{
        auth::Permission,
        usecase::{execute_with_policy, PermissionBoundary},
    },
};
use crate::{
    schedule::domain::Schedule,
    shared::usecase::{execute, UseCase, UseCaseErrorContainer},
};
use chrono_tz::Tz;

struct CreateScheduleUseCase {
    pub user_id: String,
    pub tzid: String,
}

#[derive(Debug)]
enum UseCaseErrors {
    InvalidTimezone(String),
    UserNotFound,
    Storage,
}

struct UseCaseRes {
    pub schedule: Schedule,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateScheduleUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let tz: Tz = match self.tzid.parse() {
            Ok(tz) => tz,
            Err(_) => return Err(UseCaseErrors::InvalidTimezone(self.tzid.to_string())),
        };

        let user = ctx.repos.user_repo.find(&self.user_id).await;
        if user.is_none() {
            return Err(UseCaseErrors::UserNotFound);
        }

        let schedule = Schedule::new(&self.user_id, &tz);

        let res = ctx.repos.schedule_repo.insert(&schedule).await;
        match res {
            Ok(_) => Ok(UseCaseRes { schedule }),
            Err(_) => Err(UseCaseErrors::Storage),
        }
    }
}

impl PermissionBoundary for CreateScheduleUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::CreateSchedule]
    }
}
