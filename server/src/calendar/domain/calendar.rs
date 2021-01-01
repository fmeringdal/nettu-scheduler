use serde::Serialize;

use crate::{shared::entity::Entity, user::domain::User};

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Calendar {
    pub id: String,
    pub user_id: String,
}

impl Calendar {
    pub fn is_user_owner(&self, user: &User) -> bool {
        self.user_id == user.external_id()
    }
}

impl Entity for Calendar {
    fn id(&self) -> String {
        self.id.clone()
    }
}
