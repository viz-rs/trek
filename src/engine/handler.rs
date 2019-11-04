use futures::future::{BoxFuture, Future};

use crate::{IntoResponse, Middleware, Response};

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

pub trait Cloneable<Context> {
    /// Clones `self`.
    fn clone_handler(&self) -> BoxDynHandler<Context>;
}

impl<Context, F: Handler<Context, Fut = BoxFuture<'static, Response>>, Fut> Cloneable<Context> for F
where
    F: Clone + Send + Sync + 'static + Fn(Context) -> Fut,
    Fut: Future + Send + 'static,
    Fut::Output: IntoResponse + Send + 'static,
{
    #[inline(always)]
    fn clone_handler(&self) -> BoxDynHandler<Context> {
        Box::new(self.clone())
    }
}

impl<Context> Clone for BoxDynHandler<Context> {
    #[inline(always)]
    fn clone(&self) -> BoxDynHandler<Context> {
        self.clone_handler()
    }
}

pub type DynHandler<Context> = dyn Handler<Context, Fut = BoxFuture<'static, Response>>;

pub type BoxDynHandler<Context> = Box<DynHandler<Context>>;

pub fn into_box_dyn_handler<Context>(f: impl Handler<Context> + Clone) -> BoxDynHandler<Context>
where
    Context: Send + 'static,
{
    Box::new(move |cx| f.call(cx))
}

#[allow(dead_code)]
pub fn into_middleware<Context>(f: impl Handler<Context> + Clone) -> impl Middleware<Context>
where
    Context: Send + 'static,
{
    let f = into_box_dyn_handler(f);
    Box::new(move |cx| f.call(cx))
}

pub fn box_dyn_handler_into_middleware<Context>(
    f: BoxDynHandler<Context>,
) -> impl Middleware<Context>
where
    Context: Send + 'static,
{
    Box::new(move |cx| f.call(cx))
}
