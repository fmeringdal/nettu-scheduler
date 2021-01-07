use crate::shared::auth::protect_route;
use crate::{
    api::Context,
    event::domain::event_instance::EventInstance,
    shared::usecase::{execute, Usecase},
};
use crate::{calendar::domain::calendar_view::CalendarView, event::domain::event::CalendarEvent};
use actix_web::{web, HttpRequest, HttpResponse};
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
    params: web::Path<EventPathParams>,
    query_params: web::Query<GetEventInstancesReqView>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let user = match protect_route(&http_req, &ctx).await {
        Ok(u) => u,
        Err(res) => return res,
    };

    let usecase = GetEventInstancesUseCase {
        user_id: user.id.clone(),
        event_id: params.event_id.clone(),
        view: query_params.0,
    };
    let res = execute(usecase, &ctx).await;

    match res {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => match e {
            UseCaseErrors::InvalidTimespanError => HttpResponse::UnprocessableEntity().finish(),
            UseCaseErrors::NotFoundError => HttpResponse::NotFound().finish(),
        },
    }
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
impl Usecase for GetEventInstancesUseCase {
    type Response = UseCaseResponse;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let e = ctx.repos.event_repo.find(&self.event_id).await;
        match e {
            Some(event) if self.user_id == event.user_id => {
                let view = CalendarView::create(self.view.start_ts, self.view.end_ts);
                if view.is_err() {
                    return Err(UseCaseErrors::InvalidTimespanError);
                }
                let instances = event.expand(Some(&view.unwrap()));
                Ok(UseCaseResponse { event, instances })
            }
            _ => Err(UseCaseErrors::NotFoundError {}),
        }
    }
}
