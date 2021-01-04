use crate::{
    account::domain::Account,
    calendar::repos::ICalendarRepo,
    service::{domain::Service, repos::IServiceRepo},
    shared::{
        auth::protect_account_route,
        usecase::{perform, Usecase},
    },
    user::domain::User,
};
use crate::{api::Context, user::repos::IUserRepo};
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
) -> HttpResponse {
    let account = match protect_account_route(&http_req, &ctx).await {
        Ok(a) => a,
        Err(res) => return res,
    };

    let user_id = User::create_id(&account.id, &path_params.user_id);
    let usecase = UpdateServiceUserUseCase {
        account,
        calendar_ids: body.calendar_ids.to_owned(),
        service_id: path_params.service_id.to_owned(),
        user_id,
    };
    let res = perform(usecase, &ctx).await;

    match res {
        Ok(_) => HttpResponse::Ok().body("Service successfully updated"),
        Err(e) => match e {
            UsecaseErrors::StorageError => HttpResponse::InternalServerError().finish(),
            UsecaseErrors::ServiceNotFoundError => {
                HttpResponse::NotFound().body("The requested service was not found")
            }
            UsecaseErrors::UserNotFoundError => {
                HttpResponse::NotFound().body("The specified user was not found")
            }
            UsecaseErrors::CalendarNotOwnedByUser(calendar_id) => {
                HttpResponse::Forbidden().body(format!(
                    "The calendar: {}, was not found among the calendars for the specified user",
                    calendar_id
                ))
            }
        },
    }
}

struct UpdateServiceUserUseCase {
    pub account: Account,
    pub service_id: String,
    pub user_id: String,
    pub calendar_ids: Vec<String>,
}

struct UsecaseRes {
    pub service: Service,
}

#[derive(Debug)]
enum UsecaseErrors {
    StorageError,
    ServiceNotFoundError,
    UserNotFoundError,
    CalendarNotOwnedByUser(String),
}

#[async_trait::async_trait(?Send)]
impl Usecase for UpdateServiceUserUseCase {
    type Response = UsecaseRes;

    type Errors = UsecaseErrors;

    type Context = Context;

    async fn perform(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let mut service = match ctx.repos.service_repo.find(&self.service_id).await {
            Some(service) if service.account_id == self.account.id => service,
            _ => return Err(UsecaseErrors::ServiceNotFoundError),
        };

        let user_resource = match service.find_user_mut(&self.user_id) {
            Some(res) => res,
            _ => return Err(UsecaseErrors::UserNotFoundError),
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
                return Err(UsecaseErrors::CalendarNotOwnedByUser(calendar_id.clone()));
            }
        }

        user_resource.set_calendar_ids(&self.calendar_ids);

        let res = ctx.repos.service_repo.save(&service).await;
        match res {
            Ok(_) => Ok(UsecaseRes { service }),
            Err(_) => Err(UsecaseErrors::StorageError),
        }
    }
}
