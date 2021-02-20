#[derive(Error, Debug)]
pub enum NettuError {
    #[error("data store disconnected")]
    InternalError,
    #[error("Invalid data provided: Error message: `{0}`")]
    BadClientData(String),
    #[error("There was a conflict with the request. Error message: `{0}`")]
    Conflict(String),
    #[error("Unauthorized request. Error message: `{0}`")]
    Unauthorized(String),
    #[error(
        "Unidentifiable client. Must include the `nettu-account` header. Error message: `{0}`"
    )]
    UnidentifiableClient(String),
    #[error("404 Not found. Error message: `{0}`")]
    NotFound(String),
}

impl actix_web::error::ResponseError for NettuError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            NettuError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            NettuError::BadClientData(_) => StatusCode::BAD_REQUEST,
            NettuError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            NettuError::Conflict(_) => StatusCode::CONFLICT,
            NettuError::NotFound(_) => StatusCode::NOT_FOUND,
            NettuError::UnidentifiableClient(_) => StatusCode::UNAUTHORIZED,
        }
    }
}
