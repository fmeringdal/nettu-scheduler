use crate::api::Context;
use crate::{
    account::domain::Account,
    api::NettuError,
    service::domain::Service,
    shared::{
        auth::protect_account_route,
        usecase::{execute, Usecase},
    },
    user::domain::User,
};
use actix_web::{web, HttpRequest, HttpResponse};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct PathParams {
    service_id: String,
    user_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyParams {
    calendar_ids: Vec<String>,
}

pub async fn update_service_user_controller(
    http_req: HttpRequest,
    body: web::Json<BodyParams>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let user_id = User::create_id(&account.id, &path_params.user_id);
    let usecase = UpdateServiceUserUseCase {
        account,
        calendar_ids: body.calendar_ids.to_owned(),
        service_id: path_params.service_id.to_owned(),
        user_id,
    };

    execute(usecase, &ctx).await
        .map(|_| HttpResponse::Ok().body("Service successfully updated"))
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::ServiceNotFoundError => {
                NettuError::NotFound("The requested service was not found".into())
            }
            UseCaseErrors::UserNotFoundError => {
                NettuError::NotFound("The specified user was not found".into())
            }
            UseCaseErrors::CalendarNotOwnedByUser(calendar_id) => {
                NettuError::NotFound(format!(
                    "The calendar with id: {}, was not found among the calendars for the specified user",
                    calendar_id
                ))
            }
        })
}

struct UpdateServiceUserUseCase {
    pub account: Account,
    pub service_id: String,
    pub user_id: String,
    pub calendar_ids: Vec<String>,
}

struct UseCaseRes {
    pub service: Service,
}

#[derive(Debug)]
enum UseCaseErrors {
    StorageError,
    ServiceNotFoundError,
    UserNotFoundError,
    CalendarNotOwnedByUser(String),
}

#[async_trait::async_trait(?Send)]
impl Usecase for UpdateServiceUserUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let mut service = match ctx.repos.service_repo.find(&self.service_id).await {
            Some(service) if service.account_id == self.account.id => service,
            _ => return Err(UseCaseErrors::ServiceNotFoundError),
        };

        let user_resource = match service.find_user_mut(&self.user_id) {
            Some(res) => res,
            _ => return Err(UseCaseErrors::UserNotFoundError),
        };

        let user_calendars = ctx
            .repos
            .calendar_repo
            .find_by_user(&self.user_id)
            .await
            .into_iter()
            .map(|cal| cal.id)
            .collect::<Vec<_>>();

        for calendar_id in &self.calendar_ids {
            if !user_calendars.contains(calendar_id) {
                return Err(UseCaseErrors::CalendarNotOwnedByUser(calendar_id.clone()));
            }
        }

        user_resource.set_calendar_ids(&self.calendar_ids);

        let res = ctx.repos.service_repo.save(&service).await;
        match res {
            Ok(_) => Ok(UseCaseRes { service }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}
