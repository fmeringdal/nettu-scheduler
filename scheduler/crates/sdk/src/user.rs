use crate::{shared::MetadataFindInput, APIResponse, BaseClient, ID};
use nettu_scheduler_api_structs::*;
use nettu_scheduler_domain::{IntegrationProvider, Metadata};
use reqwest::StatusCode;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserClient {
    base: Arc<BaseClient>,
}
pub struct UpdateUserInput {
    pub user_id: ID,
    pub metadata: Option<Metadata>,
}

pub type CreateUserInput = create_user::RequestBody;

impl UserClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn create(&self, input: CreateUserInput) -> APIResponse<create_user::APIResponse> {
        self.base
            .post(input, "user".into(), StatusCode::CREATED)
            .await
    }

    pub async fn get(&self, user_id: ID) -> APIResponse<get_user::APIResponse> {
        self.base
            .get(format!("user/{}", user_id), StatusCode::OK)
            .await
    }

    pub async fn delete(&self, user_id: ID) -> APIResponse<delete_user::APIResponse> {
        self.base
            .delete(format!("user/{}", user_id), StatusCode::OK)
            .await
    }

    pub async fn update(&self, input: UpdateUserInput) -> APIResponse<update_user::APIResponse> {
        let body = update_user::RequestBody {
            metadata: input.metadata,
        };
        self.base
            .put(body, format!("user/{}", input.user_id), StatusCode::OK)
            .await
    }

    pub async fn oauth(
        &self,
        user_id: ID,
        code: String,
        provider: IntegrationProvider,
    ) -> APIResponse<oauth_integration::APIResponse> {
        let body = oauth_integration::RequestBody { code, provider };
        self.base
            .post(
                body,
                format!("user/{}/oauth", user_id.to_string()),
                StatusCode::OK,
            )
            .await
    }

    pub async fn get_by_meta(
        &self,
        input: MetadataFindInput,
    ) -> APIResponse<get_users_by_meta::APIResponse> {
        self.base
            .get(
                format!("user/meta?{}", input.to_query_string()),
                StatusCode::OK,
            )
            .await
    }
}
