use crate::event::repo::IEventRepo;
use crate::{api::Context, event::domain::event::RRuleOptions};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
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
    body: web::Json<UpdateEventBody>,
    params: web::Path<EventPathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let req = UpdateEventReq {
        duration: body.duration,
        start_ts: body.start_ts,
        rrule_options: body.rrule_options.clone(),
        event_id: params.event_id.clone(),
        busy: body.busy,
    };
    let ctx = UpdateEventUseCaseCtx {
        event_repo: ctx.repos.event_repo.clone(),
    };
    let res = update_event_usecase(req, ctx).await;
    match res {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::UnprocessableEntity().finish(),
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateEventReq {
    pub event_id: String,
    pub start_ts: Option<i64>,
    pub busy: Option<bool>,
    pub duration: Option<i64>,
    pub rrule_options: Option<RRuleOptions>,
}

pub struct UpdateEventUseCaseCtx {
    pub event_repo: Arc<dyn IEventRepo>,
}

pub enum UpdateEventErrors {
    NotFoundError,
}
async fn update_event_usecase(
    req: UpdateEventReq,
    ctx: UpdateEventUseCaseCtx,
) -> Result<(), UpdateEventErrors> {
    let e = ctx.event_repo.find(&req.event_id).await;

    if e.is_none() {
        return Err(UpdateEventErrors::NotFoundError {});
    }

    let mut should_update_endtime = false;
    let mut e = e.unwrap();
    if let Some(start_ts) = req.start_ts {
        e.start_ts = start_ts;
        should_update_endtime = true;
    }
    if let Some(duration) = req.duration {
        e.duration = duration;
        should_update_endtime = true;
    }
    if let Some(busy) = req.busy {
        e.busy = busy;
    }

    if let Some(rrule_opts) = req.rrule_options.clone() {
        e.set_reccurrence(rrule_opts, true);
    } else if should_update_endtime && e.recurrence.is_some() {
        e.set_reccurrence(e.recurrence.clone().unwrap(), true);
    }

    ctx.event_repo.save(&e).await;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{api::Repos, event::repo::InMemoryEventRepo};

    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::main]
    #[test]
    async fn update_notexisting_event() {
        let event_repo = Arc::new(InMemoryEventRepo::new());

        let ctx = UpdateEventUseCaseCtx { event_repo };
        let req = UpdateEventReq {
            event_id: String::from(""),
            start_ts: Some(500),
            duration: Some(800),
            rrule_options: None,
            busy: Some(false),
        };
        let res = update_event_usecase(req, ctx).await;
        assert!(res.is_err());
    }
}
