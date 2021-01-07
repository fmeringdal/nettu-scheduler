use super::{DeleteResult, IEventRepo};
use crate::shared::inmemory_repo::*;
use crate::{calendar::domain::calendar_view::CalendarView, event::domain::event::CalendarEvent};
use std::error::Error;

pub struct InMemoryEventRepo {
    calendar_events: std::sync::Mutex<Vec<CalendarEvent>>,
}

impl InMemoryEventRepo {
    pub fn new() -> Self {
        Self {
            calendar_events: std::sync::Mutex::new(vec![]),
        }
    }
}

#[async_trait::async_trait]
impl IEventRepo for InMemoryEventRepo {
    async fn insert(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>> {
        insert(e, &self.calendar_events);
        Ok(())
    }

    async fn save(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>> {
        save(e, &self.calendar_events);
        Ok(())
    }

    async fn find(&self, event_id: &str) -> Option<CalendarEvent> {
        find(event_id, &self.calendar_events)
    }

    async fn find_by_calendar(
        &self,
        calendar_id: &str,
        view: Option<&CalendarView>,
    ) -> Result<Vec<CalendarEvent>, Box<dyn Error>> {
        let res = find_by(&self.calendar_events, |event| {
            if event.calendar_id == calendar_id {
                if let Some(v) = view {
                    // TODO: Consider if this should be strict equals or not
                    return v.get_start() <= event.end_ts && v.get_end() >= event.start_ts;
                } else {
                    return true;
                }
            }
            return false;
        });
        Ok(res)
    }

    async fn delete(&self, event_id: &str) -> Option<CalendarEvent> {
        delete(event_id, &self.calendar_events)
    }

    async fn delete_by_calendar(&self, calendar_id: &str) -> Result<DeleteResult, Box<dyn Error>> {
        let res = delete_by(&self.calendar_events, |cal| cal.calendar_id == calendar_id);
        Ok(res)
    }
}
