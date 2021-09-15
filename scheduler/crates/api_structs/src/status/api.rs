use serde::{Deserialize, Serialize};

pub mod get_service_health {
    use super::*;

    #[derive(Deserialize, Serialize)]
    pub struct APIResponse {
        pub message: String,
    }
}
