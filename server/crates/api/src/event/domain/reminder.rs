use crate::shared::entity::Entity;

#[derive(Debug, Clone)]
pub struct Reminder {
    pub id: String,
    pub event_id: String,
    pub account_id: String,
    pub remind_at: i64,
}

impl Entity for Reminder {
    fn id(&self) -> String {
        self.id.clone()
    }
}
