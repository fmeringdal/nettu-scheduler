use crate::shared::auth::protect_account_route;
use crate::shared::usecase::{perform, Usecase};
use crate::{
    api::Context,
    user::domain::{User, UserDTO},
};
use actix_web::{web, HttpRequest, HttpResponse};

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyParams {
    pub user_id: String,
}

pub async fn create_user_controller(
    http_req: HttpRequest,
    body: web::Json<BodyParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let account = match protect_account_route(&http_req, &ctx).await {
        Ok(a) => a,
        Err(res) => return res,
    };

    let usecase = CreateUserUseCase {
        account_id: account.id.clone(),
        external_user_id: body.user_id.clone(),
    };
    let res = perform(usecase, &ctx.into_inner()).await;

    match res {
        Ok(usecase_res) => {
            let res = UserDTO::new(&usecase_res.user);
            HttpResponse::Created().json(res)
        }
        Err(e) => match e {
            UsecaseErrors::StorageError => HttpResponse::InternalServerError().finish(),
            UsecaseErrors::UserAlreadyExists => HttpResponse::Conflict()
                .body("A user with that userId already exist. UserIds need to be unique."),
        },
    }
}

pub struct CreateUserUseCase {
    pub account_id: String,
    pub external_user_id: String,
}
pub struct UsecaseRes {
    pub user: User,
}

#[derive(Debug)]
pub enum UsecaseErrors {
    StorageError,
    UserAlreadyExists,
}

#[async_trait::async_trait(?Send)]
impl Usecase for CreateUserUseCase {
    type Response = UsecaseRes;
    type Errors = UsecaseErrors;
    type Context = Context;

    async fn perform(&self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let user = User::new(&self.account_id, &self.external_user_id);

        if let Some(_existing_user) = ctx.repos.user_repo.find(&user.id).await {
            return Err(UsecaseErrors::UserAlreadyExists);
        }

        let res = ctx.repos.user_repo.insert(&user).await;
        match res {
            Ok(_) => Ok(UsecaseRes { user }),
            Err(_) => Err(UsecaseErrors::StorageError),
        }
    }
}
