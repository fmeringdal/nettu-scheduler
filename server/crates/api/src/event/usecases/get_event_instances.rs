use crate::shared::usecase::{execute, UseCase};
use crate::{error::NettuError, shared::auth::protect_route};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_core::{CalendarEvent, CalendarView, EventInstance};
use nettu_scheduler_infra::Context;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct EventPathParams {
    event_id: String,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetEventInstancesReqView {
    start_ts: i64,
    end_ts: i64,
}

pub async fn get_event_instances_controller(
    http_req: HttpRequest,
    path_params: web::Path<EventPathParams>,
    query_params: web::Query<GetEventInstancesReqView>,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let (user, _policy) = protect_route(&http_req, &ctx).await?;

    let usecase = GetEventInstancesUseCase {
        user_id: user.id.clone(),
        event_id: path_params.event_id.clone(),
        view: query_params.0,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(usecase_res))
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

pub struct GetEventInstancesUseCase {
    pub user_id: String,
    pub event_id: String,
    pub view: GetEventInstancesReqView,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFoundError,
    InvalidTimespanError,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UseCaseResponse {
    pub event: CalendarEvent,
    pub instances: Vec<EventInstance>,
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetEventInstancesUseCase {
    type Response = UseCaseResponse;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let e = ctx.repos.event_repo.find(&self.event_id).await;
        match e {
            Some(event) if self.user_id == event.user_id => {
                let calendar = match ctx.repos.calendar_repo.find(&event.calendar_id).await {
                    Some(cal) => cal,
                    None => return Err(UseCaseErrors::NotFoundError {}),
                };

                let view = CalendarView::create(self.view.start_ts, self.view.end_ts);
                if view.is_err() {
                    return Err(UseCaseErrors::InvalidTimespanError);
                }
                let instances = event.expand(Some(&view.unwrap()), &calendar.settings);
                Ok(UseCaseResponse { event, instances })
            }
            _ => Err(UseCaseErrors::NotFoundError {}),
        }
    }
}
