use crate::calendar::repo::ICalendarRepo;
use crate::event::domain::event::CalendarEvent;
use crate::event::domain::event_instance::get_free_busy;
use crate::event::domain::event_instance::EventInstance;
use crate::event::repo::IEventRepo;
use crate::shared::usecase::UseCase;
use crate::{
    api::Context,
    calendar::domain::{calendar::Calendar, calendar_view::CalendarView},
};
use actix_web::{web, HttpResponse};
use async_trait::async_trait;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct UserPathParams {
    user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UserFreebusyBodyReq {
    start_ts: i64,
    end_ts: i64,
    calendar_ids: Option<Vec<String>>,
}

pub async fn get_user_freebusy_controller(
    body: web::Query<UserFreebusyBodyReq>,
    params: web::Path<UserPathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let req = GetUserFreeBusyReq {
        user_id: params.user_id.clone(),
        calendar_ids: body.calendar_ids.clone(),
        start_ts: body.start_ts,
        end_ts: body.end_ts,
    };
    let ctx = GetUserFreeBusyUseCaseCtx {
        event_repo: ctx.repos.event_repo.clone(),
        calendar_repo: ctx.repos.calendar_repo.clone(),
    };
    let res = get_user_freebusy_usecase(req, ctx).await;

    match res {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => match e {
            GetUserFreeBusyErrors::InvalidTimespanError => {
                HttpResponse::UnprocessableEntity().finish()
            }
        },
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetUserFreeBusyReq {
    pub user_id: String,
    pub calendar_ids: Option<Vec<String>>,
    pub start_ts: i64,
    pub end_ts: i64,
}

pub struct GetUserFreeBusyUseCaseCtx {
    pub event_repo: Arc<dyn IEventRepo>,
    pub calendar_repo: Arc<dyn ICalendarRepo>,
}

#[derive(Serialize)]
pub struct GetUserFreeBusyResponse {
    pub free: Vec<EventInstance>,
}

pub async fn get_user_freebusy_usecase(
    req: GetUserFreeBusyReq,
    ctx: GetUserFreeBusyUseCaseCtx,
) -> Result<GetUserFreeBusyResponse, GetUserFreeBusyErrors> {
    let view = CalendarView::create(req.start_ts, req.end_ts);
    if view.is_err() {
        return Err(GetUserFreeBusyErrors::InvalidTimespanError);
    }
    let view = view.unwrap();

    // can probably make query to event repo instead
    let mut calendars = ctx.calendar_repo.find_by_user(&req.user_id).await;
    if let Some(calendar_ids) = req.calendar_ids {
        calendars = calendars
            .into_iter()
            .filter(|cal| calendar_ids.contains(&cal.id))
            .collect();
    }

    let all_events_futures = calendars
        .iter()
        .map(|calendar| ctx.event_repo.find_by_calendar(&calendar.id, Some(&view)));
    let mut all_events_instances = join_all(all_events_futures)
        .await
        .into_iter()
        .map(|events_res| events_res.unwrap_or(vec![]))
        .map(|events| {
            events
                .into_iter()
                .map(|event| event.expand(Some(&view)))
                // It is possible that there are no instances in the expanded event, should remove them
                .filter(|instances| !instances.is_empty())
        })
        .flatten()
        .flatten()
        .collect::<Vec<_>>();
    // println!("All instances: {:?}", all_events_instances);
    let freebusy = get_free_busy(&mut all_events_instances);

    Ok(GetUserFreeBusyResponse { free: freebusy })
}

#[derive(Debug)]
pub enum GetUserFreeBusyErrors {
    InvalidTimespanError,
}

impl std::fmt::Display for GetUserFreeBusyErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            GetUserFreeBusyErrors::InvalidTimespanError => {
                write!(f, "The provided timesspan was invalid.")
            }
        }
    }
}
