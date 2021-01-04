use crate::{
    api::Context,
    event::domain::event::RRuleOptions,
    shared::usecase::{perform, Usecase},
};
use crate::{event::repos::IEventRepo, shared::auth::protect_route};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    params: web::Path<EventPathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let user = match protect_route(&http_req, &ctx).await {
        Ok(u) => u,
        Err(res) => return res,
    };

    let usecase = UpdateEventUseCase {
        user_id: user.id.clone(),
        duration: body.duration,
        start_ts: body.start_ts,
        rrule_options: body.rrule_options.clone(),
        event_id: params.event_id.clone(),
        busy: body.busy,
    };
    let res = perform(usecase, &ctx).await;
    match res {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => match e {
            UseCaseErrors::NotFoundError => HttpResponse::NotFound().finish(),
            UseCaseErrors::StorageError => HttpResponse::InternalServerError().finish(),
        },
    }
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
}

#[async_trait::async_trait(?Send)]
impl Usecase for UpdateEventUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn perform(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
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

        if let Some(rrule_opts) = self.rrule_options.clone() {
            // should we clear exdates when rrules are updated ?
            e.set_reccurrence(rrule_opts, true);
        } else if should_update_endtime && e.recurrence.is_some() {
            e.set_reccurrence(e.recurrence.clone().unwrap(), true);
        }

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
    use crate::event::repos::InMemoryEventRepo;

    #[actix_web::main]
    #[test]
    async fn update_notexisting_event() {
        let event_repo = Arc::new(InMemoryEventRepo::new());

        let usecase = UpdateEventUseCase {
            event_id: String::from(""),
            start_ts: Some(500),
            duration: Some(800),
            rrule_options: None,
            busy: Some(false),
            user_id: String::from("cool"),
        };
        let ctx = Context::create_inmemory();
        let res = usecase.perform(&ctx).await;
        assert!(res.is_err());
    }
}
