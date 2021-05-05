use super::IKVRepo;
use crate::repos::shared::{inmemory_repo::*, query_structs::MetadataFindQuery};
use nettu_scheduler_domain::{User, ID};

pub struct InMemoryKVRepo {
    users: std::sync::Mutex<Vec<User>>,
}

impl InMemoryKVRepo {
    pub fn new() -> Self {
        Self {
            users: std::sync::Mutex::new(vec![]),
        }
    }
}

#[async_trait::async_trait]
impl IKVRepo for InMemoryKVRepo {
    async fn insert(&self, user: &User) -> anyhow::Result<()> {
        insert(user, &self.users);
        Ok(())
    }

    async fn save(&self, user: &User) -> anyhow::Result<()> {
        save(user, &self.users);
        Ok(())
    }

    async fn delete(&self, user_id: &ID) -> Option<User> {
        delete(user_id, &self.users)
    }

    async fn find(&self, user_id: &ID) -> Option<User> {
        find(user_id, &self.users)
    }

    /// Ignores skip and limit as this is just used for testing
    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<User> {
        find_by_metadata(&self.users, query)
    }

    async fn find_by_account_id(&self, user_id: &ID, account_id: &ID) -> Option<User> {
        let mut user = find_by(&self.users, |u| {
            u.id == *user_id && u.account_id == *account_id
        });
        if user.is_empty() {
            return None;
        }
        Some(user.remove(0))
    }
}
