use actix_web::{HttpRequest, HttpResponse};
use company::domain::Company;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    company::{self, repos::ICompanyRepo},
    user::repos::IUserRepo,
};

lazy_static::lazy_static! {
    static ref PUBLIC_KEY: Vec<u8> = std::fs::read("./config/pubkey.pem").expect("Public signing verification key to be present");
    static ref DECODING_KEY: DecodingKey<'static> = {
        DecodingKey::from_rsa_pem(PUBLIC_KEY.as_ref()).unwrap()
    };
}

pub struct User {
    pub id: String,
}

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
            println!("Token found: {:?}", token);

            match decode_token(company, &token) {
                Ok(claims) => Some(User {
                    id: claims.user_id.clone(),
                }),
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
        let pub_key = std::fs::read("./config/test_public_rsa_key.crt").unwrap();
        let pub_key = base64::encode(pub_key);
        let company = Company {
            id: String::from("nettu"),
            public_key_b64: pub_key
        };
        ctx.company_repo.insert(&company).await;
        company
    }

    fn get_token(expired: bool) -> String {
        let priv_key = std::fs::read("./config/test_private_rsa_key.pem").unwrap();
        let exp = if expired {
            100
        } else {
            2609418990073
        };
        let claims = Claims { exp, iat: 19, user_id: String::from("cool") };
        let enc_key = EncodingKey::from_rsa_pem(&priv_key).unwrap();
        encode(&Header::new(Algorithm::RS256), &claims, &enc_key).unwrap()
    }

    #[actix_web::main]
    #[test]
    async fn decodes_valid_token() {
        let company_repo = InMemoryCompanyRepo::new();
        let user_repo = InMemoryUserRepo::new();
        let ctx = AuthContext {
            company_repo: Arc::new(company_repo),
            user_repo: Arc::new(user_repo),
        };
        let comp = setup_company(&ctx).await;

        let token = get_token(false);

        let req = TestRequest::with_header("nettu-account", comp.id)
            .header("Authorization", format!("Bearer {}", token))
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_ok());
    }

    #[actix_web::main]
    #[test]
    async fn rejects_expired_token() {
        let company_repo = InMemoryCompanyRepo::new();
        let user_repo = InMemoryUserRepo::new();
        let ctx = AuthContext {
            company_repo: Arc::new(company_repo),
            user_repo: Arc::new(user_repo),
        };
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
        let company_repo = InMemoryCompanyRepo::new();
        let user_repo = InMemoryUserRepo::new();
        let ctx = AuthContext {
            company_repo: Arc::new(company_repo),
            user_repo: Arc::new(user_repo),
        };
        let comp = setup_company(&ctx).await;

        let token = String::from("eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwidXNlcklkIjoiY29vbHRlc3R1c2VyIiwiZXhwIjoxOTE2MjM5MDIyLCJpYXQiOjE1MTYyMzkwMjJ9.TRRzG_Vakpsa3n7Gt9lYSKqwzK6FvxTSWnxMI04LUQEAXfZfhTWGooQHnAmVDxHdT-XZlExJ0MepK27NhHQ8Xww7Mu91YE_kaiRkXMo1SmdFjzLyBLOBhJrYUrqY4wYb7uzZBrIVy0rDNwKZKUtJ5MnSF2x-bPPFD1IQjNqit--D-UpHrKqoDXp5j7T0E5CzhYtyXBQsX2uUb47lbmQfyuMiKXCM85oDMNzESoTAHWv0FNhyubV5ZeGqZedPHsYI2qQr8be4vdjfaS0_tQ5mP9DpIVK_aOVJPKMzdwaFyI8Oqr886VFLkXv7kKkuIipQK__tzoFmYm1V22wEzUp0-Q");

        let req = TestRequest::with_header("Authorization", format!("Bearer {}", token))
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_err());
    }

    #[actix_web::main]
    #[test]
    async fn rejects_garbage_token_with_valid_account_header() {
        let company_repo = InMemoryCompanyRepo::new();
        let user_repo = InMemoryUserRepo::new();
        let ctx = AuthContext {
            company_repo: Arc::new(company_repo),
            user_repo: Arc::new(user_repo),
        };
        let comp = setup_company(&ctx).await;

        let token = String::from("eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwidXNlcklkIjoiY29vbHRlc3R1c2VyIiwiZXhwIjoxOTE2MjM5MDIyLCJpYXQiOjE1MTYyMzkwMjJ.TRRzG_Vakpsa3n7Gt9lYSKqwzK6FvxTSWnxMI04LUQEAXfZfhTWGooQHnAmVDxHdT-XZlExJ0MepK27NhHQ8Xww7Mu91YE_kaiRkXMo1SmdFjzLyBLOBhJrYUrqY4wYb7uzZBrIVy0rDNwKZKUtJ5MnSF2x-bPPFD1IQjNqit--D-UpHrKqoDXp5j7T0E5CzhYtyXBQsX2uUb47lbmQfyuMiKXCM85oDMNzESoTAHWv0FNhyubV5ZeGqZedPHsYI2qQr8be4vdjfaS0_tQ5mP9DpIVK_aOVJPKMzdwaFyI8Oqr886VFLkXv7kKkuIipQK__tzoFmYm1V22wEzUp0-Q");

        let req = TestRequest::with_header("Authorization", format!("Bearer {}", token))
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_err());
    }

    #[actix_web::main]
    #[test]
    async fn rejects_req_without_headers() {
        let company_repo = InMemoryCompanyRepo::new();
        let user_repo = InMemoryUserRepo::new();
        let ctx = AuthContext {
            company_repo: Arc::new(company_repo),
            user_repo: Arc::new(user_repo),
        };
        let comp = setup_company(&ctx).await;

        let req = TestRequest::default()
            .to_http_request();
        let res = protect_route(&req, &ctx).await;
        assert!(res.is_err());
    }
}
