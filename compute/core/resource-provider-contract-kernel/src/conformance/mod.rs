//! The generic conformance checks every resource provider must pass.
//!
//! Each check is a pure generic fn over a [`ConformanceFixture`]; it builds a
//! FRESH provider, drives it through the contract scenario, and returns the
//! first divergence as a typed [`ConformanceViolation`] (never panicking —
//! the assertion style belongs to the caller's test harness, the diagnosis
//! belongs here).

mod delete;
mod fixture;
mod ledger;
mod ledger_match;
mod listing;
mod write;

pub use delete::check_async_delete_operation;
pub use fixture::{ConformanceFixture, ConformanceViolation};
pub use ledger::check_operation_ledger_semantics;
pub use listing::check_stable_pagination;
pub use write::{check_create_idempotency, check_idempotent_put, check_read_after_write};

/// Poll budget for AIP-151 operations driven by the harness.
pub const MAX_OPERATION_POLLS: u32 = 32;
/// Page budget for pagination walks driven by the harness.
pub const MAX_PAGE_WALK: u32 = 100;

fn violation(check: &'static str, detail: impl Into<String>) -> ConformanceViolation {
    ConformanceViolation {
        check,
        detail: detail.into(),
    }
}

/// Run the full contract; an empty vector means the provider conforms.
pub async fn run_all_checks<F: ConformanceFixture>(fixture: &F) -> Vec<ConformanceViolation> {
    [
        check_idempotent_put(fixture).await,
        check_create_idempotency(fixture).await,
        check_read_after_write(fixture).await,
        check_stable_pagination(fixture).await,
        check_async_delete_operation(fixture).await,
        check_operation_ledger_semantics(fixture).await,
    ]
    .into_iter()
    .filter_map(Result::err)
    .collect()
}
