//! # oya-cloud-ci-accounting-registry-app
//!
//! Generates `accounting-registry.generated.json` — one record per `git ls-files`
//! path (the tracked-truth discipline; PHASE-0-FIREWALL-PLAN §5.1) — plus the three
//! companion generated faces (`ttl-policy.generated.json`, `decision-crosswalk.generated.json`,
//! `enforcement-inventory.generated.json`). The producer is the buck2 `rust_binary`
//! that GATE-2 `cloud-ci-total-accounting` owns; it is NOT an `oya` CLI command
//! (register #20 — `oya gen`/`oya gate` authority is retired).
