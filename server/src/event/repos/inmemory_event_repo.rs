use super::repos::{DeleteResult, IEventRepo};
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
        let mut events = self.calendar_events.lock().unwrap();
        events.push(e.clone());
        Ok(())
    }

    async fn save(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>> {
        let mut events = self.calendar_events.lock().unwrap();
        for i in 0..events.len() {
            if events[i].id == e.id {
                events.splice(i..i + 1, vec![e.clone()]);
            }
        }
        Ok(())
    }

    async fn find(&self, event_id: &str) -> Option<CalendarEvent> {
        let events = self.calendar_events.lock().unwrap();
        for i in 0..events.len() {
            if events[i].id == event_id {
                return Some(events[i].clone());
            }
        }
        None
    }

    async fn find_by_calendar(
        &self,
        calendar_id: &str,
        view: Option<&CalendarView>,
    ) -> Result<Vec<CalendarEvent>, Box<dyn Error>> {
        let events = self.calendar_events.lock().unwrap();
        let mut res = vec![];
        for event in events.iter() {
            if event.calendar_id == calendar_id {
                if let Some(v) = view {
                    // TODO: Consider if this should be strict equals or not
                    if v.get_start() <= event.end_ts.unwrap()
                        && v.get_end() >= event.start_ts
                    {
                        res.push(event.clone());
                    }
                } else {
                    res.push(event.clone());
                }
            }
        }
        Ok(res)
    }

    async fn delete(&self, event_id: &str) -> Option<CalendarEvent> {
        let mut events = self.calendar_events.lock().unwrap();
        for i in 0..events.len() {
            if events[i].id == event_id {
                let deleted_event = events.remove(i);
                return Some(deleted_event);
            }
        }
        None
    }

    async fn delete_by_calendar(&self, calendar_id: &str) -> Result<DeleteResult, Box<dyn Error>> {
        let mut events = self.calendar_events.lock().unwrap();
        let mut deleted_count = 0;
        for i in 0..events.len() {
            let index = events.len() - i - 1;
            if events[index].calendar_id == calendar_id {
                events.remove(index);
                deleted_count += 1;
            }
        }
        Ok(DeleteResult { deleted_count })
    }
}
