use crate::shared::{
    auth::{account_can_modify_event, protect_account_route, protect_route, Permission},
    usecase::{execute_with_policy, PermissionBoundary, UseCaseErrorContainer},
};
use crate::{
    error::NettuError,
    shared::usecase::{execute, UseCase},
};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::delete_event::*;
use nettu_scheduler_domain::{CalendarEvent, ID};
use nettu_scheduler_infra::NettuContext;

use super::sync_event_reminders::{
    EventOperation, SyncEventRemindersTrigger, SyncEventRemindersUseCase,
};

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

    let usecase = DeleteEventUseCase {
        user_id: e.user_id,
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
        user_id: user.id.clone(),
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
    pub user_id: ID,
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

    // TODO: use only one db call
    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let e = ctx.repos.event_repo.find(&self.event_id).await;
        match e {
            Some(event) if event.user_id == self.user_id => {
                ctx.repos.event_repo.delete(&event.id).await;

                let sync_event_reminders = SyncEventRemindersUseCase {
                    request: SyncEventRemindersTrigger::EventModified(
                        &event,
                        EventOperation::Deleted,
                    ),
                };

                // Sideeffect, ignore result
                let _ = execute(sync_event_reminders, ctx).await;

                Ok(event)
            }
            _ => Err(UseCaseErrors::NotFound(self.event_id.clone())),
        }
    }
}

impl PermissionBoundary for DeleteEventUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::DeleteCalendarEvent]
    }
}
