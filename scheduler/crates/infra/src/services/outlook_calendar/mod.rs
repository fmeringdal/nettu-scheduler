use crate::NettuContext;
mod auth_provider;
mod calendar_api;

use super::FreeBusyProviderQuery;
use nettu_scheduler_domain::providers::google::GoogleCalendarAccessRole;
use nettu_scheduler_domain::{CalendarEvent, CompatibleInstances, EventInstance, User};

// https://docs.microsoft.com/en-us/graph/api/resources/event?view=graph-rest-1.0

pub struct OutlookCalendarProvider {
    // api: GoogleCalendarRestApi,
}

impl OutlookCalendarProvider {
    pub async fn new(user: &mut User, ctx: &NettuContext) -> Result<Self, ()> {
        // let access_token = match auth_provider::get_access_token(user, ctx).await {
        //     Some(token) => token,
        //     None => return Err(()),
        // };
        Ok(Self {
            // api: GoogleCalendarRestApi::new(access_token),
        })
    }
}
