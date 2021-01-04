use crate::event::domain::event::{CalendarEvent, RRuleOptions};
use crate::{
    api::Context,
    shared::{
        auth::protect_route,
        usecase::{perform, Usecase},
    },
};
use actix_web::{web, HttpResponse};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventReq {
    calendar_id: String,
    start_ts: i64,
    duration: i64,
    busy: Option<bool>,
    rrule_options: Option<RRuleOptions>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventRes {
    event_id: String,
}

pub async fn create_event_controller(
    http_req: web::HttpRequest,
    req: web::Json<CreateEventReq>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let user = match protect_route(&http_req, &ctx).await {
        Ok(u) => u,
        Err(res) => return res,
    };

    let usecase = CreateEventUseCase {
        busy: req.busy,
        start_ts: req.start_ts,
        duration: req.duration,
        calendar_id: req.calendar_id.clone(),
        rrule_options: req.rrule_options.clone(),
        user_id: user.id.clone(),
    };

    let res = perform(usecase, &ctx).await;
    match res {
        Ok(e) => HttpResponse::Created().json(CreateEventRes { event_id: e.id }),
        Err(e) => match e {
            CreateCalendarEventErrors::NotFoundError => HttpResponse::NotFound().finish(),
            CreateCalendarEventErrors::StorageError => HttpResponse::InternalServerError().finish(),
        },
    }
}

struct CreateEventUseCase {
    calendar_id: String,
    user_id: String,
    start_ts: i64,
    duration: i64,
    busy: Option<bool>,
    rrule_options: Option<RRuleOptions>,
}

#[derive(Debug)]
pub enum CreateCalendarEventErrors {
    NotFoundError,
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl Usecase for CreateEventUseCase {
    type Response = CalendarEvent;

    type Errors = CreateCalendarEventErrors;

    type Context = Context;

    async fn perform(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let calendar = match ctx.repos.calendar_repo.find(&self.calendar_id).await {
            Some(calendar) if calendar.user_id == self.user_id => calendar,
            _ => return Err(CreateCalendarEventErrors::NotFoundError),
        };

        let mut e = CalendarEvent {
            id: ObjectId::new().to_string(),
            busy: self.busy.unwrap_or(false),
            start_ts: self.start_ts,
            duration: self.duration,
            recurrence: None,
            end_ts: Some(self.start_ts + self.duration), // default, if recurrence changes, this will be updated
            exdates: vec![],
            calendar_id: calendar.id,
            user_id: self.user_id.clone(),
        };
        if let Some(rrule_opts) = self.rrule_options.clone() {
            e.set_reccurrence(rrule_opts, true);
        }
        let repo_res = ctx.repos.event_repo.insert(&e).await;
        if repo_res.is_err() {
            return Err(CreateCalendarEventErrors::StorageError);
        }
        Ok(e)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{calendar::domain::calendar::Calendar, user::domain::User};

    #[actix_web::main]
    #[test]
    async fn create_event_use_case_test() {
        let ctx = Context::create_inmemory();
        let user = User::new("cool2", "cool");

        let calendar = Calendar {
            id: String::from("312312"),
            user_id: user.id.clone(),
        };
        ctx.repos.calendar_repo.insert(&calendar).await.unwrap();

        let mut usecase = CreateEventUseCase {
            start_ts: 500,
            duration: 800,
            rrule_options: None,
            busy: Some(false),
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
        };

        let res = usecase.perform(&ctx).await;

        assert!(res.is_ok());
    }
}
