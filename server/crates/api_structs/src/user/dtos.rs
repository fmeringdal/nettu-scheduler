use nettu_scheduler_domain::User;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDTO {
    pub id: String,
    pub account_id: String,
}

impl UserDTO {
    pub fn new(user: &User) -> Self {
        Self {
            id: user.external_id.clone(),
            account_id: user.account_id.clone(),
        }
    }
}
