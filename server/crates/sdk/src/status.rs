use crate::{APIResponse, BaseClient};
use actix_web::http::StatusCode;
use nettu_scheduler_api_structs::api::*;
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
