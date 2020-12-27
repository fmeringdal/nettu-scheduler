use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Calendar {
    pub id: String,
    pub user_id: String,
}
