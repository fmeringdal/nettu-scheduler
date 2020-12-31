use crate::shared::entity::Entity;

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub external_id: String,
    pub account_id: String,
}

impl Entity for User {
    fn id(&self) -> String {
        self.id.clone()
    }
}
