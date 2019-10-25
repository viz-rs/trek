//! Handler traits
//!
//! Thanks to repos:
//!     * https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=2f9af4a2114fa66ec3268bc64163026c
//!     * https://github.com/rustasync/tide/blob/master/tide-core/src/endpoint.rs
//!     * https://github.com/rustasync/tide/blob/master/tide-core/src/internal.rs
//!     * https://api.rocket.rs/v0.4/src/rocket/handler.rs.html#134-146
//!     * https://github.com/seanmonstar/warp/blob/master/src/generic.rs

use futures::future::{BoxFuture, Future};

use crate::middleware::Middleware;
use crate::response::{IntoResponse, Response};

pub trait Handler<Context>: Cloneable<Context> + Send + Sync + 'static {
    type Fut: Future<Output = Response> + Send + 'static;

    fn call(&self, cx: Context) -> Self::Fut;
}

impl<Context, F, Fut> Handler<Context> for F
where
    F: Clone + Send + Sync + 'static + Fn(Context) -> Fut,
    Fut: Future + Send + 'static,
    Fut::Output: IntoResponse + Send + 'static,
{
    type Fut = BoxFuture<'static, Response>;

    fn call(&self, cx: Context) -> Self::Fut {
        let fut = (self)(cx);
        Box::pin(async move { fut.await.into_response() })
    }
}

pub type DynHandler<Context> = dyn Handler<Context, Fut = BoxFuture<'static, Response>>;

pub trait Cloneable<Context> {
    /// Clones `self`.
    fn clone_handler(&self) -> Box<DynHandler<Context>>;
}

impl<Context, F: Handler<Context, Fut = BoxFuture<'static, Response>>, Fut> Cloneable<Context> for F
where
    F: Clone + Send + Sync + 'static + Fn(Context) -> Fut,
    Fut: Future + Send + 'static,
    Fut::Output: IntoResponse + Send + 'static,
{
    #[inline(always)]
    fn clone_handler(&self) -> Box<DynHandler<Context>> {
        Box::new(self.clone())
    }
}

impl<Context> Clone for Box<DynHandler<Context>> {
    #[inline(always)]
    fn clone(&self) -> Box<DynHandler<Context>> {
        self.clone_handler()
    }
}

pub fn into_box_dyn_handler<Context>(f: impl Handler<Context> + Clone) -> Box<DynHandler<Context>> {
    Box::new(move |cx| f.call(cx))
}

pub fn into_middleware<Context>(f: impl Handler<Context> + Clone) -> impl Middleware<Context>
where
    Context: Send + 'static,
{
    let f = into_box_dyn_handler(f);
    Box::new(move |cx| f.call(cx))
}

pub fn box_dyn_handler_into_middleware<Context>(
    f: Box<DynHandler<Context>>,
) -> impl Middleware<Context>
where
    Context: Send + 'static,
{
    Box::new(move |cx| f.call(cx))
}

// pub type DynHandler<Context> =
//     dyn (Fn(Context) -> BoxFuture<'static, Response>) + 'static + Send + Sync;

// pub fn into_box_dyn_handler<Context>(f: impl Handler<Context>) -> Box<DynHandler<Context>> {
//     Box::new(move |cx| f.call(cx).boxed())
// }

// pub fn into_middleware<Context>(f: impl Handler<Context>) -> impl Middleware<Context>
// where
//     Context: Send + 'static,
// {
//     let f = into_box_dyn_handler(f);
//     Box::new(move |cx| (f)(cx))
// }
