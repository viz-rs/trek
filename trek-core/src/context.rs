use futures::future::BoxFuture;
use http::Extensions;
use hyper::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Body, Method, Uri, Version,
};
use multipart_async::server::Multipart;
use std::{
    fmt,
    io::{Error, ErrorKind, Result},
    sync::Arc,
};

use crate::middleware::Middleware;
use crate::parameters::Parameters;
use crate::request::Request;
use crate::response::Response;

/// The `Context` of the current HTTP request.
pub struct Context<State> {
    state: Arc<State>,
    request: Request,
    params: Vec<(String, String)>,
    handlers: Vec<Arc<dyn Middleware<Self>>>,
}

impl<State: 'static> Context<State> {
    /// Create a new Context
    pub fn new(
        state: Arc<State>,
        request: Request,
        params: Vec<(String, String)>,
        handlers: Vec<Arc<dyn Middleware<Self>>>,
    ) -> Self {
        Self {
            state,
            request,
            params,
            handlers,
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

    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        self.request.headers_mut()
    }

    pub fn header(&self, key: &'static str) -> Option<&HeaderValue> {
        self.headers().get(key)
    }

    pub fn header_mut(&mut self, key: &'static str) -> Option<&mut HeaderValue> {
        self.headers_mut().get_mut(key)
    }

    /// Access the extensions to the context.
    pub fn extensions(&self) -> &Extensions {
        self.request.extensions()
    }

    /// Mutably access the extensions to the context.
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        self.request.extensions_mut()
    }

    pub fn take_body(&mut self) -> &mut Body {
        // pub fn take_body(&mut self) -> Body {
        // std::mem::replace(self.request.body_mut(), Body::empty())
        self.request.body_mut()
    }

    /// Todo: limit size
    /// https://github.com/actix/actix-web/blob/master/src/types/form.rs#L332
    /// https://github.com/stream-utils/raw-body
    /// https://github.com/rustasync/http-service/blob/master/src/lib.rs#L96
    /// https://github.com/seanmonstar/warp/blob/master/src/filters/body.rs
    pub async fn bytes(&mut self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        let body = self.take_body();
        while let Some(chunk) = body.next().await {
            bytes.extend(chunk.map_err(|_| ErrorKind::InvalidData)?);
        }
        Ok(bytes)
        // use futures::stream::TryStreamExt;
        // Ok(self.take_body().try_concat().await.unwrap().to_vec())
    }

    pub async fn json<T: serde::de::DeserializeOwned>(&mut self) -> Result<T> {
        let body = self.bytes().await?;
        Ok(serde_json::from_slice(&body).map_err(|_| ErrorKind::InvalidData)?)
    }

    pub async fn string(&mut self) -> Result<String> {
        let body = self.bytes().await?;
        Ok(String::from_utf8(body).map_err(|_| ErrorKind::InvalidData)?)
    }

    pub fn path(&self) -> &str {
        self.uri().path()
    }

    pub fn query_string(&self) -> &str {
        self.uri().query().unwrap_or("")
    }

    pub fn query<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        let query = self.query_string();
        Ok(serde_qs::from_str(query).map_err(|_| ErrorKind::InvalidData)?)
    }

    /// `application/x-www-form-urlencoded`
    pub async fn form<T: serde::de::DeserializeOwned>(&mut self) -> Result<T> {
        let body = self.bytes().await?;
        Ok(serde_urlencoded::from_bytes(&body).map_err(|_| ErrorKind::InvalidData)?)
    }

    /// https://github.com/expressjs/multer
    /// https://crates.io/crates/multipart
    /// https://github.com/abonander/multipart-async
    /// `multipart/form-data`
    // pub async fn multipart(&mut self) -> Result<Multipart<Vec<u8>>> {
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

        Ok(Multipart::with_body(self.take_body(), boundary))
    }

    pub fn params<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        Ok(Parameters::new(
            self.params
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect(),
        )
        .params()
        .map_err(|_| ErrorKind::InvalidData)?)
    }

    // generate url
    // pub fn url_for(&self) {}

    /// Next middleare
    pub fn next<'a>(mut self) -> BoxFuture<'a, Response> {
        if self.handlers.is_empty() {
            Box::pin(async { hyper::Response::new(Body::empty()) })
        } else {
            self.handlers.remove(0).call(self)
        }
    }
}

impl<State> fmt::Debug for Context<State> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Context").finish()
    }
}
