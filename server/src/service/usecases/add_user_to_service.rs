use crate::api::Context;
use crate::{
    account::domain::Account,
    service::domain::{Service, ServiceResource},
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
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyParams {
    user_id: String,
    calendar_ids: Vec<String>,
}

pub async fn add_user_to_service_controller(
    http_req: HttpRequest,
    body: web::Json<BodyParams>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let account = match protect_account_route(&http_req, &ctx).await {
        Ok(a) => a,
        Err(res) => return res,
    };

    let user_id = User::create_id(&account.id, &body.user_id);
    let usecase = AddUserToServiceUseCase {
        account,
        calendar_ids: body.calendar_ids.to_owned(),
        service_id: path_params.service_id.to_owned(),
        user_id,
    };
    let res = execute(usecase, &ctx).await;

    match res {
        Ok(_) =>
            HttpResponse::Ok().body("Service successfully updated"),
        Err(e) => match e {
            UseCaseErrors::StorageError => HttpResponse::InternalServerError().finish(),
            UseCaseErrors::ServiceNotFoundError => HttpResponse::NotFound().body("The requested service was not found"),
            UseCaseErrors::UserNotFoundError => HttpResponse::NotFound().body("The specified user was not found"),
            UseCaseErrors::UserAlreadyInService => HttpResponse::Forbidden().body("The specified user is already registered on the service, can not add the user more than once."),
            UseCaseErrors::CalendarNotOwnedByUser(calendar_id) => HttpResponse::Forbidden().body(format!("The calendar: {}, was not found among the calendars for the specified user", calendar_id)),
        }
    }
}

struct AddUserToServiceUseCase {
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
    UserAlreadyInService,
    CalendarNotOwnedByUser(String),
}

#[async_trait::async_trait(?Send)]
impl Usecase for AddUserToServiceUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let _user = match ctx.repos.user_repo.find(&self.user_id).await {
            Some(user) if user.account_id == self.account.id => user,
            _ => return Err(UseCaseErrors::UserNotFoundError),
        };

        let mut service = match ctx.repos.service_repo.find(&self.service_id).await {
            Some(service) if service.account_id == self.account.id => service,
            _ => return Err(UseCaseErrors::ServiceNotFoundError),
        };

        if let Some(_user_resource) = service.find_user(&self.user_id) {
            return Err(UseCaseErrors::UserAlreadyInService);
        }

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

        let user_resource = ServiceResource::new(&self.user_id, &self.calendar_ids);
        service.add_user(user_resource);

        let res = ctx.repos.service_repo.save(&service).await;
        match res {
            Ok(_) => Ok(UseCaseRes { service }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}
