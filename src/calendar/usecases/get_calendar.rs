use crate::calendar::domain::calendar::Calendar;
use crate::calendar::repo::ICalendarRepo;
use crate::shared::usecase::UseCase;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
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
impl UseCase<GetCalendarReq, Result<Calendar, GetCalendarErrors>> for GetCalendarUseCase {
    async fn execute(&self, req: GetCalendarReq) -> Result<Calendar, GetCalendarErrors> {
        let cal = self.calendar_repo.find(&req.calendar_id).await;
        match cal {
            Some(cal) => Ok(cal),
            None => Err(GetCalendarErrors::NotFoundError {}),
        }
    }
}
