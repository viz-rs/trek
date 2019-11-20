use http::header::CONTENT_TYPE;
use http::status::StatusCode;
use hyper::Body;

/// An HTTP response with a streaming body.
pub type Response = hyper::Response<Body>;

pub trait IntoResponse: Send + Sized {
    fn into_response(self) -> Response;
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response {
        http::Response::builder()
            .header(CONTENT_TYPE, "application/octet-stream")
            .body(Body::from(self))
            .unwrap()
    }
}

impl IntoResponse for &'_ [u8] {
    fn into_response(self) -> Response {
        self.to_vec().into_response()
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        http::Response::builder()
            .header(CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Body::from(self))
            .unwrap()
    }
}

impl IntoResponse for &'_ str {
    fn into_response(self) -> Response {
        self.to_string().into_response()
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        http::Response::builder()
            .status(self)
            .body(Body::empty())
            .unwrap()
    }
}

impl<T, U> IntoResponse for Result<T, U>
where
    T: IntoResponse,
    U: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Ok(r) => r.into_response(),
            Err(r) => {
                let res = r.into_response();
                if res.status().is_success() {
                    panic!(
                        "Attempted to yield error response with success code {:?}",
                        res.status()
                    )
                }
                res
            }
        }
    }
}

impl<T> IntoResponse for http::Response<T>
where
    T: Send + Into<Body>,
{
    fn into_response(self) -> Response {
        self.map(Into::into)
    }
}

/// A response type that modifies the status code.
#[derive(Debug)]
pub struct WithStatus<T> {
    inner: T,
    status: StatusCode,
}

impl<R> IntoResponse for WithStatus<R>
where
    R: IntoResponse,
{
    fn into_response(self) -> Response {
        let mut resp = self.inner.into_response();
        *resp.status_mut() = self.status;
        resp
    }
}

pub fn json<T>(t: &T) -> Response
where
    T: serde::Serialize,
{
    let mut res = http::Response::builder();

    match serde_json::to_vec(t) {
        Ok(v) => res
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(v)),
        Err(e) => {
            log::error!("{}", e);
            res.status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
        }
    }
    .unwrap()
}

pub fn html<T>(t: T) -> Response
where
    T: Send + Into<Body>,
{
    http::Response::builder()
        .header(CONTENT_TYPE, "text/html; charset=utf-8")
        .body(t.into())
        .unwrap()
}
