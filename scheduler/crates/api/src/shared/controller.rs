// use actix_web::{web, HttpResponse};
// use serde::Deserialize;

// use crate::{error::NettuError, user::create_user::CreateUserUseCase};

// use super::usecase::UseCase;

// // #[async_trait::async_trait(?Send)]
// // pub trait Controller<U: UseCase> {
// //     type PathParams: for<'de> Deserialize<'de>;
// //     type Body: for<'de> Deserialize<'de>;
// //     type QueryParams: for<'de> Deserialize<'de>;

// //     fn handler(
// //         path: Self::PathParams,
// //         body: Self::Body,
// //         query: Self::QueryParams,
// //     ) -> Result<U, NettuError>;

// //     fn handle_error(e: U::Error) -> NettuError;
// //     fn handle_ok(res: U::Response) -> HttpResponse;

// //     async fn execute_controller(
// //         path: web::Path<Self::PathParams>,
// //         body: web::Json<Self::Body>,
// //         query: web::Query<Self::QueryParams>,
// //     ) -> Result<HttpResponse, NettuError> {
// //         // Err(NettuError::Conflict("dfasf".into()))
// //         Ok(HttpResponse::Ok().finish())
// //     }
// // }

// #[async_trait::async_trait(?Send)]
// pub trait APIController: UseCase {
//     fn handle_error(e: Self::Error) -> NettuError;
//     fn handle_ok(res: Self::Response) -> HttpResponse;

//     async fn execute_controller<P, B, Q>(
//         path: web::Path<P>,
//         body: web::Json<B>,
//         query: web::Query<Q>,
//     ) -> Result<HttpResponse, NettuError> {
//         // Err(NettuError::Conflict("dfasf".into()))
//         Ok(HttpResponse::Ok().finish())
//     }
// }

// #[derive(Debug, Deserialize)]
// struct Params {}

// // struct Dummy;
// // impl Controller<CreateUserUseCase> for Dummy {
// //     type PathParams = Params;
// //     type Body = Params;
// //     type QueryParams = Params;

// //     fn handle_error(e: <CreateUserUseCase as UseCase>::Error) -> NettuError {
// //         todo!()
// //     }

// //     fn handle_ok(res: <CreateUserUseCase as UseCase>::Response) -> HttpResponse {
// //         todo!()
// //     }

// //     fn handler(
// //         path: Self::PathParams,
// //         body: Self::Body,
// //         query: Self::QueryParams,
// //     ) -> Result<CreateUserUseCase, NettuError> {
// //         Err(NettuError::Conflict("".into()))
// //     }
// // }

// pub fn configure_routes(cfg: &mut web::ServiceConfig) {
//     // cfg.route("/calendar", web::post().to(Dummy::execute_controller));
// }
