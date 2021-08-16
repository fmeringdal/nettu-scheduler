use crate::shared::auth::{account_can_modify_calendar, protect_route};
use crate::shared::usecase::{execute, UseCase};
use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::get_calendar_events::{APIResponse, PathParams, QueryParams};
use nettu_scheduler_domain::{Calendar, EventWithInstances, TimeSpan, ID};
use nettu_scheduler_infra::NettuContext;

pub async fn get_calendar_events_admin_controller(
    http_req: web::HttpRequest,
    query_params: web::Query<QueryParams>,
    path: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let cal = account_can_modify_calendar(&account, &path.calendar_id, &ctx).await?;

    let usecase = GetCalendarEventsUseCase {
        user_id: cal.user_id,
        calendar_id: cal.id,
        start_ts: query_params.start_ts,
        end_ts: query_params.end_ts,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| {
            HttpResponse::Ok().json(APIResponse::new(usecase_res.calendar, usecase_res.events))
        })
        .map_err(NettuError::from)
}

pub async fn get_calendar_events_controller(
    http_req: HttpRequest,
    query_params: web::Query<QueryParams>,
    path: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, _policy) = protect_route(&http_req, &ctx).await?;

    let usecase = GetCalendarEventsUseCase {
        user_id: user.id,
        calendar_id: path.calendar_id.clone(),
        start_ts: query_params.start_ts,
        end_ts: query_params.end_ts,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| {
            HttpResponse::Ok().json(APIResponse::new(usecase_res.calendar, usecase_res.events))
        })
        .map_err(NettuError::from)
}
#[derive(Debug)]
pub struct GetCalendarEventsUseCase {
    pub calendar_id: ID,
    pub user_id: ID,
    pub start_ts: i64,
    pub end_ts: i64,
}

#[derive(Debug)]
pub struct UseCaseResponse {
    calendar: Calendar,
    events: Vec<EventWithInstances>,
}

#[derive(Debug)]
pub enum UseCaseError {
    NotFound(ID),
    InvalidTimespan,
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InvalidTimespan => {
                Self::BadClientData("The start and end timespan is invalid".into())
            }
            UseCaseError::NotFound(calendar_id) => Self::NotFound(format!(
                "The calendar with id: {}, was not found.",
                calendar_id
            )),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetCalendarEventsUseCase {
    type Response = UseCaseResponse;

    type Error = UseCaseError;

    const NAME: &'static str = "GetCalendarEvents";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let calendar = ctx.repos.calendars.find(&self.calendar_id).await;

        let timespan = TimeSpan::new(self.start_ts, self.end_ts);
        if timespan.greater_than(ctx.config.event_instances_query_duration_limit) {
            return Err(UseCaseError::InvalidTimespan);
        }

        match calendar {
            Some(calendar) if calendar.user_id == self.user_id => {
                let events = ctx
                    .repos
                    .events
                    .find_by_calendar(&calendar.id, Some(&timespan))
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|event| {
                        let instances = event.expand(Some(&timespan), &calendar.settings);
                        EventWithInstances { event, instances }
                    })
                    // Also it is possible that there are no instances in the expanded event, should remove them
                    .filter(|data| !data.instances.is_empty())
                    .collect();

                Ok(UseCaseResponse { calendar, events })
            }
            _ => Err(UseCaseError::NotFound(self.calendar_id.clone())),
        }
    }
}
