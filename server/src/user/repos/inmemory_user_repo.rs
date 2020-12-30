use super::IUserRepo;
use crate::user::domain::User;
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
        let mut users = self.users.lock().unwrap();
        users.push(user.clone());
        Ok(())
    }

    async fn save(&self, user: &User) -> Result<(), Box<dyn Error>> {
        let mut users = self.users.lock().unwrap();
        for i in 0..users.len() {
            if users[i].id == user.id {
                users.splice(i..i + 1, vec![user.clone()]);
            }
        }
        Ok(())
    }

    async fn find(&self, user_id: &str) -> Option<User> {
        let users = self.users.lock().unwrap();
        for i in 0..users.len() {
            if users[i].id == user_id {
                return Some(users[i].clone());
            }
        }
        None
    }

    async fn delete(&self, user_id: &str) -> Option<User> {
        let mut users = self.users.lock().unwrap();
        for i in 0..users.len() {
            if users[i].id == user_id {
                let deleted_user = users.remove(i);
                return Some(deleted_user);
            }
        }
        None
    }
}
