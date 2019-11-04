//! Trek - Fast, effective, minimalist web framework for Rust.

#![deny(unsafe_code)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations
)]

#[macro_use]
extern crate log;

mod engine;
mod router;
mod trek;

#[doc(inline)]
pub use crate::engine::{
    context::Context,
    handler::{box_dyn_handler_into_middleware, into_box_dyn_handler, BoxDynHandler, Handler},
    middleware::Middleware,
    parameters::Parameters,
    request::Request,
    response::{html, json, IntoResponse, Response},
};

#[doc(inline)]
pub use crate::router::{
    resource::{Resource, Resources},
    router::Router,
};

#[doc(inline)]
pub use crate::trek::Trek;
