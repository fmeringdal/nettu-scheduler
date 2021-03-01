use crate::{
    error::NettuError,
    event,
    shared::auth::protect_route,
    shared::{
        auth::Permission,
        usecase::{
            execute, execute_with_policy, PermissionBoundary, UseCase, UseCaseErrorContainer,
        },
    },
};
use actix_web::{web, HttpRequest, HttpResponse};
use event::sync_event_reminders::{
    EventOperation, SyncEventRemindersTrigger, SyncEventRemindersUseCase,
};
use nettu_scheduler_api_structs::update_event::*;
use nettu_scheduler_core::{CalendarEvent, RRuleOptions};
use nettu_scheduler_infra::NettuContext;

pub async fn update_event_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = UpdateEventUseCase {
        user_id: user.id.clone(),
        duration: body.duration,
        start_ts: body.start_ts,
        rrule_options: body.rrule_options.clone(),
        event_id: path_params.event_id.clone(),
        busy: body.busy,
        services: body.services.clone(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|event| HttpResponse::Ok().json(APIResponse::new(event)))
        .map_err(|e| match e {
            UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
            UseCaseErrorContainer::UseCase(e) => match e {
                UseCaseErrors::NotFoundError => NettuError::NotFound(format!(
                    "The event with id: {}, was not found.",
                    path_params.event_id
                )),
                UseCaseErrors::InvalidRecurrenceRule => NettuError::BadClientData(
                    "Invalid recurrence rule specified for the event".into(),
                ),
                UseCaseErrors::StorageError => NettuError::InternalError,
            },
        })
}

#[derive(Debug)]
pub struct UpdateEventUseCase {
    pub user_id: String,
    pub event_id: String,
    pub start_ts: Option<i64>,
    pub busy: Option<bool>,
    pub duration: Option<i64>,
    pub rrule_options: Option<RRuleOptions>,
    pub services: Option<Vec<String>>,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFoundError,
    StorageError,
    InvalidRecurrenceRule,
}

#[async_trait::async_trait(?Send)]
impl UseCase for UpdateEventUseCase {
    type Response = CalendarEvent;

    type Errors = UseCaseErrors;

    type Context = NettuContext;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let UpdateEventUseCase {
            user_id,
            event_id,
            start_ts,
            busy,
            duration,
            rrule_options: _,
            services,
        } = self;

        let mut e = match ctx.repos.event_repo.find(&event_id).await {
            Some(event) if event.user_id == *user_id => event,
            _ => return Err(UseCaseErrors::NotFoundError),
        };

        if let Some(services) = services {
            e.services = services.clone();
        }

        let calendar = match ctx.repos.calendar_repo.find(&e.calendar_id).await {
            Some(cal) => cal,
            _ => return Err(UseCaseErrors::NotFoundError),
        };

        let mut start_or_duration_change = false;

        if let Some(start_ts) = start_ts {
            if e.start_ts != *start_ts {
                e.start_ts = *start_ts;
                e.exdates = vec![];
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

        let valid_recurrence = if let Some(rrule_opts) = self.rrule_options.clone() {
            // ? should exdates be deleted when rrules are updated
            e.set_recurrence(rrule_opts, &calendar.settings, true)
        } else if start_or_duration_change && e.recurrence.is_some() {
            e.set_recurrence(e.recurrence.clone().unwrap(), &calendar.settings, true)
        } else {
            true
        };

        if !valid_recurrence {
            return Err(UseCaseErrors::InvalidRecurrenceRule);
        };

        let repo_res = ctx.repos.event_repo.save(&e).await;
        if repo_res.is_err() {
            return Err(UseCaseErrors::StorageError);
        }

        let sync_event_reminders = SyncEventRemindersUseCase {
            request: SyncEventRemindersTrigger::EventModified(
                &e,
                EventOperation::Updated(&calendar),
            ),
        };

        // Sideeffect, ignore result
        let _ = execute(sync_event_reminders, ctx).await;

        Ok(e)
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
    async fn update_notexisting_event() {
        let mut usecase = UpdateEventUseCase {
            event_id: String::from(""),
            start_ts: Some(500),
            duration: Some(800),
            rrule_options: None,
            busy: Some(false),
            user_id: String::from("cool"),
            services: None,
        };
        let ctx = setup_context().await;
        let res = usecase.execute(&ctx).await;
        assert!(res.is_err());
    }
}
