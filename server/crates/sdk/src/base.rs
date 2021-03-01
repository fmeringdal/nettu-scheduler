use reqwest::{Client, Method, RequestBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};

pub(crate) struct BaseClient {
    address: String,
    api_key: Option<String>,
}

#[derive(Debug)]
pub enum APIError {
    Network,
    MalformedResponse,
    Unauthorized,
    Unauthenticated,
    BadClientData,
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

    fn check_status_code<T>(
        &self,
        res: &Response,
        expected_status_code: StatusCode,
    ) -> Result<(), APIError> {
        let status = res.status();
        if status != expected_status_code {
            return Err(APIError::UnexpectedStatusCode(status));
        }
        Ok(())
    }

    async fn get_json_response<T: for<'de> Deserialize<'de>>(
        &self,
        res: Response,
    ) -> APIResponse<T> {
        res.json::<T>()
            .await
            .map_err(|_| APIError::MalformedResponse)
    }

    async fn handle_api_response<T: for<'de> Deserialize<'de>>(
        &self,
        res: Response,
        expected_status_code: StatusCode,
    ) -> APIResponse<T> {
        self.check_status_code::<T>(&res, expected_status_code)?;
        self.get_json_response(res).await
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
        self.handle_api_response(res, expected_status_code).await
    }

    pub async fn delete<T: for<'de> Deserialize<'de>>(
        &self,
        path: String,
        expected_status_code: StatusCode,
    ) -> APIResponse<T> {
        let res = match self.get_client(Method::DELETE, path).send().await {
            Ok(res) => res,
            Err(_) => return Err(APIError::Network),
        };
        self.handle_api_response(res, expected_status_code).await
    }

    pub async fn put<T: for<'de> Deserialize<'de>, S: Serialize>(
        &self,
        body: S,
        path: String,
        expected_status_code: StatusCode,
    ) -> APIResponse<T> {
        let res = match self.get_client(Method::PUT, path).json(&body).send().await {
            Ok(res) => res,
            Err(_) => return Err(APIError::Network),
        };
        self.handle_api_response(res, expected_status_code).await
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
        self.handle_api_response(res, expected_status_code).await
    }
}
