use crate::event::domain::event::Calendar;
use crate::event::repo::ICalendarRepo;
use crate::shared::usecase::UseCase;
use crate::shared::errors::NotFoundError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct DeleteCalendarReq {
    pub calendar_id: String,
}

pub struct DeleteCalendarUseCase {
    pub calendar_repo: Arc<dyn ICalendarRepo>,
    pub event_repo: Arc<dyn IEventRepo>,
}

pub enum DeleteCalendarErrors {
    NotFoundError,
}

#[async_trait(?Send)]
impl UseCase<DeleteCalendarReq, Result<(), DeleteCalendarErrors>> for DeleteCalendarUseCase {
    async fn execute(&self, event_delete_req: DeleteCalendarReq) -> Result<(), DeleteCalendarErrors> {
        let calendar = self.calendar_repo.find(&event_delete_req.calendar_id).await;
        match e {
            Some(calendar) => {
                self.calendar_repo.delete(&calendar.id).await;
                self.event_repo.delete_by_calendar(&calendar.id).await;
                Ok(())
            }
            None => Err(DeleteCalendarErrors::NotFoundError {}),
        }
    }
}
