use mongodb::bson::oid::ObjectId;

use crate::shared::entity::Entity;

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub account_id: String,
}

impl User {
    pub fn new(account_id: &str) -> Self {
        Self {
            id: ObjectId::new().to_string(),
            account_id: String::from(account_id),
        }
    }
}

impl Entity for User {
    fn id(&self) -> String {
        self.id.clone()
    }
}
