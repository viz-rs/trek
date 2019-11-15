use crate::{IntoResponse, Response};

pub struct Error(Box<dyn std::error::Error + Send + Sync>);

impl Error {
    pub fn new(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self(e)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("Error").field("err", &self.0).finish()
    }
}

pub type Result<T = Response, E = Error> = std::result::Result<T, E>;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        hyper::Response::builder()
            .status(hyper::StatusCode::NOT_FOUND)
            .body(hyper::Body::empty())
            .unwrap()
    }
}
