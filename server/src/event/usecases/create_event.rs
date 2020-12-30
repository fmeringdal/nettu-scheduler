use crate::{
    api::Context,
    event::repos::IEventRepo,
    shared::auth::{protect_route, User},
};
use crate::{
    calendar::repos::ICalendarRepo,
    event::domain::event::{CalendarEvent, RRuleOptions},
};
use actix_web::{web, HttpRequest, HttpResponse};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    http_req: HttpRequest,
    req: web::Json<CreateEventReq>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    // println("Got here");
    let user = match protect_route(&http_req) {
        Ok(u) => u,
        Err(res) => return res,
    };

    let ctx = CreateEventUseCaseCtx {
        event_repo: ctx.repos.event_repo.clone(),
        calendar_repo: ctx.repos.calendar_repo.clone(),
    };

    let res = create_event_usecase(req.0, user, ctx).await;
    match res {
        Ok(e) => HttpResponse::Created().json(CreateEventRes { event_id: e.id }),
        Err(e) => match e {
            CreateCalendarEventErrors::NotFoundError => HttpResponse::NotFound().finish(),
            CreateCalendarEventErrors::StorageError => HttpResponse::InternalServerError().finish(),
        },
    }
}

pub struct CreateEventUseCaseCtx {
    pub event_repo: Arc<dyn IEventRepo>,
    pub calendar_repo: Arc<dyn ICalendarRepo>,
}
#[derive(Debug)]
pub enum CreateCalendarEventErrors {
    NotFoundError,
    StorageError,
}

async fn create_event_usecase(
    event: CreateEventReq,
    user: User,
    ctx: CreateEventUseCaseCtx,
) -> Result<CalendarEvent, CreateCalendarEventErrors> {
    let calendar = match ctx.calendar_repo.find(&event.calendar_id).await {
        Some(calendar) if calendar.user_id == user.id => calendar,
        _ => return Err(CreateCalendarEventErrors::NotFoundError),
    };

    let mut e = CalendarEvent {
        id: ObjectId::new().to_string(),
        busy: event.busy.unwrap_or(false),
        start_ts: event.start_ts,
        duration: event.duration,
        recurrence: None,
        end_ts: Some(event.start_ts + event.duration), // default, if recurrence changes, this will be updated
        exdates: vec![],
        calendar_id: calendar.id,
        user_id: user.id,
    };
    if let Some(rrule_opts) = event.rrule_options.clone() {
        e.set_reccurrence(rrule_opts, true);
    }
    let repo_res = ctx.event_repo.insert(&e).await;
    if repo_res.is_err() {
        return Err(CreateCalendarEventErrors::StorageError);
    }
    Ok(e)
}

#[cfg(test)]
mod test {
    use crate::{
        api::Repos,
        calendar::{domain::calendar::Calendar, repos::InMemoryCalendarRepo},
        event::repos::InMemoryEventRepo,
    };

    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::main]
    #[test]
    async fn create_event_use_case_test() {
        let event_repo = Arc::new(InMemoryEventRepo::new());
        let calendar_repo = Arc::new(InMemoryCalendarRepo::new());

        let user = User {
            id: String::from("2312312"),
        };

        let calendar = Calendar {
            id: String::from("312312"),
            user_id: user.id.clone(),
        };
        calendar_repo.insert(&calendar).await;

        let req = CreateEventReq {
            start_ts: 500,
            duration: 800,
            rrule_options: None,
            busy: Some(false),
            calendar_id: calendar.id.clone(),
        };

        let res = create_event_usecase(
            req,
            user,
            CreateEventUseCaseCtx {
                calendar_repo,
                event_repo,
            },
        )
        .await;

        assert!(res.is_ok());
    }
}
