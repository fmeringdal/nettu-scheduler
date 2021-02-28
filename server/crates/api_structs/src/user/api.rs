use serde::{Deserialize, Serialize};

pub mod create_user {
    use super::*;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub user_id: String,
    }
}

pub mod delete_user {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: String,
    }
}

pub mod get_user {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: String,
    }
}
