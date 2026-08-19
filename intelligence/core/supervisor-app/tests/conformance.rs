//! Conformance tests — seed read-back + tier assertions (placeholder).
//!
//! Formerly `intelligence-supervisor-conformance` (dropped — `conformance` is not a
//! valid ADR-0056 v4.1 12-layer-enum suffix; idiomatically test work lives in the
//! consuming crate per team-lead Option A decision 2026-05-15).
//!
//! Wave 2e / 2f will implement:
//!
//! ## Seed round-trips
//!   - `provider_family_round_trip::toml_string_OpenAIOrCodex_decodes_to_enum_variant_OpenAiOrCodex`
//!     (v4 §A.5 — hand-rolled try_from, not serde)
//!   - `inbox_state_valid_transitions` (Unlocked → Locked → Committed / Dead-lettered)
//!   - `spend_record_round_trip` (value type survives clone + equality check)
//!
//! ## Fixture pairs (v6 C.36)
//!   - `fixture_pair_registry_complete` — meta-test: every fixture file in
//!     `tests/fixtures/` has a matching test that asserts the documented diagnostic
//!
//! ## Failing fixtures (v6 BLOCKER-7..12, v4.50-v4.57, v5.22-v5.28)
//! These are added as individual test functions in Wave 2:
//!   - `dead_letter_on_unlocked_returns_invalid_transition`
//!   - `peek_lock_ttl_expiry_then_commit_walks_race`
//!   - `silent_switch_caught_when_account_degrades_between_snapshot_and_spawn`
//!   - `cost_ceiling_at_boundary` (parametric)
//!   - `projected_p95_warm_window_underestimate_does_not_bypass_ceiling`
//!   - `watchdog_kill_returns_fds_to_baseline`
//!   - `hung_cli_emits_exactly_one_spend_record_after_kill`

/// Placeholder — Wave 2 fills this with real conformance tests.
#[test]
fn conformance_placeholder_passes() {
    // Intentionally empty: verifies test harness compiles and links.
}
