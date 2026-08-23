# Spec: GitOps Drift-Detection Reconciliation — cloud-iac domain

**Vertical:** infra  
**Task slug:** `cloud-iac-gitops-drift-reconcile-domain`  
**Crate:** `cloud-iac-domain` (`crates/cloud-iac-domain/`)  
**ADR authority:** ADR-0131 (per-microservice flat layout), ADR-0509 (single-crate-per-service)  
**Stage:** SPEC → BUILD → VERIFY

---

## Objective

Add a pure-domain, zero-I/O drift reconciliation function to `cloud-iac-domain`.
The function compares a _desired_ `GitOpsEvidence` (what the declarative pipeline
intends) against an _observed_ `GitOpsEvidence` (what the GitOps controller last
reported) and returns a typed `GitOpsDriftReport` describing the verdict and the
identity context needed to act on it.

This stays entirely inside the domain crate contract: no filesystem, network,
OpenTofu CLI, Argo CD API, or Kubernetes client I/O. Pure functions only.

---

## Vertical context

The cloud-iac domain models the intent and evidence layer for OpenTofu-based
infrastructure managed through a GitOps controller (Argo CD). `GitOpsEvidence`
already records the controller, tenant, cell, application, commit SHA, sync
status, and health status for a single reconciliation event. The gap is a
deterministic comparator that answers: _given what we asked for and what the
controller reported, what is the current drift state?_

---

## New types

### `GitOpsDriftVerdict` (enum)

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GitOpsDriftVerdict {
    InSync,
    DriftedCommit,
    DriftedSyncStatus,
    DegradedHealth,
    IdentityMismatch,
}
```

Derives match the existing enum pattern in the crate (`Clone, Copy, Debug, Eq,
PartialEq, Ord, PartialOrd`).

### `GitOpsDriftReport` (struct)

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitOpsDriftReport {
    pub verdict: GitOpsDriftVerdict,
    pub controller: GitOpsController,
    pub tenant_id: String,
    pub cell_id: String,
    pub application_name: String,
    pub observed_commit_sha: String,
    pub observed_sync_status: GitOpsSyncStatus,
    pub observed_health_status: GitOpsHealthStatus,
}
```

Fields carry the identity tuple and observed (not desired) values so callers
can emit structured telemetry without re-reading either evidence object.
Derives match the struct pattern used by `GitOpsEvidence` in the crate
(`Clone, Debug, Eq, PartialEq`).

---

## New function

```rust
/// Compare a desired GitOps state against an observed GitOps state and
/// return a typed drift verdict.
///
/// # Identity contract
/// `desired` and `observed` must describe the same
/// `(controller, tenant_id, cell_id, application_name)` tuple.
/// If they differ, the report verdict is `IdentityMismatch` regardless of
/// any other field values.
///
/// # Drift rank order (applied only when identities match)
/// 1. `DriftedCommit`      — observed `commit_sha` != desired `commit_sha`
/// 2. `DriftedSyncStatus`  — observed `sync_status` != `GitOpsSyncStatus::Synced`
/// 3. `DegradedHealth`     — observed `health_status` != `GitOpsHealthStatus::Healthy`
/// 4. `InSync`             — all fields aligned
///
/// This function performs no I/O.
pub fn reconcile_gitops_drift(
    desired: &GitOpsEvidence,
    observed: &GitOpsEvidence,
) -> GitOpsDriftReport
```

The function is a free function (not a method), consistent with the validator
helper pattern already used in the crate.

---

## Mod layout (flat clean-arch)

This crate is single-file (`src/lib.rs`) per the flat clean-arch doctrine.
The new types and function are added directly to `src/lib.rs` in the same
file — no new mods, no new files, no new crates.

---

## Contracts

### No external API surface

This crate is a pure domain library with no HTTP, gRPC, or proto surface.
The contract is the Rust public API described above. Consumers (adapters,
use-case mods in the owning microservice) import the types directly.

### Proto3 note

If a gRPC adapter later serialises `GitOpsDriftReport` over the wire, it will
map `GitOpsDriftVerdict` variants to a proto3 enum. That mapping lives in the
adapter crate, not here.

### OpenAPI 3.2.0 note

If a REST adapter exposes drift results, it will define the JSON schema for
`GitOpsDriftReport` in its own OpenAPI document. No schema is defined in this
domain crate.

---

## Crate contract boundaries

| Allowed                          | Forbidden                                               |
|----------------------------------|---------------------------------------------------------|
| Pure Rust types and functions    | std::fs, std::net, tokio, reqwest, kube, argocd-client |
| BTreeMap / std collections       | OpenTofu CLI subprocess                                 |
| Existing crate types as inputs   | Any new `[dependencies]` entries                        |
| `#![forbid(unsafe_code)]`        | unsafe blocks                                           |

---

## Testing strategy

Tests live in `tests/cloud_iac_foundation.rs` (the existing integration test
file), following the `cloud_iac_*` naming convention already established.

Required coverage (each must be a separate `#[test]`):

| Test name                                              | Verdict expected       |
|--------------------------------------------------------|------------------------|
| `drift_identity_mismatch_beats_commit_drift`           | `IdentityMismatch`     |
| `drift_commit_sha_mismatch`                            | `DriftedCommit`        |
| `drift_sync_status_out_of_sync`                        | `DriftedSyncStatus`    |
| `drift_sync_status_unknown`                            | `DriftedSyncStatus`    |
| `drift_degraded_health`                                | `DegradedHealth`       |
| `drift_all_aligned_is_in_sync`                         | `InSync`               |

Each test builds `GitOpsEvidence` via the existing `GitOpsEvidence::new` +
`GitOpsEvidenceInput` path to exercise the full construction chain, not raw
struct literals.

---

## Acceptance criteria summary

| Subtask     | Gate                                                                 |
|-------------|----------------------------------------------------------------------|
| iac-drift-1 | `cargo check -p cloud-iac-domain --all-targets` green            |
| iac-drift-2 | `cargo nextest run -p cloud-iac-domain` green; 6 new tests pass  |
| iac-drift-3 | Both commands green; git diff additive-only within crate directory   |

---

## Non-goals

- No changes to `GitOpsEvidence`, `GitOpsEvidenceInput`, their constructors,
  field validators, or the `looks_secret_like` guard.
- No new workspace members.
- No root `Cargo.toml` edits.
- No changes to any other crate.
