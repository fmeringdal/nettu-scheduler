use crate::error::NettuError;
use crate::shared::{
    auth::protect_account_route,
    usecase::{execute, UseCase},
};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::add_user_to_service::*;
use nettu_scheduler_domain::{
    providers::google::GoogleCalendarAccessRole, Account, BusyCalendar, ServiceResource, TimePlan,
    ID,
};
use nettu_scheduler_infra::{google_calendar::GoogleCalendarProvider, NettuContext};

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
        busy: body.busy.to_owned(),
        buffer_before: body.buffer_before,
        buffer_after: body.buffer_after,
        closest_booking_time: body.closest_booking_time,
        furthest_booking_time: body.furthest_booking_time,
    };

    execute(usecase, &ctx).await
        .map(|res| HttpResponse::Ok().json(APIResponse::new(res.user)))
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::ServiceNotFound => NettuError::NotFound("The requested service was not found".into()),
            UseCaseErrors::UserNotFound => NettuError::NotFound("The specified user was not found".into()),
            UseCaseErrors::UserAlreadyInService => NettuError::Conflict("The specified user is already registered on the service, can not add the user more than once.".into()),
            UseCaseErrors::InvalidValue(e) => e.to_nettu_error(),
        })
}

#[derive(Debug)]
struct AddUserToServiceUseCase {
    pub account: Account,
    pub service_id: ID,
    pub user_id: ID,
    pub availability: Option<TimePlan>,
    pub busy: Option<Vec<BusyCalendar>>,
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
enum UseCaseErrors {
    StorageError,
    ServiceNotFound,
    UserNotFound,
    UserAlreadyInService,
    InvalidValue(UpdateServiceResourceError),
}

#[async_trait::async_trait(?Send)]
impl UseCase for AddUserToServiceUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    const NAME: &'static str = "AddUserToService";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        if ctx
            .repos
            .users
            .find_by_account_id(&self.user_id, &self.account.id)
            .await
            .is_none()
        {
            return Err(UseCaseErrors::UserNotFound);
        }

        let service = match ctx.repos.services.find(&self.service_id).await {
            Some(service) if service.account_id == self.account.id => service,
            _ => return Err(UseCaseErrors::ServiceNotFound),
        };

        let mut user_resource = ServiceResource::new(
            self.user_id.clone(),
            service.id.clone(),
            TimePlan::Empty,
            Vec::new(),
        );

        update_resource_values(
            &mut user_resource,
            &ServiceResourceUpdate {
                availability: self.availability.clone(),
                busy: self.busy.clone(),
                buffer_after: self.buffer_after,
                buffer_before: self.buffer_before,
                closest_booking_time: self.closest_booking_time,
                furthest_booking_time: self.furthest_booking_time,
            },
            ctx,
        )
        .await
        .map_err(UseCaseErrors::InvalidValue)?;

        ctx.repos
            .service_users
            .insert(&user_resource)
            .await
            .map(|_| UseCaseRes {
                user: user_resource,
            })
            .map_err(|_| UseCaseErrors::UserAlreadyInService)
    }
}

#[derive(Debug)]
pub struct ServiceResourceUpdate {
    pub availability: Option<TimePlan>,
    pub busy: Option<Vec<BusyCalendar>>,
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
    let user_calendars = ctx
        .repos
        .calendars
        .find_by_user(&user_resource.user_id)
        .await
        .into_iter()
        .map(|cal| cal.id)
        .collect::<Vec<_>>();

    let busy_google_calendars: Vec<String> = if let Some(busy) = &update.busy {
        if busy
            .iter()
            .find(|busy_cal| match busy_cal {
                BusyCalendar::Google(_) => true,
                BusyCalendar::Nettu(_) => false,
            })
            .is_some()
        {
            let mut user = ctx
                .repos
                .users
                .find(&user_resource.user_id)
                .await
                .expect("User to exist");

            match GoogleCalendarProvider::new(&mut user, ctx).await {
                Ok(provider) => match provider
                    .list(GoogleCalendarAccessRole::FreeBusyReader)
                    .await
                {
                    Ok(calendar_list) => {
                        calendar_list.items.into_iter().map(|cal| cal.id).collect()
                    }
                    Err(_) => vec![],
                },
                Err(_) => vec![],
            }
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    if let Some(busy) = &update.busy {
        for busy_cal in busy {
            match busy_cal {
                BusyCalendar::Nettu(calendar_id) => {
                    if !user_calendars.contains(calendar_id) {
                        return Err(UpdateServiceResourceError::CalendarNotOwnedByUser(
                            calendar_id.to_string(),
                        ));
                    }
                }
                BusyCalendar::Google(id) => {
                    if !busy_google_calendars.contains(id) {
                        return Err(UpdateServiceResourceError::CalendarNotOwnedByUser(
                            id.to_string(),
                        ));
                    }
                }
            }
        }
        user_resource.set_busy(busy.clone());
    }

    if let Some(availability) = &update.availability {
        match availability {
            TimePlan::Calendar(id) => {
                if !user_calendars.contains(id) {
                    return Err(UpdateServiceResourceError::CalendarNotOwnedByUser(
                        id.to_string(),
                    ));
                }
            }
            TimePlan::Schedule(id) => {
                let schedule = ctx.repos.schedules.find(id).await;
                match schedule {
                    Some(schedule) if schedule.user_id == user_resource.user_id => {}
                    _ => {
                        return Err(UpdateServiceResourceError::ScheduleNotOwnedByUser(
                            id.to_string(),
                        ))
                    }
                }
            }
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
