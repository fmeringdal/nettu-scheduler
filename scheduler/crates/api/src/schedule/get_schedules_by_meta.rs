use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::get_schedules_by_meta::*;
use nettu_scheduler_infra::{KVMetadata, MetadataFindQuery, NettuContext};

pub async fn get_schedules_by_meta_controller(
    http_req: HttpRequest,
    query_params: web::Query<QueryParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let query = MetadataFindQuery {
        account_id: account.id,
        metadata: KVMetadata {
            key: query_params.0.key,
            value: query_params.0.value,
        },
        limit: query_params.0.limit.unwrap_or(20),
        skip: query_params.0.skip.unwrap_or(0),
    };
    let schedules = ctx.repos.schedule_repo.find_by_metadata(query).await;
    Ok(HttpResponse::Ok().json(APIResponse::new(schedules)))
}
