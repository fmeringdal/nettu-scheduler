use crate::{api::Context, shared::auth::protect_route};
use crate::{
    event::repos::IEventRepo,
    shared::usecase::{execute, Usecase},
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PathParams {
    event_id: String,
}

pub async fn delete_event_controller(
    http_req: HttpRequest,
    params: web::Path<PathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let user = match protect_route(&http_req, &ctx).await {
        Ok(u) => u,
        Err(res) => return res,
    };

    let usecase = DeleteEventUseCase {
        user_id: user.id.clone(),
        event_id: params.event_id.clone(),
    };
    let res = execute(usecase, &ctx).await;
    return match res {
        Ok(_) => HttpResponse::Ok().body("Event deleted"),
        Err(_) => HttpResponse::NotFound().finish(),
    };
}

pub struct DeleteEventUseCase {
    pub user_id: String,
    pub event_id: String,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFoundError,
}

#[async_trait::async_trait(?Send)]
impl Usecase for DeleteEventUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    type Context = Context;

    // TODO: use only one db call
    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let e = ctx.repos.event_repo.find(&self.event_id).await;
        match e {
            Some(event) if event.user_id == self.user_id => {
                ctx.repos.event_repo.delete(&event.id).await;
                Ok(())
            }
            _ => Err(UseCaseErrors::NotFoundError {}),
        }
    }
}
