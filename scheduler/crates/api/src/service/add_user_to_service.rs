use crate::error::NettuError;
use crate::shared::{
    auth::protect_account_route,
    usecase::{execute, UseCase},
};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::add_user_to_service::*;
use nettu_scheduler_domain::{Account, ServiceResource, TimePlan, ID};
use nettu_scheduler_infra::NettuContext;

pub async fn add_user_to_service_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = AddUserToServiceUseCase {
        account,
        service_id: path_params.service_id.to_owned(),
        user_id: body.user_id.to_owned(),
        availability: body.availability.to_owned(),
        buffer_before: body.buffer_before,
        buffer_after: body.buffer_after,
        closest_booking_time: body.closest_booking_time,
        furthest_booking_time: body.furthest_booking_time,
    };

    execute(usecase, &ctx)
        .await
        .map(|res| HttpResponse::Ok().json(APIResponse::new(res.user)))
        .map_err(NettuError::from)
}

#[derive(Debug)]
struct AddUserToServiceUseCase {
    pub account: Account,
    pub service_id: ID,
    pub user_id: ID,
    pub availability: Option<TimePlan>,
    pub buffer_before: Option<i64>,
    pub buffer_after: Option<i64>,
    pub closest_booking_time: Option<i64>,
    pub furthest_booking_time: Option<i64>,
}

#[derive(Debug)]
struct UseCaseRes {
    pub user: ServiceResource,
}

#[derive(Debug)]
enum UseCaseError {
    ServiceNotFound,
    UserNotFound,
    UserAlreadyInService,
    InvalidValue(UpdateServiceResourceError),
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::ServiceNotFound => Self::NotFound("The requested service was not found".into()),
            UseCaseError::UserNotFound => Self::NotFound("The specified user was not found".into()),
            UseCaseError::UserAlreadyInService => Self::Conflict("The specified user is already registered on the service, can not add the user more than once.".into()),
            UseCaseError::InvalidValue(e) => e.to_nettu_error(),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for AddUserToServiceUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;

    const NAME: &'static str = "AddUserToService";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        if ctx
            .repos
            .users
            .find_by_account_id(&self.user_id, &self.account.id)
            .await
            .is_none()
        {
            return Err(UseCaseError::UserNotFound);
        }

        let service = match ctx.repos.services.find(&self.service_id).await {
            Some(service) if service.account_id == self.account.id => service,
            _ => return Err(UseCaseError::ServiceNotFound),
        };

        let mut user_resource =
            ServiceResource::new(self.user_id.clone(), service.id.clone(), TimePlan::Empty);

        update_resource_values(
            &mut user_resource,
            &ServiceResourceUpdate {
                availability: self.availability.clone(),
                buffer_after: self.buffer_after,
                buffer_before: self.buffer_before,
                closest_booking_time: self.closest_booking_time,
                furthest_booking_time: self.furthest_booking_time,
            },
            ctx,
        )
        .await
        .map_err(UseCaseError::InvalidValue)?;

        ctx.repos
            .service_users
            .insert(&user_resource)
            .await
            .map(|_| UseCaseRes {
                user: user_resource,
            })
            .map_err(|_| UseCaseError::UserAlreadyInService)
    }
}

#[derive(Debug)]
pub struct ServiceResourceUpdate {
    pub availability: Option<TimePlan>,
    pub buffer_after: Option<i64>,
    pub buffer_before: Option<i64>,
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
    pub fn to_nettu_error(&self) -> NettuError {
        match self {
            Self::InvalidBuffer => {
                NettuError::BadClientData("The provided buffer was invalid, it should be between 0 and 12 hours specified in minutes.".into())
            }
            Self::CalendarNotOwnedByUser(calendar_id) => NettuError::NotFound(format!("The calendar: {}, was not found among the calendars for the specified user", calendar_id)),
            Self::ScheduleNotOwnedByUser(schedule_id) => {
                NettuError::NotFound(format!(
                    "The schedule with id: {}, was not found among the schedules for the specified user",
                    schedule_id
                ))
            }
            Self::InvalidBookingTimespan(e) => {
                NettuError::BadClientData(e.to_string())
            }
        }
    }
}

pub async fn update_resource_values(
    user_resource: &mut ServiceResource,
    update: &ServiceResourceUpdate,
    ctx: &NettuContext,
) -> Result<(), UpdateServiceResourceError> {
    if let Some(availability) = &update.availability {
        match availability {
            TimePlan::Calendar(id) => {
                match ctx.repos.calendars.find(id).await {
                    Some(cal) if cal.user_id == user_resource.user_id => {}
                    _ => {
                        return Err(UpdateServiceResourceError::CalendarNotOwnedByUser(
                            id.to_string(),
                        ));
                    }
                };
            }
            TimePlan::Schedule(id) => match ctx.repos.schedules.find(id).await {
                Some(schedule) if schedule.user_id == user_resource.user_id => {}
                _ => {
                    return Err(UpdateServiceResourceError::ScheduleNotOwnedByUser(
                        id.to_string(),
                    ))
                }
            },
            _ => (),
        };
        user_resource.set_availability(availability.clone());
    }

    if let Some(buffer) = update.buffer_after {
        if !user_resource.set_buffer_after(buffer) {
            return Err(UpdateServiceResourceError::InvalidBuffer);
        }
    }
    if let Some(buffer) = update.buffer_before {
        if !user_resource.set_buffer_before(buffer) {
            return Err(UpdateServiceResourceError::InvalidBuffer);
        }
    }

    if let Some(closest_booking_time) = update.closest_booking_time {
        if closest_booking_time < 0 {
            return Err(UpdateServiceResourceError::InvalidBookingTimespan(
                "Closest booking time cannot be negative.".into(),
            ));
        }
        user_resource.closest_booking_time = closest_booking_time;
    }

    if let Some(furthest_booking_time) = &update.furthest_booking_time {
        if *furthest_booking_time < 0 {
            return Err(UpdateServiceResourceError::InvalidBookingTimespan(
                "Furthest booking time cannot be negative.".into(),
            ));
        }
    }
    user_resource.furthest_booking_time = update.furthest_booking_time;

    Ok(())
}
