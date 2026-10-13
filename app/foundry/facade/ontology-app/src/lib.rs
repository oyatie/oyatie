#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod auth;
pub mod authz;
mod boot;
pub mod composition;
pub mod config;
pub mod dto;
pub mod human_session;
mod link_types;
mod listing;
mod listing_query;
pub mod metrics;
mod migrate;
pub mod model_control;
mod object_set;
mod object_set_body;
pub mod observability;
pub mod observation;
pub mod pdp;
pub mod read_dto;
pub mod reads;
pub mod routes;
pub mod runbook;
mod schemas;
mod search_around;
mod search_around_body;
mod search_around_refusals;
pub mod seed;
pub mod slo;
mod status;
pub mod submit;
mod type_labels;

pub use authz::PolicyEnforcementPoint;
pub use composition::{AppState, BootError, TenantState, compose};
pub use config::{Config, ConfigError};
pub use foundry_caller_draft::{Caller, CallerVerifier, OperatorCredential};
pub use pdp::{PepError, Surface};
pub use routes::{router, router_from, router_with_model_control, router_with_user_sessions};
pub use seed::SeedError;
