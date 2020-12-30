use actix_web::{HttpRequest, HttpResponse};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

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

pub fn auth_user_req(req: &HttpRequest) -> Option<User> {
    let token = req.headers().get("authorization");
    match token {
        Some(token) => {
            let token = match token.to_str() {
                Ok(token) => parse_authtoken_header(token),
                Err(_) => return None,
            };
            let res = decode::<Claims>(&token, &DECODING_KEY, &Validation::new(Algorithm::RS256));
            match res {
                Ok(token_data) => Some(User {
                    id: token_data.claims.user_id.clone(),
                }),
                Err(_) => None,
            }
        }
        None => None,
    }
}

pub fn protect_route(req: &HttpRequest) -> Result<User, HttpResponse> {
    let res = auth_user_req(req);
    match res {
        Some(user) => Ok(user),
        None => Err(HttpResponse::Unauthorized().finish()),
    }
}
