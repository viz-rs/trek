#![deny(unsafe_code)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations
)]

use cookie::ParseError;
pub use cookie::{Cookie, CookieJar};
use futures::future::BoxFuture;
use http::status::StatusCode;
use hyper::{
    header::{HeaderMap, HeaderValue, COOKIE, SET_COOKIE},
    Body,
};
use std::sync::{Arc, RwLock};
use trek_core::{Context, Middleware, Response};

#[derive(Debug)]
pub struct Cookies(Arc<RwLock<CookieJar>>);

impl Cookies {
    pub fn from_headers(headers: &HeaderMap) -> Self {
        Self(Arc::new(RwLock::new(
            parse_from_header(
                headers
                    .get_all(COOKIE)
                    .iter()
                    .map(|raw| raw.to_str().unwrap())
                    .collect(),
            )
            .unwrap_or_default(),
        )))
    }

    pub fn jar(&self) -> Arc<RwLock<CookieJar>> {
        self.0.clone()
    }
}

/// An extension to `Context` that provides cached access to cookies
pub trait CookiesContextExt {
    /// returns all `Cookies`
    fn cookies(&self) -> Option<&Cookies>;

    /// returns a `Cookie` by name of the cookie
    fn get_cookie(&self, name: &str) -> Option<Cookie<'static>>;

    /// Add cookie to the cookie jar
    fn set_cookie(&mut self, cookie: Cookie<'static>) -> Option<()>;

    /// Removes the cookie. This instructs the `CookiesMiddleware` to send a cookie with empty value
    /// in the response.
    fn remove_cookie(&mut self, cookie: Cookie<'static>) -> Option<()>;
}

impl<State: Send + Sync + 'static> CookiesContextExt for Context<State> {
    fn cookies(&self) -> Option<&Cookies> {
        self.extensions().get::<Cookies>()
    }

    fn get_cookie(&self, name: &str) -> Option<Cookie<'static>> {
        self.cookies()?.0.read().unwrap().get(name).cloned()
    }

    fn set_cookie(&mut self, cookie: Cookie<'static>) -> Option<()> {
        self.cookies()?.0.write().unwrap().add(cookie);
        Some(())
    }

    fn remove_cookie(&mut self, cookie: Cookie<'static>) -> Option<()> {
        self.cookies()?.0.write().unwrap().remove(cookie);
        Some(())
    }
}

#[derive(Debug, Default)]
pub struct CookiesMiddleware {}

impl CookiesMiddleware {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<State: Send + Sync + 'static> Middleware<Context<State>> for CookiesMiddleware {
    fn call<'a>(&'a self, mut cx: Context<State>) -> BoxFuture<'a, Response> {
        Box::pin(async move {
            let cookies = cx
                .extensions_mut()
                .remove()
                .unwrap_or_else(|| Cookies::from_headers(cx.headers()));

            let cookie_jar = cookies.jar();

            cx.set(cookies);

            let mut res = cx.next().await;

            let headers = res.headers_mut();

            for cookie in cookie_jar.read().unwrap().delta() {
                let hv = HeaderValue::from_str(cookie.encoded().to_string().as_str());
                if let Ok(val) = hv {
                    headers.append(SET_COOKIE, val);
                } else {
                    let mut res = Response::new(Body::empty());
                    *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return res;
                }
            }

            res
        })
    }
}

#[inline]
fn parse_from_header(cs: Vec<&str>) -> Result<CookieJar, ParseError> {
    let mut jar = CookieJar::new();

    for s in cs {
        s.split(';').try_for_each(|s| -> Result<(), ParseError> {
            jar.add_original(Cookie::parse(s.trim().to_owned())?);
            Ok(())
        })?;
    }

    Ok(jar)
}
