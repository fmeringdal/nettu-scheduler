use crate::error::NettuError;
use crate::shared::{
    auth::protect_account_route,
    usecase::{execute, UseCase},
};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_core::{Account, Service, ServiceResource, TimePlan, User};
use nettu_scheduler_infra::NettuContext;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PathParams {
    service_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyParams {
    user_id: String,
    pub availibility: Option<TimePlan>,
    pub busy: Option<Vec<String>>,
    pub buffer: Option<i64>,
    pub closest_booking_time: Option<i64>,
    pub furthest_booking_time: Option<i64>,
}

pub async fn add_user_to_service_controller(
    http_req: HttpRequest,
    body: web::Json<BodyParams>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let user_id = User::create_id(&account.id, &body.user_id);
    let usecase = AddUserToServiceUseCase {
        account,
        service_id: path_params.service_id.to_owned(),
        user_id,
        availibility: body.availibility.to_owned(),
        busy: body.busy.to_owned(),
        buffer: body.buffer,
        closest_booking_time: body.closest_booking_time,
        furthest_booking_time: body.furthest_booking_time,
    };

    execute(usecase, &ctx).await
        .map(|_| HttpResponse::Ok().body("Service successfully updated"))
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::ServiceNotFoundError => NettuError::NotFound("The requested service was not found".into()),
            UseCaseErrors::UserNotFoundError => NettuError::NotFound("The specified user was not found".into()),
            UseCaseErrors::InvalidValue(e) => e.to_nettu_error(),
            UseCaseErrors::UserAlreadyInService => NettuError::Conflict("The specified user is already registered on the service, can not add the user more than once.".into()),
        })
}

struct AddUserToServiceUseCase {
    pub account: Account,
    pub service_id: String,
    pub user_id: String,
    pub availibility: Option<TimePlan>,
    pub busy: Option<Vec<String>>,
    pub buffer: Option<i64>,
    pub closest_booking_time: Option<i64>,
    pub furthest_booking_time: Option<i64>,
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
    InvalidValue(UpdateServiceResourceError),
}

#[async_trait::async_trait(?Send)]
impl UseCase for AddUserToServiceUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = NettuContext;

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

        let mut user_resource = ServiceResource::new(&self.user_id, TimePlan::Empty, vec![]);

        update_resource_values(
            &mut user_resource,
            &ServiceResourceUpdate {
                availibility: self.availibility.clone(),
                buffer: self.buffer.clone(),
                busy: self.busy.clone(),
                closest_booking_time: self.closest_booking_time,
                furthest_booking_time: self.furthest_booking_time,
            },
            ctx,
        )
        .await
        .map_err(|e| UseCaseErrors::InvalidValue(e))?;

        service.add_user(user_resource);

        let res = ctx.repos.service_repo.save(&service).await;
        match res {
            Ok(_) => Ok(UseCaseRes { service }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}

pub struct ServiceResourceUpdate {
    pub availibility: Option<TimePlan>,
    pub busy: Option<Vec<String>>,
    pub buffer: Option<i64>,
    pub closest_booking_time: Option<i64>,
    pub furthest_booking_time: Option<i64>,
}

#[derive(Debug)]
pub enum UpdateServiceResourceError {
    InvalidBuffer,
    CalendarNotOwnedByUser(String),
    ScheduleNotOwnedByUser(String),
    InvalidBookingTimespan(String),
}

impl UpdateServiceResourceError {
    pub fn to_nettu_error(self) -> NettuError {
        match self {
            Self::InvalidBuffer => {
                NettuError::BadClientData("The provided buffer was invalid, it should be netween 0 and 12 hours specified in minutes.".into())
            }
            Self::CalendarNotOwnedByUser(calendar_id) => NettuError::NotFound(format!("The calendar: {}, was not found among the calendars for the specified user", calendar_id)),
            Self::ScheduleNotOwnedByUser(schedule_id) => {
                NettuError::NotFound(format!(
                    "The schedule with id: {}, was not found among the schedules for the specified user",
                    schedule_id
                ))
            }
            Self::InvalidBookingTimespan(e) => {
                NettuError::BadClientData(e)
            }
        }
    }
}

pub async fn update_resource_values(
    user_resource: &mut ServiceResource,
    update: &ServiceResourceUpdate,
    ctx: &NettuContext,
) -> Result<(), UpdateServiceResourceError> {
    let user_calendars = ctx
        .repos
        .calendar_repo
        .find_by_user(&user_resource.user_id)
        .await
        .into_iter()
        .map(|cal| cal.id)
        .collect::<Vec<_>>();

    if let Some(busy) = &update.busy {
        for calendar_id in busy {
            if !user_calendars.contains(calendar_id) {
                return Err(UpdateServiceResourceError::CalendarNotOwnedByUser(
                    calendar_id.clone(),
                ));
            }
        }
        user_resource.set_busy(busy.clone());
    }

    if let Some(availibility) = &update.availibility {
        match availibility {
            TimePlan::Calendar(id) => {
                if !user_calendars.contains(id) {
                    return Err(UpdateServiceResourceError::CalendarNotOwnedByUser(
                        id.clone(),
                    ));
                }
            }
            TimePlan::Schedule(id) => {
                let schedule = ctx.repos.schedule_repo.find(id).await;
                match schedule {
                    Some(schedule) if schedule.user_id == user_resource.user_id => {}
                    _ => {
                        return Err(UpdateServiceResourceError::ScheduleNotOwnedByUser(
                            id.clone(),
                        ))
                    }
                }
            }
            _ => (),
        };
        user_resource.set_availibility(availibility.clone());
    }

    if let Some(buffer) = update.buffer {
        if !user_resource.set_buffer(buffer) {
            return Err(UpdateServiceResourceError::InvalidBuffer);
        }
    }

    if let Some(closest_booking_time) = update.closest_booking_time {
        if closest_booking_time < 0 {
            return Err(UpdateServiceResourceError::InvalidBookingTimespan(
                "Closest booking time cannot be negative.".into(),
            ));
        }
    }
    if let Some(furthest_booking_time) = update.furthest_booking_time {
        if furthest_booking_time < 0 {
            return Err(UpdateServiceResourceError::InvalidBookingTimespan(
                "Furthest booking time cannot be negative.".into(),
            ));
        }
    }

    Ok(())
}
