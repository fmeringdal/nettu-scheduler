use crate::event::domain::event_instance::EventInstance;
use crate::shared::auth::protect_route;
use crate::{
    event::domain::event::CalendarEvent,
    shared::usecase::{perform, Usecase},
};

use crate::{
    api::Context,
    calendar::domain::{calendar::Calendar, calendar_view::CalendarView},
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CalendarPathParams {
    calendar_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimespanParams {
    pub start_ts: i64,
    pub end_ts: i64,
}

pub async fn get_calendar_events_controller(
    http_req: HttpRequest,
    query_params: web::Query<TimespanParams>,
    params: web::Path<CalendarPathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let user = match protect_route(&http_req, &ctx).await {
        Ok(u) => u,
        Err(res) => return res,
    };

    let usecase = GetCalendarEventsUseCase {
        user_id: user.id,
        calendar_id: params.calendar_id.clone(),
        start_ts: query_params.start_ts,
        end_ts: query_params.end_ts,
    };
    let res = perform(usecase, &ctx).await;

    match res {
        Ok(calendar_events) => HttpResponse::Ok().json(calendar_events),
        Err(e) => match e {
            UseCaseErrors::InvalidTimespanError => HttpResponse::UnprocessableEntity().finish(),
            UseCaseErrors::NotFoundError => HttpResponse::NotFound().finish(),
        },
    }
}
pub struct GetCalendarEventsUseCase {
    pub calendar_id: String,
    pub user_id: String,
    pub start_ts: i64,
    pub end_ts: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UseCaseResponse {
    calendar: Calendar,
    events: Vec<EventWithInstances>,
}

#[derive(Serialize)]
struct EventWithInstances {
    pub event: CalendarEvent,
    pub instances: Vec<EventInstance>,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFoundError,
    InvalidTimespanError,
}

#[async_trait::async_trait(?Send)]
impl Usecase for GetCalendarEventsUseCase {
    type Response = UseCaseResponse;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn perform(&self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let calendar = ctx.repos.calendar_repo.find(&self.calendar_id).await;

        let view = CalendarView::create(self.start_ts, self.end_ts);
        if view.is_err() {
            return Err(UseCaseErrors::InvalidTimespanError);
        }
        let view = view.unwrap();

        match calendar {
            Some(calendar) if calendar.user_id == self.user_id => {
                let events = ctx
                    .repos
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

                Ok(UseCaseResponse { calendar, events })
            }
            _ => Err(UseCaseErrors::NotFoundError),
        }
    }
}
