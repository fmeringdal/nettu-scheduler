use reqwest::{Client, Method, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};

pub(crate) struct BaseClient {
    address: String,
    api_key: Option<String>,
}

#[derive(Debug)]
pub enum APIError {
    Network,
    MalformedResponse,
    UnexpectedStatusCode(StatusCode),
}
pub type APIResponse<T> = Result<T, APIError>;

impl BaseClient {
    pub fn new(address: String) -> Self {
        Self {
            address,
            api_key: None,
        }
    }

    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = Some(api_key);
    }

    fn get_client(&self, method: Method, path: String) -> RequestBuilder {
        let client = Client::new();
        let url = format!("{}/{}", self.address, path);
        let builder = match method {
            Method::GET => client.get(&url),
            Method::POST => client.post(&url),
            Method::PUT => client.put(&url),
            Method::DELETE => client.delete(&url),
            _ => unimplemented!(),
        };

        if let Some(api_key) = &self.api_key {
            builder.header("x-api-key", api_key.clone())
        } else {
            builder
        }
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(
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
            return Err(APIError::UnexpectedStatusCode(status));
        }

        let data = match res.json::<T>().await {
            Ok(data) => data,
            Err(_) => return Err(APIError::MalformedResponse),
        };

        Ok(data)
    }

    pub async fn post<T: for<'de> Deserialize<'de>, S: Serialize>(
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
            return Err(APIError::UnexpectedStatusCode(status));
        }

        let data = match res.json::<T>().await {
            Ok(data) => data,
            Err(_) => return Err(APIError::MalformedResponse),
        };

        Ok(data)
    }
}
