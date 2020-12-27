use crate::calendar::domain::{calendar::Calendar, calendar_view::CalendarView};
use crate::calendar::repo::ICalendarRepo;
use crate::event::domain::event::CalendarEvent;
use crate::event::domain::event_instance::get_free_busy;
use crate::event::domain::event_instance::EventInstance;
use crate::event::repo::IEventRepo;
use crate::shared::usecase::UseCase;
use async_trait::async_trait;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct GetUserFreeBusyReq {
    pub user_id: String,
    pub calendar_ids: Option<Vec<String>>,
    pub start_ts: i64,
    pub end_ts: i64,
}

pub struct GetUserFreeBusyUseCase {
    pub event_repo: Arc<dyn IEventRepo>,
    pub calendar_repo: Arc<dyn ICalendarRepo>,
}

#[derive(Serialize)]
pub struct GetUserFreeBusyResponse {
    pub free: Vec<EventInstance>,
}

#[async_trait(?Send)]
impl UseCase<GetUserFreeBusyReq, Result<GetUserFreeBusyResponse, GetUserFreeBusyErrors>>
    for GetUserFreeBusyUseCase
{
    async fn execute(
        &self,
        req: GetUserFreeBusyReq,
    ) -> Result<GetUserFreeBusyResponse, GetUserFreeBusyErrors> {
        let view = CalendarView::create(req.start_ts, req.end_ts);
        if view.is_err() {
            return Err(GetUserFreeBusyErrors::InvalidTimespanError);
        }
        let view = view.unwrap();

        let mut calendars = self.calendar_repo.find_by_user(&req.user_id).await;
        if let Some(calendar_ids) = req.calendar_ids {
            calendars = calendars
                .into_iter()
                .filter(|cal| calendar_ids.contains(&cal.id))
                .collect();
        }

        let all_events_futures = calendars
            .iter()
            .map(|calendar| self.event_repo.find_by_calendar(&calendar.id, Some(&view)));
        let mut all_events_instances = join_all(all_events_futures)
            .await
            .into_iter()
            .map(|events_res| events_res.unwrap_or(vec![]))
            .map(|events| {
                events
                    .into_iter()
                    .map(|event| event.expand(Some(&view)))
                    // It is possible that there are no instances in the expanded event, should remove them
                    .filter(|instances| !instances.is_empty())
            })
            .flatten()
            .flatten()
            .collect::<Vec<_>>();
        // println!("All instances: {:?}", all_events_instances);
        let freebusy = get_free_busy(&mut all_events_instances);

        Ok(GetUserFreeBusyResponse { free: freebusy })
    }
}

#[derive(Debug)]
pub enum GetUserFreeBusyErrors {
    InvalidTimespanError,
}

impl std::fmt::Display for GetUserFreeBusyErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            GetUserFreeBusyErrors::InvalidTimespanError => {
                write!(f, "The provided timesspan was invalid.")
            }
        }
    }
}
