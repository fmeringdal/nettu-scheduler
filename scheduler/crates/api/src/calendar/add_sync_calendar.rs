use crate::shared::auth::{account_can_modify_user, Permission};
use crate::shared::{auth::protect_account_route, usecase::PermissionBoundary};
use crate::{
    error::NettuError,
    shared::usecase::{execute, UseCase},
};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::add_sync_calendar::{APIResponse, PathParams, RequestBody};
use nettu_scheduler_domain::IntegrationProvider;
use nettu_scheduler_domain::{
    providers::{google::GoogleCalendarAccessRole, outlook::OutlookCalendarAccessRole},
    SyncedCalendar, User, ID,
};
use nettu_scheduler_infra::{
    google_calendar::GoogleCalendarProvider, outlook_calendar::OutlookCalendarProvider,
    NettuContext,
};

fn error_handler(e: UseCaseErrors) -> NettuError {
    match e {
        UseCaseErrors::StorageError => NettuError::InternalError,
        UseCaseErrors::ExternalCalendarNotFound => NettuError::NotFound("The external calendar was not found. Make sure it exists and that user has write access to that calendar".into()),
        UseCaseErrors::CalendarAlreadySynced => NettuError::Conflict("The calendar is already synced to the given external calendar".into()),
        UseCaseErrors::NoProviderIntegration => NettuError::NotFound("The user has not integrated with the given provider".into()),
    }
}

pub async fn add_sync_calendar_admin_controller(
    http_req: web::HttpRequest,
    path_params: web::Path<PathParams>,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let user = account_can_modify_user(&account, &path_params.user_id, &ctx).await?;

    let body = body.0;
    let usecase = AddSyncCalendarUseCase {
        user,
        calendar_id: body.calendar_id,
        ext_calendar_id: body.ext_calendar_id,
        provider: body.provider,
    };

    execute(usecase, &ctx)
        .await
        .map(|_| HttpResponse::Ok().json(APIResponse::from("Calendar sync created")))
        .map_err(error_handler)
}

// pub async fn add_sync_calendar_controller(
//     http_req: web::HttpRequest,
//     body: web::Json<RequestBody>,
//     ctx: web::Data<NettuContext>,
// ) -> Result<HttpResponse, NettuError> {
//     let (user, policy) = protect_route(&http_req, &ctx).await?;

//     let body = body.0;

//     let usecase = AddSyncCalendarUseCase {
//         user,
//         calendar_id: body.calendar_id,
//         ext_calendar_id: body.ext_calendar_id,
//         provider: body.provider,
//     };

//     execute_with_policy(usecase, &policy, &ctx)
//         .await
//         .map(|_| HttpResponse::Ok().json(APIResponse::from("Calendar sync created")))
//         .map_err(|e| match e {
//             UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
//             UseCaseErrorContainer::UseCase(e) => error_handler(e),
//         })
// }

#[derive(Debug)]
struct AddSyncCalendarUseCase {
    pub user: User,
    pub provider: IntegrationProvider,
    pub calendar_id: ID,
    pub ext_calendar_id: String,
}

#[derive(Debug)]
enum UseCaseErrors {
    NoProviderIntegration,
    ExternalCalendarNotFound,
    CalendarAlreadySynced,
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for AddSyncCalendarUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    const NAME: &'static str = "AddSyncCalendar";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        // Check that user has integrated to that provider
        ctx.repos
            .user_integrations
            .find(&self.user.id)
            .await
            .map_err(|_| UseCaseErrors::StorageError)?
            .into_iter()
            .find(|i| i.provider == self.provider)
            .ok_or(UseCaseErrors::NoProviderIntegration)?;

        // Check if calendar sync already exists
        if ctx
            .repos
            .calendar_synced
            .find_by_calendar(&self.calendar_id)
            .await
            .map_err(|_| UseCaseErrors::StorageError)?
            .into_iter()
            .find(|c| c.provider == self.provider && c.ext_calendar_id == self.ext_calendar_id)
            .is_some()
        {
            return Err(UseCaseErrors::CalendarAlreadySynced);
        }

        // Check that user has write access to the given external calendar.
        match self.provider {
            IntegrationProvider::Google => {
                let google_provider = GoogleCalendarProvider::new(&self.user, ctx)
                    .await
                    .map_err(|_| UseCaseErrors::StorageError)?;
                let google_calendars = google_provider
                    .list(GoogleCalendarAccessRole::Writer)
                    .await
                    .map_err(|_| UseCaseErrors::StorageError)?;

                if google_calendars
                    .items
                    .into_iter()
                    .map(|c| c.id)
                    .find(|google_calendar_id| google_calendar_id == &self.ext_calendar_id)
                    .is_none()
                {
                    return Err(UseCaseErrors::ExternalCalendarNotFound);
                }
            }
            IntegrationProvider::Outlook => {
                let outlook_provider = OutlookCalendarProvider::new(&self.user, ctx)
                    .await
                    .map_err(|_| UseCaseErrors::StorageError)?;
                let outlook_calendars = outlook_provider
                    .list(OutlookCalendarAccessRole::Writer)
                    .await
                    .map_err(|_| UseCaseErrors::StorageError)?;

                if outlook_calendars
                    .value
                    .into_iter()
                    .map(|c| c.id)
                    .find(|outlook_calendar_id| outlook_calendar_id == &self.ext_calendar_id)
                    .is_none()
                {
                    return Err(UseCaseErrors::ExternalCalendarNotFound);
                }
            }
        }

        let synced_calendar = SyncedCalendar {
            calendar_id: self.calendar_id.clone(),
            ext_calendar_id: self.ext_calendar_id.clone(),
            provider: self.provider.clone(),
            user_id: self.user.id.clone(),
        };

        ctx.repos
            .calendar_synced
            .insert(&synced_calendar)
            .await
            .map_err(|_| UseCaseErrors::StorageError)
    }
}

impl PermissionBoundary for AddSyncCalendarUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::UpdateCalendar]
    }
}
