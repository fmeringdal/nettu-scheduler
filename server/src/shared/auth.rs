use actix_web::{HttpRequest, HttpResponse};
use company::domain::Company;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{company::{self, repos::ICompanyRepo}, user::{domain::User, repos::IUserRepo}};

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
    pub company_repo: Arc<dyn ICompanyRepo>,
}

async fn create_user_if_not_exists(external_user_id: &str, company_id: &str, ctx: &AuthContext) -> User {
    match ctx.user_repo.find(external_user_id, company_id).await {
        Some(user) => user,
        None => {
            // create user
            // todo: in future there will be a create user admin endpoint
            let user = User {
                id: ObjectId::new().to_string(),
                external_id: String::from(external_user_id),
                company_id: String::from(company_id)
            };

            ctx.user_repo.insert(&user).await;

            user
        }
    }
}

pub async fn auth_user_req(
    req: &HttpRequest,
    company: &Company,
    ctx: &AuthContext,
) -> Option<User> {
    let token = req.headers().get("authorization");
    match token {
        Some(token) => {
            let token = match token.to_str() {
                Ok(token) => parse_authtoken_header(token),
                Err(_) => return None,
            };

            match decode_token(company, &token) {
                Ok(claims) => {
                    let user = create_user_if_not_exists(&claims.user_id, &company.id, ctx).await;
                    Some(user)
                },
                Err(_) => None,
            }
        }
        None => None,
    }
}

pub async fn get_client_company(req: &HttpRequest, ctx: &AuthContext) -> Option<Company> {
    let account = req.headers().get("nettu-account");

    match account {
        Some(acc) => match acc.to_str() {
            Ok(acc) => ctx.company_repo.find(acc).await,
            Err(_) => None,
        },
        None => None,
    }
}

fn decode_token(company: &Company, token: &str) -> anyhow::Result<Claims> {
    let public_key = base64::decode(&company.public_key_b64)?;
    let decoding_key = DecodingKey::from_rsa_pem(&public_key)?;
    let token_data = decode::<Claims>(&token, &decoding_key, &Validation::new(Algorithm::RS256))?;
    Ok(token_data.claims)
}

pub async fn protect_route(req: &HttpRequest, ctx: &AuthContext) -> Result<User, HttpResponse> {
    let company = match get_client_company(req, ctx).await {
        Some(comp) => comp,
        None => {
            return Err(HttpResponse::Unauthorized()
                .body("Unable to find the company the client belongs to"))
        }
    };
    let res = auth_user_req(req, &company, ctx).await;

    match res {
        Some(user) => Ok(user),
        None => Err(HttpResponse::Unauthorized().finish()),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{company::repos::InMemoryCompanyRepo, user::repos::InMemoryUserRepo};
    use actix_web::test::TestRequest;
    use jsonwebtoken::{encode, Header, EncodingKey, Algorithm};

    async fn setup_company(ctx: &AuthContext) -> Company {
        let company = get_company();
        ctx.company_repo.insert(&company).await.unwrap();
        company
    }

    fn setup_ctx() -> AuthContext {
        let company_repo = InMemoryCompanyRepo::new();
        let user_repo = InMemoryUserRepo::new();
        AuthContext {
            company_repo: Arc::new(company_repo),
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

    fn get_company() -> Company {
        let pub_key = std::fs::read("./config/test_public_rsa_key.crt").unwrap();
        let public_key_b64 = base64::encode(pub_key);
        Company {
            id: String::from("nettu"),
            public_key_b64
        }
    }

    #[actix_web::main]
    #[test]
    async fn decodes_valid_token_and_creates_user_if_not_found() {
        let ctx = setup_ctx();
        let comp = setup_company(&ctx).await;
        let token = get_token(false);

        let req = TestRequest::with_header("nettu-account", comp.id.clone())
            .header("Authorization", format!("Bearer {}", token))
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_ok());
        assert!(ctx.user_repo.find(&get_external_user_id(), &comp.id).await.is_some());
    }

    #[actix_web::main]
    #[test]
    async fn decodes_valid_token_and_for_existing_user() {
        let ctx = setup_ctx();
        let comp = setup_company(&ctx).await;
        let token = get_token(false);
        let user = User {
            id: ObjectId::new().to_string(),
            external_id: get_external_user_id(),
            company_id: comp.id.clone()
        };
        ctx.user_repo.insert(&user).await;

        let req = TestRequest::with_header("nettu-account", comp.id)
            .header("Authorization", format!("Bearer {}", token))
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_ok());
    }

    #[actix_web::main]
    #[test]
    async fn rejects_expired_token() {
        let ctx = setup_ctx();
        let comp = setup_company(&ctx).await;
        let token = get_token(true);

        let req = TestRequest::with_header("nettu-account", comp.id)
            .header("Authorization", format!("Bearer {}", token))
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_err());
    }

    #[actix_web::main]
    #[test]
    async fn rejects_valid_token_without_account_header() {
        let ctx = setup_ctx();
        let comp = setup_company(&ctx).await;
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
        let comp = setup_company(&ctx).await;
        let token = "sajfosajfposajfopaso12";

        let req = TestRequest::with_header("nettu-account", comp.id+"s")
            .header("Authorization", format!("Bearer {}", token))
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_err());
    }

    #[actix_web::main]
    #[test]
    async fn rejects_garbage_token_with_valid_account_header() {
        let ctx = setup_ctx();
        let comp = setup_company(&ctx).await;
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
        let comp = setup_company(&ctx).await;

        let req = TestRequest::default()
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_err());
    }
}
