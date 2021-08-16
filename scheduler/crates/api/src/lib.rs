mod account;
mod calendar;
mod error;
mod event;
mod job_schedulers;
mod schedule;
mod service;
mod shared;
mod status;
mod user;

use actix_cors::Cors;
use actix_web::{dev::Server, middleware, web, App, HttpServer};
use job_schedulers::{start_reminder_generation_job_scheduler, start_send_reminders_job};
use nettu_scheduler_domain::{
    Account, AccountIntegration, AccountWebhookSettings, IntegrationProvider, PEMKey, ID,
};
use nettu_scheduler_infra::NettuContext;
use std::net::TcpListener;
use tracing::warn;
use tracing_actix_web::TracingLogger;

pub fn configure_server_api(cfg: &mut web::ServiceConfig) {
    account::configure_routes(cfg);
    calendar::configure_routes(cfg);
    event::configure_routes(cfg);
    schedule::configure_routes(cfg);
    service::configure_routes(cfg);
    status::configure_routes(cfg);
    user::configure_routes(cfg);
}

pub struct Application {
    server: Server,
    port: u16,
    context: NettuContext,
}

impl Application {
    pub async fn new(context: NettuContext) -> Result<Self, std::io::Error> {
        let (server, port) = Application::configure_server(context.clone()).await?;
        Application::start_job_schedulers(context.clone());

        Ok(Self {
            server,
            port,
            context,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    fn start_job_schedulers(context: NettuContext) {
        start_send_reminders_job(context.clone());
        start_reminder_generation_job_scheduler(context);
    }

    async fn configure_server(context: NettuContext) -> Result<(Server, u16), std::io::Error> {
        let port = context.config.port;
        let address = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();

        let server = HttpServer::new(move || {
            let ctx = context.clone();

            App::new()
                .wrap(Cors::permissive())
                .wrap(middleware::Compress::default())
                .wrap(TracingLogger)
                .data(ctx)
                .service(web::scope("/api/v1").configure(|cfg| configure_server_api(cfg)))
        })
        .listen(listener)?
        .workers(4)
        .run();

        Ok((server, port))
    }

    pub async fn start(self) -> Result<(), std::io::Error> {
        self.init_default_account().await;
        self.server.await
    }

    async fn init_default_account(&self) {
        let secret_api_key = match std::env::var("ACCOUNT_API_KEY") {
            Ok(key) => key,
            Err(_) => Account::generate_secret_api_key(),
        };
        if self
            .context
            .repos
            .accounts
            .find_by_apikey(&secret_api_key)
            .await
            .is_none()
        {
            let mut account = Account::default();
            let account_id = std::env::var("ACCOUNT_ID")
                .unwrap_or_default()
                .parse::<ID>()
                .unwrap_or_default();
            account.id = account_id;
            account.secret_api_key = secret_api_key;
            account.settings.webhook = match std::env::var("ACCOUNT_WEBHOOK_URL") {
                Ok(url) => Some(AccountWebhookSettings {
                    url,
                    key: Default::default(),
                }),
                Err(_) => None,
            };

            if let Ok(mut verification_key) = std::env::var("ACCOUNT_PUB_KEY") {
                verification_key = verification_key.replace("\\n", "\n");
                match PEMKey::new(verification_key) {
                    Ok(k) => account.set_public_jwt_key(Some(k)),
                    Err(e) => warn!("Invalid ACCOUNT_PUB_KEY provided: {:?}", e),
                };
            }

            self.context
                .repos
                .accounts
                .insert(&account)
                .await
                .expect("To insert default account");

            let account_google_client_id_env = "ACCOUNT_GOOGLE_CLIENT_ID";
            let account_google_client_secret_env = "ACCOUNT_GOOGLE_CLIENT_SECRET";
            let account_google_redirect_uri_env = "ACCOUNT_GOOGLE_REDIRECT_URI";
            if let Ok(google_client_id) = std::env::var(account_google_client_id_env) {
                let google_client_secret = std::env::var(account_google_client_secret_env)
                    .unwrap_or_else(|_| {
                        panic!(
                            "{} should be specified also when {} is specified.",
                            account_google_client_secret_env, account_google_client_id_env
                        )
                    });
                let google_redirect_uri = std::env::var(account_google_redirect_uri_env)
                    .unwrap_or_else(|_| {
                        panic!(
                            "{} should be specified also when {} is specified.",
                            account_google_redirect_uri_env, account_google_client_id_env
                        )
                    });
                self.context
                    .repos
                    .account_integrations
                    .insert(&AccountIntegration {
                        account_id: account.id.clone(),
                        client_id: google_client_id,
                        client_secret: google_client_secret,
                        redirect_uri: google_redirect_uri,
                        provider: IntegrationProvider::Google,
                    })
                    .await
                    .expect("To insert google account integration");
            }
            let account_outlook_client_id_env = "ACCOUNT_OUTLOOK_CLIENT_ID";
            let account_outlook_client_secret_env = "ACCOUNT_OUTLOOK_CLIENT_SECRET";
            let account_outlook_redirect_uri_env = "ACCOUNT_OUTLOOK_REDIRECT_URI";
            if let Ok(outlook_client_id) = std::env::var(account_outlook_client_id_env) {
                let outlook_client_secret = std::env::var(account_outlook_client_secret_env)
                    .unwrap_or_else(|_| {
                        panic!(
                            "{} should be specified also when {} is specified.",
                            account_outlook_client_secret_env, account_outlook_client_id_env
                        )
                    });
                let outlook_redirect_uri = std::env::var(account_outlook_redirect_uri_env)
                    .unwrap_or_else(|_| {
                        panic!(
                            "{} should be specified also when {} is specified.",
                            account_outlook_redirect_uri_env, account_outlook_client_id_env
                        )
                    });
                self.context
                    .repos
                    .account_integrations
                    .insert(&AccountIntegration {
                        account_id: account.id.clone(),
                        client_id: outlook_client_id,
                        client_secret: outlook_client_secret,
                        redirect_uri: outlook_redirect_uri,
                        provider: IntegrationProvider::Outlook,
                    })
                    .await
                    .expect("To insert outlook account integration");
            }
        }
    }
}
