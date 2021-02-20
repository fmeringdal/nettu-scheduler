use super::IUserRepo;
use crate::shared::inmemory_repo::*;
use nettu_scheduler_core::domain::User;
use std::error::Error;

pub struct InMemoryUserRepo {
    users: std::sync::Mutex<Vec<User>>,
}

impl InMemoryUserRepo {
    pub fn new() -> Self {
        Self {
            users: std::sync::Mutex::new(vec![]),
        }
    }
}

#[async_trait::async_trait]
impl IUserRepo for InMemoryUserRepo {
    async fn insert(&self, user: &User) -> Result<(), Box<dyn Error>> {
        insert(user, &self.users);
        Ok(())
    }

    async fn save(&self, user: &User) -> Result<(), Box<dyn Error>> {
        save(user, &self.users);
        Ok(())
    }

    async fn delete(&self, user_id: &str) -> Option<User> {
        delete(user_id, &self.users)
    }

    async fn find(&self, user_id: &str) -> Option<User> {
        find(user_id, &self.users)
    }
}
