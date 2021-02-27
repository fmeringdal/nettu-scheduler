use std::collections::HashMap;

use nettu_scheduler_api::{
    dev::account::{Account, CreateAccountResponse},
    Application,
};
use nettu_scheduler_infra::{setup_context, Config};

pub struct TestApp {
    pub address: String,
    pub config: Config,
}

#[derive(Debug)]
pub enum APIError {
    Network,
    MalformedResponse,
    UnexpectedStatusCode(reqwest::Response),
}
pub type APIResponse<T> = Result<T, APIError>;

impl TestApp {
    pub async fn get_account(&self, api_key: &str) -> APIResponse<Account> {
        let expected_status_code = reqwest::StatusCode::OK;

        let res = match reqwest::Client::new()
            .get(&format!("{}/account", &self.address))
            .header("x-api-key", api_key)
            .send()
            .await
        {
            Ok(res) => res,
            Err(_) => return Err(APIError::Network),
        };

        let status = res.status();
        if status != expected_status_code {
            return Err(APIError::UnexpectedStatusCode(res));
        }

        let data = match res.json::<Account>().await {
            Ok(data) => data,
            Err(_) => return Err(APIError::MalformedResponse),
        };

        Ok(data)
    }

    pub async fn create_account(&self, code: &str) -> APIResponse<CreateAccountResponse> {
        let expected_status_code = reqwest::StatusCode::CREATED;

        let mut body = HashMap::new();
        body.insert("code", code);

        let res = match reqwest::Client::new()
            .post(&format!("{}/account", &self.address))
            .json(&body)
            .send()
            .await
        {
            Ok(res) => res,
            Err(_) => return Err(APIError::Network),
        };

        let status = res.status();
        if status != expected_status_code {
            return Err(APIError::UnexpectedStatusCode(res));
        }

        let data = match res.json::<CreateAccountResponse>().await {
            Ok(data) => data,
            Err(_) => return Err(APIError::MalformedResponse),
        };

        Ok(data)
    }

    pub async fn check_health(&self) -> reqwest::Response {
        reqwest::Client::new()
            .get(&format!("{}/", &self.address))
            .header("Content-Type", "application/json")
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

// Launch the application as a background task
pub async fn spawn_app() -> TestApp {
    let mut ctx = setup_context().await;
    ctx.config.port = 0; // Random port
    ctx.config.create_account_secret_code = "123".into(); // Overriding create account secret

    let config = ctx.config.clone();
    let application = Application::new(ctx)
        .await
        .expect("Failed to build application.");

    let address = format!("http://localhost:{}", application.port());
    println!("{}", address);
    let _ = actix_web::rt::spawn(async move {
        application
            .start()
            .await
            .expect("Expected application to start");
    });

    TestApp { address, config }
}
