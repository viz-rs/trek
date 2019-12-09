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

mod router;
mod trek;

pub mod middleware;

#[doc(inline)]
pub use trek_core::{
    box_dyn_handler_into_middleware, helpers, html, into_box_dyn_handler, json, Body,
    BoxDynHandler, Context, Error, ErrorResponse, Handler, IntoResponse, Middleware, Parameters,
    Request, Response, Result, StatusCode,
};

#[doc(inline)]
pub use trek_router::{Resource, Resources, Router};

#[doc(inline)]
pub use crate::trek::Trek;
