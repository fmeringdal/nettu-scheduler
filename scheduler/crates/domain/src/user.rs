use crate::{
    shared::entity::{Entity, ID},
    Meta, Metadata,
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

impl Entity<ID> for User {
    fn id(&self) -> ID {
        self.id.clone()
    }
}

impl Meta<ID> for User {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    fn account_id(&self) -> &ID {
        &self.account_id
    }
}
