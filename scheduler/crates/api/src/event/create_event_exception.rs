use crate::shared::{
    auth::{account_can_modify_event, protect_account_route, protect_route, Permission},
    usecase::{execute, execute_with_policy, PermissionBoundary, UseCaseErrorContainer},
};
use crate::{error::NettuError, shared::usecase::UseCase};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::create_event_exception::*;
use nettu_scheduler_domain::{CalendarEvent, ID};
use nettu_scheduler_infra::NettuContext;

fn handle_error(e: UseCaseErrors) -> NettuError {
    match e {
        UseCaseErrors::NotFound(event_id) => NettuError::NotFound(format!(
            "The calendar event with id: {}, was not found.",
            event_id
        )),
        UseCaseErrors::StorageError => NettuError::InternalError,
    }
}

pub async fn create_event_exception_admin_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let e = account_can_modify_event(&account, &path_params.event_id, &ctx).await?;

    let usecase = CreateEventExceptionUseCase {
        user_id: e.user_id,
        event_id: e.id,
        exception_ts: body.exception_ts,
    };

    execute(usecase, &ctx)
        .await
        .map(|event| HttpResponse::Ok().json(APIResponse::new(event)))
        .map_err(handle_error)
}

pub async fn create_event_exception_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = CreateEventExceptionUseCase {
        event_id: path_params.event_id.clone(),
        user_id: user.id.clone(),
        exception_ts: body.exception_ts,
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|event| HttpResponse::Created().json(APIResponse::new(event)))
        .map_err(|e| match e {
            UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
            UseCaseErrorContainer::UseCase(e) => handle_error(e),
        })
}

#[derive(Debug)]
pub struct CreateEventExceptionUseCase {
    event_id: ID,
    exception_ts: i64,
    user_id: ID,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFound(ID),
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateEventExceptionUseCase {
    type Response = CalendarEvent;

    type Errors = UseCaseErrors;

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let mut event = match ctx.repos.event_repo.find(&self.event_id).await {
            Some(event) if event.user_id == self.user_id => event,
            _ => return Err(UseCaseErrors::NotFound(self.event_id.clone())),
        };

        event.exdates.push(self.exception_ts);

        let repo_res = ctx.repos.event_repo.save(&event).await;
        if repo_res.is_err() {
            return Err(UseCaseErrors::StorageError);
        }

        Ok(event)
    }
}

impl PermissionBoundary for CreateEventExceptionUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::UpdateCalendarEvent]
    }
}
