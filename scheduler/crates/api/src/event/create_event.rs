use super::subscribers::CreateRemindersOnEventCreated;
use crate::shared::{
    auth::{account_can_modify_user, protect_route, Permission},
    usecase::{
        execute, execute_with_policy, PermissionBoundary, Subscriber, UseCase,
        UseCaseErrorContainer,
    },
};
use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::create_event::*;
use nettu_scheduler_domain::{CalendarEvent, CalendarEventReminder, Metadata, RRuleOptions, ID};
use nettu_scheduler_infra::NettuContext;

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
        user_id: user.id,
        calendar_id: body.calendar_id,
        rrule_options: body.rrule_options,
        account_id: account.id,
        reminder: body.reminder,
        services: body.services.unwrap_or_default(),
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
        rrule_options: body.rrule_options,
        user_id: user.id,
        account_id: user.account_id,
        reminder: body.reminder,
        services: body.services.unwrap_or_default(),
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
    pub account_id: ID,
    pub calendar_id: ID,
    pub user_id: ID,
    pub start_ts: i64,
    pub duration: i64,
    pub busy: bool,
    pub rrule_options: Option<RRuleOptions>,
    pub reminder: Option<CalendarEventReminder>,
    pub services: Vec<String>,
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
        let calendar = match ctx.repos.calendar_repo.find(&self.calendar_id).await {
            Some(calendar) if calendar.user_id == self.user_id => calendar,
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
            exdates: vec![],
            calendar_id: calendar.id.clone(),
            user_id: self.user_id.clone(),
            account_id: self.account_id.clone(),
            reminder: self.reminder.clone(),
            services: self.services.clone(),
            metadata: self.metadata.clone(),
        };
        if let Some(rrule_opts) = self.rrule_options.clone() {
            if !e.set_recurrence(rrule_opts, &calendar.settings, true) {
                return Err(UseCaseErrors::InvalidRecurrenceRule);
            };
        }

        if let Some(reminder) = &e.reminder {
            if !reminder.is_valid() {
                return Err(UseCaseErrors::InvalidReminder);
            }
        }

        let repo_res = ctx.repos.event_repo.insert(&e).await;
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
    use nettu_scheduler_domain::{Calendar, User};
    use nettu_scheduler_infra::setup_context;

    struct TestContext {
        ctx: NettuContext,
        calendar: Calendar,
        user: User,
    }

    async fn setup() -> TestContext {
        let ctx = setup_context().await;
        let user = User::new(Default::default());

        let account_id = ID::default();
        let calendar = Calendar::new(&user.id, &account_id);

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
            rrule_options: Some(Default::default()),
            busy: false,
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: user.account_id,
            reminder: None,
            services: vec![],
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
            rrule_options: Some(Default::default()),
            busy: false,
            calendar_id: ID::default(),
            user_id: user.id.clone(),
            account_id: user.account_id,
            reminder: None,
            services: vec![],
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
                metadata: Default::default(),
            };

            let res = usecase.execute(&ctx).await;

            assert!(res.is_err());
        }
    }
}
