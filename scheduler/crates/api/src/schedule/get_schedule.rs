use crate::shared::{
    auth::{account_can_modify_schedule, protect_account_route},
    usecase::{execute, UseCase},
};
use crate::{error::NettuError, shared::auth::protect_route};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::get_schedule::*;
use nettu_scheduler_domain::{Schedule, ID};
use nettu_scheduler_infra::NettuContext;

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
        .map_err(NettuError::from)
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
        .map_err(NettuError::from)
}

#[derive(Debug)]
struct GetScheduleUseCase {
    pub schedule_id: ID,
}

#[derive(Debug)]
enum UseCaseError {
    NotFound(ID),
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::NotFound(schedule_id) => Self::NotFound(format!(
                "The schedule with id: {}, was not found.",
                schedule_id
            )),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetScheduleUseCase {
    type Response = Schedule;

    type Error = UseCaseError;

    const NAME: &'static str = "GetSchedule";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let schedule = ctx.repos.schedules.find(&self.schedule_id).await;
        match schedule {
            Some(schedule) => Ok(schedule),
            _ => Err(UseCaseError::NotFound(self.schedule_id.clone())),
        }
    }
}
