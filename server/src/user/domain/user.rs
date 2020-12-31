use crate::shared::entity::Entity;

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
}

impl Entity for User {
    fn id(&self) -> String {
        self.id.clone()
    }
}
