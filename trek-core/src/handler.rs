//! Handler traits
//!
//! Thanks to repos:
//!     * https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=2f9af4a2114fa66ec3268bc64163026c
//!     * https://github.com/rustasync/tide/blob/master/tide-core/src/endpoint.rs
//!     * https://github.com/rustasync/tide/blob/master/tide-core/src/internal.rs
//!     * https://github.com/seanmonstar/warp/blob/master/src/generic.rs

use futures::future::{BoxFuture, Future};

use crate::middleware::DynMiddleware;
use crate::response::{IntoResponse, Response};

pub trait Handler<Context, Output>: Send + Sync + 'static {
    type Fut: Future<Output = Output> + Send + 'static;

    fn call(&self, cx: Context) -> Self::Fut;
}

impl<Context, Output, F, Fut> Handler<Context, Output> for F
where
    F: Send + Sync + 'static + Fn(Context) -> Fut,
    Fut: Future<Output = Output> + Send + 'static,
    Fut::Output: Send + 'static,
{
    type Fut = Fut;

    fn call(&self, cx: Context) -> Self::Fut {
        (self)(cx)
    }
}

pub type DynHandler<Context, Output> =
    dyn (Fn(Context) -> BoxFuture<'static, Output>) + 'static + Send + Sync;

pub fn into_dyn_handler<Context, Output>(
    f: impl Handler<Context, Output>,
) -> Box<DynHandler<Context, Output>> {
    Box::new(move |cx| Box::pin(f.call(cx)))
}

pub fn wrap_handler<Context, Output>(
    f: impl Handler<Context, Output>,
) -> impl Handler<Context, Response>
where
    Output: IntoResponse + Send + 'static,
{
    Box::new(move |cx| {
        let fut = f.call(cx);
        Box::pin(async move { fut.await.into_response() })
    })
}

pub fn into_dyn_middleware<Context, Output>(
    f: impl Handler<Context, Output>,
) -> Box<DynMiddleware<Context, Response>>
where
    Output: IntoResponse + Send + 'static,
{
    Box::new(move |cx| {
        let fut = f.call(cx);
        Box::pin(async move { fut.await.into_response() })
    })
}
