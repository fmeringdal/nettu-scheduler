use crate::shared::entity::Entity;

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub external_id: String,
    pub account_id: String,
}

impl User {
    pub fn new(account_id: &str, external_id: &str) -> Self {
        Self {
            id: Self::create_id(account_id, external_id),
            account_id: String::from(account_id),
            external_id: String::from(external_id),
        }
    }

    // todo: make sure that no external id contains the seperator chat
    pub fn create_id(account_id: &str, external_id: &str) -> String {
        format!("{}#{}", account_id, external_id)
    }
}

impl Entity for User {
    fn id(&self) -> String {
        self.id.clone()
    }
}
