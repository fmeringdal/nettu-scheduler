use crate::event::domain::event::{CalendarEvent, RRuleOptions};
use crate::event::repo::IEventRepo;
use crate::shared::usecase::UseCase;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct GetEventReq {
    pub event_id: String,
}

pub struct GetEventUseCase {
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

pub enum GetEventErrors {
    NotFoundError
}

#[async_trait(?Send)]
impl UseCase<GetEventReq, Result<CalendarEvent, GetEventErrors>> for GetEventUseCase {
    async fn execute(&self, event_update_req: GetEventReq) -> Result<CalendarEvent, GetEventErrors> {
        let e = self.event_repo.find(&event_update_req.event_id).await;
        match e {
            Some(event) => {
                Ok(event)
            }
            None => Err(GetEventErrors::NotFoundError {})
        }
        
    }
}
