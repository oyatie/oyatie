---
id: ADR-0360
title: CI/CD pipeline optimization program — affected-target precision, gate-only overlay, warm shared cache, test sharding, pinned+signed agent image, speculative merge queue, content-addressed gate caching
status: Rejected
planning_impact: true
date: 2026-05-25
owner_team:
  - council-architecture
  - ops-platform
  - axis-dev-cli
  - ops-sre-reliability
owners:
  - council-architecture
  - ops-platform
  - axis-dev-cli
amends:
  - ADR-0346-oya-verify-must-run-full-ci-mirror.md
supersedes: []
---
# ADR-0360: CI/CD pipeline optimization program — affected-target precision, gate-only overlay, warm shared cache, test sharding, pinned+signed agent image, speculative merge queue, content-addressed gate caching

## Status

Proposed — 2026-05-25. Amends ADR-0346 (adds an affected-scope presubmit mode while preserving `--ci-required` as the authoritative full mirror). Evidence-blocked: every throughput/cache-hit/latency claim stays `blocked_until_required_evidence_is_green` until measured on the CI farm.

## Context

Direct observation (2026-05-25): `oya verify --ci-required` runs `cargo {check,clippy,nextest} --workspace --all-targets` with **no affected-target selection**, so a change touching only docs/specs/evidence YAML still triggers a whole-workspace cargo + test mirror (observed: a 1342-file diff that was ~99% non-Rust ran the full mirror for 10+ minutes). Hyperscaler CI (Google TAP, Bazel) instead runs the **reverse-dependency closure** of the change, shards tests, reuses a warm remote cache populated by trunk, and gates merges with a speculative always-green queue. `specs/cloud-toolchain-target.json` already names these as targets; this ADR commits the program and its correctness rules, grounded in `docs/ideas/pipeline-optimization.md` and best-practice research (Google SWE book Ch.23, Bazel remote-cache/query, cargo-nextest, sccache, Zuul/GitHub merge queue, cosign/Kyverno).

## Decision

Adopt a seven-part CI/CD optimization program. Each part has a hard correctness rule so optimization never weakens the governance gates.

- **O1 — Affected-target precision.** Add an additive `oya verify --affected [--base <ref>]` presubmit mode. Classify the changed-file set vs the base into: **Full** (any of `Cargo.lock`, root/`[workspace]` `Cargo.toml`, a `workspace-hack` manifest, `rust-toolchain*`, `.cargo/config*`, CI config, the `oya-dev-cli` gate engine, or any `build.rs`), **NoRust** (docs/`.md`/YAML/JSON/evidence/specs ⇒ skip the cargo mirror, run gates only), or **Crates** (map changed files → owning package by nearest manifest, take the transitive reverse-dependency closure via `cargo metadata` `resolve.nodes` including dev+build edges, run `-p` per crate). Correctness: `--ci-required` is UNCHANGED and remains the authoritative whole-workspace mirror that runs on trunk/merge as the backstop (presubmit affected-selection is ~95% safe by Google's own measure; the full trunk run closes the gap). Governance gates (`gate run-all`) ALWAYS run regardless of scope.
- **O2 — oya as governance-only overlay.** `oya verify` consumes the lane's already-produced test results (nextest JUnit) rather than re-running cargo, eliminating the double build. Until the results-ingest contract lands, `--ci-required` keeps running the mirror.
- **O3 — Warm shared cache + cached downloads.** Trunk/postsubmit builds get read-write to the blessed sccache prefix; PR builds are read-through (read blessed, write a PR-scoped prefix, promote on merge) — the write principal must equal the trust boundary. `SCCACHE_S3_KEY_PREFIX` encodes the toolchain identity; `CARGO_INCREMENTAL=0`; basedir normalization for path-independent keys. Crate downloads are served by a sparse-registry mirror (Panamax) + a warm read-only `CARGO_HOME` (sccache caches compilation, not downloads).
- **O4 — Distributed test sharding.** `cargo nextest run --partition slice:m/n` across agents (NOT deprecated `count:`); composes with O1 (shard only the affected set).
- **O5 — Pinned, signed agent image.** Prebuilt image: digest-pinned base, non-root numeric UID, baked toolchain + sccache + nextest + git + warm registry; cosign-signed by digest; enforced by the existing `infra/kyverno/policies/require-signed-images.yaml`; S3 creds via external-secrets/OpenBao with the O3 RO/RW split.
- **O6 — Speculative merge queue.** Implement ADR-0111 projected state as a speculative queue: test A, A+B, A+B+C… in parallel against projected trunk; land the green prefix; on failure eject the culprit and re-project dependents (bisect a failing batch); an adaptive window grows on success and halves on failure. Always-green-trunk (Not-Rocket-Science Rule) is the invariant.
- **O7 — Content-addressed gate caching.** `verdict_key = H(merkle(declared_inputs) ‖ gate_version ‖ config_digest ‖ env_subset)`; cache hit ⇒ reuse verdict, skip the gate. Correctness rule (load-bearing): a gate is cacheable ONLY if it declares all its inputs and is deterministic; per-file gates key on their file set; cross-file/global gates key on the whole corpus digest; **a gate that cannot enumerate its inputs is un-cacheable and always runs** — never risk a false PASS.

## Consequences

- Presubmit latency/cost drop toward O(change) instead of O(workspace); the full mirror still backstops trunk, so safety is preserved by construction.
- New correctness surfaces: affected-selection under-test (mitigated by full-triggers + dev/build edges + trunk backstop + `cargo-hakari` feature unification), and gate-cache false-PASS (mitigated by the input-declaration rule + un-cacheable fallback). These warrant their own fitness gates.
- `--ci-required` semantics are preserved (ADR-0346 amended, not superseded): it stays the authoritative full mirror.
- All performance claims remain blocked until measured on the farm; this ADR commits the design + correctness rules, not measured numbers.
