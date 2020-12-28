use crate::{api::Context, event::repo::IEventRepo};
use crate::{
    calendar::repo::ICalendarRepo,
    event::domain::event::{CalendarEvent, RRuleOptions},
};
use actix_web::{web, HttpResponse};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct CreateEventReq {
    calendar_id: String,
    start_ts: i64,
    duration: i64,
    busy: Option<bool>,
    rrule_options: Option<RRuleOptions>,
}

pub async fn create_event_controller(
    req: web::Json<CreateEventReq>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let ctx = CreateEventUseCaseCtx {
        event_repo: ctx.repos.event_repo.clone(),
        calendar_repo: ctx.repos.calendar_repo.clone(),
    };
    let res = create_event_usecase(req.0, ctx).await;
    match res {
        Ok(e) => HttpResponse::Created().json(e),
        Err(_e) => HttpResponse::UnprocessableEntity().finish(),
    }
}

pub struct CreateEventUseCaseCtx {
    pub event_repo: Arc<dyn IEventRepo>,
    pub calendar_repo: Arc<dyn ICalendarRepo>,
}

pub enum CreateCalendarEventErrors {
    NotFoundError,
}

async fn create_event_usecase(
    event: CreateEventReq,
    ctx: CreateEventUseCaseCtx,
) -> Result<CalendarEvent, CreateCalendarEventErrors> {
    let calendar = ctx.calendar_repo.find(&event.calendar_id).await;
    if calendar.is_none() {
        return Err(CreateCalendarEventErrors::NotFoundError);
    }
    let calendar = calendar.unwrap();

    let mut e = CalendarEvent {
        id: ObjectId::new().to_string(),
        busy: event.busy.unwrap_or(false),
        start_ts: event.start_ts,
        duration: event.duration,
        recurrence: None,
        end_ts: None,
        exdates: vec![],
        calendar_id: calendar.id,
        user_id: calendar.user_id,
    };
    if let Some(rrule_opts) = event.rrule_options.clone() {
        e.set_reccurrence(rrule_opts, true);
    };
    ctx.event_repo.insert(&e).await;
    Ok(e)
}

#[cfg(test)]
mod test {
    use async_trait::async_trait;
    use mongodb::results::DeleteResult;
    use std::error::Error;

    use crate::{calendar::domain::calendar_view::CalendarView, shared::errors::NotFoundError};

    use super::*;

    struct MockEventRepo {}

    #[async_trait]
    impl IEventRepo for MockEventRepo {
        async fn insert(&self, _e: &CalendarEvent) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
        async fn save(&self, _e: &CalendarEvent) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
        async fn find(&self, _event_id: &str) -> Option<CalendarEvent> {
            None
        }
        async fn find_by_calendar(
            &self,
            _calendar_id: &str,
            _cal_view: Option<&CalendarView>,
        ) -> Result<Vec<CalendarEvent>, Box<dyn Error>> {
            Ok(vec![])
        }
        async fn delete(&self, _event_id: &str) -> Option<CalendarEvent> {
            None
        }
        async fn delete_by_calendar(
            &self,
            _event_id: &str,
        ) -> Result<DeleteResult, Box<dyn Error>> {
            Err(Box::new(NotFoundError))
        }
    }

    #[actix_web::main]
    #[test]
    #[ignore = "calendar repo mock"]
    async fn create_event_use_case_test() {
        // let use_case = CreateEventUseCase {
        //     event_repo: Arc::new(MockEventRepo {}),
        // };
        // let res = use_case
        //     .execute(CreateEventReq {
        //         start_ts: 500,
        //         duration: 800,
        //         rrule_options: None,
        //         busy: Some(false),
        //         calendar_id: String::from("1231"),
        //     })
        //     .await;
        // assert!(res.is_ok());
    }
}
