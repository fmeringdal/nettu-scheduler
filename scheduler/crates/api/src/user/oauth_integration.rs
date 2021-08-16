use crate::shared::{
    auth::{account_can_modify_user, protect_route},
    usecase::{execute, UseCase},
};
use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use nettu_scheduler_api_structs::oauth_integration::*;
use nettu_scheduler_domain::{IntegrationProvider, User, UserIntegration};
use nettu_scheduler_infra::{CodeTokenRequest, NettuContext, ProviderOAuth};

pub async fn oauth_integration_admin_controller(
    http_req: HttpRequest,
    path: web::Path<PathParams>,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let user = account_can_modify_user(&account, &path.user_id, &ctx).await?;

    let usecase = OAuthIntegrationUseCase {
        user,
        code: body.0.code,
        provider: body.0.provider,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.user)))
        .map_err(NettuError::from)
}

pub async fn oauth_integration_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, _) = protect_route(&http_req, &ctx).await?;

    let usecase = OAuthIntegrationUseCase {
        user,
        code: body.0.code,
        provider: body.0.provider,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.user)))
        .map_err(NettuError::from)
}

#[derive(Debug)]
pub struct OAuthIntegrationUseCase {
    pub user: User,
    pub code: String,
    pub provider: IntegrationProvider,
}

#[derive(Debug)]
pub struct UseCaseRes {
    pub user: User,
}

#[derive(Debug)]
pub enum UseCaseError {
    StorageError,
    AccountDoesntSupportProvider,
    IntegrationAlreadyExists,
    OAuthFailed,
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::OAuthFailed => Self::BadClientData(
                "Bad client data made the oauth process fail. Make sure the code and redirect_uri is correct".into(),
            ),
            UseCaseError::IntegrationAlreadyExists => Self::Conflict(
                "User already has an integration to that provider".into(),
            ),
            UseCaseError::AccountDoesntSupportProvider => Self::Conflict("The account does not have an integration to that provider".into())
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for OAuthIntegrationUseCase {
    type Response = UseCaseRes;
    type Error = UseCaseError;

    const NAME: &'static str = "OAuthIntegration";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let account_integrations = ctx
            .repos
            .account_integrations
            .find(&self.user.account_id)
            .await
            .map_err(|_| UseCaseError::StorageError)?;
        let acc_provider_integration = match account_integrations
            .into_iter()
            .find(|i| i.provider == self.provider)
        {
            Some(data) => data,
            None => return Err(UseCaseError::AccountDoesntSupportProvider),
        };
        let user_integrations = ctx
            .repos
            .user_integrations
            .find(&self.user.id)
            .await
            .map_err(|_| UseCaseError::StorageError)?;
        if user_integrations
            .into_iter()
            .any(|i| i.provider == self.provider)
        {
            return Err(UseCaseError::IntegrationAlreadyExists);
        };

        let req = CodeTokenRequest {
            client_id: acc_provider_integration.client_id,
            client_secret: acc_provider_integration.client_secret,
            redirect_uri: acc_provider_integration.redirect_uri,
            code: self.code.clone(),
        };
        println!("---------------");
        println!("---------------");
        println!("---------------");
        println!("---------------");
        println!("{:?}", self.provider);
        let res = self
            .provider
            .exchange_code_token(req)
            .await
            .map_err(|_| UseCaseError::OAuthFailed)?;

        let now = Utc::now().timestamp_millis();
        let expires_in_millis = res.expires_in * 1000;
        let user_integration = UserIntegration {
            account_id: self.user.account_id.clone(),
            user_id: self.user.id.clone(),
            access_token: res.access_token,
            access_token_expires_ts: now + expires_in_millis,
            refresh_token: res.refresh_token,
            provider: self.provider.clone(),
        };

        ctx.repos
            .user_integrations
            .insert(&user_integration)
            .await
            .map(|_| UseCaseRes {
                user: self.user.clone(),
            })
            .map_err(|_| UseCaseError::StorageError)
    }
}
