use super::IServiceUserRepo;
use crate::repos::shared::{inmemory_repo::*, query_structs::MetadataFindQuery};
use nettu_scheduler_domain::{Service, ServiceResource, ID};

pub struct InMemoryServiceUserRepo {
    service_users: std::sync::Mutex<Vec<ServiceResource>>,
}

impl InMemoryServiceUserRepo {
    pub fn new() -> Self {
        Self {
            service_users: std::sync::Mutex::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl IServiceUserRepo for InMemoryServiceUserRepo {
    async fn insert(&self, user: &ServiceResource) -> anyhow::Result<()> {
        insert(user, &self.service_users);
        Ok(())
    }

    async fn save(&self, user: &ServiceResource) -> anyhow::Result<()> {
        save(user, &self.service_users);
        Ok(())
    }

    async fn find(&self, service_id: &ID, user_id: &ID) -> Option<ServiceResource> {
        let id = format!("{}#{}", service_id, user_id);
        find(&id, &self.service_users)
    }

    async fn delete(&self, service_id: &ID, user_id: &ID) -> anyhow::Result<()> {
        delete_by(&self.service_users, |u| {
            u.user_id == *user_id && u.service_id == *service_id
        });
        Ok(())
    }
}
