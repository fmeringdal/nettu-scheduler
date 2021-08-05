use super::get_service_bookingslots;
use crate::error::NettuError;
use crate::shared::{
    auth::protect_account_route,
    usecase::{execute, UseCase},
};
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{Duration, TimeZone, Utc};
use chrono_tz::UTC;
use get_service_bookingslots::GetServiceBookingSlotsUseCase;
use nettu_scheduler_api_structs::create_service_event_intend::*;
use nettu_scheduler_domain::{
    format_date,
    scheduling::{
        RoundRobinAlgorithm, RoundRobinAvailabilityAssignment,
        RoundRobinEqualDistributionAssignment,
    },
    ServiceMultiPersonOptions, User,
};
use nettu_scheduler_domain::{Account, ID};
use nettu_scheduler_infra::NettuContext;

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
        host_user_ids: body.host_user_ids,
        duration: body.duration,
        timestamp: body.timestamp,
        interval: body.interval,
    };

    execute(usecase, &ctx)
        .await
        .map(|res| {
            HttpResponse::Ok().json(APIResponse::new(
                res.selected_hosts,
                res.create_event_for_hosts,
            ))
        })
        .map_err(|e| match e {
            UseCaseErrors::UserNotAvailable => {
                NettuError::BadClientData("The user is not available at the given time".into())
            }
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::BookingSlotsQuery(e) => e.into(),
        })
}

#[derive(Debug)]
struct CreateServiceEventIntendUseCase {
    pub account: Account,
    pub service_id: ID,
    pub host_user_ids: Option<Vec<ID>>,
    pub timestamp: i64,
    pub duration: i64,
    pub interval: i64,
}

#[derive(Debug)]
struct UseCaseRes {
    pub selected_hosts: Vec<User>,
    pub create_event_for_hosts: bool,
}

#[derive(Debug)]
enum UseCaseErrors {
    UserNotAvailable,
    StorageError,
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
            host_user_ids: self.host_user_ids.clone(),
        };
        let res = execute(get_bookingslots_usecase, ctx)
            .await
            .map_err(|e| UseCaseErrors::BookingSlotsQuery(e))?;
        let service = res.service;
        let booking_slots_dates = res.booking_slots.dates;

        let mut create_event_for_hosts = true;
        let selected_host_user_ids = if let Some(host_user_ids) = &self.host_user_ids {
            let mut found_slot = false;
            for date in booking_slots_dates {
                for slot in date.slots {
                    if slot.start == self.timestamp {
                        // Check that all host users are available
                        for host_user_id in host_user_ids {
                            if !slot.user_ids.contains(&host_user_id) {
                                return Err(UseCaseErrors::UserNotAvailable);
                            }
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
            host_user_ids.clone()
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
            } else {
                let user_ids_at_slot = hosts_at_slot
                    .iter()
                    .map(|h| h.user_id.clone())
                    .collect::<Vec<_>>();
                // Do round robin to get host
                match &service.multi_person {
                    ServiceMultiPersonOptions::RoundRobinAlgorithm(round_robin) => {
                        match round_robin {
                            RoundRobinAlgorithm::Availability => {
                                if hosts_at_slot.len() == 1 {
                                    vec![hosts_at_slot[0].user_id.clone()]
                                } else {
                                    let events = ctx
                                        .repos
                                        .events
                                        .find_most_recently_created_service_events(
                                            &service.id,
                                            &user_ids_at_slot,
                                        )
                                        .await;

                                    let query = RoundRobinAvailabilityAssignment {
                                        members: events
                                            .into_iter()
                                            .map(|e| (e.user_id, e.created))
                                            .collect(),
                                    };
                                    let selected_user_id = query.assign().expect("At least one host can be picked when there are at least one host available");
                                    vec![selected_user_id]
                                }
                            }
                            RoundRobinAlgorithm::EqualDistribution => {
                                if hosts_at_slot.len() == 1 {
                                    vec![hosts_at_slot[0].user_id.clone()]
                                } else {
                                    let now = Utc::now().timestamp_millis();
                                    let timestamp_in_two_months = now + 1000 * 60 * 60 * 24 * 61;

                                    let service_events = ctx
                                        .repos
                                        .events
                                        .find_by_service(
                                            &service.id,
                                            &user_ids_at_slot,
                                            now,
                                            timestamp_in_two_months,
                                        )
                                        .await;

                                    let query = RoundRobinEqualDistributionAssignment {
                                        events: service_events,
                                        user_ids: user_ids_at_slot,
                                    };
                                    let selected_user_id = query.assign().expect("At least one host can be picked when there are at least one host available");
                                    vec![selected_user_id]
                                }
                            }
                        }
                    }
                    ServiceMultiPersonOptions::Collective => {
                        let all_hosts_user_ids: Vec<_> = service
                            .users
                            .iter()
                            .map(|resource| resource.user_id.clone())
                            .collect();

                        // Check that all the hosts are available
                        if user_ids_at_slot.len() < all_hosts_user_ids.len() {
                            return Err(UseCaseErrors::UserNotAvailable);
                        }

                        all_hosts_user_ids
                    }
                    ServiceMultiPersonOptions::Group(max_count) => {
                        let all_hosts_user_ids: Vec<_> = service
                            .users
                            .iter()
                            .map(|resource| resource.user_id.clone())
                            .collect();

                        // Check that all the hosts are available
                        if user_ids_at_slot.len() < all_hosts_user_ids.len() {
                            return Err(UseCaseErrors::UserNotAvailable);
                        }

                        let reservations = ctx
                            .repos
                            .reservations
                            .count(&service.id, self.timestamp)
                            .await
                            .map_err(|_| UseCaseErrors::StorageError)?;
                        if reservations + 1 < *max_count {
                            // Client do not need to create service event yet
                            create_event_for_hosts = false;
                        }

                        ctx.repos
                            .reservations
                            .increment(&service.id, self.timestamp)
                            .await
                            .map_err(|_| UseCaseErrors::StorageError)?;

                        all_hosts_user_ids
                    }
                }
            }
        };

        let selected_hosts = ctx.repos.users.find_many(&selected_host_user_ids).await;

        Ok(UseCaseRes {
            selected_hosts,
            create_event_for_hosts,
        })
    }
}
