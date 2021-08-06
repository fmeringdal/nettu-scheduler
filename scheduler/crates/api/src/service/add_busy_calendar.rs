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
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::CalendarNotFound => {
                NettuError::NotFound("The requested calendar was not found or user is missing permissions to read the calendar".into())
            }
            UseCaseErrors::UserNotFound => {
                NettuError::NotFound("The specified user was not found".into())
            }
            UseCaseErrors::CalendarAlreadyRegistered => NettuError::Conflict(
                "The busy calendar is already registered on the service user".into(),
            ),
        })
}

#[derive(Debug)]
struct AddBusyCalendarUseCase {
    pub account: Account,
    pub service_id: ID,
    pub user_id: ID,
    pub busy: BusyCalendar,
}

#[derive(Debug)]
enum UseCaseErrors {
    StorageError,
    UserNotFound,
    CalendarAlreadyRegistered,
    CalendarNotFound,
}

#[async_trait::async_trait(?Send)]
impl UseCase for AddBusyCalendarUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    const NAME: &'static str = "AddBusyCalendar";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let user = ctx
            .repos
            .users
            .find_by_account_id(&self.user_id, &self.account.id)
            .await
            .ok_or(UseCaseErrors::UserNotFound)?;

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
                    return Err(UseCaseErrors::CalendarAlreadyRegistered);
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
                    return Err(UseCaseErrors::CalendarAlreadyRegistered);
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
                    return Err(UseCaseErrors::CalendarAlreadyRegistered);
                }
            }
        }

        // Validate calendar permissions
        match &self.busy {
            BusyCalendar::Google(g_cal_id) => {
                let provider = GoogleCalendarProvider::new(&user, ctx)
                    .await
                    .map_err(|_| UseCaseErrors::StorageError)?;

                let g_calendars = provider
                    .list(GoogleCalendarAccessRole::FreeBusyReader)
                    .await
                    .map_err(|_| UseCaseErrors::StorageError)?;

                if g_calendars
                    .items
                    .into_iter()
                    .find(|g_calendar| g_calendar.id == *g_cal_id)
                    .is_none()
                {
                    return Err(UseCaseErrors::CalendarNotFound);
                }
            }
            BusyCalendar::Outlook(o_cal_id) => {
                let provider = OutlookCalendarProvider::new(&user, ctx)
                    .await
                    .map_err(|_| UseCaseErrors::StorageError)?;

                let o_calendars = provider
                    .list(OutlookCalendarAccessRole::Reader)
                    .await
                    .map_err(|_| UseCaseErrors::StorageError)?;

                if o_calendars
                    .value
                    .into_iter()
                    .find(|o_calendar| o_calendar.id == *o_cal_id)
                    .is_none()
                {
                    return Err(UseCaseErrors::CalendarNotFound);
                }
            }
            BusyCalendar::Nettu(n_cal_id) => match ctx.repos.calendars.find(&n_cal_id).await {
                Some(cal) if cal.user_id == user.id => (),
                _ => return Err(UseCaseErrors::CalendarNotFound),
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
                    .map_err(|_| UseCaseErrors::StorageError)
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
                    .map_err(|_| UseCaseErrors::StorageError)
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
                    .map_err(|_| UseCaseErrors::StorageError)
            }
        }
    }
}
