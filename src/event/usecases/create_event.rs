use serde::{Serialize, Deserialize};
use std::error::Error;
use crate::shared::usecase::UseCase;
use crate::event::domain::event::{RRuleOptions, CalendarEvent};
use std::sync::Arc;
use async_trait::async_trait;
use crate::event::repo::IEventRepo;

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
            start_ts: event.start_ts,
            duration: event.duration,
            recurrence: None,
            end_ts: None,
            exdates: vec![],
            calendar_id: String::from("1"),
            user_id: String::from("2"),
        };
        if let Some(rrule_opts) = event.rrule_options.clone() {
            e.set_reccurrence(rrule_opts);
        };
        self.event_repo.insert(&e).await;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct MockEventRepo {}

    #[async_trait]
    impl IEventRepo for MockEventRepo {
        async fn insert(&self, e: &CalendarEvent) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        async fn find(&self, event_id: &str) -> Option<CalendarEvent> {
            None
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
