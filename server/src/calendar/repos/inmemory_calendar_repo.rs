use super::ICalendarRepo;
use crate::calendar::domain::calendar::Calendar;
use crate::shared::inmemory_repo::*;
use std::error::Error;

pub struct InMemoryCalendarRepo {
    calendars: std::sync::Mutex<Vec<Calendar>>,
}

impl InMemoryCalendarRepo {
    pub fn new() -> Self {
        Self {
            calendars: std::sync::Mutex::new(vec![]),
        }
    }
}

#[async_trait::async_trait]
impl ICalendarRepo for InMemoryCalendarRepo {
    async fn insert(&self, calendar: &Calendar) -> Result<(), Box<dyn Error>> {
        insert(calendar, &self.calendars);
        Ok(())
    }

    async fn save(&self, calendar: &Calendar) -> Result<(), Box<dyn Error>> {
        save(calendar, &self.calendars);
        Ok(())
    }

    async fn find(&self, calendar_id: &str) -> Option<Calendar> {
        find(calendar_id, &self.calendars)
    }

    async fn find_by_user(&self, user_id: &str) -> Vec<Calendar> {
        let calendars = self.calendars.lock().unwrap();
        let mut res = vec![];
        for i in 0..calendars.len() {
            if calendars[i].user_id == user_id {
                res.push(calendars[i].clone());
            }
        }
        res
    }

    async fn delete(&self, calendar_id: &str) -> Option<Calendar> {
        delete(calendar_id, &self.calendars)
    }
}
