use nettu_scheduler_domain::{User, ID};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDTO {
    pub id: ID,
    pub account_id: ID,
}

impl UserDTO {
    pub fn new(user: User) -> Self {
        Self {
            id: user.id.clone(),
            account_id: user.account_id.clone(),
        }
    }
}
