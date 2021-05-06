use super::add_user_to_service::{
    update_resource_values, ServiceResourceUpdate, UpdateServiceResourceError,
};
use crate::{
    error::NettuError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::update_service_user::*;
use nettu_scheduler_domain::{Account, BusyCalendar, Service, TimePlan, ID};
use nettu_scheduler_infra::NettuContext;

pub async fn update_service_user_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = UpdateServiceUserUseCase {
        account,
        service_id: path_params.service_id.to_owned(),
        user_id: path_params.user_id.to_owned(),
        availibility: body.availibility.to_owned(),
        busy: body.busy.to_owned(),
        buffer: body.buffer,
        closest_booking_time: body.closest_booking_time,
        furthest_booking_time: body.furthest_booking_time,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.service)))
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::ServiceNotFound => {
                NettuError::NotFound("The requested service was not found".into())
            }
            UseCaseErrors::UserNotFound => {
                NettuError::NotFound("The specified user was not found".into())
            }
            UseCaseErrors::InvalidValue(e) => e.to_nettu_error(),
        })
}

#[derive(Debug)]
struct UpdateServiceUserUseCase {
    pub account: Account,
    pub service_id: ID,
    pub user_id: ID,
    pub availibility: Option<TimePlan>,
    pub busy: Option<Vec<BusyCalendar>>,
    pub buffer: Option<i64>,
    pub closest_booking_time: Option<i64>,
    pub furthest_booking_time: Option<i64>,
}

#[derive(Debug)]
struct UseCaseRes {
    pub service: Service,
}

#[derive(Debug)]
enum UseCaseErrors {
    StorageError,
    ServiceNotFound,
    UserNotFound,
    InvalidValue(UpdateServiceResourceError),
}

#[async_trait::async_trait(?Send)]
impl UseCase for UpdateServiceUserUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    const NAME: &'static str = "UpdateServiceUser";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let mut service = match ctx.repos.service_repo.find(&self.service_id).await {
            Some(service) if service.account_id == self.account.id => service,
            _ => return Err(UseCaseErrors::ServiceNotFound),
        };

        let mut user_resource = match service.find_user_mut(&self.user_id) {
            Some(res) => res,
            _ => return Err(UseCaseErrors::UserNotFound),
        };

        update_resource_values(
            &mut user_resource,
            &ServiceResourceUpdate {
                availibility: self.availibility.clone(),
                busy: self.busy.clone(),
                buffer: self.buffer,
                closest_booking_time: self.closest_booking_time,
                furthest_booking_time: self.furthest_booking_time,
            },
            ctx,
        )
        .await
        .map_err(UseCaseErrors::InvalidValue)?;

        let res = ctx.repos.service_repo.save(&service).await;
        match res {
            Ok(_) => Ok(UseCaseRes { service }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}
