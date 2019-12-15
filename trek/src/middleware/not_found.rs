use crate::{Body, Context, Middleware, Response};
use futures::future::BoxFuture;
use http::status::StatusCode;

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
            let mut res = Response::new(Body::empty());
            *res.status_mut() = StatusCode::NOT_FOUND;
            res
        })
    }
}
