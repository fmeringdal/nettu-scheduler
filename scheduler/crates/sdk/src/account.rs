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

    pub async fn create_webhook(&self, url: &str) -> APIResponse<set_account_webhook::APIResponse> {
        let body = set_account_webhook::RequestBody {
            webhook_url: url.into(),
        };
        self.base
            .put(body, "account/webhook".into(), StatusCode::OK)
            .await
    }

    pub async fn delete_webhook(&self) -> APIResponse<delete_account_webhook::APIResponse> {
        self.base
            .delete("account/webhook".into(), StatusCode::OK)
            .await
    }

    pub async fn set_account_pub_key(
        &self,
        key: Option<String>,
    ) -> APIResponse<set_account_pub_key::APIResponse> {
        let body = set_account_pub_key::RequestBody {
            public_jwt_key: key,
        };
        self.base
            .put(body, "account/pubkey".into(), StatusCode::OK)
            .await
    }
}
