/// An HTTP request with a streaming body.
pub type Request = hyper::Request<hyper::Body>;

pub trait FromRequest {}
