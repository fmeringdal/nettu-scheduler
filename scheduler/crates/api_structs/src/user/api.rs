use nettu_scheduler_domain::{User, ID};
use serde::{Deserialize, Serialize};

use crate::dtos::UserDTO;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub user: UserDTO,
}

impl UserResponse {
    pub fn new(user: User) -> Self {
        Self {
            user: UserDTO::new(user),
        }
    }
}

pub mod get_me {
    use super::*;

    pub type APIResponse = UserResponse;
}

pub mod create_user {
    use nettu_scheduler_domain::Metadata;

    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        #[serde(default)]
        pub metadata: Option<Metadata>,
    }

    pub type APIResponse = UserResponse;
}

pub mod oauth_google {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub code: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    pub type APIResponse = UserResponse;
}

pub mod update_user {
    use super::*;
    use nettu_scheduler_domain::Metadata;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        #[serde(default)]
        pub metadata: Option<Metadata>,
    }

    #[derive(Debug, Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    pub type APIResponse = UserResponse;
}

pub mod delete_user {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    pub type APIResponse = UserResponse;
}

pub mod get_user {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    pub type APIResponse = UserResponse;
}

pub mod get_users_by_meta {
    use super::*;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub key: String,
        pub value: String,
        #[serde(default)]
        pub skip: Option<usize>,
        pub limit: Option<usize>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct APIResponse {
        pub users: Vec<UserDTO>,
    }

    impl APIResponse {
        pub fn new(users: Vec<User>) -> Self {
            Self {
                users: users.into_iter().map(UserDTO::new).collect(),
            }
        }
    }
}
