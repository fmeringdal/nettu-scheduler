use crate::shared::{
    auth::{
        account_can_modify_event, account_can_modify_user, protect_account_route, protect_route,
        Permission,
    },
    usecase::{execute_with_policy, PermissionBoundary, UseCaseErrorContainer},
};
use crate::{
    error::NettuError,
    shared::usecase::{execute, UseCase},
};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::delete_event::*;
use nettu_scheduler_domain::{CalendarEvent, IntegrationProvider, User, ID};
use nettu_scheduler_infra::{
    google_calendar::GoogleCalendarProvider, outlook_calendar::OutlookCalendarProvider,
    NettuContext,
};
use tracing::error;

fn handle_error(e: UseCaseErrors) -> NettuError {
    match e {
        UseCaseErrors::NotFound(event_id) => NettuError::NotFound(format!(
            "The calendar event with id: {}, was not found.",
            event_id
        )),
    }
}

pub async fn delete_event_admin_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let e = account_can_modify_event(&account, &path_params.event_id, &ctx).await?;
    let user = account_can_modify_user(&account, &e.user_id, &ctx).await?;

    let usecase = DeleteEventUseCase {
        user,
        event_id: e.id,
    };

    execute(usecase, &ctx)
        .await
        .map(|event| HttpResponse::Ok().json(APIResponse::new(event)))
        .map_err(handle_error)
}

pub async fn delete_event_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = DeleteEventUseCase {
        user,
        event_id: path_params.event_id.clone(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|event| HttpResponse::Ok().json(APIResponse::new(event)))
        .map_err(|e| match e {
            UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
            UseCaseErrorContainer::UseCase(e) => handle_error(e),
        })
}

#[derive(Debug)]
pub struct DeleteEventUseCase {
    pub user: User,
    pub event_id: ID,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFound(ID),
}

#[async_trait::async_trait(?Send)]
impl UseCase for DeleteEventUseCase {
    type Response = CalendarEvent;

    type Errors = UseCaseErrors;

    const NAME: &'static str = "DeleteEvent";

    // TODO: use only one db call
    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let e = match ctx.repos.events.find(&self.event_id).await {
            Some(e) if e.user_id == self.user.id => e,
            _ => return Err(UseCaseErrors::NotFound(self.event_id.clone())),
        };

        self.delete_synced_events(&e, ctx).await;

        ctx.repos.events.delete(&e.id).await;

        Ok(e)
    }
}

impl PermissionBoundary for DeleteEventUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::DeleteCalendarEvent]
    }
}

impl DeleteEventUseCase {
    pub async fn delete_synced_events(&self, e: &CalendarEvent, ctx: &NettuContext) {
        let synced_events = match ctx.repos.event_synced.find_by_event(&e.id).await {
            Ok(synced_events) => synced_events,
            Err(e) => {
                error!("Unable to query synced events from repo: {:?}", e);
                return;
            }
        };

        let synced_outlook_events = synced_events
            .iter()
            .filter(|o_event| o_event.provider == IntegrationProvider::Outlook)
            .collect::<Vec<_>>();
        let synced_google_events = synced_events
            .iter()
            .filter(|g_event| g_event.provider == IntegrationProvider::Google)
            .collect::<Vec<_>>();

        if synced_google_events.is_empty() && synced_outlook_events.is_empty() {
            return;
        }

        let user = match ctx.repos.users.find(&e.user_id).await {
            Some(u) => u,
            None => {
                error!("Unable to find user when deleting sync events");
                return;
            }
        };

        if !synced_outlook_events.is_empty() {
            let provider = match OutlookCalendarProvider::new(&user, ctx).await {
                Ok(p) => p,
                Err(_) => {
                    error!("Unable to create outlook calendar provider");
                    return;
                }
            };
            for cal in synced_outlook_events {
                if provider
                    .delete_event(cal.ext_calendar_id.clone(), cal.ext_event_id.clone())
                    .await
                    .is_err()
                {
                    error!("Unable to delete external outlook calendar event");
                };
            }
        }

        if !synced_google_events.is_empty() {
            let provider = match GoogleCalendarProvider::new(&user, ctx).await {
                Ok(p) => p,
                Err(_) => {
                    error!("Unable to create google calendar provider");
                    return;
                }
            };
            for cal in synced_google_events {
                if provider
                    .delete_event(cal.ext_calendar_id.clone(), cal.ext_event_id.clone())
                    .await
                    .is_err()
                {
                    error!("Unable to delete google external calendar event");
                };
            }
        }
    }
}
