use super::subscribers::CreateRemindersOnEventCreated;
use crate::error::NettuError;
use crate::shared::{
    auth::{account_can_modify_user, protect_account_route, protect_route, Permission},
    usecase::{
        execute, execute_with_policy, PermissionBoundary, Subscriber, UseCase,
        UseCaseErrorContainer,
    },
};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::create_event::*;
use nettu_scheduler_domain::{
    CalendarEvent, CalendarEventReminder, Metadata, RRuleOptions, SyncedCalendar,
    SyncedCalendarEvent, SyncedCalendarProvider, User, ID,
};
use nettu_scheduler_infra::{google_calendar::GoogleCalendarProvider, NettuContext};
use tracing::info;

fn handle_error(e: UseCaseErrors) -> NettuError {
    match e {
        UseCaseErrors::NotFound(calendar_id) => NettuError::NotFound(format!(
            "The calendar with id: {}, was not found.",
            calendar_id
        )),
        UseCaseErrors::InvalidRecurrenceRule => {
            NettuError::BadClientData("Invalid recurrence rule specified for the event".into())
        }
        UseCaseErrors::InvalidReminder => {
            NettuError::BadClientData("Invalid reminder specified for the event".into())
        }
        UseCaseErrors::StorageError => NettuError::InternalError,
    }
}

pub async fn create_event_admin_controller(
    http_req: web::HttpRequest,
    path_params: web::Path<PathParams>,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let user = account_can_modify_user(&account, &path_params.user_id, &ctx).await?;

    let body = body.0;
    let usecase = CreateEventUseCase {
        busy: body.busy.unwrap_or(false),
        start_ts: body.start_ts,
        duration: body.duration,
        user,
        calendar_id: body.calendar_id,
        recurrence: body.recurrence,
        reminder: body.reminder,
        service_id: body.service_id,
        metadata: body.metadata.unwrap_or_default(),
    };

    execute(usecase, &ctx)
        .await
        .map(|event| HttpResponse::Created().json(APIResponse::new(event)))
        .map_err(handle_error)
}

pub async fn create_event_controller(
    http_req: web::HttpRequest,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let body = body.0;
    let usecase = CreateEventUseCase {
        busy: body.busy.unwrap_or(false),
        start_ts: body.start_ts,
        duration: body.duration,
        calendar_id: body.calendar_id,
        recurrence: body.recurrence,
        user,
        reminder: body.reminder,
        service_id: body.service_id,
        metadata: body.metadata.unwrap_or_default(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|event| HttpResponse::Created().json(APIResponse::new(event)))
        .map_err(|e| match e {
            UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
            UseCaseErrorContainer::UseCase(e) => handle_error(e),
        })
}

#[derive(Debug)]
pub struct CreateEventUseCase {
    pub calendar_id: ID,
    pub user: User,
    pub start_ts: i64,
    pub duration: i64,
    pub busy: bool,
    pub recurrence: Option<RRuleOptions>,
    pub reminder: Option<CalendarEventReminder>,
    pub service_id: Option<ID>,
    pub metadata: Metadata,
}

#[derive(Debug, PartialEq)]
pub enum UseCaseErrors {
    InvalidRecurrenceRule,
    InvalidReminder,
    NotFound(ID),
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateEventUseCase {
    type Response = CalendarEvent;

    type Errors = UseCaseErrors;

    const NAME: &'static str = "CreateEvent";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let calendar = match ctx.repos.calendars.find(&self.calendar_id).await {
            Some(calendar) if calendar.user_id == self.user.id => calendar,
            _ => return Err(UseCaseErrors::NotFound(self.calendar_id.clone())),
        };

        let mut e = CalendarEvent {
            id: Default::default(),
            busy: self.busy,
            start_ts: self.start_ts,
            duration: self.duration,
            created: ctx.sys.get_timestamp_millis(),
            updated: ctx.sys.get_timestamp_millis(),
            recurrence: None,
            end_ts: self.start_ts + self.duration, // default, if recurrence changes, this will be updated
            exdates: Vec::new(),
            calendar_id: calendar.id.clone(),
            user_id: self.user.id.clone(),
            account_id: self.user.account_id.clone(),
            reminder: self.reminder.clone(),
            service_id: self.service_id.clone(),
            metadata: self.metadata.clone(),
            synced_events: Default::default(),
        };
        info!("Metadata got from event!: {:?}", e.metadata);
        if let Some(rrule_opts) = self.recurrence.clone() {
            if !e.set_recurrence(rrule_opts, &calendar.settings, true) {
                return Err(UseCaseErrors::InvalidRecurrenceRule);
            };
        }

        if let Some(reminder) = &e.reminder {
            if !reminder.is_valid() {
                return Err(UseCaseErrors::InvalidReminder);
            }
        }

        let synced_google_calendar_ids = calendar
            .synced
            .iter()
            .map(|synced| match synced {
                SyncedCalendar::Google(id) => id.clone(),
            })
            .collect::<Vec<_>>();
        if !synced_google_calendar_ids.is_empty() {
            if let Ok(provider) = GoogleCalendarProvider::new(&mut self.user, ctx).await {
                for synced_google_calendar_id in synced_google_calendar_ids {
                    if let Ok(google_event) = provider
                        .create_event(synced_google_calendar_id.clone(), e.clone())
                        .await
                    {
                        e.synced_events.push(SyncedCalendarEvent {
                            calendar_id: synced_google_calendar_id,
                            event_id: google_event.id,
                            provider: SyncedCalendarProvider::Google,
                        })
                    }
                }
            }
        }

        println!("Created this event: {:?}", e);
        let repo_res = ctx.repos.events.insert(&e).await;
        if repo_res.is_err() {
            return Err(UseCaseErrors::StorageError);
        }

        Ok(e)
    }

    fn subscribers() -> Vec<Box<dyn Subscriber<Self>>> {
        vec![Box::new(CreateRemindersOnEventCreated)]
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
    use nettu_scheduler_domain::{Account, Calendar, User};
    use nettu_scheduler_infra::setup_context;

    struct TestContext {
        ctx: NettuContext,
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
            recurrence: None,
            busy: false,
            calendar_id: calendar.id.clone(),
            user,
            reminder: None,
            service_id: None,
            metadata: Default::default(),
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
            recurrence: Some(Default::default()),
            busy: false,
            calendar_id: calendar.id.clone(),
            user,
            reminder: None,
            service_id: None,
            metadata: Default::default(),
        };

        let res = usecase.execute(&ctx).await;

        assert!(res.is_ok());
    }

    #[actix_web::main]
    #[test]
    async fn rejects_invalid_calendar_id() {
        let TestContext {
            ctx,
            calendar: _,
            user,
        } = setup().await;

        let mut usecase = CreateEventUseCase {
            start_ts: 500,
            duration: 800,
            recurrence: Some(Default::default()),
            busy: false,
            calendar_id: ID::default(),
            user,
            reminder: None,
            service_id: None,
            metadata: Default::default(),
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            UseCaseErrors::NotFound(usecase.calendar_id)
        );
    }

    #[actix_web::main]
    #[test]
    async fn rejects_event_with_invalid_recurrence() {
        let TestContext {
            ctx,
            calendar,
            user,
        } = setup().await;

        let mut invalid_rrules = Vec::new();
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
                recurrence: Some(rrule),
                busy: false,
                calendar_id: calendar.id.clone(),
                user: user.clone(),
                reminder: None,
                service_id: None,
                metadata: Default::default(),
            };

            let res = usecase.execute(&ctx).await;

            assert!(res.is_err());
        }
    }
}
