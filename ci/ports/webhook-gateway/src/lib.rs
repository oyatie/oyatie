//! Webhook-gateway ports (ADR-0562).
//!
//! The facade consumes this crate instead of `ci-webhook-gateway-kernel`. Types
//! and traits remain defined in the kernel; this face is the only legal path
//! from `ci/facade` into that core.

#![forbid(unsafe_code)]

pub use ci_webhook_gateway_kernel::*;
