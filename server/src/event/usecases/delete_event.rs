use crate::{api::Context, shared::auth::protect_route};
use crate::{
    api::NettuError,
    shared::usecase::{execute, UseCase},
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

use super::sync_event_reminders::{EventOperation, SyncEventRemindersUseCase};

#[derive(Deserialize)]
pub struct PathParams {
    event_id: String,
}

pub async fn delete_event_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let user = protect_route(&http_req, &ctx).await?;

    let usecase = DeleteEventUseCase {
        user_id: user.id.clone(),
        event_id: path_params.event_id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|_| HttpResponse::Ok().body("Event deleted"))
        .map_err(|e| match e {
            UseCaseErrors::NotFound => NettuError::NotFound(format!(
                "The event with id: {}, was not found",
                path_params.event_id
            )),
        })
}

pub struct DeleteEventUseCase {
    pub user_id: String,
    pub event_id: String,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFound,
}

#[async_trait::async_trait(?Send)]
impl UseCase for DeleteEventUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    type Context = Context;

    // TODO: use only one db call
    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let e = ctx.repos.event_repo.find(&self.event_id).await;
        match e {
            Some(event) if event.user_id == self.user_id => {
                ctx.repos.event_repo.delete(&event.id).await;

                let sync_event_reminders = SyncEventRemindersUseCase {
                    event: &event,
                    op: EventOperation::Deleted,
                };
                // TODO: handl err
                execute(sync_event_reminders, ctx).await;

                Ok(())
            }
            _ => Err(UseCaseErrors::NotFound),
        }
    }
}
