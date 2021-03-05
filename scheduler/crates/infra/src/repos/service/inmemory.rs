use super::IServiceRepo;
use crate::repos::shared::inmemory_repo::*;
use nettu_scheduler_domain::{Service, ID};

pub struct InMemoryServiceRepo {
    services: std::sync::Mutex<Vec<Service>>,
}

impl InMemoryServiceRepo {
    pub fn new() -> Self {
        Self {
            services: std::sync::Mutex::new(vec![]),
        }
    }
}

#[async_trait::async_trait]
impl IServiceRepo for InMemoryServiceRepo {
    async fn insert(&self, service: &Service) -> anyhow::Result<()> {
        insert(service, &self.services);
        Ok(())
    }

    async fn save(&self, service: &Service) -> anyhow::Result<()> {
        save(service, &self.services);
        Ok(())
    }

    async fn find(&self, service_id: &ID) -> Option<Service> {
        find(service_id, &self.services)
    }

    async fn delete(&self, service_id: &ID) -> Option<Service> {
        delete(service_id, &self.services)
    }

    async fn remove_calendar_from_services(&self, calendar_id: &ID) -> anyhow::Result<()> {
        update_many(
            &self.services,
            |service| {
                for user in &service.users {
                    if user.contains_calendar(&calendar_id) {
                        return true;
                    }
                }
                false
            },
            |service| {
                for user in &mut service.users {
                    user.remove_calendar(&calendar_id);
                }
            },
        );
        Ok(())
    }

    async fn remove_schedule_from_services(&self, schedule_id: &ID) -> anyhow::Result<()> {
        update_many(
            &self.services,
            |service| {
                for user in &service.users {
                    if user.contains_schedule(&schedule_id) {
                        return true;
                    }
                }
                false
            },
            |service| {
                for user in &mut service.users {
                    user.remove_schedule(&schedule_id);
                }
            },
        );
        Ok(())
    }

    async fn remove_user_from_services(&self, user_id: &ID) -> anyhow::Result<()> {
        update_many(
            &self.services,
            |service| {
                service
                    .users
                    .iter()
                    .find(|u| u.user_id == *user_id)
                    .is_some()
            },
            |service| {
                service.users.retain(|u| u.user_id != *user_id);
            },
        );
        Ok(())
    }
}
