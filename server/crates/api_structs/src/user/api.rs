use nettu_scheduler_domain::User;
use serde::{Deserialize, Serialize};

use crate::dtos::UserDTO;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    user: UserDTO,
}

impl UserResponse {
    pub fn new(user: User) -> Self {
        Self {
            user: UserDTO::new(&user),
        }
    }
}

pub mod create_user {
    use super::*;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub user_id: String,
    }

    pub type APIResponse = UserResponse;
}

pub mod delete_user {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: String,
    }

    pub type APIResponse = UserResponse;
}

pub mod get_user {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: String,
    }

    pub type APIResponse = UserResponse;
}
