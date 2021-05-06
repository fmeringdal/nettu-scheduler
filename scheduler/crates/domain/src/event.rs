use crate::{
    calendar::CalendarSettings,
    shared::entity::Entity,
    shared::{metadata::Metadata, recurrence::RRuleOptions},
    timespan::TimeSpan,
    Meta,
};
use crate::{event_instance::EventInstance, shared::entity::ID};
use chrono::{prelude::*, Duration};
use rrule::{RRule, RRuleSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct CalendarEvent {
    pub id: ID,
    pub start_ts: i64,
    pub duration: i64,
    pub busy: bool,
    pub end_ts: i64,
    pub created: i64,
    pub updated: i64,
    pub recurrence: Option<RRuleOptions>,
    pub exdates: Vec<i64>,
    pub calendar_id: ID,
    pub user_id: ID,
    pub account_id: ID,
    pub reminder: Option<CalendarEventReminder>,
    pub is_service: bool,
    pub metadata: Metadata,
    pub synced_events: Vec<SyncedCalendarEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncedCalendarProvider {
    Google,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedCalendarEvent {
    pub event_id: String,
    pub calendar_id: String,
    pub provider: SyncedCalendarProvider,
}

impl Entity for CalendarEvent {
    fn id(&self) -> &ID {
        &self.id
    }
}

impl Meta for CalendarEvent {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    fn account_id(&self) -> &ID {
        &self.account_id
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventReminder {
    pub minutes_before: i64,
}

impl CalendarEventReminder {
    // This isnt ideal at all, shouldnt be possible to construct
    // this type of it is not valid, but for now it is good enough
    pub fn is_valid(&self) -> bool {
        self.minutes_before >= 0 && self.minutes_before <= 60 * 24
    }
}

impl CalendarEvent {
    fn update_endtime(&mut self, calendar_settings: &CalendarSettings) -> bool {
        match self.recurrence.clone() {
            Some(recurrence) => {
                let rrule_options = recurrence.get_parsed_options(self.start_ts, calendar_settings);
                if (rrule_options.count.is_some() && rrule_options.count.unwrap() > 0)
                    || rrule_options.until.is_some()
                {
                    let expand = self.expand(None, calendar_settings);
                    self.end_ts = expand.last().unwrap().end_ts;
                } else {
                    self.end_ts = Self::get_max_timestamp();
                }
                true
            }
            None => true,
        }
    }

    pub fn set_recurrence(
        &mut self,
        reccurence: RRuleOptions,
        calendar_settings: &CalendarSettings,
        update_endtime: bool,
    ) -> bool {
        let valid_recurrence = reccurence.is_valid(self.start_ts);
        if !valid_recurrence {
            return false;
        }

        self.recurrence = Some(reccurence);
        if update_endtime {
            return self.update_endtime(calendar_settings);
        }
        true
    }

    pub fn get_max_timestamp() -> i64 {
        5609882500905 // Mon Oct 09 2147 06:41:40 GMT+0200 (Central European Summer Time)
    }

    pub fn get_rrule_set(&self, calendar_settings: &CalendarSettings) -> Option<RRuleSet> {
        self.recurrence.clone().map(|recurrence| {
            let rrule_options = recurrence.get_parsed_options(self.start_ts, calendar_settings);
            let tzid = rrule_options.tzid;
            let mut rrule_set = RRuleSet::new();
            for exdate in &self.exdates {
                let exdate = tzid.timestamp_millis(*exdate);
                rrule_set.exdate(exdate);
            }
            let rrule = RRule::new(rrule_options);
            rrule_set.rrule(rrule);
            rrule_set
        })
    }

    pub fn expand(
        &self,
        timespan: Option<&TimeSpan>,
        calendar_settings: &CalendarSettings,
    ) -> Vec<EventInstance> {
        match self.recurrence.clone() {
            Some(recurrence) => {
                let rrule_options = recurrence.get_parsed_options(self.start_ts, calendar_settings);
                let tzid = rrule_options.tzid;
                let rrule_set = self.get_rrule_set(calendar_settings).unwrap();

                let instances = match timespan {
                    Some(timespan) => {
                        let timespan = timespan.as_datetime(&tzid);

                        // Also take the duration of events into consideration as the rrule library
                        // does not support duration on events.
                        let end = timespan.end - Duration::milliseconds(self.duration);

                        // RRule v0.5.5 is not inclusive on start, so just by subtracting one millisecond
                        // will make it inclusive
                        let start = timespan.start - Duration::milliseconds(1);

                        rrule_set.between(start, end, true)
                    }
                    None => rrule_set.all(),
                };

                instances
                    .iter()
                    .map(|occurence| {
                        let start_ts = occurence.timestamp_millis();

                        EventInstance {
                            start_ts,
                            end_ts: start_ts + self.duration,
                            busy: self.busy,
                        }
                    })
                    .collect()
            }
            None => {
                if self.exdates.contains(&self.start_ts) {
                    vec![]
                } else {
                    vec![EventInstance {
                        start_ts: self.start_ts,
                        end_ts: self.start_ts + self.duration,
                        busy: self.busy,
                    }]
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{shared::recurrence::WeekDay, RRuleFrequenzy};
    use chrono_tz::UTC;

    #[test]
    fn daily_calendar_event() {
        let settings = CalendarSettings {
            timezone: UTC,
            week_start: 0,
        };
        let event = CalendarEvent {
            id: Default::default(),
            start_ts: 1521317491239,
            busy: false,
            duration: 1000 * 60 * 60,
            recurrence: Some(RRuleOptions {
                freq: RRuleFrequenzy::Daily,
                interval: 1,
                count: Some(4),
                ..Default::default()
            }),
            end_ts: 2521317491239,
            exdates: vec![1521317491239],
            calendar_id: Default::default(),
            user_id: Default::default(),
            account_id: Default::default(),
            reminder: None,
            is_service: false,
            metadata: Default::default(),
            created: Default::default(),
            updated: Default::default(),
            synced_events: Default::default(),
        };

        let oc = event.expand(None, &settings);
        assert_eq!(oc.len(), 3);
    }

    #[test]
    fn calendar_event_without_recurrence() {
        let settings = CalendarSettings {
            timezone: UTC,
            week_start: 0,
        };
        let mut event = CalendarEvent {
            id: Default::default(),
            start_ts: 1521317491239,
            busy: false,
            duration: 1000 * 60 * 60,
            recurrence: None,
            end_ts: 2521317491239,
            exdates: vec![],
            calendar_id: Default::default(),
            user_id: Default::default(),
            account_id: Default::default(),
            reminder: None,
            is_service: false,
            metadata: Default::default(),
            created: Default::default(),
            updated: Default::default(),
            synced_events: Default::default(),
        };

        let oc = event.expand(None, &settings);
        assert_eq!(oc.len(), 1);

        // Without recurrence but with exdate at start time
        event.exdates = vec![event.start_ts];
        let oc = event.expand(None, &settings);
        assert_eq!(oc.len(), 0);
    }

    #[test]
    fn rejects_event_with_invalid_recurrence() {
        let settings = CalendarSettings {
            timezone: UTC,
            week_start: 0,
        };
        let mut invalid_rrules = vec![];
        invalid_rrules.push(RRuleOptions {
            count: Some(1000), // too big count
            ..Default::default()
        });
        invalid_rrules.push(RRuleOptions {
            until: Some(Utc.ymd(2150, 1, 1).and_hms(0, 0, 0).timestamp_millis() as isize), // too big until
            ..Default::default()
        });
        invalid_rrules.push(RRuleOptions {
            // Only bysetpos and no by*
            bysetpos: Some(vec![1]),
            freq: RRuleFrequenzy::Monthly,
            ..Default::default()
        });
        for rrule in invalid_rrules {
            let mut event = CalendarEvent {
                id: Default::default(),
                start_ts: 1521317491239,
                busy: false,
                duration: 1000 * 60 * 60,
                end_ts: 2521317491239,
                exdates: vec![],
                calendar_id: Default::default(),
                user_id: Default::default(),
                account_id: Default::default(),
                recurrence: None,
                reminder: None,
                is_service: false,
                metadata: Default::default(),
                created: Default::default(),
                updated: Default::default(),
                synced_events: Default::default(),
            };

            assert!(!event.set_recurrence(rrule, &settings, true));
        }
    }

    #[test]
    fn allows_event_with_valid_recurrence() {
        let settings = CalendarSettings {
            timezone: UTC,
            week_start: 0,
        };
        let mut valid_rrules = vec![];
        let start_ts = 1521317491239;
        valid_rrules.push(Default::default());
        valid_rrules.push(RRuleOptions {
            count: Some(100),
            ..Default::default()
        });
        valid_rrules.push(RRuleOptions {
            until: Some(start_ts + 1000 * 60 * 60 * 24 * 100),
            ..Default::default()
        });
        valid_rrules.push(RRuleOptions {
            byweekday: Some(vec![WeekDay::new(1).unwrap()]),
            ..Default::default()
        });
        valid_rrules.push(RRuleOptions {
            byweekday: Some(vec![WeekDay::new_nth(1, 1).unwrap()]),
            freq: RRuleFrequenzy::Monthly,
            ..Default::default()
        });
        for rrule in valid_rrules {
            let mut event = CalendarEvent {
                id: Default::default(),
                start_ts: start_ts as i64,
                busy: false,
                duration: 1000 * 60 * 60,
                end_ts: 2521317491239,
                exdates: vec![],
                calendar_id: Default::default(),
                account_id: Default::default(),
                user_id: Default::default(),
                recurrence: None,
                reminder: None,
                is_service: false,
                metadata: Default::default(),
                created: Default::default(),
                updated: Default::default(),
                synced_events: Default::default(),
            };

            assert!(event.set_recurrence(rrule, &settings, true));
        }
    }
}
