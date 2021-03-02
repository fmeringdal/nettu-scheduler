use crate::{APIResponse, BaseClient};
use nettu_scheduler_api_structs::*;
use reqwest::StatusCode;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserClient {
    base: Arc<BaseClient>,
}

impl UserClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn create(&self, user_id: String) -> APIResponse<create_user::APIResponse> {
        let body = create_user::RequestBody { user_id };

        self.base
            .post(body, "user".into(), StatusCode::CREATED)
            .await
    }
}
