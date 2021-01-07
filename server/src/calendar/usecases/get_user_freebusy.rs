use crate::{api::NettuError, event::domain::event_instance::EventInstance};
use crate::{api::Context, calendar::domain::calendar_view::CalendarView};
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
}

pub async fn get_user_freebusy_controller(
    http_req: HttpRequest,
    query_params: web::Query<UserFreebusyQuery>,
    params: web::Path<UserPathParams>,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let account = ensure_nettu_acct_header(&http_req)?;

    let calendar_ids = match &query_params.calendar_ids {
        Some(calendar_ids) => Some(calendar_ids.split(',').map(String::from).collect()),
        None => None,
    };

    let usecase = GetUserFreeBusyUseCase {
        user_id: User::create_id(&account, &params.external_user_id),
        calendar_ids,
        start_ts: query_params.start_ts,
        end_ts: query_params.end_ts,
    };
    
    execute(usecase, &ctx).await
        .map(|usecase_res| HttpResponse::Ok().json(usecase_res))
        .map_err(|e| match e {
            GetUserFreeBusyErrors::InvalidTimespanError => {
                NettuError::BadClientData("The provided start_ts and end_ts is invalid".into())
            }
        })
}

pub struct GetUserFreeBusyUseCase {
    pub user_id: String,
    pub calendar_ids: Option<Vec<String>>,
    pub start_ts: i64,
    pub end_ts: i64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetUserFreeBusyResponse {
    pub free: Vec<EventInstance>,
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

        // can probably make query to event repo instead
        let mut calendars = ctx.repos.calendar_repo.find_by_user(&self.user_id).await;

        if let Some(calendar_ids) = &self.calendar_ids {
            if !calendar_ids.is_empty() {
                calendars = calendars
                    .into_iter()
                    .filter(|cal| calendar_ids.contains(&cal.id))
                    .collect();
            }
        }

        let all_events_futures = calendars.iter().map(|calendar| {
            ctx.repos
                .event_repo
                .find_by_calendar(&calendar.id, Some(&view))
        });
        let mut all_events_instances = join_all(all_events_futures)
            .await
            .into_iter()
            .map(|events_res| events_res.unwrap_or_default())
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

        let freebusy = get_free_busy(&mut all_events_instances);

        Ok(GetUserFreeBusyResponse { free: freebusy })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        calendar::domain::calendar::Calendar,
        event::domain::event::{CalendarEvent, RRuleFrequenzy, RRuleOptions},
        shared::entity::Entity,
        user::domain::User,
    };

    #[actix_web::main]
    #[test]
    async fn freebusy_works() {
        let ctx = Context::create_inmemory();
        let user = User::new("yoyoyo", "cool");

        let calendar = Calendar {
            id: String::from("312312"),
            user_id: user.id(),
        };
        ctx.repos.calendar_repo.insert(&calendar).await.unwrap();
        let one_hour = 1000 * 60 * 60;
        let mut e1 = CalendarEvent {
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            busy: false,
            duration: one_hour,
            end_ts: CalendarEvent::get_max_timestamp(),
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
            freq: RRuleFrequenzy::Daily,
            interval: 1,
            tzid: String::from("UTC"),
            until: None,
            wkst: 0,
        };
        e1.set_reccurrence(e1rr, true).unwrap();

        let mut e2 = CalendarEvent {
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            busy: false,
            duration: one_hour,
            end_ts: CalendarEvent::get_max_timestamp(),
            exdates: vec![],
            id: String::from("2"),
            start_ts: one_hour * 4,
            recurrence: None,
        };
        let e2rr = RRuleOptions {
            bynweekday: Default::default(),
            bysetpos: Default::default(),
            byweekday: Default::default(),
            count: Some(100),
            freq: RRuleFrequenzy::Daily,
            interval: 1,
            tzid: String::from("UTC"),
            until: None,
            wkst: 0,
        };
        e2.set_reccurrence(e2rr, true).unwrap();

        let mut e3 = CalendarEvent {
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            busy: false,
            duration: one_hour,
            end_ts: one_hour,
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
            freq: RRuleFrequenzy::Daily,
            interval: 2,
            tzid: String::from("UTC"),
            until: None,
            wkst: 0,
        };
        e3.set_reccurrence(e3rr, true).unwrap();

        ctx.repos.event_repo.insert(&e1).await.unwrap();
        ctx.repos.event_repo.insert(&e2).await.unwrap();
        ctx.repos.event_repo.insert(&e3).await.unwrap();

        let mut usecase = GetUserFreeBusyUseCase {
            user_id: user.id(),
            calendar_ids: Some(vec![calendar.id.clone()]),
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
