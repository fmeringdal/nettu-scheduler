use crate::calendar::domain::calendar::Calendar;
use crate::calendar::repo::ICalendarRepo;
use crate::shared::usecase::UseCase;
use async_trait::async_trait;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct CreateCalendarReq {
    user_id: String,
}

pub struct CreateCalendarUseCase {
    pub calendar_repo: Arc<dyn ICalendarRepo>,
}

#[async_trait(?Send)]
impl UseCase<CreateCalendarReq, Result<(), Box<dyn Error>>> for CreateCalendarUseCase {
    async fn execute(&self, req: CreateCalendarReq) -> Result<(), Box<dyn Error>> {
        let calendar = Calendar {
            id: ObjectId::new().to_string(),
            user_id: req.user_id,
        };
        self.calendar_repo.insert(&calendar).await;
        Ok(())
    }
}
