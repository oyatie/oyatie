//! Native SCIM resource, server and store contracts with atomic protected writes.
#![forbid(unsafe_code)]

use core::future::Future;
use core::pin::Pin;
use iam_identity_domain::{UserId, provisioned_role_is_valid};
use serde::{Deserialize, Serialize};
use std::fmt;

mod filter_eval;
mod filters;
mod group_patch;
mod memory_groups;
mod memory_users;
mod reference;
mod resources;
mod server;
mod server_port;
mod store_port;
mod timestamps;
mod user_patch;
use filter_eval::*;
pub use filters::*;
use group_patch::*;
pub use memory_groups::*;
pub use memory_users::*;
pub use reference::*;
pub use resources::*;
pub use server_port::*;
pub use store_port::*;
use timestamps::*;
use user_patch::*;
