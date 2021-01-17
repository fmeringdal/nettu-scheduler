use mongodb::bson::oid::ObjectId;
use serde::Serialize;

use crate::shared::entity::Entity;

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Calendar {
    pub id: String,
    pub user_id: String,
    pub settings: CalendarSettings,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CalendarSettings {
    pub wkst: isize,
}

impl CalendarSettings {
    pub fn set_wkst(&mut self, wkst: isize) -> bool {
        if wkst >= 0 && wkst <= 6 {
            self.wkst = wkst;
            true
        } else {
            false
        }
    }
}

impl Calendar {
    pub fn new(user_id: &str) -> Self {
        Self {
            id: ObjectId::new().to_hex(),
            user_id: user_id.to_string(),
            settings: CalendarSettings { wkst: 0 },
        }
    }
}

impl Entity for Calendar {
    fn id(&self) -> String {
        self.id.clone()
    }
}
