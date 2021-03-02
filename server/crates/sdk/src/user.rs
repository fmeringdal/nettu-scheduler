use crate::{APIResponse, BaseClient};
use nettu_scheduler_api_structs::*;
use reqwest::StatusCode;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct UserClient {
    base: Arc<BaseClient>,
}

impl UserClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn create(&self) -> APIResponse<create_user::APIResponse> {
        let empty = HashMap::<String, String>::new();
        self.base
            .post(empty, "user".into(), StatusCode::CREATED)
            .await
    }
}
