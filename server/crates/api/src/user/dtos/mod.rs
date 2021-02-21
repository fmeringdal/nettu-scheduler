use nettu_scheduler_core::User;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDTO {
    id: String,
    account_id: String,
}

impl UserDTO {
    pub fn new(user: &User) -> Self {
        Self {
            id: user.external_id.clone(),
            account_id: user.account_id.clone(),
        }
    }
}
