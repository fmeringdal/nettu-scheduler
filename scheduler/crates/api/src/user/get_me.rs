use crate::{error::NettuError, shared::auth::protect_route};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::get_me::*;
use nettu_scheduler_infra::NettuContext;

pub async fn get_me_controller(
    http_req: HttpRequest,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, _) = protect_route(&http_req, &ctx).await?;

    Ok(HttpResponse::Ok().json(APIResponse::new(user)))
}
