use crate::shared::{
    auth::{account_can_modify_event, protect_account_route},
    usecase::{execute, UseCase},
};
use crate::{error::NettuError, shared::auth::protect_route};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::get_event_instances::*;
use nettu_scheduler_domain::{CalendarEvent, EventInstance, TimeSpan, ID};
use nettu_scheduler_infra::NettuContext;

fn handle_error(e: UseCaseErrors) -> NettuError {
    match e {
        UseCaseErrors::InvalidTimespan => {
            NettuError::BadClientData("The provided start_ts and end_ts is invalid".into())
        }
        UseCaseErrors::NotFound(entity, event_id) => NettuError::NotFound(format!(
            "The {} with id: {}, was not found.",
            entity, event_id
        )),
    }
}

pub async fn get_event_instances_admin_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    query_params: web::Query<QueryParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let e = account_can_modify_event(&account, &path_params.event_id, &ctx).await?;

    let usecase = GetEventInstancesUseCase {
        user_id: e.user_id,
        event_id: e.id,
        timespan: query_params.0,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| {
            HttpResponse::Ok().json(APIResponse::new(usecase_res.event, usecase_res.instances))
        })
        .map_err(handle_error)
}

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
        .map_err(handle_error)
}

#[derive(Debug)]
pub struct GetEventInstancesUseCase {
    pub user_id: ID,
    pub event_id: ID,
    pub timespan: QueryParams,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFound(String, ID),
    InvalidTimespan,
}

#[derive(Debug)]
pub struct UseCaseResponse {
    pub event: CalendarEvent,
    pub instances: Vec<EventInstance>,
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetEventInstancesUseCase {
    type Response = UseCaseResponse;

    type Errors = UseCaseErrors;

    const NAME: &'static str = "GetEventInstances";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let e = ctx.repos.events.find(&self.event_id).await;
        match e {
            Some(event) if self.user_id == event.user_id => {
                let calendar = match ctx.repos.calendars.find(&event.calendar_id).await {
                    Some(cal) => cal,
                    None => {
                        return Err(UseCaseErrors::NotFound(
                            "Calendar".into(),
                            event.calendar_id,
                        ))
                    }
                };

                let timespan = TimeSpan::new(self.timespan.start_ts, self.timespan.end_ts);
                if timespan.greater_than(ctx.config.event_instances_query_duration_limit) {
                    return Err(UseCaseErrors::InvalidTimespan);
                }

                let instances = event.expand(Some(&timespan), &calendar.settings);
                Ok(UseCaseResponse { event, instances })
            }
            _ => Err(UseCaseErrors::NotFound(
                "Calendar Event".into(),
                self.event_id.clone(),
            )),
        }
    }
}
