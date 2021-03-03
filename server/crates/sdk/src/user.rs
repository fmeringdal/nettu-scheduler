use crate::{APIResponse, BaseClient};
use add_user_to_service::PathParams;
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

    pub async fn get(&self, user_id: String) -> APIResponse<get_user::APIResponse> {
        self.base
            .get(format!("user/{}", user_id), StatusCode::OK)
            .await
    }

    pub async fn delete(&self, user_id: String) -> APIResponse<delete_user::APIResponse> {
        self.base
            .delete(format!("user/{}", user_id), StatusCode::OK)
            .await
    }
}
