extern crate nettu_scheduler;

mod helpers;

use actix_web::{http, test};

#[actix_web::main]
#[test]
async fn test_status_ok() {
    let req = test::TestRequest::with_uri("/");
    let res = helpers::perform(req).await;
    assert_eq!(res.status(), http::StatusCode::OK);
}
