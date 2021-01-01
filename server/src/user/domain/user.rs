use crate::shared::entity::Entity;

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub external_id: String,
    pub account_id: String,
}

impl User {
    pub fn external_id(&self) -> String {
        Self::create_external_id(&self.account_id, &self.external_id)
    }

    // todo: make sure that no external id contains the seperator chat
    pub fn create_external_id(account_id: &str, external_id: &str) -> String {
        format!("{}#{}", account_id, external_id)
    }
}

impl Entity for User {
    fn id(&self) -> String {
        self.id.clone()
    }
}
