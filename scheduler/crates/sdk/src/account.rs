use crate::{APIResponse, BaseClient};
use nettu_scheduler_api_structs::*;
use reqwest::StatusCode;
use std::sync::Arc;

#[derive(Clone)]
pub struct AccountClient {
    base: Arc<BaseClient>,
}

impl AccountClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn get(&self) -> APIResponse<get_account::APIResponse> {
        self.base.get("account".into(), StatusCode::OK).await
    }

    pub async fn create(&self, code: &str) -> APIResponse<create_account::APIResponse> {
        let body = create_account::RequestBody { code: code.into() };
        self.base
            .post(body, "account".into(), StatusCode::CREATED)
            .await
    }
}
