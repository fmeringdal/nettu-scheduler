use super::sync_event_reminders::{
    EventOperation, SyncEventRemindersTrigger, SyncEventRemindersUseCase,
};
use crate::error::NettuError;
use crate::shared::{
    auth::{protect_route, Permission},
    usecase::{execute, execute_with_policy, PermissionBoundary, UseCase, UseCaseErrorContainer},
};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::{api::create_event::RequestBody, dtos::CalendarEventDTO};
use nettu_scheduler_core::{CalendarEvent, CalendarEventReminder, RRuleOptions};
use nettu_scheduler_infra::NettuContext;
use nettu_scheduler_infra::ObjectId;

pub async fn create_event_controller(
    http_req: web::HttpRequest,
    req: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = CreateEventUseCase {
        busy: req.busy.unwrap_or(false),
        start_ts: req.start_ts,
        duration: req.duration,
        calendar_id: req.calendar_id.clone(),
        rrule_options: req.rrule_options.clone(),
        user_id: user.id.clone(),
        account_id: user.account_id,
        reminder: req.reminder.clone(),
        services: req.services.clone().unwrap_or(vec![]),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|event| HttpResponse::Created().json(CalendarEventDTO::new(&event)))
        .map_err(|e| match e {
            UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
            UseCaseErrorContainer::UseCase(e) => match e {
                UseCaseErrors::NotFoundError => NettuError::NotFound(format!(
                    "The calendar with id: {}, was not found.",
                    req.calendar_id
                )),
                UseCaseErrors::InvalidRecurrenceRule => NettuError::BadClientData(
                    "Invalid recurrence rule specified for the event".into(),
                ),
                UseCaseErrors::StorageError => NettuError::InternalError,
            },
        })
}

#[derive(Debug)]
pub struct CreateEventUseCase {
    pub account_id: String,
    pub calendar_id: String,
    pub user_id: String,
    pub start_ts: i64,
    pub duration: i64,
    pub busy: bool,
    pub rrule_options: Option<RRuleOptions>,
    pub reminder: Option<CalendarEventReminder>,
    pub services: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum UseCaseErrors {
    InvalidRecurrenceRule,
    NotFoundError,
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateEventUseCase {
    type Response = CalendarEvent;

    type Errors = UseCaseErrors;

    type Context = NettuContext;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let calendar = match ctx.repos.calendar_repo.find(&self.calendar_id).await {
            Some(calendar) if calendar.user_id == self.user_id => calendar,
            _ => return Err(UseCaseErrors::NotFoundError),
        };

        let mut e = CalendarEvent {
            id: ObjectId::new().to_string(),
            busy: self.busy,
            start_ts: self.start_ts,
            duration: self.duration,
            recurrence: None,
            end_ts: self.start_ts + self.duration, // default, if recurrence changes, this will be updated
            exdates: vec![],
            calendar_id: calendar.id.clone(),
            user_id: self.user_id.clone(),
            account_id: self.account_id.clone(),
            reminder: self.reminder.clone(),
            services: self.services.clone(),
        };
        if let Some(rrule_opts) = self.rrule_options.clone() {
            if !e.set_recurrence(rrule_opts, &calendar.settings, true) {
                return Err(UseCaseErrors::InvalidRecurrenceRule);
            };
        }

        let repo_res = ctx.repos.event_repo.insert(&e).await;
        if repo_res.is_err() {
            return Err(UseCaseErrors::StorageError);
        }

        let sync_event_reminders = SyncEventRemindersUseCase {
            request: SyncEventRemindersTrigger::EventModified(
                &e,
                EventOperation::Created(&calendar),
            ),
        };

        // Sideeffect, ignore result
        let _ = execute(sync_event_reminders, ctx).await;

        Ok(e)
    }
}

impl PermissionBoundary for CreateEventUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::CreateCalendarEvent]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::prelude::*;
    use chrono::Utc;
    use nettu_scheduler_core::{Calendar, User};
    use nettu_scheduler_infra::setup_context;

    struct TestContext {
        ctx: NettuContext,
        calendar: Calendar,
        user: User,
    }

    async fn setup() -> TestContext {
        let ctx = setup_context().await;
        let user = User::new("cool2", "cool");

        let calendar = Calendar::new(&user.id);

        ctx.repos.calendar_repo.insert(&calendar).await.unwrap();
        TestContext {
            user,
            calendar,
            ctx,
        }
    }

    #[actix_web::main]
    #[test]
    async fn creates_event_without_recurrence() {
        let TestContext {
            ctx,
            calendar,
            user,
        } = setup().await;

        let mut usecase = CreateEventUseCase {
            start_ts: 500,
            duration: 800,
            rrule_options: None,
            busy: false,
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: user.account_id,
            reminder: None,
            services: vec![],
        };

        let res = usecase.execute(&ctx).await;

        assert!(res.is_ok());
    }

    #[actix_web::main]
    #[test]
    async fn creates_event_with_recurrence() {
        let TestContext {
            ctx,
            calendar,
            user,
        } = setup().await;

        let mut usecase = CreateEventUseCase {
            start_ts: 500,
            duration: 800,
            rrule_options: Some(Default::default()),
            busy: false,
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: user.account_id,
            reminder: None,
            services: vec![],
        };

        let res = usecase.execute(&ctx).await;

        assert!(res.is_ok());
    }

    #[actix_web::main]
    #[test]
    async fn rejects_invalid_calendar_id() {
        let TestContext {
            ctx,
            calendar,
            user,
        } = setup().await;

        let mut usecase = CreateEventUseCase {
            start_ts: 500,
            duration: 800,
            rrule_options: Some(Default::default()),
            busy: false,
            calendar_id: format!("1{}", calendar.id),
            user_id: user.id.clone(),
            account_id: user.account_id,
            reminder: None,
            services: vec![],
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), UseCaseErrors::NotFoundError);
    }

    #[actix_web::main]
    #[test]
    async fn rejects_event_with_invalid_recurrence() {
        let TestContext {
            ctx,
            calendar,
            user,
        } = setup().await;

        let mut invalid_rrules = vec![];
        invalid_rrules.push(RRuleOptions {
            count: Some(1000), // too big count
            ..Default::default()
        });
        invalid_rrules.push(RRuleOptions {
            until: Some(Utc.ymd(2150, 1, 1).and_hms(0, 0, 0).timestamp_millis() as isize), // too big until
            ..Default::default()
        });
        for rrule in invalid_rrules {
            let mut usecase = CreateEventUseCase {
                start_ts: 500,
                duration: 800,
                rrule_options: Some(rrule),
                busy: false,
                calendar_id: calendar.id.clone(),
                user_id: user.id.clone(),
                account_id: user.account_id.to_owned(),
                reminder: None,
                services: vec![],
            };

            let res = usecase.execute(&ctx).await;

            assert!(res.is_err());
        }
    }
}
