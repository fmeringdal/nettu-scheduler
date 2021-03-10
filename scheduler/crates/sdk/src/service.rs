use crate::{shared::MetadataFindInput, APIResponse, BaseClient, TimePlan, ID};
use nettu_scheduler_api_structs::*;
use nettu_scheduler_domain::Metadata;
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
    pub availibility: Option<TimePlan>,
    pub busy: Option<Vec<ID>>,
    pub buffer: Option<i64>,
    pub closest_booking_time: Option<i64>,
    pub furthest_booking_time: Option<i64>,
}

pub struct UpdateServiceUserInput {
    pub service_id: ID,
    pub user_id: ID,
    pub availibility: Option<TimePlan>,
    pub busy: Option<Vec<ID>>,
    pub buffer: Option<i64>,
    pub closest_booking_time: Option<i64>,
    pub furthest_booking_time: Option<i64>,
}

pub struct RemoveServiceUserInput {
    pub service_id: ID,
    pub user_id: ID,
}

pub struct GetSerivceBookingSlotsInput {
    pub service_id: ID,
    pub iana_tz: Option<String>,
    pub duration: i64,
    pub interval: i64,
    pub date: String,
}

pub struct UpdateServiceInput {
    pub service_id: ID,
    pub metadata: Option<Metadata>,
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
        input: GetSerivceBookingSlotsInput,
    ) -> APIResponse<get_service_bookingslots::APIResponse> {
        let mut query_string = format!(
            "duration={}&interval={}&date={}",
            input.duration, input.interval, input.date
        );
        if let Some(timezone) = input.iana_tz {
            query_string = format!("{}&ianaTz={}", query_string, timezone);
        }

        self.base
            .get(
                format!("service/{}/booking?{}", input.service_id, query_string),
                StatusCode::OK,
            )
            .await
    }

    pub async fn delete(&self, service_id: ID) -> APIResponse<delete_service::APIResponse> {
        self.base
            .delete(format!("service/{}", service_id), StatusCode::OK)
            .await
    }

    pub async fn create(&self) -> APIResponse<create_service::APIResponse> {
        let body = Empty {};
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
            availibility: input.availibility,
            buffer: input.buffer,
            busy: input.busy,
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
            availibility: input.availibility,
            buffer: input.buffer,
            busy: input.busy,
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
