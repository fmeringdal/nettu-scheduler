use crate::{APIResponse, BaseClient};
use actix_web::http::StatusCode;
use nettu_scheduler_api_structs::api::*;
use std::{collections::HashMap, sync::Arc};

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
        let mut body = HashMap::new();
        body.insert("code", code);
        self.base
            .post(body, "account".into(), StatusCode::CREATED)
            .await
    }
}
