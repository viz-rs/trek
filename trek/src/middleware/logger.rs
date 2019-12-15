use crate::{Context, Middleware, Response};
use futures::future::BoxFuture;
use std::time::Instant;

#[derive(Debug, Clone, Default)]
pub struct Logger;

impl Logger {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<State: Send + Sync + 'static> Middleware<Context<State>> for Logger {
    fn call<'a>(&'a self, cx: Context<State>) -> BoxFuture<'a, Response> {
        Box::pin(async move {
            let start = Instant::now();
            let path = cx.uri().path().to_owned();
            let method = cx.method().as_str().to_owned();
            log::trace!("IN => {} {}", method, path);
            let res = cx.next().await;
            log::info!(
                "{} {} {} {}ms",
                method,
                path,
                res.status().as_str(),
                start.elapsed().as_millis()
            );
            res
        })
    }
}
