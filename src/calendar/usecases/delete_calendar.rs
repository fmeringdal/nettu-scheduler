use crate::calendar::repo::ICalendarRepo;
use crate::shared::errors::NotFoundError;
use crate::shared::usecase::UseCase;
use crate::{calendar::domain::calendar::Calendar, event::repo::IEventRepo};
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
    async fn execute(&self, req: DeleteCalendarReq) -> Result<(), DeleteCalendarErrors> {
        let calendar = self.calendar_repo.find(&req.calendar_id).await;
        match calendar {
            Some(calendar) => {
                self.calendar_repo.delete(&calendar.id).await;
                self.event_repo.delete_by_calendar(&calendar.id).await;
                Ok(())
            }
            None => Err(DeleteCalendarErrors::NotFoundError {}),
        }
    }
}
