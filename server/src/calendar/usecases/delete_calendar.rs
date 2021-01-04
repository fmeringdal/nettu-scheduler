use crate::event::repos::IEventRepo;
use crate::{api::Context, shared::auth::protect_route};
use crate::{
    calendar::repos::ICalendarRepo,
    shared::usecase::{perform, Usecase},
};
use actix_web::{web, HttpResponse};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct DeleteCalendarReq {
    calendar_id: String,
}

pub async fn delete_calendar_controller(
    http_req: web::HttpRequest,
    req: web::Path<DeleteCalendarReq>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let user = match protect_route(&http_req, &ctx).await {
        Ok(u) => u,
        Err(res) => return res,
    };

    let usecase = DeleteCalendarUseCase {
        user_id: user.id,
        calendar_id: req.calendar_id.clone(),
    };

    let res = perform(usecase, &ctx).await;
    match res {
        Ok(_) => HttpResponse::Ok().body("Calendar deleted"),
        Err(e) => match e {
            UseCaseErrors::NotFoundError => HttpResponse::NotFound().finish(),
            UseCaseErrors::UnableToDelete => HttpResponse::InternalServerError().finish(),
        },
    }
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFoundError,
    UnableToDelete,
}

pub struct DeleteCalendarUseCase {
    calendar_id: String,
    user_id: String,
}

#[async_trait::async_trait(?Send)]
impl Usecase for DeleteCalendarUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn perform(&self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let calendar = ctx.repos.calendar_repo.find(&self.calendar_id).await;
        match calendar {
            Some(calendar) if calendar.user_id == self.user_id => {
                ctx.repos.calendar_repo.delete(&calendar.id).await;
                let repo_res = ctx.repos.event_repo.delete_by_calendar(&calendar.id).await;
                if repo_res.is_err() {
                    return Err(UseCaseErrors::UnableToDelete);
                }

                Ok(())
            }
            _ => Err(UseCaseErrors::NotFoundError),
        }
    }
}
