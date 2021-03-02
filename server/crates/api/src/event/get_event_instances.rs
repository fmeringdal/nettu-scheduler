use crate::shared::usecase::{execute, UseCase};
use crate::{error::NettuError, shared::auth::protect_route};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::get_event_instances::*;
use nettu_scheduler_domain::{CalendarEvent, EventInstance, TimeSpan};
use nettu_scheduler_infra::NettuContext;

pub async fn get_event_instances_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    query_params: web::Query<QueryParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, _policy) = protect_route(&http_req, &ctx).await?;

    let usecase = GetEventInstancesUseCase {
        user_id: user.id.clone(),
        event_id: path_params.event_id.clone(),
        timespan: query_params.0,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| {
            HttpResponse::Ok().json(APIResponse::new(usecase_res.event, usecase_res.instances))
        })
        .map_err(|e| match e {
            UseCaseErrors::InvalidTimespanError => {
                NettuError::BadClientData("The provided start_ts and end_ts is invalid".into())
            }
            UseCaseErrors::NotFoundError => NettuError::NotFound(format!(
                "The event with id: {}, was not found",
                path_params.event_id
            )),
        })
}

#[derive(Debug)]
pub struct GetEventInstancesUseCase {
    pub user_id: String,
    pub event_id: String,
    pub timespan: QueryParams,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFoundError,
    InvalidTimespanError,
}

pub struct UseCaseResponse {
    pub event: CalendarEvent,
    pub instances: Vec<EventInstance>,
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetEventInstancesUseCase {
    type Response = UseCaseResponse;

    type Errors = UseCaseErrors;

    type Context = NettuContext;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let e = ctx.repos.event_repo.find(&self.event_id).await;
        match e {
            Some(event) if self.user_id == event.user_id => {
                let calendar = match ctx.repos.calendar_repo.find(&event.calendar_id).await {
                    Some(cal) => cal,
                    None => return Err(UseCaseErrors::NotFoundError {}),
                };

                let timespan = TimeSpan::create(self.timespan.start_ts, self.timespan.end_ts);
                if timespan.is_err() {
                    return Err(UseCaseErrors::InvalidTimespanError);
                }
                let instances = event.expand(Some(&timespan.unwrap()), &calendar.settings);
                Ok(UseCaseResponse { event, instances })
            }
            _ => Err(UseCaseErrors::NotFoundError {}),
        }
    }
}
