use super::IServiceRepo;
use crate::shared::inmemory_repo::*;
use nettu_scheduler_core::Service;
use std::error::Error;

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
    async fn insert(&self, service: &Service) -> Result<(), Box<dyn Error>> {
        insert(service, &self.services);
        Ok(())
    }

    async fn save(&self, service: &Service) -> Result<(), Box<dyn Error>> {
        save(service, &self.services);
        Ok(())
    }

    async fn find(&self, service_id: &str) -> Option<Service> {
        find(service_id, &self.services)
    }

    async fn delete(&self, service_id: &str) -> Option<Service> {
        delete(service_id, &self.services)
    }

    async fn remove_calendar_from_services(&self, calendar_id: &str) -> Result<(), Box<dyn Error>> {
        let calendar_id = calendar_id.to_string();
        update_many(
            &self.services,
            |service| {
                for user in &service.users {
                    if user.calendar_ids.contains(&calendar_id) {
                        return true;
                    }
                }
                false
            },
            |service| {
                for user in &mut service.users {
                    user.calendar_ids.retain(|cal_id| *cal_id != calendar_id);
                }
            },
        );
        Ok(())
    }

    async fn remove_schedule_from_services(&self, schedule_id: &str) -> Result<(), Box<dyn Error>> {
        let schedule_id = schedule_id.to_string();
        update_many(
            &self.services,
            |service| {
                for user in &service.users {
                    if user.schedule_ids.contains(&schedule_id) {
                        return true;
                    }
                }
                false
            },
            |service| {
                for user in &mut service.users {
                    user.schedule_ids.retain(|id| *id != schedule_id);
                }
            },
        );
        Ok(())
    }
}
