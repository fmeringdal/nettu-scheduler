use crate::{APIResponse, BaseClient};
use nettu_scheduler_api_structs::*;
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

    pub async fn create(&self) -> APIResponse<create_schedule::APIResponse> {
        let body = create_schedule::RequstBody {
            timezone: "UTC".into(),
        };
        let path = create_schedule::AdminPathParams {
            user_id: "1234213".into(),
        };

        self.base
            .post(
                body,
                format!("schedule/{}", path.user_id),
                StatusCode::CREATED,
            )
            .await
    }
}
