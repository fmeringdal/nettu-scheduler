use crate::event::domain::event::{CalendarEvent, RRuleOptions};
use crate::event::repo::IEventRepo;
use crate::shared::usecase::UseCase;
use async_trait::async_trait;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct CreateEventReq {
    start_ts: i64,
    duration: i64,
    rrule_options: Option<RRuleOptions>,
}

pub struct CreateEventUseCase {
    pub event_repo: Arc<dyn IEventRepo>,
}

#[async_trait(?Send)]
impl UseCase<CreateEventReq, Result<(), Box<dyn Error>>> for CreateEventUseCase {
    async fn execute(&self, event: CreateEventReq) -> Result<(), Box<dyn Error>> {
        let mut e = CalendarEvent {
            id: ObjectId::new().to_string(),
            start_ts: event.start_ts,
            duration: event.duration,
            recurrence: None,
            end_ts: None,
            exdates: vec![],
            calendar_id: String::from("1"),
            user_id: String::from("2"),
        };
        if let Some(rrule_opts) = event.rrule_options.clone() {
            e.set_reccurrence(rrule_opts, true);
        };
        self.event_repo.insert(&e).await;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use mongodb::results::DeleteResult;
    use std::error::Error;

    use crate::shared::errors::NotFoundError;

    use super::*;

    struct MockEventRepo {}

    #[async_trait]
    impl IEventRepo for MockEventRepo {
        async fn insert(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
        async fn save(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
        async fn find(&self, event_id: &str) -> Option<CalendarEvent> {
            None
        }
        async fn find_by_calendar(
            &self,
            calendar_id: &str,
        ) -> Result<Vec<CalendarEvent>, Box<dyn Error>> {
            Ok(vec![])
        }
        async fn delete(&self, event_id: &str) -> Option<CalendarEvent> {
            None
        }
        async fn delete_by_calendar(&self, event_id: &str) -> Result<DeleteResult, Box<dyn Error>> {
            Err(Box::new(NotFoundError))
        }
    }

    #[actix_web::main]
    #[test]
    async fn create_event_use_case_test() {
        let use_case = CreateEventUseCase {
            event_repo: Arc::new(MockEventRepo {}),
        };
        let res = use_case
            .execute(CreateEventReq {
                start_ts: 500,
                duration: 800,
                rrule_options: None,
            })
            .await;
        assert!(res.is_ok());
    }
}
