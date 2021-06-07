use crate::shared::{
    auth::{
        account_can_modify_event, account_can_modify_user, protect_account_route, protect_route,
        Permission,
    },
    usecase::{execute_with_policy, PermissionBoundary, Subscriber, UseCaseErrorContainer},
};
use crate::{
    error::NettuError,
    shared::usecase::{execute, UseCase},
};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::delete_event::*;
use nettu_scheduler_domain::{CalendarEvent, SyncedCalendarProvider, User, ID};
use nettu_scheduler_infra::{google_calendar::GoogleCalendarProvider, NettuContext};

use super::subscribers::DeleteRemindersOnEventDeleted;

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

        let synced_google_events = e
            .synced_events
            .iter()
            .filter(|synced_event| synced_event.provider == SyncedCalendarProvider::Google)
            .collect::<Vec<_>>();
        if !synced_google_events.is_empty() {
            if let Ok(provider) = GoogleCalendarProvider::new(&mut self.user, ctx).await {
                for synced_google_event in synced_google_events {
                    let _ = provider
                        .delete_event(
                            synced_google_event.calendar_id.clone(),
                            synced_google_event.event_id.clone(),
                        )
                        .await;
                }
            }
        }

        ctx.repos.events.delete(&e.id).await;

        Ok(e)
    }

    fn subscribers() -> Vec<Box<dyn Subscriber<Self>>> {
        vec![Box::new(DeleteRemindersOnEventDeleted)]
    }
}

impl PermissionBoundary for DeleteEventUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::DeleteCalendarEvent]
    }
}
