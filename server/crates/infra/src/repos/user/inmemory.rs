use super::IUserRepo;
use crate::repos::shared::inmemory_repo::*;
use nettu_scheduler_domain::User;
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
    async fn insert(&self, user: &User) -> anyhow::Result<()> {
        insert(user, &self.users);
        Ok(())
    }

    async fn save(&self, user: &User) -> anyhow::Result<()> {
        save(user, &self.users);
        Ok(())
    }

    async fn delete(&self, user_id: &str) -> Option<User> {
        delete(user_id, &self.users)
    }

    async fn find(&self, user_id: &str) -> Option<User> {
        find(user_id, &self.users)
    }

    async fn find_by_account_id(&self, user_id: &str, account_id: &str) -> Option<User> {
        let mut user = find_by(&self.users, |u| {
            u.id == user_id && u.account_id == account_id
        });
        if user.is_empty() {
            return None;
        }
        Some(user.remove(0))
    }
}
