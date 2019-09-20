use http::Extensions;
use hyper::{
    header::{HeaderMap, HeaderValue},
    Body, Method, Uri, Version,
};
use std::{
    io::{ErrorKind, Result},
    sync::Arc,
};

use crate::request::Request;

/// The `Context` of the current HTTP request.
#[derive(Debug)]
pub struct Context<State> {
    state: Arc<State>,
    request: Request,
}

impl<State> Context<State> {
    /// Create a new Context
    pub fn new(state: Arc<State>, request: Request) -> Self {
        Self { state, request }
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
    pub async fn bytes(&mut self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        let body = self.take_body();
        while let Some(chunk) = body.next().await {
            bytes.extend(chunk.map_err(|_| ErrorKind::InvalidData)?);
        }
        Ok(bytes)
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

    pub fn param(&self) {}

    /// `application/x-www-form-urlencoded`
    pub async fn form<T: serde::de::DeserializeOwned>(&mut self) -> Result<T> {
        let body = self.bytes().await?;
        Ok(serde_urlencoded::from_bytes(&body).map_err(|_| ErrorKind::InvalidData)?)
    }

    /// https://github.com/expressjs/multer
    /// https://crates.io/crates/multipart
    /// https://github.com/abonander/multipart-async
    /// `multipart/form-data`
    pub fn multipart(&self) {}

    pub fn cookies(&self) {}

    // generate url
    pub fn url_for(&self) {}
}
