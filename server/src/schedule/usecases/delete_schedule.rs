use crate::{api::Context, shared::auth::protect_route};
use crate::{
    api::NettuError,
    shared::usecase::{execute, UseCase},
};
use actix_web::{web, HttpResponse};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct DeleteScheduleReq {
    schedule_id: String,
}

pub async fn delete_schedule_controller(
    http_req: web::HttpRequest,
    req: web::Path<DeleteScheduleReq>,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let user = protect_route(&http_req, &ctx).await?;

    let usecase = DeleteScheduleUseCase {
        user_id: user.id,
        schedule_id: req.schedule_id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|_| HttpResponse::Ok().body("Schedule deleted"))
        .map_err(|e| match e {
            UseCaseErrors::NotFoundError => NettuError::NotFound(format!(
                "The schedule with id: {}, was not found.",
                req.schedule_id
            )),
            UseCaseErrors::UnableToDelete => NettuError::InternalError,
        })
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFoundError,
    UnableToDelete,
}

pub struct DeleteScheduleUseCase {
    schedule_id: String,
    user_id: String,
}

#[async_trait::async_trait(?Send)]
impl UseCase for DeleteScheduleUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let schedule = ctx.repos.schedule_repo.find(&self.schedule_id).await;
        match schedule {
            Some(schedule) if schedule.user_id == self.user_id => {
                let res = ctx.repos.schedule_repo.delete(&schedule.id).await;
                if res.is_none() {
                    return Err(UseCaseErrors::UnableToDelete);
                }
                let res = ctx
                    .repos
                    .service_repo
                    .remove_schedule_from_services(&schedule.id)
                    .await;
                if res.is_err() {
                    return Err(UseCaseErrors::UnableToDelete);
                }

                Ok(())
            }
            _ => Err(UseCaseErrors::NotFoundError),
        }
    }
}
