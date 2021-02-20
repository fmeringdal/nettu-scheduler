use crate::{
    api::Context,
    shared::{
        auth::{protect_route, Permission},
        usecase::{execute_with_policy, PermissionBoundary, UseCaseErrorContainer},
    },
};
use crate::{
    api::NettuError,
    shared::usecase::{execute, UseCase},
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateEventExceptionPathParams {
    event_id: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventExceptionBody {
    exception_ts: i64,
}

pub async fn create_event_exception_controller(
    http_req: HttpRequest,
    path_params: web::Path<CreateEventExceptionPathParams>,
    body: web::Json<CreateEventExceptionBody>,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = CreateEventExceptionUseCase {
        event_id: path_params.event_id.clone(),
        exception_ts: body.exception_ts,
        user_id: user.id.clone(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|_| HttpResponse::Created().finish())
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
    type Response = ();

    type Errors = UseCaseErrors;

    type Context = Context;

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

        Ok(())
    }
}

impl PermissionBoundary for CreateEventExceptionUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::UpdateCalendarEvent]
    }
}
