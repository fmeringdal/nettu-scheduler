use actix_web::{HttpRequest, HttpResponse};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};


pub struct User {
    pub id: String
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Claims {
    exp: usize,          // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize,          // Optional. Issued at (as UTC timestamp)
    user_id: String,         // Optional. Subject (whom token refers to)
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
                Err(_) => return None
            };
            println!("Token i got: {}", token);
            let signing_secret = "nettubookingtest";
            let key = DecodingKey::from_secret(signing_secret.as_ref());
            let res = decode::<Claims>(&token, &key, &Validation::default());
            println!("Parsing res: {:?}", res);
            match res {
                Ok(token_data) => Some(User {
                    id: token_data.claims.user_id.clone()
                }),
                Err(_) => None
            }  
        }
        None => None
    }
}

pub fn protect_route(req: &HttpRequest) -> Result<User, HttpResponse> {
    let res = auth_user_req(req);
    match res {
        Some(user ) => Ok(user),
        None => Err(HttpResponse::Unauthorized().finish())
    }
}