use crate::{event::repos::IEventRepo, shared::auth::protect_route};
use crate::{api::Context, event::domain::event_instance::EventInstance};
use crate::{calendar::domain::calendar_view::CalendarView, event::domain::event::CalendarEvent};
use actix_web::{HttpRequest, HttpResponse, web};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct EventPathParams {
    event_id: String,
}

pub async fn get_event_instances_controller(
    http_req: HttpRequest,
    params: web::Path<EventPathParams>,
    query_params: web::Query<GetEventInstancesReqView>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let user = match protect_route(&http_req) {
        Ok(u) => u,
        Err(res) => return res
    };

    let req = GetEventInstancesReq {
        user_id: user.id.clone(),
        event_id: params.event_id.clone(),
        view: query_params.0,
    };
    let ctx = GetEventInstancesUseCaseCtx {
        event_repo: ctx.repos.event_repo.clone(),
    };
    let res = get_event_instances_usecase(req, ctx).await;

    match res {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => match e {
            GetEventInstancesErrors::InvalidTimespanError => {
                HttpResponse::UnprocessableEntity().finish()
            }
            GetEventInstancesErrors::NotFoundError => HttpResponse::NotFound().finish(),
        },
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetEventInstancesReq {
    pub user_id: String,
    pub event_id: String,
    pub view: GetEventInstancesReqView,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetEventInstancesReqView {
    start_ts: i64,
    end_ts: i64,
}

pub struct GetEventInstancesUseCaseCtx {
    pub event_repo: Arc<dyn IEventRepo>,
}

pub enum GetEventInstancesErrors {
    NotFoundError,
    InvalidTimespanError,
}

#[derive(Serialize)]
pub struct GetEventInstancesResponse {
    pub event: CalendarEvent,
    pub instances: Vec<EventInstance>,
}
async fn get_event_instances_usecase(
    req: GetEventInstancesReq,
    ctx: GetEventInstancesUseCaseCtx,
) -> Result<GetEventInstancesResponse, GetEventInstancesErrors> {
    let e = ctx.event_repo.find(&req.event_id).await;
    match e {
        Some(event) if req.user_id == event.user_id => {
            let view = CalendarView::create(req.view.start_ts, req.view.end_ts);
            if view.is_err() {
                return Err(GetEventInstancesErrors::InvalidTimespanError);
            }
            let instances = event.expand(Some(&view.unwrap()));
            Ok(GetEventInstancesResponse { event, instances })
        }
        _ => Err(GetEventInstancesErrors::NotFoundError {}),
    }
}
