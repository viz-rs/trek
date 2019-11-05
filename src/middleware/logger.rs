use crate::{Context, Middleware, Response};
use futures::future::BoxFuture;

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
            let path = cx.uri().path().to_owned();
            let method = cx.method().as_str().to_owned();
            log::trace!("IN => {} {}", method, path);
            let start = std::time::Instant::now();
            let res = cx.next().await;
            let status = res.status();
            log::info!(
                "{} {} {} {}ms",
                method,
                path,
                status.as_str(),
                start.elapsed().as_millis()
            );
            res
        })
    }
}
