mod postgres;

use crate::repos::shared::query_structs::MetadataFindQuery;
use nettu_scheduler_domain::{CalendarEvent, TimeSpan, ID};
pub use postgres::PostgresEventRepo;

#[async_trait::async_trait]
pub trait IEventRepo: Send + Sync {
    async fn insert(&self, e: &CalendarEvent) -> anyhow::Result<()>;
    async fn save(&self, e: &CalendarEvent) -> anyhow::Result<()>;
    async fn find(&self, event_id: &ID) -> Option<CalendarEvent>;
    async fn find_many(&self, event_ids: &[ID]) -> anyhow::Result<Vec<CalendarEvent>>;
    async fn find_by_calendar(
        &self,
        calendar_id: &ID,
        timespan: Option<&TimeSpan>,
    ) -> anyhow::Result<Vec<CalendarEvent>>;
    async fn find_most_recent_service_event(
        &self,
        service_id: &ID,
        user_id: &ID,
    ) -> Option<CalendarEvent>;
    async fn find_by_service(
        &self,
        service_id: &ID,
        min_ts: i64,
        max_ts: i64,
    ) -> Vec<CalendarEvent>;
    async fn delete(&self, event_id: &ID) -> Option<CalendarEvent>;
    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<CalendarEvent>;
}

#[cfg(test)]
mod tests {
    use crate::{setup_context, NettuContext};
    use nettu_scheduler_domain::{Account, Calendar, CalendarEvent, Entity, TimeSpan, User, ID};

    fn generate_default_event(account_id: &ID, calendar_id: &ID, user_id: &ID) -> CalendarEvent {
        CalendarEvent {
            account_id: account_id.clone(),
            busy: Default::default(),
            calendar_id: calendar_id.clone(),
            created: Default::default(),
            duration: Default::default(),
            end_ts: Default::default(),
            exdates: Default::default(),
            id: Default::default(),
            service_id: Default::default(),
            metadata: Default::default(),
            recurrence: Default::default(),
            reminder: Default::default(),
            start_ts: Default::default(),
            updated: Default::default(),
            synced_events: Default::default(),
            user_id: user_id.clone(),
        }
    }

    struct TestContext {
        ctx: NettuContext,
        account: Account,
        calendar: Calendar,
        user: User,
    }

    async fn setup() -> TestContext {
        let ctx = setup_context().await;
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();
        let user = User::new(account.id.clone());
        ctx.repos.users.insert(&user).await.unwrap();
        let calendar = Calendar::new(&user.id, &account.id);
        ctx.repos.calendars.insert(&calendar).await.unwrap();

        TestContext {
            account,
            calendar,
            user,
            ctx,
        }
    }

    #[tokio::test]
    async fn create_and_delete() {
        let TestContext {
            ctx,
            account,
            calendar,
            user,
        } = setup().await;
        let event = generate_default_event(&account.id, &calendar.id, &user.id);

        // Insert
        assert!(ctx.repos.events.insert(&event).await.is_ok());

        // Different find methods
        let get_event_res = ctx.repos.events.find(&event.id).await.unwrap();
        assert!(get_event_res.eq(&event));
        let get_event_res = ctx
            .repos
            .events
            .find_many(&vec![event.id.clone()])
            .await
            .expect("To find many events");
        assert!(get_event_res[0].eq(&event));

        // Delete
        let delete_res = ctx
            .repos
            .events
            .delete(&event.id)
            .await
            .expect("To delete event by id");
        assert!(delete_res.eq(&event));

        // Find
        assert!(ctx.repos.events.find(&event.id).await.is_none());
    }

    #[tokio::test]
    async fn update() {
        let TestContext {
            ctx,
            account,
            calendar,
            user,
        } = setup().await;
        let mut event = generate_default_event(&account.id, &calendar.id, &user.id);

        // Insert
        assert!(ctx.repos.events.insert(&event).await.is_ok());

        event.updated += 1;

        // Save
        assert!(ctx.repos.events.save(&event).await.is_ok());

        // Find
        assert!(ctx
            .repos
            .events
            .find(&event.id)
            .await
            .expect("To be event")
            .eq(&event));
    }

    #[tokio::test]
    async fn delete_by_user() {
        let TestContext {
            ctx,
            account,
            calendar,
            user,
        } = setup().await;
        let event = generate_default_event(&account.id, &calendar.id, &user.id);

        // Insert
        assert!(ctx.repos.events.insert(&event).await.is_ok());

        // Delete
        assert!(ctx.repos.users.delete(&user.id).await.is_some());

        // Find after delete
        assert!(ctx.repos.events.find(&event.id).await.is_none());
    }

    #[tokio::test]
    async fn delete_by_calendar() {
        let TestContext {
            ctx,
            account,
            calendar,
            user,
        } = setup().await;
        let event = generate_default_event(&account.id, &calendar.id, &user.id);

        // Insert
        assert!(ctx.repos.events.insert(&event).await.is_ok());

        // Delete
        assert!(ctx.repos.calendars.delete(&calendar.id).await.is_ok());

        // Find after delete
        assert!(ctx.repos.events.find(&event.id).await.is_none());
    }

    async fn generate_event_with_time(
        account_id: &ID,
        calendar_id: &ID,
        user_id: &ID,
        start_ts: i64,
        end_ts: i64,
        ctx: &NettuContext,
    ) -> CalendarEvent {
        let mut event = generate_default_event(&account_id, &calendar_id, &user_id);
        event.calendar_id = calendar_id.clone();
        event.start_ts = start_ts;
        event.end_ts = end_ts;
        ctx.repos
            .events
            .insert(&event)
            .await
            .expect("To insert event");
        event
    }

    #[tokio::test]
    async fn find_by_calendar_and_timespan() {
        let TestContext {
            ctx,
            account,
            calendar,
            user,
        } = setup().await;
        let start_ts = 100;
        let end_ts = 200;
        // All the possible combination of intervals
        let event_1 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            start_ts - 2,
            start_ts - 1,
            &ctx,
        )
        .await;
        let event_2 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            start_ts - 1,
            start_ts,
            &ctx,
        )
        .await;
        let event_3 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            start_ts - 1,
            start_ts + 1,
            &ctx,
        )
        .await;
        let event_4 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            start_ts - 1,
            end_ts,
            &ctx,
        )
        .await;
        let event_5 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            start_ts - 1,
            end_ts + 1,
            &ctx,
        )
        .await;
        let event_6 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            start_ts,
            end_ts - 1,
            &ctx,
        )
        .await;
        let event_7 =
            generate_event_with_time(&account.id, &calendar.id, &user.id, start_ts, end_ts, &ctx)
                .await;
        let event_8 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            start_ts,
            end_ts + 1,
            &ctx,
        )
        .await;
        let event_9 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            start_ts + 1,
            end_ts - 1,
            &ctx,
        )
        .await;
        let event_10 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            start_ts + 1,
            end_ts,
            &ctx,
        )
        .await;
        let event_11 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            start_ts + 1,
            end_ts + 1,
            &ctx,
        )
        .await;
        let event_12 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            end_ts,
            end_ts + 1,
            &ctx,
        )
        .await;
        let event_13 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            end_ts + 1,
            end_ts + 2,
            &ctx,
        )
        .await;

        let actual_events_in_timespan = vec![
            event_2.clone(),
            event_3.clone(),
            event_4.clone(),
            event_5.clone(),
            event_6.clone(),
            event_7.clone(),
            event_8.clone(),
            event_9.clone(),
            event_10.clone(),
            event_11.clone(),
            event_12.clone(),
        ];

        let mut actual_events_in_calendar = actual_events_in_timespan.clone();
        actual_events_in_calendar.push(event_1.clone());
        actual_events_in_calendar.push(event_13.clone());

        // Find
        let events_in_calendar_and_timespan = ctx
            .repos
            .events
            .find_by_calendar(&calendar.id, Some(&TimeSpan::new(start_ts, end_ts)))
            .await
            .expect("To get events");

        assert_eq!(
            events_in_calendar_and_timespan.len(),
            actual_events_in_timespan.len()
        );
        for actual_event in actual_events_in_timespan {
            assert!(events_in_calendar_and_timespan
                .iter()
                .find(|e| e.id() == actual_event.id())
                .is_some());
        }

        let events_in_calendar = ctx
            .repos
            .events
            .find_by_calendar(&calendar.id, None)
            .await
            .expect("To get events");
        assert_eq!(actual_events_in_calendar.len(), events_in_calendar.len());
        for actual_event in actual_events_in_calendar {
            assert!(events_in_calendar
                .iter()
                .find(|e| e.id() == actual_event.id())
                .is_some());
        }
    }
}
