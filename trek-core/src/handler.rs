//! Handler traits
//!
//! Thanks to repos:
//!     * https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=2f9af4a2114fa66ec3268bc64163026c
//!     * https://github.com/rustasync/tide/blob/master/tide-core/src/endpoint.rs
//!     * https://github.com/rustasync/tide/blob/master/tide-core/src/internal.rs
//!     * https://github.com/seanmonstar/warp/blob/master/src/generic.rs

use futures::future::{BoxFuture, Future, FutureExt};

use crate::middleware::Middleware;
use crate::response::{IntoResponse, Response};

pub trait Handler<Context>: Send + Sync + 'static {
    type Fut: Future<Output = Response> + Send + 'static;

    fn call(&self, cx: Context) -> Self::Fut;
}

impl<Context, F, Fut> Handler<Context> for F
where
    F: Send + Sync + 'static + Fn(Context) -> Fut,
    Fut: Future + Send + 'static,
    Fut::Output: IntoResponse + Send + 'static,
{
    type Fut = BoxFuture<'static, Response>;

    fn call(&self, cx: Context) -> Self::Fut {
        let fut = (self)(cx);
        Box::pin(async move { fut.await.into_response() })
    }
}

pub type DynHandler<Context> =
    dyn (Fn(Context) -> BoxFuture<'static, Response>) + 'static + Send + Sync;

pub fn into_dyn_handler<Context>(f: impl Handler<Context>) -> Box<DynHandler<Context>> {
    Box::new(move |cx| f.call(cx).boxed())
}

pub fn into_middleware<Context>(f: impl Handler<Context>) -> impl Middleware<Context>
where
    Context: Send + 'static,
{
    let f = into_dyn_handler(f);
    Box::new(move |cx| (f)(cx))
}
