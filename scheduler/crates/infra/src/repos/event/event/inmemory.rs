use super::IEventRepo;
use crate::repos::shared::repo::DeleteResult;
use crate::repos::shared::{inmemory_repo::*, query_structs::MetadataFindQuery};
use nettu_scheduler_domain::{CalendarEvent, TimeSpan, ID};

pub struct InMemoryEventRepo {
    calendar_events: std::sync::Mutex<Vec<CalendarEvent>>,
}

impl InMemoryEventRepo {
    pub fn new() -> Self {
        Self {
            calendar_events: std::sync::Mutex::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl IEventRepo for InMemoryEventRepo {
    async fn insert(&self, e: &CalendarEvent) -> anyhow::Result<()> {
        insert(e, &self.calendar_events);
        Ok(())
    }

    async fn save(&self, e: &CalendarEvent) -> anyhow::Result<()> {
        save(e, &self.calendar_events);
        Ok(())
    }

    async fn find(&self, event_id: &ID) -> Option<CalendarEvent> {
        find(event_id, &self.calendar_events)
    }

    async fn find_many(&self, event_ids: &[ID]) -> anyhow::Result<Vec<CalendarEvent>> {
        let res = find_by(&self.calendar_events, |event| event_ids.contains(&event.id));
        Ok(res)
    }

    async fn find_by_calendar(
        &self,
        calendar_id: &ID,
        timespan: Option<&TimeSpan>,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        let res = find_by(&self.calendar_events, |event| {
            if event.calendar_id == *calendar_id {
                if let Some(span) = timespan {
                    // TODO: Consider if this should be strict equals or not
                    span.start() <= event.end_ts && span.end() >= event.start_ts
                } else {
                    true
                }
            } else {
                false
            }
        });
        Ok(res)
    }

    async fn delete(&self, event_id: &ID) -> Option<CalendarEvent> {
        delete(event_id, &self.calendar_events)
    }

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<CalendarEvent> {
        find_by_metadata(&self.calendar_events, query)
    }
}
