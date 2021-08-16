use crate::{
    error::NettuError,
    event::{self, subscribers::UpdateSyncedEventsOnEventUpdated},
    shared::auth::protect_route,
    shared::{
        auth::{
            account_can_modify_event, account_can_modify_user, protect_account_route, Permission,
        },
        usecase::{
            execute, execute_with_policy, PermissionBoundary, Subscriber, UseCase,
            UseCaseErrorContainer,
        },
    },
};
use actix_web::{web, HttpRequest, HttpResponse};
use event::subscribers::SyncRemindersOnEventUpdated;
use nettu_scheduler_api_structs::update_event::*;
use nettu_scheduler_domain::{
    CalendarEvent, CalendarEventReminder, Metadata, RRuleOptions, User, ID,
};
use nettu_scheduler_infra::NettuContext;

fn handle_error(e: UseCaseErrors) -> NettuError {
    match e {
        UseCaseErrors::NotFound(entity, event_id) => NettuError::NotFound(format!(
            "The {} with id: {}, was not found.",
            entity, event_id
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

pub async fn update_event_admin_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let e = account_can_modify_event(&account, &path_params.event_id, &ctx).await?;
    let user = account_can_modify_user(&account, &e.user_id, &ctx).await?;

    let body = body.0;
    let usecase = UpdateEventUseCase {
        user,
        event_id: e.id,
        duration: body.duration,
        start_ts: body.start_ts,
        reminders: body.reminders,
        recurrence: body.recurrence,
        busy: body.busy,
        service_id: body.service_id,
        exdates: body.exdates,
        metadata: body.metadata,
    };

    execute(usecase, &ctx)
        .await
        .map(|event| HttpResponse::Ok().json(APIResponse::new(event)))
        .map_err(handle_error)
}

pub async fn update_event_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let body = body.0;
    let usecase = UpdateEventUseCase {
        user,
        event_id: path_params.event_id.clone(),
        duration: body.duration,
        start_ts: body.start_ts,
        reminders: body.reminders,
        recurrence: body.recurrence,
        busy: body.busy,
        service_id: body.service_id,
        exdates: body.exdates,
        metadata: body.metadata,
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|event| HttpResponse::Ok().json(APIResponse::new(event)))
        .map_err(|e| match e {
            UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
            UseCaseErrorContainer::UseCase(e) => handle_error(e),
        })
}

#[derive(Debug)]
pub struct UpdateEventUseCase {
    pub user: User,
    pub event_id: ID,
    pub start_ts: Option<i64>,
    pub busy: Option<bool>,
    pub duration: Option<i64>,
    pub reminders: Option<Vec<CalendarEventReminder>>,
    pub recurrence: Option<RRuleOptions>,
    pub service_id: Option<ID>,
    pub exdates: Option<Vec<i64>>,
    pub metadata: Option<Metadata>,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFound(String, ID),
    InvalidReminder,
    StorageError,
    InvalidRecurrenceRule,
}

#[async_trait::async_trait(?Send)]
impl UseCase for UpdateEventUseCase {
    type Response = CalendarEvent;

    type Errors = UseCaseErrors;

    const NAME: &'static str = "UpdateEvent";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let UpdateEventUseCase {
            user,
            event_id,
            start_ts,
            busy,
            duration,
            recurrence,
            exdates,
            reminders,
            service_id,
            metadata,
        } = self;

        let mut e = match ctx.repos.events.find(event_id).await {
            Some(event) if event.user_id == user.id => event,
            _ => {
                return Err(UseCaseErrors::NotFound(
                    "Calendar Event".into(),
                    event_id.clone(),
                ))
            }
        };

        e.service_id = service_id.clone();

        if let Some(exdates) = exdates {
            e.exdates = exdates.clone();
        }
        if let Some(metadata) = metadata {
            e.metadata = metadata.clone();
        }

        if let Some(reminders) = &reminders {
            for reminder in reminders {
                if !reminder.is_valid() {
                    return Err(UseCaseErrors::InvalidReminder);
                }
            }
            e.reminders = reminders.clone();
        }

        let calendar = match ctx.repos.calendars.find(&e.calendar_id).await {
            Some(cal) => cal,
            _ => {
                return Err(UseCaseErrors::NotFound(
                    "Calendar".into(),
                    e.calendar_id.clone(),
                ))
            }
        };

        let mut start_or_duration_change = false;

        if let Some(start_ts) = start_ts {
            if e.start_ts != *start_ts {
                e.start_ts = *start_ts;
                e.exdates = Vec::new();
                start_or_duration_change = true;
            }
        }
        if let Some(duration) = duration {
            if e.duration != *duration {
                e.duration = *duration;
                start_or_duration_change = true;
            }
        }
        if let Some(busy) = busy {
            e.busy = *busy;
        }

        let valid_recurrence = if let Some(rrule_opts) = recurrence.clone() {
            // ? should exdates be deleted when rrules are updated
            e.set_recurrence(rrule_opts, &calendar.settings, true)
        } else if start_or_duration_change && e.recurrence.is_some() {
            e.set_recurrence(e.recurrence.clone().unwrap(), &calendar.settings, true)
        } else {
            e.recurrence = None;
            true
        };

        if !valid_recurrence {
            return Err(UseCaseErrors::InvalidRecurrenceRule);
        };

        e.updated = ctx.sys.get_timestamp_millis();

        ctx.repos
            .events
            .save(&e)
            .await
            .map(|_| e.clone())
            .map_err(|_| UseCaseErrors::StorageError)
    }

    fn subscribers() -> Vec<Box<dyn Subscriber<Self>>> {
        vec![
            Box::new(SyncRemindersOnEventUpdated),
            Box::new(UpdateSyncedEventsOnEventUpdated),
        ]
    }
}

impl PermissionBoundary for UpdateEventUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::UpdateCalendarEvent]
    }
}

#[cfg(test)]
mod test {
    use nettu_scheduler_infra::setup_context;

    use super::*;

    #[actix_web::main]
    #[test]
    async fn update_nonexisting_event() {
        let mut usecase = UpdateEventUseCase {
            user: User::new(Default::default()),
            event_id: Default::default(),
            start_ts: Some(500),
            duration: Some(800),
            reminders: None,
            recurrence: None,
            busy: Some(false),
            service_id: None,
            exdates: None,
            metadata: None,
        };
        let ctx = setup_context().await;
        let res = usecase.execute(&ctx).await;
        assert!(res.is_err());
    }
}
