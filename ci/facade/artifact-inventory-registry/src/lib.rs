//! # oya-cloud-ci-accounting-registry-app
//!
//! Generates `accounting-registry.generated.json` — one record per `git ls-files`
//! path (the tracked-truth discipline; PHASE-0-FIREWALL-PLAN §5.1) — plus the three
//! companion generated faces (`ttl-policy.generated.json`, `decision-crosswalk.generated.json`,
//! `enforcement-inventory.generated.json`). The producer is the buck2 `rust_binary`
//! that GATE-2 `cloud-ci-total-accounting` owns; it is NOT an `oya` CLI command
//! (register #20 — `oya gen`/`oya gate` authority is retired).
//!
//! ## Invariants (10-gates-registry §A.2)
//! 1. `committed == regenerated` — the output is fully deterministic (no wall-clock in
//!    the row data; `_provenance` carries a content digest, not a timestamp), so the
//!    `registry-drift` test can byte-diff a fresh run against the committed face.
//! 2. Total coverage — `set(rows.path) == set(git ls-files) − ephemeral` (ephemeral
//!    carve-out rows are excluded by CLASS, resolved from the DATA table, never by row).
//! 3. Carve-outs (vendor/generated/ephemeral/...) live as DATA in the bundled
//!    oya-ci-config unit-class + ttl tables (`Policy::from_config`), never as scanner
//!    branches (Linus: the exception lives in the table). The classifier walks the
//!    table; it has no hard-coded special cases.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]
