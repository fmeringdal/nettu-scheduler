use crate::shared::entity::{Entity, ID};

#[derive(Debug, Clone)]
pub struct User {
    pub id: ID,
    pub account_id: ID,
}

impl User {
    pub fn new(account_id: ID) -> Self {
        Self {
            id: Default::default(),
            account_id,
        }
    }
}

impl Entity for User {
    fn id(&self) -> &ID {
        &self.id
    }
}
