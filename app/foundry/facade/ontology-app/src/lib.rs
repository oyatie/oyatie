#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod auth;
pub mod authz;
mod boot;
pub mod composition;
pub mod config;
pub mod dto;
mod listing;
mod listing_query;
pub mod metrics;
mod migrate;
mod object_set;
mod object_set_body;
pub mod observability;
pub mod observation;
pub mod pdp;
pub mod read_dto;
pub mod reads;
pub mod routes;
mod search_around;
mod search_around_body;
mod search_around_refusals;
pub mod seed;
pub mod slo;
mod status;
mod submission;
mod submission_limits;
pub mod submit;

pub use authz::PolicyEnforcementPoint;
pub use composition::{AppState, BootError, TenantState, compose};
pub use config::{Config, ConfigError};
pub use foundry_caller_draft::{Caller, CallerVerifier, OperatorCredential};
pub use pdp::{PepError, Surface};
pub use routes::{router, router_from};
pub use seed::SeedError;
