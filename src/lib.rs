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
pub use self::engine::context::Context;
pub use self::engine::handler::into_box_dyn_handler;
pub use self::engine::handler::Handler;
pub use self::engine::middleware::Middleware;
pub use self::engine::response::Response;
pub use self::router::{
    resource::{Resource, Resources},
    router::Router,
};
pub use self::trek::Trek;
