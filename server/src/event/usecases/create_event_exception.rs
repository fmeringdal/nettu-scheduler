use crate::shared::usecase::{perform, Usecase};
use crate::{api::Context, event::repos::IEventRepo, shared::auth::protect_route};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateEventExceptionPathParams {
    event_id: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventExceptionBody {
    exception_ts: i64,
}

pub async fn create_event_exception_controller(
    http_req: HttpRequest,
    path_params: web::Path<CreateEventExceptionPathParams>,
    body: web::Json<CreateEventExceptionBody>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let user = match protect_route(&http_req, &ctx).await {
        Ok(u) => u,
        Err(res) => return res,
    };

    let usecase = CreateEventExceptionUseCase {
        event_id: path_params.event_id.clone(),
        exception_ts: body.exception_ts,
        user_id: user.id.clone(),
    };

    let res = perform(usecase, &ctx).await;
    match res {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => match e {
            UsecaseErrors::NotFoundError => HttpResponse::NotFound().finish(),
            UsecaseErrors::StorageError => HttpResponse::InternalServerError().finish(),
        },
    }
}

pub struct CreateEventExceptionUseCase {
    event_id: String,
    exception_ts: i64,
    user_id: String,
}

#[derive(Debug)]
pub enum UsecaseErrors {
    NotFoundError,
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl Usecase for CreateEventExceptionUseCase {
    type Response = ();

    type Errors = UsecaseErrors;

    type Context = Context;

    async fn perform(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let mut event = match ctx.repos.event_repo.find(&self.event_id).await {
            Some(event) if event.user_id == self.user_id => event,
            _ => return Err(UsecaseErrors::NotFoundError),
        };

        event.exdates.push(self.exception_ts);

        let repo_res = ctx.repos.event_repo.save(&event).await;
        if repo_res.is_err() {
            return Err(UsecaseErrors::StorageError);
        }

        Ok(())
    }
}
