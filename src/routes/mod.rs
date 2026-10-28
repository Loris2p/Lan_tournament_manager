pub(crate) mod auth;
pub(crate) mod public;
pub(crate) mod captain;
pub(crate) mod admin;
mod router;

pub use router::create_router;