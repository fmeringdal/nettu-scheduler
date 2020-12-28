use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct Calendar {
    pub id: String,
    pub user_id: String,
}
