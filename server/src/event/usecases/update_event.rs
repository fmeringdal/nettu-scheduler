use crate::{
    api::Context,
    event::domain::event::RRuleOptions,
    shared::usecase::{execute, Usecase},
};
use crate::{api::NettuError, shared::auth::protect_route};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEventBody {
    start_ts: Option<i64>,
    duration: Option<i64>,
    busy: Option<bool>,
    rrule_options: Option<RRuleOptions>,
}

#[derive(Deserialize)]
pub struct EventPathParams {
    event_id: String,
}

pub async fn update_event_controller(
    http_req: HttpRequest,
    body: web::Json<UpdateEventBody>,
    path_params: web::Path<EventPathParams>,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let user = protect_route(&http_req, &ctx).await?;

    let usecase = UpdateEventUseCase {
        user_id: user.id.clone(),
        duration: body.duration,
        start_ts: body.start_ts,
        rrule_options: body.rrule_options.clone(),
        event_id: path_params.event_id.clone(),
        busy: body.busy,
    };

    execute(usecase, &ctx)
        .await
        .map(|_| HttpResponse::Ok().finish())
        .map_err(|e| match e {
            UseCaseErrors::NotFoundError => NettuError::NotFound(format!(
                "The event with id: {}, was not found.",
                path_params.event_id
            )),
            UseCaseErrors::InvalidRecurrenceRule => {
                NettuError::BadClientData("Invalid recurrence rule specified for the event".into())
            }
            UseCaseErrors::StorageError => NettuError::InternalError,
        })
}

pub struct UpdateEventUseCase {
    pub user_id: String,
    pub event_id: String,
    pub start_ts: Option<i64>,
    pub busy: Option<bool>,
    pub duration: Option<i64>,
    pub rrule_options: Option<RRuleOptions>,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFoundError,
    StorageError,
    InvalidRecurrenceRule,
}

#[async_trait::async_trait(?Send)]
impl Usecase for UpdateEventUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let mut e = match ctx.repos.event_repo.find(&self.event_id).await {
            Some(event) if event.user_id == self.user_id => event,
            _ => return Err(UseCaseErrors::NotFoundError),
        };

        let mut should_update_endtime = false;

        if let Some(start_ts) = self.start_ts {
            if e.start_ts != start_ts {
                e.start_ts = start_ts;
                e.exdates = vec![];
                should_update_endtime = true;
            }
        }
        if let Some(duration) = self.duration {
            if e.duration != duration {
                e.duration = duration;
                should_update_endtime = true;
            }
        }
        if let Some(busy) = self.busy {
            e.busy = busy;
        }

        let recurrence_res = if let Some(rrule_opts) = self.rrule_options.clone() {
            // should we clear exdates when rrules are updated ?
            e.set_reccurrence(rrule_opts, true)
        } else if should_update_endtime && e.recurrence.is_some() {
            e.set_reccurrence(e.recurrence.clone().unwrap(), true)
        } else {
            Ok(())
        };

        if recurrence_res.is_err() {
            return Err(UseCaseErrors::InvalidRecurrenceRule);
        };

        let repo_res = ctx.repos.event_repo.save(&e).await;
        if repo_res.is_err() {
            return Err(UseCaseErrors::StorageError);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
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
        };
        let ctx = Context::create_inmemory();
        let res = usecase.execute(&ctx).await;
        assert!(res.is_err());
    }
}
