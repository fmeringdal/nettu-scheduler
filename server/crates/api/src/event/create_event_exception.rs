use crate::shared::{
    auth::{protect_route, Permission},
    usecase::{execute_with_policy, PermissionBoundary, UseCaseErrorContainer},
};
use crate::{error::NettuError, shared::usecase::UseCase};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::create_event_exception::*;
use nettu_scheduler_domain::CalendarEvent;
use nettu_scheduler_infra::NettuContext;

pub async fn create_event_exception_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = CreateEventExceptionUseCase {
        event_id: path_params.event_id.clone(),
        exception_ts: body.exception_ts,
        user_id: user.id.clone(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|event| HttpResponse::Created().json(APIResponse::new(event)))
        .map_err(|e| match e {
            UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
            UseCaseErrorContainer::UseCase(e) => match e {
                UseCaseErrors::NotFoundError => NettuError::NotFound(format!(
                    "The event with id: {}, was not found",
                    path_params.event_id
                )),
                UseCaseErrors::StorageError => NettuError::InternalError,
            },
        })
}

#[derive(Debug)]
pub struct CreateEventExceptionUseCase {
    event_id: String,
    exception_ts: i64,
    user_id: String,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFoundError,
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateEventExceptionUseCase {
    type Response = CalendarEvent;

    type Errors = UseCaseErrors;

    type Context = NettuContext;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let mut event = match ctx.repos.event_repo.find(&self.event_id).await {
            Some(event) if event.user_id == self.user_id => event,
            _ => return Err(UseCaseErrors::NotFoundError),
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
