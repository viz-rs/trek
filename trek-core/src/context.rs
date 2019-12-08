use bytes::BytesMut;
use futures::future::BoxFuture;
use futures::stream::StreamExt;
use http::Extensions;
use hyper::{
    header::{HeaderMap, HeaderValue},
    Body, Method, Uri, Version,
};
use std::{
    fmt,
    io::{ErrorKind, Result},
    sync::Arc,
};

#[cfg(feature = "multipart")]
use hyper::header::CONTENT_TYPE;
#[cfg(feature = "multipart")]
use std::io::Error;

use crate::{Middleware, Parameters, Request, Response};

/// The `Context` of the current HTTP request.
pub struct Context<State> {
    state: Arc<State>,
    request: Request,
    pub params: Vec<(String, String)>,
    pub middleware: Vec<Arc<dyn Middleware<Self>>>,
}

impl<State: Send + Sync + 'static> Context<State> {
    /// Create a new Context
    pub fn new(
        state: Arc<State>,
        request: Request,
        params: Vec<(String, String)>,
        middleware: Vec<Arc<dyn Middleware<Self>>>,
    ) -> Self {
        Self {
            state,
            request,
            params,
            middleware,
        }
    }

    ///  Access the state.
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Access the request's HTTP method.
    pub fn method(&self) -> &Method {
        self.request.method()
    }

    pub fn method_mut(&mut self) -> &mut Method {
        self.request.method_mut()
    }

    /// Access the request's full URI method.
    pub fn uri(&self) -> &Uri {
        self.request.uri()
    }

    /// Access the request's HTTP version.
    pub fn version(&self) -> Version {
        self.request.version()
    }

    /// Access the entrie request.
    pub fn request(&self) -> &Request {
        &self.request
    }

    /// Access a mutable handle to the entire request.
    pub fn request_mut(&mut self) -> &mut Request {
        &mut self.request
    }

    /// Access the request's headers.
    pub fn headers(&self) -> &HeaderMap {
        self.request.headers()
    }

    /// Access a mutable request's headers.
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        self.request.headers_mut()
    }

    /// Access a request's header.
    pub fn header(&self, key: &'static str) -> Option<&HeaderValue> {
        self.headers().get(key)
    }

    /// Access a mutable request's header.
    pub fn header_mut(&mut self, key: &'static str) -> Option<&mut HeaderValue> {
        self.headers_mut().get_mut(key)
    }

    /// Access the request's extensions.
    pub fn extensions(&self) -> &Extensions {
        self.request.extensions()
    }

    /// Access a mutable request's extensions.
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        self.request.extensions_mut()
    }

    /// Access the request's path.
    pub fn path(&self) -> &str {
        self.uri().path()
    }

    /// Access the request's path.
    pub fn query_string(&self) -> &str {
        self.uri().query().unwrap_or("")
    }

    /// From `?query=string`.
    /// TODO: check 'string-length'
    pub fn query<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        let query = self.query_string();
        Ok(serde_qs::from_str(query).map_err(|_| ErrorKind::InvalidData)?)
    }

    // Access a mutable request's body.
    pub fn take_body(&mut self) -> &mut Body {
        self.request.body_mut()
    }

    /// Todo: limit size
    /// https://github.com/actix/actix-web/blob/master/src/types/form.rs#L332
    /// https://github.com/stream-utils/raw-body
    /// https://github.com/rustasync/http-service/blob/master/src/lib.rs#L96
    /// https://github.com/seanmonstar/warp/blob/master/src/filters/body.rs
    pub async fn bytes(&mut self) -> Result<Vec<u8>> {
        let mut bytes = BytesMut::with_capacity(8 * 1024);
        let body = self.take_body();
        while let Some(chunk) = body.next().await {
            bytes.extend_from_slice(&chunk.map_err(|_| ErrorKind::InvalidData)?);
        }
        Ok(bytes.to_vec())
    }

    /// From `application/json`
    /// TODO: check `content-type` and 'content-length'
    pub async fn json<T: serde::de::DeserializeOwned>(&mut self) -> Result<T> {
        let body = self.bytes().await?;
        Ok(serde_json::from_slice(&body).map_err(|_| ErrorKind::InvalidData)?)
    }

    pub async fn string(&mut self) -> Result<String> {
        let body = self.bytes().await?;
        Ok(String::from_utf8(body).map_err(|_| ErrorKind::InvalidData)?)
    }

    /// From `application/x-www-form-urlencoded`
    /// TODO: check `content-type` and 'content-length'
    pub async fn form<T: serde::de::DeserializeOwned>(&mut self) -> Result<T> {
        let body = self.bytes().await?;
        Ok(serde_urlencoded::from_bytes(&body).map_err(|_| ErrorKind::InvalidData)?)
    }

    /// https://github.com/expressjs/multer
    /// https://crates.io/crates/multipart
    /// https://github.com/abonander/multipart-async
    /// From `multipart/form-data`
    /// TODO: check `content-type` and 'content-length'
    #[cfg(feature = "multipart")]
    pub fn multipart(&mut self) -> Result<Multipart<&mut Body>> {
        const BOUNDARY: &str = "boundary=";

        let boundary = self
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|ct| {
                let ct = ct.to_str().ok()?;
                let idx = ct.find(BOUNDARY)?;
                Some(ct[idx + BOUNDARY.len()..].to_string())
            })
            .ok_or_else(|| Error::new(ErrorKind::Other, "no boundary found"))?;

        Ok(multipart_async::server::Multipart::with_body(
            self.take_body(),
            boundary,
        ))
    }

    pub fn params<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        Ok(Parameters::from_vec_string(&self.params)
            .parse()
            .map_err(|_| ErrorKind::InvalidData)?)
    }

    // generate url
    // pub fn url_for(&self) {}

    /// Next middleare
    pub fn next<'a>(mut self) -> BoxFuture<'a, Response> {
        if self.middleware.is_empty() {
            Box::pin(async { hyper::Response::new(Body::empty()) })
        } else {
            Box::pin(async move { self.middleware.remove(0).call(self).await })
        }
    }
}

impl<State> fmt::Debug for Context<State> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Context").finish()
    }
}
