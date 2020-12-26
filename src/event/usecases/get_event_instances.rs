use crate::event::domain::event::CalendarEvent;
use crate::event::domain::event_instance::EventInstance;
use crate::event::repo::IEventRepo;
use crate::shared::usecase::UseCase;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct GetEventInstancesReq {
    pub event_id: String,
}

pub struct GetEventInstancesUseCase {
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

pub enum GetEventInstancesErrors {
    NotFoundError,
}

#[derive(Serialize)]
pub struct GetEventInstancesResponse {
    pub event: CalendarEvent,
    pub instances: Vec<EventInstance>,
}

#[async_trait(?Send)]
impl UseCase<GetEventInstancesReq, Result<GetEventInstancesResponse, GetEventInstancesErrors>>
    for GetEventInstancesUseCase
{
    async fn execute(
        &self,
        event_update_req: GetEventInstancesReq,
    ) -> Result<GetEventInstancesResponse, GetEventInstancesErrors> {
        let e = self.event_repo.find(&event_update_req.event_id).await;
        match e {
            Some(event) => {
                let instances = event.expand();
                Ok(GetEventInstancesResponse { event, instances })
            }
            None => Err(GetEventInstancesErrors::NotFoundError {}),
        }
    }
}
