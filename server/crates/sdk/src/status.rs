use crate::{APIResponse, BaseClient};
use actix_web::http::StatusCode;
use nettu_scheduler_api::dev::status::StatusResponse;
use std::sync::Arc;

pub struct StatusClient {
    base: Arc<BaseClient>,
}

impl StatusClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn check_health(&self) -> APIResponse<StatusResponse> {
        self.base.get("".into(), StatusCode::OK).await
    }
}
