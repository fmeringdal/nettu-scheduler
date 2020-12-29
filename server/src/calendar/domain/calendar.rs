use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Calendar {
    pub id: String,
    pub user_id: String,
}
