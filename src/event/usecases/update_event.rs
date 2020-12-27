use crate::event::domain::event::{CalendarEvent, RRuleOptions};
use crate::event::repo::IEventRepo;
use crate::shared::usecase::UseCase;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct UpdateEventReq {
    pub event_id: String,
    pub start_ts: Option<i64>,
    pub duration: Option<i64>,
    pub rrule_options: Option<RRuleOptions>,
}

pub struct UpdateEventUseCase {
    pub event_repo: Arc<dyn IEventRepo>,
}
#[derive(Debug)]
struct NotFoundError;

impl Error for NotFoundError {}

impl std::fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Oh no, something bad went down")
    }
}

pub enum UpdateEventErrors {
    NotFoundError,
}

#[async_trait(?Send)]
impl UseCase<UpdateEventReq, Result<(), UpdateEventErrors>> for UpdateEventUseCase {
    async fn execute(&self, event_update_req: UpdateEventReq) -> Result<(), UpdateEventErrors> {
        let e = self.event_repo.find(&event_update_req.event_id).await;
        println!("Event found: {:?}", e);
        if e.is_none() {
            return Err(UpdateEventErrors::NotFoundError {});
        }
        let mut should_update_endtime = false;
        let mut e = e.unwrap();
        if let Some(start_ts) = event_update_req.start_ts {
            e.start_ts = start_ts;
            should_update_endtime = true;
        }
        if let Some(duration) = event_update_req.duration {
            e.duration = duration;
            should_update_endtime = true;
        }

        if let Some(rrule_opts) = event_update_req.rrule_options.clone() {
            e.set_reccurrence(rrule_opts, true);
        } else if should_update_endtime && e.recurrence.is_some() {
            e.set_reccurrence(e.recurrence.clone().unwrap(), true);
        }

        self.event_repo.save(&e).await;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use mongodb::results::DeleteResult;
    use std::error::Error;

    use super::*;

    struct MockEventRepo {}

    #[async_trait]
    impl IEventRepo for MockEventRepo {
        async fn insert(&self, e: &CalendarEvent) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        async fn save(&self, e: &CalendarEvent) -> Result<(), Box<dyn std::error::Error>> {
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
    async fn update_notexisting_event() {
        let use_case = UpdateEventUseCase {
            event_repo: Arc::new(MockEventRepo {}),
        };
        let res = use_case
            .execute(UpdateEventReq {
                event_id: String::from(""),
                start_ts: Some(500),
                duration: Some(800),
                rrule_options: None,
            })
            .await;
        assert!(res.is_err());
    }
}
