/// An HTTP response with a streaming body.
pub type Response = hyper::Response<hyper::Body>;

pub trait IntoResponse: Send + Sized {
    fn into_response(self) -> Response;
}
