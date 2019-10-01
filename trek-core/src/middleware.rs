//! Middleware traits
//!
//! Thanks to repos:
//!     * https://play.rust-lang.org/?version=nightly&mode=debug&edition=2018&gist=07fa435c700c1dfab4c112cc07d1543d
//!     * https://play.rust-lang.org/?version=nightly&mode=debug&edition=2018&gist=2f9af4a2114fa66ec3268bc64163026c
//!     * https://github.com/rustasync/tide/blob/master/tide-core/src/middleware.rs
//!     * https://github.com/gotham-rs/gotham/blob/master/gotham/src/middleware/mod.rs
//!     * https://github.com/trezm/Thruster/blob/master/thruster-core-async-await/src/middleware.rs
//!     * https://github.com/iron/iron/blob/master/iron/src/middleware/mod.rs#L135
//!     * https://github.com/koajs/compose/blob/master/index.js

use crate::response::Response;
use futures::future::BoxFuture;

pub trait Middleware<Context>: Send + Sync + 'static {
    fn call(&self, cx: Context) -> BoxFuture<'static, Response>;
}

impl<Context, F> Middleware<Context> for F
where
    F: Send + Sync + 'static + Fn(Context) -> BoxFuture<'static, Response>,
{
    fn call(&self, cx: Context) -> BoxFuture<'static, Response> {
        (self)(cx)
    }
}
