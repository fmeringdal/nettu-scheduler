use crate::shared::entity::Entity;

#[derive(Debug, Clone)]
pub struct Company {
    pub id: String
}

impl Entity for Company {
    fn id(&self) -> String {
        self.id.clone()
    }
}