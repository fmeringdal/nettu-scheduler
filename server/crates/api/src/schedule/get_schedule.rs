use crate::shared::usecase::{execute, UseCase};
use crate::{error::NettuError, shared::auth::protect_route};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::get_schedule::*;
use nettu_scheduler_core::Schedule;
use nettu_scheduler_infra::NettuContext;

pub async fn get_schedule_controller(
    http_req: HttpRequest,
    req: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, _policy) = protect_route(&http_req, &ctx).await?;

    let usecase = GetScheduleUseCase {
        user_id: user.id.clone(),
        schedule_id: req.schedule_id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|schedule| HttpResponse::Ok().json(APIResponse::new(schedule)))
        .map_err(|e| match e {
            UseCaseErrors::NotFound => NettuError::NotFound(format!(
                "The schedule with id: {}, was not found.",
                req.schedule_id
            )),
        })
}

#[derive(Debug)]
struct GetScheduleUseCase {
    pub user_id: String,
    pub schedule_id: String,
}

#[derive(Debug)]
enum UseCaseErrors {
    NotFound,
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetScheduleUseCase {
    type Response = Schedule;

    type Errors = UseCaseErrors;

    type Context = NettuContext;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let cal = ctx.repos.schedule_repo.find(&self.schedule_id).await;
        match cal {
            Some(cal) if cal.user_id == self.user_id => Ok(cal),
            _ => Err(UseCaseErrors::NotFound),
        }
    }
}
