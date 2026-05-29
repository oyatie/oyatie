# Plan: cloud-iac-gitops-drift-reconcile-domain

**Lane:** infra  
**Crate:** `oya-cloud-iac-domain` (the ONLY crate this task may touch)  
**Branch:** `feat/task-cloud-iac-gitops-drift-reconcile-domain-2026-05-28`  
**Base:** `origin/dev`

---

## Objective

Extend the pure cloud-iac domain crate with a GitOps drift-detection reconciliation
function that compares a desired `GitOpsEvidence` against an observed `GitOpsEvidence`
for the same controller/tenant/cell/application identity tuple and returns a typed
`GitOpsDriftReport`.

No I/O is introduced. The crate contract (no filesystem, network, OpenTofu CLI,
Argo CD API, or Kubernetes client) is preserved.

---

## Subtasks

### [iac-drift-1] Add `GitOpsDriftVerdict` and `GitOpsDriftReport` types

**What:**  
- `GitOpsDriftVerdict` enum: `InSync | DriftedCommit | DriftedSyncStatus | DegradedHealth | IdentityMismatch`  
- `GitOpsDriftReport` struct:
  - `verdict: GitOpsDriftVerdict`
  - `controller: GitOpsController`
  - `tenant_id: String`
  - `cell_id: String`
  - `application_name: String`
  - `observed_commit_sha: String`
  - `observed_sync_status: GitOpsSyncStatus`
  - `observed_health_status: GitOpsHealthStatus`
- Derive the same trait set used across the crate: `Clone, Debug, Eq, PartialEq`
- Add `Ord, PartialOrd` to the verdict enum (consistent with other enums in the crate)
- No new dependencies

**Acceptance:**
- `cargo check -p oya-cloud-iac-domain --all-targets` passes
- New types compile under `#![forbid(unsafe_code)]` and workspace lints
- No new entries in `[dependencies]`

---

### [iac-drift-2] Implement `reconcile_gitops_drift`

**What:**  
`pub fn reconcile_gitops_drift(desired: &GitOpsEvidence, observed: &GitOpsEvidence) -> GitOpsDriftReport`

**Logic (deterministic rank order):**
1. Identity check: if any of `(controller, tenant_id, cell_id, application_name)` differ
   between `desired` and `observed` → `IdentityMismatch`
2. `desired.commit_sha != observed.commit_sha` → `DriftedCommit`
3. `observed.sync_status != GitOpsSyncStatus::Synced` → `DriftedSyncStatus`
4. `observed.health_status != GitOpsHealthStatus::Healthy` → `DegradedHealth`
5. All aligned → `InSync`

Report always carries the observed fields (not desired) so callers can diagnose
what the controller actually reported.

**Acceptance:**
- `cargo nextest run -p oya-cloud-iac-domain` green
- Tests cover:
  - `IdentityMismatch` when identity differs, even when commit_sha also differs (precedence)
  - `DriftedCommit` when identity matches but sha differs
  - `DriftedSyncStatus` when identity+sha match but sync_status != Synced
  - `DegradedHealth` when identity+sha+sync match but health_status != Healthy
  - `InSync` when all fields aligned

---

### [iac-drift-3] Expose from `lib.rs` + rustdoc + verify no regressions

**What:**  
- Re-export `GitOpsDriftVerdict`, `GitOpsDriftReport`, `reconcile_gitops_drift` from `lib.rs`
- Add crate-level rustdoc comment to `lib.rs` extending the existing module-doc to
  describe the desired-vs-observed contract
- Confirm no changes to existing `GitOpsEvidence` / `GitOpsEvidenceInput` fields,
  validators, or secret-like guard

**Acceptance:**
- `cargo check -p oya-cloud-iac-domain --all-targets` green
- `cargo nextest run -p oya-cloud-iac-domain` green
- `git diff` shows only additive changes within `crates/oya-cloud-iac-domain/`
- No root `Cargo.toml` edit, no new workspace member, no changes to other crates
