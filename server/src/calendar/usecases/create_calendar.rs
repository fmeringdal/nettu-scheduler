use crate::{
    api::Context,
    shared::auth::{protect_account_route, protect_route},
    user::repos::IUserRepo,
};
use crate::{
    calendar::domain::calendar::Calendar,
    shared::usecase::{perform, Usecase},
};
use crate::{calendar::repos::ICalendarRepo, user::domain::User};
use actix_web::{web, HttpResponse};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AdminControllerPathParams {
    user_id: String,
}

pub async fn create_calendar_admin_controller(
    http_req: web::HttpRequest,
    path_params: web::Json<AdminControllerPathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let account = match protect_account_route(&http_req, &ctx).await {
        Ok(u) => u,
        Err(res) => return res,
    };

    let user_id = User::create_id(&account.id, &path_params.user_id);
    let usecase = CreateCalendarUseCase { user_id };
    let res = perform(usecase, &ctx).await;

    match res {
        Ok(json) => HttpResponse::Created().json(json),
        Err(_) => HttpResponse::UnprocessableEntity().finish(),
    }
}

pub async fn create_calendar_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let user = match protect_route(&http_req, &ctx).await {
        Ok(u) => u,
        Err(res) => return res,
    };
    let usecase = CreateCalendarUseCase { user_id: user.id };
    let res = perform(usecase, &ctx).await;

    match res {
        Ok(json) => HttpResponse::Created().json(json),
        Err(_) => HttpResponse::UnprocessableEntity().finish(),
    }
}

struct CreateCalendarUseCase {
    pub user_id: String,
}

#[derive(Debug)]
enum UseCaseErrors {
    UserNotFoundError,
    StorageError,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UseCaseRes {
    pub calendar_id: String,
}

#[async_trait::async_trait(?Send)]
impl Usecase for CreateCalendarUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn perform(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let user = ctx.repos.user_repo.find(&self.user_id).await;
        if user.is_none() {
            return Err(UseCaseErrors::UserNotFoundError);
        }

        let calendar = Calendar {
            id: ObjectId::new().to_string(),
            user_id: self.user_id.clone(),
        };
        let res = ctx.repos.calendar_repo.insert(&calendar).await;
        match res {
            Ok(_) => Ok(UseCaseRes {
                calendar_id: calendar.id.clone(),
            }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}
