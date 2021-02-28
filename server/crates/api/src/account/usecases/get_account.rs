use crate::error::NettuError;
use crate::shared::auth::protect_account_route;
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::api::get_account::APIResponse;
use nettu_scheduler_infra::NettuContext;

pub async fn get_account_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    Ok(HttpResponse::Ok().json(APIResponse::new(account)))
}
