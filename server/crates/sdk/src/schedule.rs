use crate::{APIResponse, BaseClient};
use nettu_scheduler_api_structs::*;
use nettu_scheduler_domain::ID;
use reqwest::StatusCode;
use std::sync::Arc;

#[derive(Clone)]
pub struct ScheduleClient {
    base: Arc<BaseClient>,
}

impl ScheduleClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn create(
        &self,
        input: CreateScheduleInput,
    ) -> APIResponse<create_schedule::APIResponse> {
        let body = create_schedule::RequstBody {
            timezone: input.timezone,
        };
        let path = create_schedule::AdminPathParams {
            user_id: input.user_id,
        };

        self.base
            .post(
                body,
                format!("user/{}/schedule", path.user_id),
                StatusCode::CREATED,
            )
            .await
    }
}

pub struct CreateScheduleInput {
    pub timezone: String,
    pub user_id: ID,
}
