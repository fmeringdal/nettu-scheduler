use crate::shared::entity::Entity;

#[derive(Debug, Clone)]
pub struct Company {
    pub id: String,
    pub public_key_b64: String,
}

impl Entity for Company {
    fn id(&self) -> String {
        self.id.clone()
    }
}
