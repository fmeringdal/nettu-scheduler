use crate::error::NettuError;
use crate::shared::{
    auth::protect_account_route,
    usecase::{execute, UseCase},
};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::add_busy_calendar::*;
use nettu_scheduler_domain::IntegrationProvider;
use nettu_scheduler_domain::{
    providers::{google::GoogleCalendarAccessRole, outlook::OutlookCalendarAccessRole},
    Account, BusyCalendar, ID,
};
use nettu_scheduler_infra::{
    google_calendar::GoogleCalendarProvider, outlook_calendar::OutlookCalendarProvider,
    NettuContext,
};
use nettu_scheduler_infra::{BusyCalendarIdentifier, ExternalBusyCalendarIdentifier};

pub async fn add_busy_calendar_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let path = path_params.0;
    let body = body.0;
    let usecase = AddBusyCalendarUseCase {
        account,
        service_id: path.service_id.to_owned(),
        user_id: path.user_id.to_owned(),
        busy: body.busy,
    };

    execute(usecase, &ctx)
        .await
        .map(|_| HttpResponse::Ok().json(APIResponse::from("Busy calendar added to service user")))
        .map_err(NettuError::from)
}

#[derive(Debug)]
struct AddBusyCalendarUseCase {
    pub account: Account,
    pub service_id: ID,
    pub user_id: ID,
    pub busy: BusyCalendar,
}

#[derive(Debug)]
enum UseCaseError {
    StorageError,
    UserNotFound,
    CalendarAlreadyRegistered,
    CalendarNotFound,
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::CalendarNotFound => {
                Self::NotFound("The requested calendar was not found or user is missing permissions to read the calendar".into())
            }
            UseCaseError::UserNotFound => {
                Self::NotFound("The specified user was not found".into())
            }
            UseCaseError::CalendarAlreadyRegistered => Self::Conflict(
                "The busy calendar is already registered on the service user".into(),
            ),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for AddBusyCalendarUseCase {
    type Response = ();

    type Error = UseCaseError;

    const NAME: &'static str = "AddBusyCalendar";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let user = ctx
            .repos
            .users
            .find_by_account_id(&self.user_id, &self.account.id)
            .await
            .ok_or(UseCaseError::UserNotFound)?;

        // Check if busy calendar already exists
        match &self.busy {
            BusyCalendar::Google(g_cal_id) => {
                let identifier = ExternalBusyCalendarIdentifier {
                    ext_calendar_id: g_cal_id.clone(),
                    provider: IntegrationProvider::Google,
                    service_id: self.service_id.clone(),
                    user_id: user.id.clone(),
                };
                if ctx
                    .repos
                    .service_user_busy_calendars
                    .exists_ext(identifier)
                    .await
                    .unwrap_or(false)
                {
                    return Err(UseCaseError::CalendarAlreadyRegistered);
                }
            }
            BusyCalendar::Outlook(o_cal_id) => {
                let identifier = ExternalBusyCalendarIdentifier {
                    ext_calendar_id: o_cal_id.clone(),
                    provider: IntegrationProvider::Outlook,
                    service_id: self.service_id.clone(),
                    user_id: user.id.clone(),
                };
                if ctx
                    .repos
                    .service_user_busy_calendars
                    .exists_ext(identifier)
                    .await
                    .unwrap_or(false)
                {
                    return Err(UseCaseError::CalendarAlreadyRegistered);
                }
            }
            BusyCalendar::Nettu(n_cal_id) => {
                let identifier = BusyCalendarIdentifier {
                    calendar_id: n_cal_id.clone(),
                    service_id: self.service_id.clone(),
                    user_id: user.id.clone(),
                };
                if ctx
                    .repos
                    .service_user_busy_calendars
                    .exists(identifier)
                    .await
                    .unwrap_or(false)
                {
                    return Err(UseCaseError::CalendarAlreadyRegistered);
                }
            }
        }

        // Validate calendar permissions
        match &self.busy {
            BusyCalendar::Google(g_cal_id) => {
                let provider = GoogleCalendarProvider::new(&user, ctx)
                    .await
                    .map_err(|_| UseCaseError::StorageError)?;

                let g_calendars = provider
                    .list(GoogleCalendarAccessRole::FreeBusyReader)
                    .await
                    .map_err(|_| UseCaseError::StorageError)?;

                if !g_calendars
                    .items
                    .into_iter()
                    .any(|g_calendar| g_calendar.id == *g_cal_id)
                {
                    return Err(UseCaseError::CalendarNotFound);
                }
            }
            BusyCalendar::Outlook(o_cal_id) => {
                let provider = OutlookCalendarProvider::new(&user, ctx)
                    .await
                    .map_err(|_| UseCaseError::StorageError)?;

                let o_calendars = provider
                    .list(OutlookCalendarAccessRole::Reader)
                    .await
                    .map_err(|_| UseCaseError::StorageError)?;

                if !o_calendars
                    .value
                    .into_iter()
                    .any(|o_calendar| o_calendar.id == *o_cal_id)
                {
                    return Err(UseCaseError::CalendarNotFound);
                }
            }
            BusyCalendar::Nettu(n_cal_id) => match ctx.repos.calendars.find(n_cal_id).await {
                Some(cal) if cal.user_id == user.id => (),
                _ => return Err(UseCaseError::CalendarNotFound),
            },
        }

        // Insert busy calendar
        match &self.busy {
            BusyCalendar::Google(g_cal_id) => {
                let identifier = ExternalBusyCalendarIdentifier {
                    ext_calendar_id: g_cal_id.clone(),
                    provider: IntegrationProvider::Google,
                    service_id: self.service_id.clone(),
                    user_id: user.id.clone(),
                };
                ctx.repos
                    .service_user_busy_calendars
                    .insert_ext(identifier)
                    .await
                    .map_err(|_| UseCaseError::StorageError)
            }
            BusyCalendar::Outlook(o_cal_id) => {
                let identifier = ExternalBusyCalendarIdentifier {
                    ext_calendar_id: o_cal_id.clone(),
                    provider: IntegrationProvider::Outlook,
                    service_id: self.service_id.clone(),
                    user_id: user.id.clone(),
                };
                ctx.repos
                    .service_user_busy_calendars
                    .insert_ext(identifier)
                    .await
                    .map_err(|_| UseCaseError::StorageError)
            }
            BusyCalendar::Nettu(n_cal_id) => {
                let identifier = BusyCalendarIdentifier {
                    calendar_id: n_cal_id.clone(),
                    service_id: self.service_id.clone(),
                    user_id: user.id.clone(),
                };
                ctx.repos
                    .service_user_busy_calendars
                    .insert(identifier)
                    .await
                    .map_err(|_| UseCaseError::StorageError)
            }
        }
    }
}
