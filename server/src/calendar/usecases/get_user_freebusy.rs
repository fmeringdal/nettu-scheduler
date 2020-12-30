use crate::calendar::repos::ICalendarRepo;

use crate::event::domain::event_instance::get_free_busy;
use crate::event::domain::event_instance::EventInstance;
use crate::event::repos::IEventRepo;

use crate::{api::Context, calendar::domain::calendar_view::CalendarView};
use actix_web::{web, HttpResponse};

use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct UserPathParams {
    user_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserFreebusyQuery {
    start_ts: i64,
    end_ts: i64,
    calendar_ids: Option<String>,
}

pub async fn get_user_freebusy_controller(
    query_params: web::Query<UserFreebusyQuery>,
    params: web::Path<UserPathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let calendar_ids = match &query_params.calendar_ids {
        Some(calendar_ids) => Some(calendar_ids.split(",").map(|s| String::from(s)).collect()),
        None => None,
    };

    let req = GetUserFreeBusyReq {
        user_id: params.user_id.clone(),
        calendar_ids,
        start_ts: query_params.start_ts,
        end_ts: query_params.end_ts,
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

#[derive(Debug)]
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

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetUserFreeBusyResponse {
    pub free: Vec<EventInstance>,
}

pub async fn get_user_freebusy_usecase(
    req: GetUserFreeBusyReq,
    ctx: GetUserFreeBusyUseCaseCtx,
) -> Result<GetUserFreeBusyResponse, GetUserFreeBusyErrors> {
    println!("req into freebusy: {:?}", req);
    let view = CalendarView::create(req.start_ts, req.end_ts);
    if view.is_err() {
        return Err(GetUserFreeBusyErrors::InvalidTimespanError);
    }
    let view = view.unwrap();
    println!("view freebusy: {:?}", view);

    // can probably make query to event repo instead
    let mut calendars = ctx.calendar_repo.find_by_user(&req.user_id).await;
    if let Some(calendar_ids) = req.calendar_ids {
        if !calendar_ids.is_empty() {
            calendars = calendars
                .into_iter()
                .filter(|cal| calendar_ids.contains(&cal.id))
                .collect();
        }
    }

    let all_events_futures = calendars
        .iter()
        .map(|calendar| ctx.event_repo.find_by_calendar(&calendar.id, Some(&view)));
    let mut all_events_instances = join_all(all_events_futures)
        .await
        .into_iter()
        // .map(|events_res| {
        //     println!("events_res: {:?}", events_res);
        //     events_res
        // })
        .map(|events_res| events_res.unwrap_or_default())
        .map(|events| {
            events
                .into_iter()
                .map(|event| {
                    if event.id == String::from("1") {
                        println!("Expanding event: {:?}", event);
                        println!("Expanding event in view: {:?}", view);
                        println!("With result: {:?}", event.expand(Some(&view)));
                    }
                    event
                })
                .map(|event| event.expand(Some(&view)))
                // It is possible that there are no instances in the expanded event, should remove them
                .filter(|instances| !instances.is_empty())
        })
        .flatten()
        .flatten()
        .collect::<Vec<_>>();

    println!("all fre: {:?}", all_events_instances);
    let freebusy = get_free_busy(&mut all_events_instances);

    println!("res from freebusy: {:?}", freebusy);
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        calendar::{domain::calendar::Calendar, repos::InMemoryCalendarRepo},
        event::{
            domain::event::{CalendarEvent, RRuleOptions},
            repos::InMemoryEventRepo,
        },
        shared::auth::User,
    };
    use std::sync::Arc;

    #[actix_web::main]
    #[test]
    async fn freebusy_works() {
        let event_repo = InMemoryEventRepo::new();
        let calendar_repo = InMemoryCalendarRepo::new();

        let user = User {
            id: String::from("2312312"),
        };

        let calendar = Calendar {
            id: String::from("312312"),
            user_id: user.id.clone(),
        };
        calendar_repo.insert(&calendar).await;

        let mut e1 = CalendarEvent {
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            busy: false,
            duration: 1000 * 60 * 60,
            end_ts: None,
            exdates: vec![],
            id: String::from("1"),
            start_ts: 0,
            recurrence: None,
        };
        let e1rr = RRuleOptions {
            bynweekday: Default::default(),
            bysetpos: Default::default(),
            byweekday: Default::default(),
            count: Some(100),
            freq: 4,
            interval: 1,
            tzid: String::from("UTC"),
            until: None,
            wkst: 0,
        };
        e1.set_reccurrence(e1rr, true);

        let mut e2 = CalendarEvent {
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            busy: false,
            duration: 1000 * 60 * 60,
            end_ts: None,
            exdates: vec![],
            id: String::from("2"),
            start_ts: 1000 * 60 * 60 * 4,
            recurrence: None,
        };
        let e2rr = RRuleOptions {
            bynweekday: Default::default(),
            bysetpos: Default::default(),
            byweekday: Default::default(),
            count: Some(100),
            freq: 4,
            interval: 1,
            tzid: String::from("UTC"),
            until: None,
            wkst: 0,
        };
        e2.set_reccurrence(e2rr, true);

        let mut e3 = CalendarEvent {
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            busy: false,
            duration: 1000 * 60 * 60,
            end_ts: None,
            exdates: vec![],
            id: String::from("3"),
            start_ts: 0,
            recurrence: None,
        };
        let e3rr = RRuleOptions {
            bynweekday: Default::default(),
            bysetpos: Default::default(),
            byweekday: Default::default(),
            count: Some(100),
            freq: 4,
            interval: 2,
            tzid: String::from("UTC"),
            until: None,
            wkst: 0,
        };
        e3.set_reccurrence(e3rr, true);

        event_repo.insert(&e1).await;
        event_repo.insert(&e2).await;
        event_repo.insert(&e3).await;

        let ctx = GetUserFreeBusyUseCaseCtx {
            event_repo: Arc::new(event_repo),
            calendar_repo: Arc::new(calendar_repo),
        };

        let req = GetUserFreeBusyReq {
            user_id: user.id.clone(),
            calendar_ids: Some(vec![calendar.id.clone()]),
            start_ts: 86400000,
            end_ts: 172800000,
        };

        let res = get_user_freebusy_usecase(req, ctx).await;
        assert!(res.is_ok());
        let instances = res.unwrap().free.clone();
        assert_eq!(instances.len(), 2);
        assert_eq!(
            instances[0],
            EventInstance {
                busy: false,
                start_ts: 86400000,
                end_ts: 90000000,
            }
        );
        assert_eq!(
            instances[1],
            EventInstance {
                busy: false,
                start_ts: 100800000,
                end_ts: 104400000,
            }
        );
    }
}
