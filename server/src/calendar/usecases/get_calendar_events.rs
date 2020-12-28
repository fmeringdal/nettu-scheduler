use crate::calendar::repos::ICalendarRepo;
use crate::event::domain::event::CalendarEvent;
use crate::event::domain::event_instance::EventInstance;
use crate::event::repos::IEventRepo;

use crate::{
    api::Context,
    calendar::domain::{calendar::Calendar, calendar_view::CalendarView},
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct CalendarPathParams {
    calendar_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TimespanBodyReq {
    pub start_ts: i64,
    pub end_ts: i64,
}

pub async fn get_calendar_events_controller(
    query_params: web::Query<TimespanBodyReq>,
    params: web::Path<CalendarPathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let ctx = GetCalendarEventsUseCaseCtx {
        calendar_repo: ctx.repos.calendar_repo.clone(),
        event_repo: ctx.repos.event_repo.clone(),
    };
    let req = GetCalendarEventsReq {
        calendar_id: params.calendar_id.clone(),
        start_ts: query_params.start_ts,
        end_ts: query_params.end_ts,
    };
    let res = get_calendar_events_usecase(req, ctx).await;

    match res {
        Ok(calendar_events) => HttpResponse::Ok().json(calendar_events),
        Err(e) => match e {
            GetCalendarEventsErrors::InvalidTimespanError => {
                HttpResponse::UnprocessableEntity().finish()
            }
            GetCalendarEventsErrors::NotFoundError => HttpResponse::NotFound().finish(),
        },
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetCalendarEventsReq {
    pub calendar_id: String,
    pub start_ts: i64,
    pub end_ts: i64,
}

pub struct GetCalendarEventsUseCaseCtx {
    pub event_repo: Arc<dyn IEventRepo>,
    pub calendar_repo: Arc<dyn ICalendarRepo>,
}

#[derive(Serialize)]
pub struct GetCalendarEventsResponse {
    calendar: Calendar,
    events: Vec<EventWithInstances>,
}

#[derive(Serialize)]
struct EventWithInstances {
    pub event: CalendarEvent,
    pub instances: Vec<EventInstance>,
}

async fn get_calendar_events_usecase(
    req: GetCalendarEventsReq,
    ctx: GetCalendarEventsUseCaseCtx,
) -> Result<GetCalendarEventsResponse, GetCalendarEventsErrors> {
    let calendar = ctx.calendar_repo.find(&req.calendar_id).await;

    let view = CalendarView::create(req.start_ts, req.end_ts);
    if view.is_err() {
        return Err(GetCalendarEventsErrors::InvalidTimespanError);
    }
    let view = view.unwrap();

    match calendar {
        Some(calendar) => {
            let events = ctx
                .event_repo
                .find_by_calendar(&calendar.id, Some(&view))
                .await
                .unwrap()
                .into_iter()
                .map(|event| {
                    let instances = event.expand(Some(&view));
                    EventWithInstances { event, instances }
                })
                // Also it is possible that there are no instances in the expanded event, should remove them
                .filter(|data| !data.instances.is_empty())
                .collect();

            Ok(GetCalendarEventsResponse { calendar, events })
        }
        None => Err(GetCalendarEventsErrors::NotFoundError {}),
    }
}

// ERRORS

#[derive(Debug)]
pub enum GetCalendarEventsErrors {
    NotFoundError,
    InvalidTimespanError,
}

impl std::fmt::Display for GetCalendarEventsErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            GetCalendarEventsErrors::NotFoundError => write!(f, "The calendar was not found"),
            GetCalendarEventsErrors::InvalidTimespanError => {
                write!(f, "The provided timesspan was invalid.")
            }
        }
    }
}
