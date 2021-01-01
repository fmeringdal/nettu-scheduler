use actix_web::{HttpRequest, HttpResponse};
use account::domain::Account;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{account::{self, repos::IAccountRepo}, user::{domain::User, repos::IUserRepo}};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Claims {
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    user_id: String, // Optional. Subject (whom token refers to)
}

fn parse_authtoken_header(token_header_value: &str) -> String {
    let mut token = token_header_value.replace("Bearer", "");
    token = token.replace("bearer", "");
    String::from(token.trim())
}

pub struct AuthContext {
    pub user_repo: Arc<dyn IUserRepo>,
    pub account_repo: Arc<dyn IAccountRepo>,
}

async fn create_user_if_not_exists(external_user_id: &str, account_id: &str, ctx: &AuthContext) -> User {
    let user_id = User::create_id(account_id, external_user_id);
    match ctx.user_repo.find(&user_id).await {
        Some(user) => user,
        None => {
            // create user
            // todo: in future there will be a create user admin endpoint
            let user = User::new(account_id, external_user_id);

            ctx.user_repo.insert(&user).await;

            user
        }
    }
}

pub async fn auth_user_req(
    req: &HttpRequest,
    account: &Account,
    ctx: &AuthContext,
) -> Option<User> {
    let token = req.headers().get("authorization");
    match token {
        Some(token) => {
            let token = match token.to_str() {
                Ok(token) => parse_authtoken_header(token),
                Err(_) => return None,
            };
            match decode_token(account, &token) {
                Ok(claims) => {
                    let user = create_user_if_not_exists(&claims.user_id, &account.id, ctx).await;
                    Some(user)
                },
                Err(e) => None,
            }
        }
        None => None,
    }
}

pub async fn get_client_account(req: &HttpRequest, ctx: &AuthContext) -> Option<Account> {
    let account = req.headers().get("nettu-account");
    match account {
        Some(acc) => match acc.to_str() {
            Ok(acc) => ctx.account_repo.find(acc).await,
            Err(_) => None,
        },
        None => None,
    }
}

fn decode_token(account: &Account, token: &str) -> anyhow::Result<Claims> {
    let public_key_b64 = match &account.public_key_b64 {
        Some(public_key_b64) => public_key_b64,
        None => return Err(anyhow::Error::msg("Account does not support user tokens"))
    };
    let public_key = base64::decode(&public_key_b64)?;
    let decoding_key = DecodingKey::from_rsa_pem(&public_key)?;
    let token_data = decode::<Claims>(&token, &decoding_key, &Validation::new(Algorithm::RS256))?;
    Ok(token_data.claims)
}

pub async fn protect_route(req: &HttpRequest, ctx: &AuthContext) -> Result<User, HttpResponse> {
    let account = match get_client_account(req, ctx).await {
        Some(account) => account,
        None => {
            return Err(HttpResponse::Unauthorized()
                .body("Unable to find the account the client belongs to"))
        }
    };
    let res = auth_user_req(req, &account, ctx).await;

    match res {
        Some(user) => Ok(user),
        None => Err(HttpResponse::Unauthorized().finish()),
    }
}

pub fn ensure_nettu_acct_header(req: &HttpRequest) -> Result<String, HttpResponse> {
    match req.headers().get("nettu-account") {
        Some(acc_id) => {
            match acc_id.to_str() {
                Ok(acc_id) => Ok(String::from(acc_id)),
                Err(_) => return Err(HttpResponse::Unauthorized().body(format!("Malformed nettu account header provided: {:?}", acc_id))),
            }
        }
        None => return Err(HttpResponse::Unauthorized().body("Unable to find nettu account header"))
    }
}

pub struct AccountAuthContext {
    pub account_repo: Arc<dyn IAccountRepo>,
}

pub async fn protect_account_route(req: &HttpRequest, ctx: &AccountAuthContext) -> Result<Account, HttpResponse> {
    let api_key = match req.headers().get("x-api-key") {
        Some(api_key) => {
            match api_key.to_str() {
                Ok(api_key) => api_key,
                Err(_) => return Err(HttpResponse::Unauthorized().body("Malformed api key provided")),
            }
        }
        None => return Err(HttpResponse::Unauthorized().body("Unable to find api-key in x-api-key header"))
    };


    let account = ctx.account_repo.find_by_apikey(api_key).await;

    match account {
        Some(acc) => Ok(acc),
        None => Err(HttpResponse::Unauthorized().body("Invalid api key provided")),
    }
}





#[cfg(test)]
mod test {
    use super::*;
    use crate::{account::repos::InMemoryAccountRepo, user::repos::InMemoryUserRepo};
    use actix_web::test::TestRequest;
    use jsonwebtoken::{encode, Header, EncodingKey, Algorithm};

    async fn setup_account(ctx: &AuthContext) -> Account {
        let account = get_account();
        ctx.account_repo.insert(&account).await.unwrap();
        account
    }

    fn setup_ctx() -> AuthContext {
        let account_repo = InMemoryAccountRepo::new();
        let user_repo = InMemoryUserRepo::new();
        AuthContext {
            account_repo: Arc::new(account_repo),
            user_repo: Arc::new(user_repo),
        }
    }

    fn get_token(expired: bool) -> String {
        let priv_key = std::fs::read("./config/test_private_rsa_key.pem").unwrap();
        let exp = if expired {
            100 // year 1970
        } else {
            5609418990073 // year 2147
        };
        let claims = Claims { exp, iat: 19, user_id: get_external_user_id()  };
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
            secret_api_key: String::from("yoyo")
        }
    }

    #[actix_web::main]
    #[test]
    async fn decodes_valid_token_and_creates_user_if_not_found() {
        let ctx = setup_ctx();
        let account = setup_account(&ctx).await;
        let token = get_token(false);

        let req = TestRequest::with_header("nettu-account", account.id.clone())
            .header("Authorization", format!("Bearer {}", token))
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_ok());
        let user_id = User::create_id(&account.id, &get_external_user_id());
        assert!(ctx.user_repo.find(&user_id).await.is_some());
    }

    #[actix_web::main]
    #[test]
    async fn decodes_valid_token_and_for_existing_user() {
        let ctx = setup_ctx();
        let account = setup_account(&ctx).await;
        let token = get_token(false);
        let user = User::new(&account.id, &get_external_user_id());
        ctx.user_repo.insert(&user).await.unwrap();

        let req = TestRequest::with_header("nettu-account", account.id)
            .header("Authorization", format!("Bearer {}", token))
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_ok());
    }

    #[actix_web::main]
    #[test]
    async fn rejects_expired_token() {
        let ctx = setup_ctx();
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
        let ctx = setup_ctx();
        let account = setup_account(&ctx).await;
        let token = get_token(false);

        let req = TestRequest::with_header("Authorization", format!("Bearer {}", token))
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_err());
    }


    #[actix_web::main]
    #[test]
    async fn rejects_valid_token_with_valid_invalid_account_header() {
        let ctx = setup_ctx();
        let account = setup_account(&ctx).await;
        let token = "sajfosajfposajfopaso12";

        let req = TestRequest::with_header("nettu-account", account.id+"s")
            .header("Authorization", format!("Bearer {}", token))
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_err());
    }

    #[actix_web::main]
    #[test]
    async fn rejects_garbage_token_with_valid_account_header() {
        let ctx = setup_ctx();
        let account = setup_account(&ctx).await;
        let token = "sajfosajfposajfopaso12";

        let req = TestRequest::with_header("Authorization", format!("Bearer {}", token))
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_err());
    }


    #[actix_web::main]
    #[test]
    async fn rejects_req_without_headers() {
        let ctx = setup_ctx();
        let account = setup_account(&ctx).await;

        let req = TestRequest::default()
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_err());
    }
}
