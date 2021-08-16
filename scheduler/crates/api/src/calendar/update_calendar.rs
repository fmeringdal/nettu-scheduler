use crate::shared::{
    auth::{
        account_can_modify_calendar, account_can_modify_user, protect_account_route, Permission,
    },
    usecase::{execute, execute_with_policy, PermissionBoundary, UseCase},
};
use crate::{error::NettuError, shared::auth::protect_route};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::update_calendar::{APIResponse, PathParams, RequestBody};
use nettu_scheduler_domain::{Calendar, Metadata, User, ID};
use nettu_scheduler_infra::NettuContext;

pub async fn update_calendar_admin_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<NettuContext>,
    path: web::Path<PathParams>,
    body: web::Json<RequestBody>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let cal = account_can_modify_calendar(&account, &path.calendar_id, &ctx).await?;
    let user = account_can_modify_user(&account, &cal.user_id, &ctx).await?;

    let usecase = UpdateCalendarUseCase {
        user,
        calendar_id: cal.id,
        week_start: body.0.settings.week_start,
        timezone: body.0.settings.timezone,
        metadata: body.0.metadata,
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar| HttpResponse::Ok().json(APIResponse::new(calendar)))
        .map_err(NettuError::from)
}

pub async fn update_calendar_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<NettuContext>,
    path: web::Path<PathParams>,
    body: web::Json<RequestBody>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = UpdateCalendarUseCase {
        user,
        calendar_id: path.0.calendar_id,
        week_start: body.0.settings.week_start,
        timezone: body.0.settings.timezone,
        metadata: body.0.metadata,
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|calendar| HttpResponse::Ok().json(APIResponse::new(calendar)))
        .map_err(NettuError::from)
}

#[derive(Debug)]
struct UpdateCalendarUseCase {
    pub user: User,
    pub calendar_id: ID,
    pub week_start: Option<isize>,
    pub timezone: Option<String>,
    pub metadata: Option<Metadata>,
}

#[derive(Debug)]
enum UseCaseError {
    CalendarNotFound,
    StorageError,
    InvalidSettings(String),
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::CalendarNotFound => Self::NotFound("The calendar was not found.".into()),
            UseCaseError::InvalidSettings(err) => Self::BadClientData(format!(
                "Bad calendar settings provided. Error message: {}",
                err
            )),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for UpdateCalendarUseCase {
    type Response = Calendar;

    type Error = UseCaseError;

    const NAME: &'static str = "UpdateCalendar";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let mut calendar = match ctx.repos.calendars.find(&self.calendar_id).await {
            Some(cal) if cal.user_id == self.user.id => cal,
            _ => return Err(UseCaseError::CalendarNotFound),
        };

        if let Some(wkst) = self.week_start {
            if !calendar.settings.set_week_start(wkst) {
                return Err(UseCaseError::InvalidSettings(format!(
                    "Invalid week start: {}, must be between 0 and 6",
                    wkst
                )));
            }
        }

        if let Some(timezone) = &self.timezone {
            if !calendar.settings.set_timezone(timezone) {
                return Err(UseCaseError::InvalidSettings(format!(
                    "Invalid timezone: {}, must be a valid IANA Timezone string",
                    timezone
                )));
            }
        }

        if let Some(metadata) = &self.metadata {
            calendar.metadata = metadata.clone();
        }

        ctx.repos
            .calendars
            .save(&calendar)
            .await
            .map(|_| calendar)
            .map_err(|_| UseCaseError::StorageError)
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

    use nettu_scheduler_domain::{Account, Calendar, User};
    use nettu_scheduler_infra::setup_context;

    use super::*;

    #[actix_web::main]
    #[test]
    async fn it_rejects_invalid_wkst() {
        let ctx = setup_context().await;
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();
        let user = User::new(account.id.clone());
        ctx.repos.users.insert(&user).await.unwrap();
        let calendar = Calendar::new(&user.id, &account.id);
        ctx.repos.calendars.insert(&calendar).await.unwrap();

        let mut usecase = UpdateCalendarUseCase {
            user,
            calendar_id: calendar.id.into(),
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
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();
        let user = User::new(account.id.clone());
        ctx.repos.users.insert(&user).await.unwrap();
        let calendar = Calendar::new(&user.id, &account.id);
        ctx.repos.calendars.insert(&calendar).await.unwrap();

        assert_eq!(calendar.settings.week_start, 0);
        let new_wkst = 3;
        let mut usecase = UpdateCalendarUseCase {
            user,
            calendar_id: calendar.id.clone(),
            week_start: Some(new_wkst),
            timezone: None,
            metadata: Some(HashMap::new()),
        };
        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());

        // Check that calendar settings have been updated
        let calendar = ctx.repos.calendars.find(&calendar.id).await.unwrap();
        assert_eq!(calendar.settings.week_start, new_wkst);
    }
}
