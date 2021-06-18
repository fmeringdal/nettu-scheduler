use crate::error::NettuError;
use crate::shared::{
    auth::protect_account_route,
    usecase::{execute, UseCase},
};
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{Duration, TimeZone, Utc};
use chrono_tz::UTC;
use get_service_bookingslots::GetServiceBookingSlotsUseCase;
use itertools::Itertools;
use nettu_scheduler_api_structs::create_service_event_intend::*;
use nettu_scheduler_domain::{format_date, RoundRobinAlgorithm, ServiceMultiPersonOptions, User};
use nettu_scheduler_domain::{Account, ID};
use nettu_scheduler_infra::NettuContext;
use rand::{thread_rng, Rng};

use super::get_service_bookingslots;

pub async fn create_service_event_intend_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let body = body.0;
    let usecase = CreateServiceEventIntendUseCase {
        account,
        service_id: path_params.service_id.to_owned(),
        host_user_id: body.host_user_id,
        duration: body.duration,
        timestamp: body.timestamp,
        interval: body.interval,
    };

    execute(usecase, &ctx)
        .await
        .map(|res| HttpResponse::Ok().json(APIResponse::new(res.selected_host)))
        .map_err(|e| match e {
            UseCaseErrors::UserNotAvailable => {
                NettuError::BadClientData("The user is not available at the given time".into())
            }
            UseCaseErrors::UserNotInService => {
                NettuError::NotFound("The user is not in a member of the service".into())
            }
            UseCaseErrors::BookingSlotsQuery(e) => e.into(),
        })
}

#[derive(Debug)]
struct CreateServiceEventIntendUseCase {
    pub account: Account,
    pub service_id: ID,
    pub host_user_id: Option<ID>,
    pub timestamp: i64,
    pub duration: i64,
    pub interval: i64,
}

#[derive(Debug)]
struct UseCaseRes {
    pub selected_host: User,
}

#[derive(Debug)]
enum UseCaseErrors {
    UserNotAvailable,
    UserNotInService,
    BookingSlotsQuery(get_service_bookingslots::UseCaseErrors),
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateServiceEventIntendUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    const NAME: &'static str = "CreateServiceEventIntend";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let start = UTC.timestamp_millis(self.timestamp);
        let start_date = format_date(&start);
        let day_after = start + Duration::days(1);
        let end_date = format_date(&day_after);

        let get_bookingslots_usecase = GetServiceBookingSlotsUseCase {
            duration: self.duration,
            service_id: self.service_id.clone(),
            end_date,
            start_date,
            iana_tz: Some("UTC".to_string()),
            interval: self.interval,
        };
        let res = execute(get_bookingslots_usecase, ctx)
            .await
            .map_err(|e| UseCaseErrors::BookingSlotsQuery(e))?;
        let service = res.service;
        let booking_slots_dates = res.booking_slots.dates;

        let selected_host = if let Some(host_user_id) = &self.host_user_id {
            let service_member = match service.users.iter().find(|u| u.user_id == *host_user_id) {
                Some(e) => e,
                None => return Err(UseCaseErrors::UserNotInService),
            };
            let mut found_slot = false;
            for date in booking_slots_dates {
                for slot in date.slots {
                    if slot.start == self.timestamp {
                        if !slot.user_ids.contains(&service_member.user_id) {
                            return Err(UseCaseErrors::UserNotAvailable);
                        }
                        found_slot = true;
                        break;
                    }
                    if slot.start > self.timestamp {
                        break;
                    }
                }
                if found_slot {
                    break;
                }
            }
            if !found_slot {
                return Err(UseCaseErrors::UserNotAvailable);
            }
            service_member
        } else {
            let mut hosts_at_slot = vec![];
            for date in booking_slots_dates {
                for slot in date.slots {
                    if slot.start == self.timestamp {
                        hosts_at_slot = slot.user_ids.clone();
                        break;
                    }
                    if slot.start > self.timestamp {
                        return Err(UseCaseErrors::UserNotAvailable);
                    }
                }
                if !hosts_at_slot.is_empty() {
                    break;
                }
            }
            let hosts_at_slot = service
                .users
                .iter()
                .filter(|member| hosts_at_slot.contains(&member.user_id))
                .collect::<Vec<_>>();

            if hosts_at_slot.is_empty() {
                return Err(UseCaseErrors::UserNotAvailable);
            } else if hosts_at_slot.len() == 1 {
                &hosts_at_slot[0]
            } else {
                // Do round robin to get host
                match &service.multi_person {
                    ServiceMultiPersonOptions::RoundRobinAlgorithm(round_robin) => {
                        match round_robin {
                            RoundRobinAlgorithm::Availability => {
                                // Availability
                                // - least recently booked host for this service
                                let mut least_recently_booked_member = hosts_at_slot[0];
                                let mut least_recently_booked_time = None;
                                for member in hosts_at_slot {
                                    if let Some(service_event) = ctx
                                        .repos
                                        .events
                                        .find_most_recent_service_event(
                                            &service.id,
                                            &member.user_id,
                                        )
                                        .await
                                    {
                                        if least_recently_booked_time.is_none()
                                            || service_event.created
                                                < least_recently_booked_time.unwrap()
                                        {
                                            least_recently_booked_time =
                                                Some(service_event.created);
                                            least_recently_booked_member = member;
                                        }
                                    }
                                }

                                least_recently_booked_member
                            }
                            RoundRobinAlgorithm::EqualDistribution => {
                                // Equal distribution
                                // - check which user has the least number of service events for this service for the next two weeks
                                let now = Utc::now().timestamp_millis();
                                let timestamp_in_two_weeks = now + 1000 * 60 * 60 * 24 * 14;

                                let user_with_least_service_events = ctx
                                    .repos
                                    .events
                                    .find_by_service(&service.id, now, timestamp_in_two_weeks)
                                    .await
                                    .into_iter()
                                    .group_by(|e1| e1.user_id.clone())
                                    .into_iter()
                                    .map(|(id, events)| (id, events.count()))
                                    .min_by_key(|(id, events_count)| *events_count)
                                    .map(|(user_id, _)| user_id);

                                match user_with_least_service_events {
                                    Some(selected_user_id) => service
                                        .users
                                        .iter()
                                        .find(|member| member.user_id == selected_user_id)
                                        .expect("User to be member of service"),
                                    None => {
                                        // Just pick random
                                        let mut rng = thread_rng();
                                        let rand_user_index = rng.gen_range(0..service.users.len());
                                        &service.users[rand_user_index]
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };
        let selected_host = ctx
            .repos
            .users
            .find(&selected_host.user_id)
            .await
            .expect("To find selected host user");

        Ok(UseCaseRes { selected_host })
    }
}
