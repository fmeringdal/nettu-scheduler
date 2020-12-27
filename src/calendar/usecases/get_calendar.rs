use crate::event::domain::event::{CalendarEvent, RRuleOptions};
use crate::event::repo::ICalendarRepo;
use crate::shared::usecase::UseCase;
use crate::shared::errors::NotFoundError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct GetCalendarReq {
    pub calendar_id: String,
}

pub struct GetCalendarUseCase {
    pub calendar_repo: Arc<dyn ICalendarRepo>,
}

pub enum GetCalendarErrors {
    NotFoundError,
}

#[async_trait(?Send)]
impl UseCase<GetCalendarReq, Result<CalendarEvent, GetCalendarErrors>> for GetCalendarUseCase {
    async fn execute(
        &self,
        req: GetCalendarReq,
    ) -> Result<CalendarEvent, GetCalendarErrors> {
        let cal = self.calendar_repo.find(&req.calendar_id).await;
        match cal {
            Some(cal) => Ok(cal),
            None => Err(GetCalendarErrors::NotFoundError {}),
        }
    }
}
