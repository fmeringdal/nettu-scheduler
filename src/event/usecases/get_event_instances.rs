use crate::event::domain::event_instance::EventInstance;
use crate::event::repo::IEventRepo;
use crate::shared::errors::NotFoundError;
use crate::shared::usecase::UseCase;
use crate::{calendar::domain::calendar_view::CalendarView, event::domain::event::CalendarEvent};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct GetEventInstancesReq {
    pub event_id: String,
    pub view: GetEventInstancesReqView,
}

#[derive(Serialize, Deserialize)]
pub struct GetEventInstancesReqView {
    start_ts: i64,
    end_ts: i64,
}

pub struct GetEventInstancesUseCase {
    pub event_repo: Arc<dyn IEventRepo>,
}

pub enum GetEventInstancesErrors {
    NotFoundError,
    InvalidTimespanError,
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
        req: GetEventInstancesReq,
    ) -> Result<GetEventInstancesResponse, GetEventInstancesErrors> {
        let e = self.event_repo.find(&req.event_id).await;
        match e {
            Some(event) => {
                let view = CalendarView::create(req.view.start_ts, req.view.end_ts);
                if view.is_err() {
                    return Err(GetEventInstancesErrors::InvalidTimespanError);
                }
                let instances = event.expand(Some(&view.unwrap()));
                Ok(GetEventInstancesResponse { event, instances })
            }
            None => Err(GetEventInstancesErrors::NotFoundError {}),
        }
    }
}
