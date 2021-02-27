use crate::{
    error::NettuError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_core::{Account, Service, TimePlan, User};
use nettu_scheduler_infra::NettuContext;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PathParams {
    service_id: String,
    user_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyParams {
    availibility: TimePlan,
    busy: Vec<String>,
    buffer: i64,
}

pub async fn update_service_user_controller(
    http_req: HttpRequest,
    body: web::Json<BodyParams>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let user_id = User::create_id(&account.id, &path_params.user_id);
    let usecase = UpdateServiceUserUseCase {
        account,
        service_id: path_params.service_id.to_owned(),
        user_id,
        availibility: body.availibility.to_owned(),
        busy: body.busy.to_owned(),
        buffer: body.buffer,
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
            UseCaseErrors::InvalidBuffer => {
                NettuError::BadClientData("The provided buffer was invalid, it should be netween 0 and 12 hours specified in minutes.".into())
            }
            UseCaseErrors::ScheduleNotOwnedByUser(schedule_id) => {
                NettuError::NotFound(format!(
                    "The schedule with id: {}, was not found among the schedules for the specified user",
                    schedule_id
                ))
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
    pub availibility: TimePlan,
    pub busy: Vec<String>,
    pub buffer: i64,
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
    ScheduleNotOwnedByUser(String),
    InvalidBuffer,
}

#[async_trait::async_trait(?Send)]
impl UseCase for UpdateServiceUserUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = NettuContext;

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

        for calendar_id in &self.busy {
            if !user_calendars.contains(calendar_id) {
                return Err(UseCaseErrors::CalendarNotOwnedByUser(calendar_id.clone()));
            }
        }
        match &self.availibility {
            TimePlan::Calendar(id) => {
                if !user_calendars.contains(id) {
                    return Err(UseCaseErrors::CalendarNotOwnedByUser(id.clone()));
                }
            }
            TimePlan::Schedule(id) => {
                let schedule = ctx.repos.schedule_repo.find(id).await;
                match schedule {
                    Some(schedule) if schedule.user_id == self.user_id => {}
                    _ => return Err(UseCaseErrors::ScheduleNotOwnedByUser(id.clone())),
                }
            }
            _ => (),
        };

        if !user_resource.set_buffer(self.buffer) {
            return Err(UseCaseErrors::InvalidBuffer);
        }

        user_resource.set_availibility(self.availibility.clone());
        user_resource.set_busy(self.busy.clone());

        let res = ctx.repos.service_repo.save(&service).await;
        match res {
            Ok(_) => Ok(UseCaseRes { service }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}
