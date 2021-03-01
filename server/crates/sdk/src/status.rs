use crate::{APIResponse, BaseClient};
use nettu_scheduler_api_structs::*;
use reqwest::StatusCode;
use std::sync::Arc;

#[derive(Clone)]
pub struct StatusClient {
    base: Arc<BaseClient>,
}

impl StatusClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn check_health(&self) -> APIResponse<get_service_health::APIResponse> {
        self.base.get("".into(), StatusCode::OK).await
    }
}
