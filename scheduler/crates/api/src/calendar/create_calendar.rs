use crate::shared::auth::{account_can_modify_user, Permission};
use crate::shared::{
    auth::{protect_account_route, protect_route},
    usecase::{execute_with_policy, PermissionBoundary, UseCaseErrorContainer},
};
use crate::{
    error::NettuError,
    shared::usecase::{execute, UseCase},
};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::create_calendar::{APIResponse, PathParams, RequestBody};
use nettu_scheduler_domain::{Calendar, CalendarSettings, Metadata, ID};
use nettu_scheduler_infra::NettuContext;

fn error_handler(e: UseCaseErrors) -> NettuError {
    match e {
        UseCaseErrors::StorageError => NettuError::InternalError,
        UseCaseErrors::InvalidCalendarSetting(e) => NettuError::BadClientData(e),
        UseCaseErrors::UserNotFound => {
            NettuError::NotFound("The requested user was not found.".to_string())
        }
    }
}

pub async fn create_calendar_admin_controller(
    http_req: web::HttpRequest,
    path_params: web::Path<PathParams>,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let user = account_can_modify_user(&account, &path_params.user_id, &ctx).await?;

    let usecase = CreateCalendarUseCase {
        user_id: user.id,
        account_id: account.id,
        week_start: body.0.week_start,
        timezone: body.0.timezone,
        metadata: body.0.metadata.unwrap_or_default(),
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar| HttpResponse::Created().json(APIResponse::new(calendar)))
        .map_err(error_handler)
}

pub async fn create_calendar_controller(
    http_req: web::HttpRequest,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = CreateCalendarUseCase {
        user_id: user.id,
        account_id: user.account_id,
        week_start: body.0.week_start,
        timezone: body.0.timezone,
        metadata: body.0.metadata.unwrap_or_default(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|calendar| HttpResponse::Created().json(APIResponse::new(calendar)))
        .map_err(|e| match e {
            UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
            UseCaseErrorContainer::UseCase(e) => error_handler(e),
        })
}

#[derive(Debug)]
struct CreateCalendarUseCase {
    pub user_id: ID,
    pub account_id: ID,
    pub week_start: isize,
    pub timezone: String,
    pub metadata: Metadata,
}

#[derive(Debug)]
enum UseCaseErrors {
    UserNotFound,
    InvalidCalendarSetting(String),
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateCalendarUseCase {
    type Response = Calendar;

    type Errors = UseCaseErrors;

    const NAME: &'static str = "CreateCalendar";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let user = match ctx.repos.user_repo.find(&self.user_id).await {
            Some(user) if user.account_id == self.account_id => user,
            _ => return Err(UseCaseErrors::UserNotFound),
        };

        let mut settings = CalendarSettings::default();
        if !settings.set_timezone(&self.timezone) {
            return Err(UseCaseErrors::InvalidCalendarSetting(format!(
                "Invalid timezone given: {:?}",
                self.timezone
            )));
        }
        if !settings.set_week_start(self.week_start) {
            return Err(UseCaseErrors::InvalidCalendarSetting(format!(
                "Invalid week_start given: {:?}",
                self.week_start
            )));
        }

        let mut calendar = Calendar::new(&self.user_id, &user.account_id);
        calendar.settings = settings;
        calendar.metadata = self.metadata.clone();

        let res = ctx.repos.calendar_repo.insert(&calendar).await;
        match res {
            Ok(_) => Ok(calendar),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}

impl PermissionBoundary for CreateCalendarUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::CreateCalendar]
    }
}
