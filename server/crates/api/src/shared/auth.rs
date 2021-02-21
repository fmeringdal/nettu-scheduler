use std::collections::HashMap;

use actix_web::HttpRequest;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use nettu_scheduler_core::{Account, User};
use nettu_scheduler_infra::NettuContext;
use serde::{Deserialize, Serialize};

use crate::user::usecases::create_user::CreateUserUseCase;
use crate::{error::NettuError, shared::usecase::execute};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Claims {
    exp: usize,      // Expiration time (as UTC timestamp)
    iat: usize,      // Issued at (as UTC timestamp)
    user_id: String, // Subject (whom token refers to)
    scheduler_policy: Option<Policy>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Policy {
    allow: Option<Vec<Permission>>,
    reject: Option<Vec<Permission>>,
}

impl Policy {
    pub fn authorize(&self, permissions: &Vec<Permission>) -> bool {
        if permissions.is_empty() {
            return true;
        }

        if let Some(rejected) = &self.reject {
            for rejected_permission in rejected {
                if *rejected_permission == Permission::All {
                    return false;
                }
                if permissions.contains(rejected_permission) {
                    return false;
                }
            }
        }

        if let Some(allowed) = &self.allow {
            // First loop to check if All exists
            if allowed.contains(&Permission::All) {
                return true;
            }

            // Check that all permissions are in allowed
            for permission in permissions {
                if !allowed.contains(permission) {
                    return false;
                }
            }

            return true;
        }

        false
    }

    pub fn empty() -> Self {
        Self {
            allow: None,
            reject: None,
        }
    }
}

impl Default for Policy {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    #[serde(rename = "*")]
    All,
    CreateCalendar,
    DeleteCalendar,
    UpdateCalendar,
    CreateCalendarEvent,
    DeleteCalendarEvent,
    UpdateCalendarEvent,
    CreateSchedule,
    UpdateSchedule,
    DeleteSchedule,
}

fn parse_authtoken_header(token_header_value: &str) -> String {
    token_header_value
        .replace("Bearer", "")
        .replace("bearer", "")
        .trim()
        .to_string()
}

async fn create_user_if_not_exists(
    external_user_id: &str,
    account_id: &str,
    ctx: &NettuContext,
) -> Option<User> {
    let user_id = User::create_id(account_id, external_user_id);
    let user = ctx.repos.user_repo.find(&user_id).await;
    if let Some(user) = user {
        return Some(user);
    }

    let usecase = CreateUserUseCase {
        account_id: String::from(account_id),
        external_user_id: String::from(external_user_id),
    };
    let res = execute(usecase, ctx).await;
    match res {
        Ok(res) => Some(res.user),
        Err(_) => None,
    }
}

pub async fn auth_user_req(
    req: &HttpRequest,
    account: &Account,
    ctx: &NettuContext,
) -> Option<(User, Policy)> {
    let token = req.headers().get("authorization");
    match token {
        Some(token) => {
            let token = match token.to_str() {
                Ok(token) => parse_authtoken_header(token),
                Err(_) => return None,
            };
            match decode_token(account, &token) {
                Ok(claims) => create_user_if_not_exists(&claims.user_id, &account.id, ctx)
                    .await
                    .map(|u| (u, claims.scheduler_policy.unwrap_or_default())),
                Err(_e) => None,
            }
        }
        None => None,
    }
}

pub async fn get_client_account(req: &HttpRequest, ctx: &NettuContext) -> Option<Account> {
    let account = req.headers().get("nettu-account");
    match account {
        Some(acc) => match acc.to_str() {
            Ok(acc) => ctx.repos.account_repo.find(acc).await,
            Err(_) => None,
        },
        None => None,
    }
}

fn decode_token(account: &Account, token: &str) -> anyhow::Result<Claims> {
    let public_key_b64 = match &account.public_key_b64 {
        Some(public_key_b64) => public_key_b64,
        None => return Err(anyhow::Error::msg("Account does not support user tokens")),
    };
    let public_key = base64::decode(&public_key_b64)?;
    let decoding_key = DecodingKey::from_rsa_pem(&public_key)?;
    let claims =
        decode::<Claims>(&token, &decoding_key, &Validation::new(Algorithm::RS256))?.claims;

    Ok(claims)
}

pub async fn protect_route(
    req: &HttpRequest,
    ctx: &NettuContext,
) -> Result<(User, Policy), NettuError> {
    let account = match get_client_account(req, ctx).await {
        Some(account) => account,
        None => {
            return Err(NettuError::Unauthorized(
                "Unable to find the account the client belongs to".into(),
            ))
        }
    };
    let res = auth_user_req(req, &account, ctx).await;

    match res {
        Some(user_and_policy) => Ok(user_and_policy),
        None => Err(NettuError::Unauthorized(
            "Unable to find user from credentials".into(),
        )),
    }
}

pub fn ensure_nettu_acct_header(req: &HttpRequest) -> Result<String, NettuError> {
    match req.headers().get("nettu-account") {
        Some(acc_id) => match acc_id.to_str() {
            Ok(acc_id) => Ok(String::from(acc_id)),
            Err(_) => Err(NettuError::UnidentifiableClient(format!(
                "Malformed nettu account header provided: {:?}",
                acc_id
            ))),
        },
        None => Err(NettuError::UnidentifiableClient(
            "Unable to find nettu account header".into(),
        )),
    }
}

pub async fn protect_account_route(
    req: &HttpRequest,
    ctx: &NettuContext,
) -> Result<Account, NettuError> {
    let api_key = match req.headers().get("x-api-key") {
        Some(api_key) => match api_key.to_str() {
            Ok(api_key) => api_key,
            Err(_) => {
                return Err(NettuError::Unauthorized(
                    "Malformed api key provided".to_string(),
                ))
            }
        },
        None => {
            return Err(NettuError::Unauthorized(
                "Unable to find api-key in x-api-key header".to_string(),
            ))
        }
    };

    let account = ctx.repos.account_repo.find_by_apikey(api_key).await;

    match account {
        Some(acc) => Ok(acc),
        None => Err(NettuError::Unauthorized(format!(
            "Invalid api-key provided in x-api-key header"
        ))),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_web::test::TestRequest;
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use nettu_scheduler_infra::setup_context;

    async fn setup_account(ctx: &NettuContext) -> Account {
        let account = get_account();
        ctx.repos.account_repo.insert(&account).await.unwrap();
        account
    }

    fn get_token(expired: bool) -> String {
        let priv_key = std::fs::read("./config/test_private_rsa_key.pem").unwrap();
        let exp = if expired {
            100 // year 1970
        } else {
            5609418990073 // year 2147
        };
        let claims = Claims {
            exp,
            iat: 19,
            user_id: get_external_user_id(),
            scheduler_policy: None,
        };
        let enc_key = EncodingKey::from_rsa_pem(&priv_key).unwrap();
        encode(&Header::new(Algorithm::RS256), &claims, &enc_key).unwrap()
    }

    fn get_external_user_id() -> String {
        String::from("cool")
    }

    fn get_account() -> Account {
        let pub_key = std::fs::read("./config/test_public_rsa_key.crt").unwrap();
        let public_key_b64 = base64::encode(pub_key);
        Account {
            id: String::from("nettu"),
            public_key_b64: Some(public_key_b64),
            secret_api_key: String::from("yoyo"),
            settings: Default::default(),
        }
    }

    #[actix_web::main]
    #[test]
    async fn decodes_valid_token_and_creates_user_if_not_found() {
        let ctx = setup_context().await;
        let account = setup_account(&ctx).await;
        let token = get_token(false);

        let req = TestRequest::with_header("nettu-account", account.id.clone())
            .header("Authorization", format!("Bearer {}", token))
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_ok());
        let user_id = User::create_id(&account.id, &get_external_user_id());
        assert!(ctx.repos.user_repo.find(&user_id).await.is_some());
    }

    #[actix_web::main]
    #[test]
    async fn decodes_valid_token_for_existing_user() {
        let ctx = setup_context().await;
        let account = setup_account(&ctx).await;
        let token = get_token(false);
        let user = User::new(&account.id, &get_external_user_id());
        ctx.repos.user_repo.insert(&user).await.unwrap();

        let req = TestRequest::with_header("nettu-account", account.id)
            .header("Authorization", format!("Bearer {}", token))
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_ok());
    }

    #[actix_web::main]
    #[test]
    async fn rejects_expired_token() {
        let ctx = setup_context().await;
        let account = setup_account(&ctx).await;
        let token = get_token(true);

        let req = TestRequest::with_header("nettu-account", account.id)
            .header("Authorization", format!("Bearer {}", token))
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_err());
    }

    #[actix_web::main]
    #[test]
    async fn rejects_valid_token_without_account_header() {
        let ctx = setup_context().await;
        let _account = setup_account(&ctx).await;
        let token = get_token(false);

        let req = TestRequest::with_header("Authorization", format!("Bearer {}", token))
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_err());
    }

    #[actix_web::main]
    #[test]
    async fn rejects_valid_token_with_valid_invalid_account_header() {
        let ctx = setup_context().await;
        let account = setup_account(&ctx).await;
        let token = "sajfosajfposajfopaso12";

        let req = TestRequest::with_header("nettu-account", account.id + "s")
            .header("Authorization", format!("Bearer {}", token))
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_err());
    }

    #[actix_web::main]
    #[test]
    async fn rejects_garbage_token_with_valid_account_header() {
        let ctx = setup_context().await;
        let _account = setup_account(&ctx).await;
        let token = "sajfosajfposajfopaso12";

        let req = TestRequest::with_header("Authorization", format!("Bearer {}", token))
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_err());
    }

    #[actix_web::main]
    #[test]
    async fn rejects_req_without_headers() {
        let ctx = setup_context().await;
        let _account = setup_account(&ctx).await;

        let req = TestRequest::default().to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_err());
    }

    #[test]
    fn permissions() {
        let policy = Policy::empty();
        assert!(policy.authorize(&vec![]));
        assert!(!policy.authorize(&vec![Permission::CreateCalendar]));

        let policy = Policy {
            allow: Some(vec![Permission::All]),
            reject: None,
        };
        assert!(policy.authorize(&vec![Permission::CreateCalendar]));

        let policy = Policy {
            allow: Some(vec![Permission::All]),
            reject: Some(vec![Permission::CreateCalendar]),
        };
        assert!(!policy.authorize(&vec![Permission::CreateCalendar]));

        let policy = Policy {
            allow: Some(vec![Permission::CreateCalendar]),
            reject: Some(vec![]),
        };
        assert!(policy.authorize(&vec![Permission::CreateCalendar]));

        let policy = Policy {
            allow: Some(vec![Permission::CreateCalendar]),
            reject: Some(vec![Permission::CreateCalendar]),
        };
        assert!(!policy.authorize(&vec![Permission::CreateCalendar]));

        let policy = Policy {
            allow: Some(vec![Permission::CreateCalendar]),
            reject: Some(vec![Permission::All]),
        };
        assert!(!policy.authorize(&vec![Permission::CreateCalendar]));

        let policy = Policy {
            allow: Some(vec![Permission::CreateCalendar, Permission::UpdateCalendar]),
            reject: Some(vec![Permission::DeleteCalendar]),
        };
        assert!(policy.authorize(&vec![Permission::CreateCalendar]));
        assert!(policy.authorize(&vec![
            Permission::CreateCalendar,
            Permission::UpdateCalendar
        ]));

        let policy = Policy {
            allow: Some(vec![Permission::CreateCalendar, Permission::UpdateCalendar]),
            reject: Some(vec![Permission::UpdateCalendar]),
        };
        assert!(policy.authorize(&vec![Permission::CreateCalendar]));
        assert!(!policy.authorize(&vec![
            Permission::CreateCalendar,
            Permission::UpdateCalendar
        ]));

        let policy = Policy {
            allow: Some(vec![Permission::All]),
            reject: Some(vec![Permission::UpdateCalendar]),
        };
        assert!(policy.authorize(&vec![Permission::CreateCalendar]));
        assert!(policy.authorize(&vec![
            Permission::CreateCalendar,
            Permission::DeleteCalendar,
        ]));
        assert!(!policy.authorize(&vec![
            Permission::CreateCalendar,
            Permission::DeleteCalendar,
            Permission::UpdateCalendar,
        ]));
    }
}
