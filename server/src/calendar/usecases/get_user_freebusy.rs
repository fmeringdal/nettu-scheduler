use std::collections::HashMap;

use crate::{
    api::Context,
    calendar::domain::{Calendar, CalendarView},
};
use crate::{api::NettuError, event::domain::event_instance::EventInstance};
use crate::{
    event::domain::event_instance::get_free_busy,
    shared::usecase::{execute, Usecase},
};
use crate::{shared::auth::ensure_nettu_acct_header, user::domain::User};
use actix_web::{web, HttpRequest, HttpResponse};
use futures::future::join_all;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct UserPathParams {
    external_user_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserFreebusyQuery {
    start_ts: i64,
    end_ts: i64,
    calendar_ids: Option<String>,
    schedule_ids: Option<String>,
}

pub async fn get_user_freebusy_controller(
    http_req: HttpRequest,
    query_params: web::Query<UserFreebusyQuery>,
    params: web::Path<UserPathParams>,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let account = ensure_nettu_acct_header(&http_req)?;

    let calendar_ids = match &query_params.calendar_ids {
        Some(ids) => Some(ids.split(',').map(String::from).collect()),
        None => None,
    };
    let schedule_ids = match &query_params.schedule_ids {
        Some(ids) => Some(ids.split(',').map(String::from).collect()),
        None => None,
    };

    let usecase = GetUserFreeBusyUseCase {
        user_id: User::create_id(&account, &params.external_user_id),
        calendar_ids,
        schedule_ids,
        start_ts: query_params.start_ts,
        end_ts: query_params.end_ts,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(usecase_res))
        .map_err(|e| match e {
            GetUserFreeBusyErrors::InvalidTimespanError => {
                NettuError::BadClientData("The provided start_ts and end_ts is invalid".into())
            }
        })
}

#[derive(Debug)]
pub struct GetUserFreeBusyUseCase {
    pub user_id: String,
    pub calendar_ids: Option<Vec<String>>,
    pub schedule_ids: Option<Vec<String>>,
    pub start_ts: i64,
    pub end_ts: i64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetUserFreeBusyResponse {
    pub free: Vec<EventInstance>,
    pub user_id: String
}

#[derive(Debug)]
pub enum GetUserFreeBusyErrors {
    InvalidTimespanError,
}

#[async_trait::async_trait(?Send)]
impl Usecase for GetUserFreeBusyUseCase {
    type Response = GetUserFreeBusyResponse;

    type Errors = GetUserFreeBusyErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let view = match CalendarView::create(self.start_ts, self.end_ts) {
            Ok(view) => view,
            Err(_) => return Err(GetUserFreeBusyErrors::InvalidTimespanError),
        };

        let mut all_event_instances = vec![
            self.get_event_instances_from_calendars(&view, ctx).await,
            self.get_event_instances_from_schedules(&view, ctx).await,
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let freebusy = get_free_busy(&mut all_event_instances);

        Ok(GetUserFreeBusyResponse { free: freebusy, user_id: self.user_id.to_owned() })
    }
}

impl GetUserFreeBusyUseCase {
    async fn get_event_instances_from_calendars(
        &self,
        view: &CalendarView,
        ctx: &Context,
    ) -> Vec<EventInstance> {
        let calendar_ids = match &self.calendar_ids {
            Some(ids) if !ids.is_empty() => ids,
            _ => return vec![],
        };

        // can probably make query to event repo instead
        let mut calendars = ctx.repos.calendar_repo.find_by_user(&self.user_id).await;

        if !calendar_ids.is_empty() {
            calendars = calendars
                .into_iter()
                .filter(|cal| calendar_ids.contains(&cal.id))
                .collect();
        }

        let calendars_lookup: HashMap<_, _> = calendars.iter().map(|cal| (&cal.id, cal)).collect();

        let all_events_futures = calendars.iter().map(|calendar| {
            ctx.repos
                .event_repo
                .find_by_calendar(&calendar.id, Some(&view))
        });

        let all_events_instances = join_all(all_events_futures)
            .await
            .into_iter()
            .map(|events_res| events_res.unwrap_or_default())
            .map(|events| {
                events
                    .into_iter()
                    .map(|event| {
                        let calendar = calendars_lookup.get(&event.calendar_id).unwrap();
                        event.expand(Some(&view), &calendar.settings)
                    })
                    // It is possible that there are no instances in the expanded event, should remove them
                    .filter(|instances| !instances.is_empty())
            })
            .flatten()
            .flatten()
            .collect::<Vec<_>>();

        all_events_instances
    }

    async fn get_event_instances_from_schedules(
        &self,
        view: &CalendarView,
        ctx: &Context,
    ) -> Vec<EventInstance> {
        let schedule_ids = match &self.schedule_ids {
            Some(ids) if !ids.is_empty() => ids,
            _ => return vec![],
        };

        // can probably make query to event repo instead
        let mut schedules = ctx.repos.schedule_repo.find_by_user(&self.user_id).await;
        println!("Schedules found: {:?}", schedules);
        if !schedule_ids.is_empty() {
            schedules = schedules
                .into_iter()
                .filter(|cal| schedule_ids.contains(&cal.id))
                .collect();
        }

        schedules
            .iter()
            .map(|schedule| schedule.freebusy(view))
            .flatten()
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        calendar::domain::Calendar,
        event::domain::event::{CalendarEvent, RRuleFrequenzy, RRuleOptions},
        shared::entity::Entity,
        user::domain::User,
    };

    #[actix_web::main]
    #[test]
    async fn freebusy_works() {
        let ctx = Context::create_inmemory();
        let user = User::new("yoyoyo", "cool");

        let calendar = Calendar::new(&user.id());

        ctx.repos.calendar_repo.insert(&calendar).await.unwrap();
        let one_hour = 1000 * 60 * 60;
        let mut e1 = CalendarEvent {
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: user.account_id.clone(),
            busy: false,
            duration: one_hour,
            end_ts: CalendarEvent::get_max_timestamp(),
            exdates: vec![],
            id: String::from("1"),
            start_ts: 0,
            recurrence: None,
            reminder: None,
        };
        let e1rr = RRuleOptions {
            bynweekday: Default::default(),
            bysetpos: Default::default(),
            byweekday: Default::default(),
            count: Some(100),
            freq: RRuleFrequenzy::Daily,
            interval: 1,
            until: None,
        };
        e1.set_recurrence(e1rr, &calendar.settings, true);

        let mut e2 = CalendarEvent {
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: user.account_id.clone(),
            busy: false,
            duration: one_hour,
            end_ts: CalendarEvent::get_max_timestamp(),
            exdates: vec![],
            id: String::from("2"),
            start_ts: one_hour * 4,
            recurrence: None,
            reminder: None,
        };
        let e2rr = RRuleOptions {
            bynweekday: Default::default(),
            bysetpos: Default::default(),
            byweekday: Default::default(),
            count: Some(100),
            freq: RRuleFrequenzy::Daily,
            interval: 1,
            until: None,
        };
        e2.set_recurrence(e2rr, &calendar.settings, true);

        let mut e3 = CalendarEvent {
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: user.account_id.clone(),
            busy: false,
            duration: one_hour,
            end_ts: one_hour,
            exdates: vec![],
            id: String::from("3"),
            start_ts: 0,
            recurrence: None,
            reminder: None,
        };
        let e3rr = RRuleOptions {
            bynweekday: Default::default(),
            bysetpos: Default::default(),
            byweekday: Default::default(),
            count: Some(100),
            freq: RRuleFrequenzy::Daily,
            interval: 2,
            until: None,
        };
        e3.set_recurrence(e3rr, &calendar.settings, true);

        ctx.repos.event_repo.insert(&e1).await.unwrap();
        ctx.repos.event_repo.insert(&e2).await.unwrap();
        ctx.repos.event_repo.insert(&e3).await.unwrap();

        let mut usecase = GetUserFreeBusyUseCase {
            user_id: user.id(),
            calendar_ids: Some(vec![calendar.id.clone()]),
            schedule_ids: None,
            start_ts: 86400000,
            end_ts: 172800000,
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());
        let instances = res.unwrap().free;
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
