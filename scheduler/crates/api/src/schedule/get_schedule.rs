use crate::shared::{
    auth::{account_can_modify_schedule, protect_account_route},
    usecase::{execute, UseCase},
};
use crate::{error::NettuError, shared::auth::protect_route};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::get_schedule::*;
use nettu_scheduler_domain::{Schedule, ID};
use nettu_scheduler_infra::NettuContext;

fn handle_error(e: UseCaseErrors) -> NettuError {
    match e {
        UseCaseErrors::NotFound(schedule_id) => NettuError::NotFound(format!(
            "The schedule with id: {}, was not found.",
            schedule_id
        )),
    }
}

pub async fn get_schedule_admin_controller(
    http_req: HttpRequest,
    path: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let schedule = account_can_modify_schedule(&account, &path.schedule_id, &ctx).await?;

    let usecase = GetScheduleUseCase {
        schedule_id: schedule.id,
    };

    execute(usecase, &ctx)
        .await
        .map(|schedule| HttpResponse::Ok().json(APIResponse::new(schedule)))
        .map_err(handle_error)
}

pub async fn get_schedule_controller(
    http_req: HttpRequest,
    req: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (_user, _policy) = protect_route(&http_req, &ctx).await?;

    let usecase = GetScheduleUseCase {
        schedule_id: req.schedule_id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|schedule| HttpResponse::Ok().json(APIResponse::new(schedule)))
        .map_err(handle_error)
}

#[derive(Debug)]
struct GetScheduleUseCase {
    pub schedule_id: ID,
}

#[derive(Debug)]
enum UseCaseErrors {
    NotFound(ID),
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetScheduleUseCase {
    type Response = Schedule;

    type Errors = UseCaseErrors;

    const NAME: &'static str = "GetSchedule";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let schedule = ctx.repos.schedules.find(&self.schedule_id).await;
        match schedule {
            Some(schedule) => Ok(schedule),
            _ => Err(UseCaseErrors::NotFound(self.schedule_id.clone())),
        }
    }
}
