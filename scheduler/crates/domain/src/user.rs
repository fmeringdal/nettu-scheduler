use crate::{
    shared::entity::{Entity, ID},
    Metadata,
};

#[derive(Debug, Clone)]
pub struct User {
    pub id: ID,
    pub account_id: ID,
    pub metadata: Metadata,
}

impl User {
    pub fn new(account_id: ID) -> Self {
        Self {
            id: Default::default(),
            account_id,
            metadata: Default::default(),
        }
    }
}

impl Entity for User {
    fn id(&self) -> &ID {
        &self.id
    }
}
