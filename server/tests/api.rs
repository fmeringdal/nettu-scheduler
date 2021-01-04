extern crate nettu_scheduler;

mod helpers;

use actix_web::{http::StatusCode, test::TestRequest};

#[actix_web::main]
#[test]
async fn test_status_ok() {
    let req = TestRequest::with_uri("/");
    let res = helpers::perform(req).await;
    assert_eq!(res.status(), StatusCode::OK);
}
