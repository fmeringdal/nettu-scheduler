use crate::shared::{
    auth::{account_can_modify_calendar, protect_account_route, Permission},
    usecase::{execute, execute_with_policy, PermissionBoundary, UseCase, UseCaseErrorContainer},
};
use crate::{error::NettuError, shared::auth::protect_route};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::update_calendar::{APIResponse, PathParams, RequestBody};
use nettu_scheduler_domain::{Calendar, Metadata, ID};
use nettu_scheduler_infra::NettuContext;

fn handle_errors(e: UseCaseErrors) -> NettuError {
    match e {
        UseCaseErrors::StorageError => NettuError::InternalError,
        UseCaseErrors::CalendarNotFound => {
            NettuError::NotFound("The calendar was not found.".into())
        }
        UseCaseErrors::InvalidSettings(err) => NettuError::BadClientData(format!(
            "Bad calendar settings provided. Error message: {}",
            err
        )),
    }
}

pub async fn update_calendar_admin_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<NettuContext>,
    path: web::Path<PathParams>,
    body: web::Json<RequestBody>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let cal = account_can_modify_calendar(&account, &path.calendar_id, &ctx).await?;

    let usecase = UpdateCalendarUseCase {
        user_id: cal.user_id,
        calendar_id: cal.id,
        week_start: body.0.settings.week_start,
        timezone: body.0.settings.timezone,
        metadata: body.0.metadata,
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar| HttpResponse::Ok().json(APIResponse::new(calendar)))
        .map_err(handle_errors)
}

pub async fn update_calendar_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<NettuContext>,
    path: web::Path<PathParams>,
    body: web::Json<RequestBody>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = UpdateCalendarUseCase {
        user_id: user.id,
        calendar_id: path.0.calendar_id,
        week_start: body.0.settings.week_start,
        timezone: body.0.settings.timezone,
        metadata: body.0.metadata,
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|calendar| HttpResponse::Ok().json(APIResponse::new(calendar)))
        .map_err(|e| match e {
            UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
            UseCaseErrorContainer::UseCase(e) => handle_errors(e),
        })
}

#[derive(Debug)]
struct UpdateCalendarUseCase {
    pub user_id: ID,
    pub calendar_id: ID,
    pub week_start: Option<isize>,
    pub timezone: Option<String>,
    pub metadata: Option<Metadata>,
}

#[derive(Debug)]
enum UseCaseErrors {
    CalendarNotFound,
    StorageError,
    InvalidSettings(String),
}

#[async_trait::async_trait(?Send)]
impl UseCase for UpdateCalendarUseCase {
    type Response = Calendar;

    type Errors = UseCaseErrors;

    const NAME: &'static str = "UpdateCalendar";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let mut calendar = match ctx.repos.calendar_repo.find(&self.calendar_id).await {
            Some(cal) if cal.user_id == self.user_id => cal,
            _ => return Err(UseCaseErrors::CalendarNotFound),
        };

        if let Some(wkst) = self.week_start {
            if !calendar.settings.set_week_start(wkst) {
                return Err(UseCaseErrors::InvalidSettings(format!(
                    "Invalid week start: {}, must be between 0 and 6",
                    wkst
                )));
            }
        }

        if let Some(timezone) = &self.timezone {
            if !calendar.settings.set_timezone(timezone) {
                return Err(UseCaseErrors::InvalidSettings(format!(
                    "Invalid timezone: {}, must be a valid IANA Timezone string",
                    timezone
                )));
            }
        }

        if let Some(metadata) = &self.metadata {
            calendar.metadata = metadata.clone();
        }

        let repo_res = ctx.repos.calendar_repo.save(&calendar).await;
        match repo_res {
            Ok(_) => Ok(calendar),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}

impl PermissionBoundary for UpdateCalendarUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::UpdateCalendar]
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use nettu_scheduler_domain::Calendar;
    use nettu_scheduler_infra::setup_context;

    use super::*;

    #[actix_web::main]
    #[test]
    async fn it_rejects_invalid_wkst() {
        let ctx = setup_context().await;
        let user_id = ID::default();
        let account_id = ID::default();
        let calendar = Calendar::new(&user_id, &account_id);
        ctx.repos.calendar_repo.insert(&calendar).await.unwrap();

        let mut usecase = UpdateCalendarUseCase {
            calendar_id: calendar.id.into(),
            user_id: user_id.into(),
            week_start: Some(20),
            timezone: None,
            metadata: None,
        };
        let res = usecase.execute(&ctx).await;
        assert!(res.is_err());
    }

    #[actix_web::main]
    #[test]
    async fn it_update_settings_with_valid_wkst() {
        let ctx = setup_context().await;
        let user_id = ID::default();
        let account_id = ID::default();
        let calendar = Calendar::new(&user_id, &account_id);
        ctx.repos.calendar_repo.insert(&calendar).await.unwrap();

        assert_eq!(calendar.settings.week_start, 0);
        let new_wkst = 3;
        let mut usecase = UpdateCalendarUseCase {
            calendar_id: calendar.id.clone(),
            user_id,
            week_start: Some(new_wkst),
            timezone: None,
            metadata: Some(HashMap::new()),
        };
        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());

        // Check that calendar settings have been updated
        let calendar = ctx.repos.calendar_repo.find(&calendar.id).await.unwrap();
        assert_eq!(calendar.settings.week_start, new_wkst);
    }
}
