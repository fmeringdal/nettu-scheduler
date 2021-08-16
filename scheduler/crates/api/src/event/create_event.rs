use super::subscribers::CreateRemindersOnEventCreated;
use crate::error::NettuError;
use crate::event::subscribers::CreateSyncedEventsOnEventCreated;
use crate::shared::{
    auth::{account_can_modify_user, protect_account_route, protect_route, Permission},
    usecase::{execute, execute_with_policy, PermissionBoundary, Subscriber, UseCase},
};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::create_event::*;
use nettu_scheduler_domain::{
    CalendarEvent, CalendarEventReminder, Metadata, RRuleOptions, User, ID,
};
use nettu_scheduler_infra::NettuContext;

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
        reminders: body.reminders,
        service_id: body.service_id,
        metadata: body.metadata.unwrap_or_default(),
    };

    execute(usecase, &ctx)
        .await
        .map(|event| HttpResponse::Created().json(APIResponse::new(event)))
        .map_err(NettuError::from)
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
        reminders: body.reminders,
        service_id: body.service_id,
        metadata: body.metadata.unwrap_or_default(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|event| HttpResponse::Created().json(APIResponse::new(event)))
        .map_err(NettuError::from)
}

#[derive(Debug)]
pub struct CreateEventUseCase {
    pub calendar_id: ID,
    pub user: User,
    pub start_ts: i64,
    pub duration: i64,
    pub busy: bool,
    pub recurrence: Option<RRuleOptions>,
    pub reminders: Vec<CalendarEventReminder>,
    pub service_id: Option<ID>,
    pub metadata: Metadata,
}

#[derive(Debug, PartialEq)]
pub enum UseCaseError {
    InvalidRecurrenceRule,
    InvalidReminder,
    NotFound(ID),
    StorageError,
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::NotFound(calendar_id) => Self::NotFound(format!(
                "The calendar with id: {}, was not found.",
                calendar_id
            )),
            UseCaseError::InvalidRecurrenceRule => {
                Self::BadClientData("Invalid recurrence rule specified for the event".into())
            }
            UseCaseError::InvalidReminder => {
                Self::BadClientData("Invalid reminder specified for the event".into())
            }
            UseCaseError::StorageError => Self::InternalError,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateEventUseCase {
    type Response = CalendarEvent;

    type Error = UseCaseError;

    const NAME: &'static str = "CreateEvent";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let calendar = match ctx.repos.calendars.find(&self.calendar_id).await {
            Some(calendar) if calendar.user_id == self.user.id => calendar,
            _ => return Err(UseCaseError::NotFound(self.calendar_id.clone())),
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
            reminders: self.reminders.clone(),
            service_id: self.service_id.clone(),
            metadata: self.metadata.clone(),
        };

        if let Some(rrule_opts) = self.recurrence.clone() {
            if !e.set_recurrence(rrule_opts, &calendar.settings, true) {
                return Err(UseCaseError::InvalidRecurrenceRule);
            };
        }

        // TODO: maybe have reminders length restriction
        for reminder in &self.reminders {
            if !reminder.is_valid() {
                return Err(UseCaseError::InvalidReminder);
            }
        }

        ctx.repos
            .events
            .insert(&e)
            .await
            .map_err(|_| UseCaseError::StorageError)?;

        Ok(e)
    }

    fn subscribers() -> Vec<Box<dyn Subscriber<Self>>> {
        vec![
            Box::new(CreateRemindersOnEventCreated),
            Box::new(CreateSyncedEventsOnEventCreated),
        ]
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
            reminders: vec![],
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
            reminders: vec![],
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
            reminders: vec![],
            service_id: None,
            metadata: Default::default(),
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            UseCaseError::NotFound(usecase.calendar_id)
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
                reminders: vec![],
                service_id: None,
                metadata: Default::default(),
            };

            let res = usecase.execute(&ctx).await;

            assert!(res.is_err());
        }
    }
}
