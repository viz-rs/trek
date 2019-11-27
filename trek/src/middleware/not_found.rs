use crate::{Context, Middleware, Response};
use futures::future::BoxFuture;

#[derive(Debug, Clone, Default)]
pub struct NotFound;

impl NotFound {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<State: Send + Sync + 'static> Middleware<Context<State>> for NotFound {
    fn call<'a>(&'a self, _: Context<State>) -> BoxFuture<'a, Response> {
        Box::pin(async {
            hyper::Response::builder()
                .status(http::status::StatusCode::NOT_FOUND)
                .body(hyper::Body::empty())
                .unwrap()
        })
    }
}
