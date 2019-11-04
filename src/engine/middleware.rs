use crate::Response;
use futures::future::BoxFuture;

pub trait Middleware<Context>: Send + Sync + 'static {
    fn call<'a>(&'a self, cx: Context) -> BoxFuture<'a, Response>;
}

impl<Context, F> Middleware<Context> for F
where
    F: Send + Sync + 'static + Fn(Context) -> BoxFuture<'static, Response>,
{
    fn call<'a>(&'a self, cx: Context) -> BoxFuture<'a, Response> {
        (self)(cx)
    }
}
