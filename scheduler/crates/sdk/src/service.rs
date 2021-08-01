use crate::{shared::MetadataFindInput, APIResponse, BaseClient, TimePlan, ID};
use nettu_scheduler_api_structs::*;
use nettu_scheduler_domain::{BusyCalendar, Metadata, ServiceMultiPersonOptions};
use reqwest::StatusCode;
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServiceClient {
    base: Arc<BaseClient>,
}

pub struct AddServiceUserInput {
    pub service_id: ID,
    pub user_id: ID,
    pub availability: Option<TimePlan>,
    pub buffer_after: Option<i64>,
    pub buffer_before: Option<i64>,
    pub closest_booking_time: Option<i64>,
    pub furthest_booking_time: Option<i64>,
}

pub struct UpdateServiceUserInput {
    pub service_id: ID,
    pub user_id: ID,
    pub availability: Option<TimePlan>,
    pub buffer_after: Option<i64>,
    pub buffer_before: Option<i64>,
    pub closest_booking_time: Option<i64>,
    pub furthest_booking_time: Option<i64>,
}

pub struct CreateBookingIntendInput {
    pub service_id: ID,
    pub host_user_ids: Option<Vec<ID>>,
    pub timestamp: i64,
    pub duration: i64,
    pub interval: i64,
}

pub struct RemoveBookingIntendInput {
    pub service_id: ID,
    pub timestamp: i64,
}

pub struct RemoveServiceUserInput {
    pub service_id: ID,
    pub user_id: ID,
}

#[derive(Debug, Clone)]
pub struct GetServiceBookingSlotsInput {
    pub service_id: ID,
    pub iana_tz: Option<String>,
    pub duration: i64,
    pub interval: i64,
    pub start_date: String,
    pub end_date: String,
    pub host_user_ids: Option<Vec<ID>>,
}

pub struct UpdateServiceInput {
    pub service_id: ID,
    pub metadata: Option<Metadata>,
    pub multi_person: Option<ServiceMultiPersonOptions>,
}

pub struct CreateServiceInput {
    pub metadata: Option<Metadata>,
    pub multi_person: Option<ServiceMultiPersonOptions>,
}

#[derive(Serialize)]
struct Empty {}

impl ServiceClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn get(&self, service_id: ID) -> APIResponse<get_service::APIResponse> {
        self.base
            .get(format!("service/{}", service_id), StatusCode::OK)
            .await
    }

    pub async fn bookingslots(
        &self,
        input: GetServiceBookingSlotsInput,
    ) -> APIResponse<get_service_bookingslots::APIResponse> {
        let mut query_string = format!(
            "duration={}&interval={}&startDate={}&endDate={}",
            input.duration, input.interval, input.start_date, input.end_date
        );

        if let Some(timezone) = input.iana_tz {
            query_string = format!("{}&ianaTz={}", query_string, timezone);
        }
        if let Some(host_user_ids) = input.host_user_ids {
            query_string = format!(
                "{}&hostUserIds={}",
                query_string,
                host_user_ids
                    .into_iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }

        self.base
            .get(
                format!("service/{}/booking?{}", input.service_id, query_string),
                StatusCode::OK,
            )
            .await
    }

    pub async fn create_booking_intend(
        &self,
        input: CreateBookingIntendInput,
    ) -> APIResponse<create_service_event_intend::APIResponse> {
        let body = create_service_event_intend::RequestBody {
            duration: input.duration,
            host_user_ids: input.host_user_ids,
            interval: input.interval,
            timestamp: input.timestamp,
        };
        self.base
            .post(
                body,
                format!("service/{}/booking-intend", input.service_id),
                StatusCode::OK,
            )
            .await
    }

    pub async fn remove_booking_intend(
        &self,
        input: RemoveBookingIntendInput,
    ) -> APIResponse<remove_service_event_intend::APIResponse> {
        self.base
            .delete(
                format!(
                    "service/{}/booking-intend?timestamp={}",
                    input.service_id, input.timestamp
                ),
                StatusCode::OK,
            )
            .await
    }

    pub async fn delete(&self, service_id: ID) -> APIResponse<delete_service::APIResponse> {
        self.base
            .delete(format!("service/{}", service_id), StatusCode::OK)
            .await
    }

    pub async fn create(
        &self,
        input: CreateServiceInput,
    ) -> APIResponse<create_service::APIResponse> {
        let body = create_service::RequestBody {
            metadata: input.metadata,
            multi_person: input.multi_person,
        };
        self.base
            .post(body, "service".into(), StatusCode::CREATED)
            .await
    }

    pub async fn update(
        &self,
        input: UpdateServiceInput,
    ) -> APIResponse<update_service::APIResponse> {
        let body = update_service::RequestBody {
            metadata: input.metadata,
            multi_person: input.multi_person,
        };
        self.base
            .put(
                body,
                format!("service/{}", input.service_id),
                StatusCode::OK,
            )
            .await
    }

    pub async fn get_by_meta(
        &self,
        input: MetadataFindInput,
    ) -> APIResponse<get_services_by_meta::APIResponse> {
        self.base
            .get(
                format!("service/meta?{}", input.to_query_string()),
                StatusCode::OK,
            )
            .await
    }

    pub async fn remove_user(
        &self,
        input: RemoveServiceUserInput,
    ) -> APIResponse<remove_user_from_service::APIResponse> {
        self.base
            .delete(
                format!("service/{}/users/{}", input.service_id, input.user_id),
                StatusCode::OK,
            )
            .await
    }

    pub async fn update_user(
        &self,
        input: UpdateServiceUserInput,
    ) -> APIResponse<update_service_user::APIResponse> {
        let user_id = input.user_id.clone();
        let service_id = input.service_id.clone();
        let body = update_service_user::RequestBody {
            availability: input.availability,
            buffer_after: input.buffer_after,
            buffer_before: input.buffer_before,
            closest_booking_time: input.closest_booking_time,
            furthest_booking_time: input.furthest_booking_time,
        };

        self.base
            .put(
                body,
                format!("service/{}/users/{}", service_id, user_id),
                StatusCode::OK,
            )
            .await
    }

    pub async fn add_user(
        &self,
        input: AddServiceUserInput,
    ) -> APIResponse<add_user_to_service::APIResponse> {
        let service_id = input.service_id.clone();
        let body = add_user_to_service::RequestBody {
            user_id: input.user_id,
            availability: input.availability,
            buffer_after: input.buffer_after,
            buffer_before: input.buffer_before,
            closest_booking_time: input.closest_booking_time,
            furthest_booking_time: input.furthest_booking_time,
        };

        self.base
            .post(
                body,
                format!("service/{}/users", service_id),
                StatusCode::OK,
            )
            .await
    }
}
