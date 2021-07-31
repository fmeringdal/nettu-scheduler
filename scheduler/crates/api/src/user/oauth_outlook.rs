use crate::shared::{
    auth::{account_can_modify_user, protect_route},
    usecase::{execute, UseCase},
};
use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use nettu_scheduler_api_structs::oauth_outlook::*;
use nettu_scheduler_domain::{User, UserIntegration, UserOutlookIntegrationData};
use nettu_scheduler_infra::{outlook_calendar::auth_provider, NettuContext};

fn handle_error(e: UseCaseErrors) -> NettuError {
    match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::OAuthFailed => NettuError::BadClientData(
                "Bad client data made the oauth process fail. Make sure the code and redirect_uri is correct".into(),
            ),
            UseCaseErrors::AccountDoesntHaveOutlookSupport => NettuError::Conflict("The account does not have outlook integration enabled".into())
    }
}

pub async fn oauth_outlook_admin_controller(
    http_req: HttpRequest,
    path: web::Path<PathParams>,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let user = account_can_modify_user(&account, &path.user_id, &ctx).await?;

    let usecase = OAuthOutlookUseCase {
        user,
        code: body.0.code,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.user)))
        .map_err(handle_error)
}

pub async fn oauth_outlook_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, _) = protect_route(&http_req, &ctx).await?;

    let usecase = OAuthOutlookUseCase {
        user,
        code: body.0.code,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.user)))
        .map_err(handle_error)
}

#[derive(Debug)]
pub struct OAuthOutlookUseCase {
    pub user: User,
    pub code: String,
}

#[derive(Debug)]
pub struct UseCaseRes {
    pub user: User,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    StorageError,
    AccountDoesntHaveOutlookSupport,
    OAuthFailed,
}

#[async_trait::async_trait(?Send)]
impl UseCase for OAuthOutlookUseCase {
    type Response = UseCaseRes;
    type Errors = UseCaseErrors;

    const NAME: &'static str = "OAuthOutlook";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let account = match ctx.repos.accounts.find(&self.user.account_id).await {
            Some(acc) => acc,
            None => return Err(UseCaseErrors::StorageError),
        };
        let acc_outlook_settings = match account.settings.outlook {
            Some(data) => data,
            None => return Err(UseCaseErrors::AccountDoesntHaveOutlookSupport),
        };
        let req = auth_provider::CodeTokenRequest {
            client_id: acc_outlook_settings.client_id,
            client_secret: acc_outlook_settings.client_secret,
            redirect_uri: acc_outlook_settings.redirect_uri,
            code: self.code.clone(),
        };
        let res = match auth_provider::exchange_code_token(req).await {
            Ok(res) => res,
            Err(_) => return Err(UseCaseErrors::OAuthFailed),
        };
        let now = Utc::now().timestamp_millis();
        let expires_in_millis = res.expires_in * 1000;
        let user_integration = UserOutlookIntegrationData {
            access_token: res.access_token,
            access_token_expires_ts: now + expires_in_millis,
            refresh_token: res.refresh_token,
        };

        if let Some(existing_outlook_integration) =
            self.user
                .integrations
                .iter_mut()
                .find_map(|integration| match integration {
                    UserIntegration::Outlook(data) => Some(data),
                    _ => None,
                })
        {
            existing_outlook_integration.access_token = user_integration.access_token;
            existing_outlook_integration.access_token_expires_ts =
                user_integration.access_token_expires_ts;
            existing_outlook_integration.refresh_token = user_integration.refresh_token;
        } else {
            self.user
                .integrations
                .push(UserIntegration::Outlook(user_integration));
        }

        ctx.repos
            .users
            .save(&self.user)
            .await
            .map(|_| UseCaseRes {
                user: self.user.clone(),
            })
            .map_err(|_| UseCaseErrors::StorageError)
    }
}
