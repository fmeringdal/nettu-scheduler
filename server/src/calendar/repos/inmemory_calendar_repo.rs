use crate::calendar::domain::calendar::Calendar;
use std::error::Error;
use super::ICalendarRepo;


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
        let mut calendars = self.calendars.lock().unwrap();
        calendars.push(calendar.clone());
        Ok(())
    }

    async fn save(&self, calendar: &Calendar) -> Result<(), Box<dyn Error>> {
        let mut calendars = self.calendars.lock().unwrap();
        for i in 0..calendars.len() {
            if calendars[i].id == calendar.id {
                calendars.splice(i..i + 1, vec![calendar.clone()]);
            }
        }
        Ok(())
    }

    async fn find(&self, calendar_id: &str) -> Option<Calendar> {
        let calendars = self.calendars.lock().unwrap();
        for i in 0..calendars.len() {
            if calendars[i].id == calendar_id {
                return Some(calendars[i].clone());
            }
        }
        None
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
        let mut calendars = self.calendars.lock().unwrap();
        for i in 0..calendars.len() {
            if calendars[i].id == calendar_id {
                let deleted_calendar = calendars.remove(i);
                return Some(deleted_calendar);
            }
        }
        None
    }
}
