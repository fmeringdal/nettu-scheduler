use crate::shared::{
    auth::{account_can_modify_user, protect_account_route},
    usecase::{execute, UseCase},
};
use crate::{error::NettuError, shared::auth::protect_route};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::get_google_calendars::{APIResponse, PathParams, QueryParams};
use nettu_scheduler_domain::{
    providers::google::{GoogleCalendarAccessRole, GoogleCalendarListEntry},
    User,
};
use nettu_scheduler_infra::{google_calendar::GoogleCalendarProvider, NettuContext};

pub async fn get_google_calendars_admin_controller(
    http_req: web::HttpRequest,
    path: web::Path<PathParams>,
    query: web::Query<QueryParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let user = account_can_modify_user(&account, &path.user_id, &ctx).await?;

    let usecase = GetGoogleCalendarsUseCase {
        user,
        min_access_role: query.0.min_access_role,
    };

    execute(usecase, &ctx)
        .await
        .map(|calendars| HttpResponse::Ok().json(APIResponse::new(calendars)))
        .map_err(NettuError::from)
}

pub async fn get_google_calendars_controller(
    http_req: HttpRequest,
    query: web::Query<QueryParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, _policy) = protect_route(&http_req, &ctx).await?;

    let usecase = GetGoogleCalendarsUseCase {
        user,
        min_access_role: query.0.min_access_role,
    };

    execute(usecase, &ctx)
        .await
        .map(|calendars| HttpResponse::Ok().json(APIResponse::new(calendars)))
        .map_err(NettuError::from)
}

#[derive(Debug)]
struct GetGoogleCalendarsUseCase {
    pub user: User,
    pub min_access_role: GoogleCalendarAccessRole,
}

#[derive(Debug)]
enum UseCaseError {
    UserNotConnectedToGoogle,
    GoogleQuery,
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::UserNotConnectedToGoogle => {
                Self::BadClientData("The user is not connected to google.".into())
            }
            UseCaseError::GoogleQuery => Self::InternalError,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetGoogleCalendarsUseCase {
    type Response = Vec<GoogleCalendarListEntry>;

    type Error = UseCaseError;

    const NAME: &'static str = "GetGoogleCalendars";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let provider = GoogleCalendarProvider::new(&self.user, ctx)
            .await
            .map_err(|_| UseCaseError::UserNotConnectedToGoogle)?;

        provider
            .list(self.min_access_role.clone())
            .await
            .map_err(|_| UseCaseError::GoogleQuery)
            .map(|res| res.items)
    }
}
