use crate::{helpers::Writer, IntoResponse, Response};
use bytes::BytesMut;
use hyper::{header, Body, StatusCode};
use std::{
    error, fmt,
    io::{self, Write},
    result,
};

pub type Result<T = Response, E = Error> = result::Result<T, E>;

pub struct Error {
    e: Box<dyn ErrorResponse + Send + Sync>,
}

impl Error {
    pub fn new(e: Box<dyn ErrorResponse + Send + Sync>) -> Self {
        Self { e }
    }

    pub fn as_error_response(&self) -> &dyn ErrorResponse {
        self.e.as_ref()
    }
}

pub trait ErrorResponse: error::Error + Send + Sync {
    fn error_response(&self) -> Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }

    fn render_response(&self) -> Response {
        let mut resp = self.error_response();
        let mut buf = BytesMut::new();
        let _ = write!(Writer(&mut buf), "{}", self);
        resp.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        *resp.body_mut() = Body::from(buf.freeze());
        resp
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.e, f)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", &self.e)
    }
}

impl error::Error for Error {}

/// `Error` for any error that implements `ErrorResponse`
impl<T: ErrorResponse + error::Error + 'static> From<T> for Error {
    fn from(e: T) -> Error {
        Error::new(Box::new(e))
    }
}

/// Return `InternalServerError` for `io::Error`
impl ErrorResponse for io::Error {
    fn error_response(&self) -> Response {
        match self.kind() {
            io::ErrorKind::NotFound => StatusCode::NOT_FOUND,
            io::ErrorKind::PermissionDenied => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        self.as_error_response().error_response()
    }
}
