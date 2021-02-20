use crate::{
    context::Context,
    shared::{
        auth::Permission,
        usecase::{execute_with_policy, PermissionBoundary, UseCaseErrorContainer},
    },
};
use crate::{
    schedule::domain::{Schedule, ScheduleRule},
    shared::usecase::UseCase,
};
use chrono_tz::Tz;

struct UpdateScheduleUseCase {
    pub user_id: String,
    pub schedule_id: String,
    pub timezone: Option<String>,
    pub rules: Option<Vec<ScheduleRule>>,
}

#[derive(Debug)]
enum UseCaseErrors {
    ScheduleNotFoundError,
    StorageError,
    InvalidSettings(String),
}
struct UseCaseRes {
    pub schedule: Schedule,
}

#[async_trait::async_trait(?Send)]
impl UseCase for UpdateScheduleUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let mut schedule = match ctx.repos.schedule_repo.find(&self.schedule_id).await {
            Some(cal) if cal.user_id == self.user_id => cal,
            _ => return Err(UseCaseErrors::ScheduleNotFoundError),
        };

        if let Some(tz) = &self.timezone {
            match tz.parse::<Tz>() {
                Ok(tz) => schedule.timezone = tz,
                Err(_) => {
                    return Err(UseCaseErrors::InvalidSettings(format!(
                        "Invalid timezone provided: {}",
                        tz
                    )))
                }
            }
        };
        if let Some(rules) = &self.rules {
            schedule.set_rules(rules);
        }

        let repo_res = ctx.repos.schedule_repo.save(&schedule).await;
        match repo_res {
            Ok(_) => Ok(UseCaseRes { schedule }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}

impl PermissionBoundary for UpdateScheduleUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::UpdateSchedule]
    }
}
