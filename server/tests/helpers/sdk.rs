use nettu_scheduler_api::dev::{
    account::{Account, CreateAccountResponse},
    status::StatusResponse,
};
use reqwest::{Client, Method, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct NettuSDK {
    address: String,
    api_key: Option<String>,
}

#[derive(Debug)]
pub enum APIError {
    Network,
    MalformedResponse,
    UnexpectedStatusCode(reqwest::Response),
}
pub type APIResponse<T> = Result<T, APIError>;

impl NettuSDK {
    pub fn new(address: String) -> Self {
        Self {
            address,
            api_key: None,
        }
    }

    pub fn set_admin_key(&mut self, api_key: String) {
        self.api_key = Some(api_key);
    }

    fn get_client(&self, method: Method, path: String) -> reqwest::RequestBuilder {
        let client = Client::new();
        let url = format!("{}/{}", self.address, path);
        let builder = match method {
            Method::GET => client.get(&url),
            Method::POST => client.post(&url),
            _ => unimplemented!(),
        };

        if let Some(api_key) = &self.api_key {
            builder.header("x-api-key", api_key)
        } else {
            builder
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        path: String,
        expected_status_code: StatusCode,
    ) -> APIResponse<T> {
        let res = match self.get_client(Method::GET, path).send().await {
            Ok(res) => res,
            Err(_) => return Err(APIError::Network),
        };

        let status = res.status();
        if status != expected_status_code {
            return Err(APIError::UnexpectedStatusCode(res));
        }

        let data = match res.json::<T>().await {
            Ok(data) => data,
            Err(_) => return Err(APIError::MalformedResponse),
        };

        Ok(data)
    }

    async fn post<T: for<'de> Deserialize<'de>, S: Serialize>(
        &self,
        body: S,
        path: String,
        expected_status_code: StatusCode,
    ) -> APIResponse<T> {
        let res = match self.get_client(Method::POST, path).json(&body).send().await {
            Ok(res) => res,
            Err(_) => return Err(APIError::Network),
        };

        let status = res.status();
        if status != expected_status_code {
            return Err(APIError::UnexpectedStatusCode(res));
        }

        let data = match res.json::<T>().await {
            Ok(data) => data,
            Err(_) => return Err(APIError::MalformedResponse),
        };

        Ok(data)
    }

    pub async fn get_account(&self) -> APIResponse<Account> {
        self.get("account".into(), StatusCode::OK).await
    }

    pub async fn create_account(&self, code: &str) -> APIResponse<CreateAccountResponse> {
        let mut body = HashMap::new();
        body.insert("code", code);
        self.post(body, "account".into(), StatusCode::CREATED).await
    }

    pub async fn check_health(&self) -> APIResponse<StatusResponse> {
        self.get("".into(), StatusCode::OK).await
    }
}
