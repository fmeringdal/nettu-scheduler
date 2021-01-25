use crate::{api::Context, shared::auth::protect_route};
use crate::{
    api::NettuError,
    shared::usecase::{execute, Usecase},
};
use actix_web::{web, HttpResponse};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct DeleteCalendarReq {
    calendar_id: String,
}

pub async fn delete_calendar_controller(
    http_req: web::HttpRequest,
    req: web::Path<DeleteCalendarReq>,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let user = protect_route(&http_req, &ctx).await?;

    let usecase = DeleteCalendarUseCase {
        user_id: user.id,
        calendar_id: req.calendar_id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|_| HttpResponse::Ok().body("Calendar deleted"))
        .map_err(|e| match e {
            UseCaseErrors::NotFoundError => NettuError::NotFound(format!(
                "The calendar with id: {}, was not found.",
                req.calendar_id
            )),
            UseCaseErrors::UnableToDelete => NettuError::InternalError,
        })
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFoundError,
    UnableToDelete,
}

pub struct DeleteCalendarUseCase {
    calendar_id: String,
    user_id: String,
}

#[async_trait::async_trait(?Send)]
impl Usecase for DeleteCalendarUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let calendar = ctx.repos.calendar_repo.find(&self.calendar_id).await;
        match calendar {
            Some(calendar) if calendar.user_id == self.user_id => {
                ctx.repos.calendar_repo.delete(&calendar.id).await;
                let repo_res = ctx.repos.event_repo.delete_by_calendar(&calendar.id).await;
                if repo_res.is_err() {
                    return Err(UseCaseErrors::UnableToDelete);
                }
                let repo_res = ctx
                    .repos
                    .service_repo
                    .remove_calendar_from_services(&calendar.id)
                    .await;
                if repo_res.is_err() {
                    return Err(UseCaseErrors::UnableToDelete);
                }

                Ok(())
            }
            _ => Err(UseCaseErrors::NotFoundError),
        }
    }
}
