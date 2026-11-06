#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::module_name_repetitions)]
// Fires across the `Hooks` and `Mailer` trait methods, which return `Result` —
// already `#[must_use]` — and are exported API a `#[must_use]` cannot be added
// to without churn. Suppressed on Linux CI but not on macOS, so it also made
// the lint gate platform-dependent.
#![allow(clippy::must_use_candidate)]
#![doc = include_str!("../README.md")]

pub use self::errors::Error;

/// Experimental statically linked module runtime from MoonWitness ORM.
///
/// Enable the `mw-modules` Cargo feature to use this API. The runtime currently
/// coordinates module validation and synchronous lifecycle hooks; it does not
/// execute database migrations or provide tenant isolation.
#[cfg(feature = "mw-modules")]
pub mod modules {
    pub use moonwitness_module::*;
}

mod banner;
pub mod bgworker;
mod byte_size;
mod depcheck;
pub mod initializers;
pub mod prelude;

pub mod data;
pub mod doctor;

#[cfg(feature = "with-db")]
pub mod db;
#[cfg(feature = "with-db")]
pub mod model;
#[cfg(feature = "with-db")]
pub mod schema;
#[cfg(all(feature = "db-sqlite", any(feature = "with-db", feature = "worker")))]
pub(crate) mod sqlite_pragmas;
mod tera;

pub mod app;
pub mod auth;
pub mod boot;
pub mod cache;
#[cfg(feature = "cli")]
pub mod cli;
pub mod config;
pub mod controller;
mod env_vars;
pub mod environment;
pub mod errors;
pub mod hash;
pub mod logger;
#[cfg(feature = "mailer")]
pub mod mailer;
#[cfg(feature = "notifications")]
pub mod notifications;
pub mod scheduler;
pub mod task;
#[cfg(feature = "with-db")]
pub mod tenant;
#[cfg(feature = "testing-core")]
pub mod testing;
#[cfg(feature = "testing-core")]
pub use axum_test::TestServer;
pub mod storage;
#[cfg(feature = "testing-core")]
pub mod tests_cfg;
pub mod validation;
pub use validator;
#[cfg(feature = "with-db")]
pub mod cargo_config;

/// Application results options list
pub type Result<T, E = Error> = std::result::Result<T, E>;
