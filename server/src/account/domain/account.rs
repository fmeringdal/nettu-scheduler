use crate::shared::entity::Entity;

#[derive(Debug, Clone)]
pub struct Account {
    pub id: String,
    pub public_key_b64: String,
}

impl Entity for Account {
    fn id(&self) -> String {
        self.id.clone()
    }
}
