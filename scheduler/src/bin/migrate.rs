use nettu_scheduler_infra::run_migration;

#[actix_web::main]
async fn main() -> () {
    run_migration().await.unwrap()
}
