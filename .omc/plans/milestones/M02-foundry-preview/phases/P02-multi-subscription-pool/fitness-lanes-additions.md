---
purpose: "Delta-spec for new fitness lanes introduced by P02. This file is the *proposal*; the authoritative lane definitions land under `.omc/fitness-lanes/<lane>.md` once approved (M-CC-P01 owns lane-file shape)."
---

---
doc_class: FitnessLaneDelta
parent: ./INDEX.md
milestone: M02
phase: P02-multi-subscription-pool
status: pending approval
purpose: |
  Delta-spec for new fitness lanes introduced by P02. This file is the *proposal*; the
  authoritative lane definitions land under `.omc/fitness-lanes/<lane>.md` once approved
  (M-CC-P01 owns lane-file shape). Do not edit existing lanes from this file.
length_cap: 120
---

# Fitness-lane additions for P02-multi-subscription-pool

## 1. `oya-foundry-fitness-pool-routing-honor` (BLOCKER)

- **Owner:** axis-foundry.
- **Severity:** BLOCKER (PR cannot merge if lane red).
- **Trigger:** every PR touching `crates/oya-foundry-provider-pool-kernel/**`, `crates/oya-foundry-policy-kernel/**`, or `crates/oya-foundry-adapter-*-compat-api/**`.
- **Check:** simulated routing decision sequence (100 deterministic inputs) → assert every emitted `EVT-PROVIDER-POOL-ROUTING` audit event carries (a) the account_id returned by `pick_account`, (b) the routing_reason, (c) a non-null `tos_ack_ref` when `pool_size > 1`, (d) the trace_id propagated from the inbound request. Any silent account switch (i.e., adapter calls a ProviderAccount not named in the decision) fails the lane.
- **Implementation:** `tools/oya-foundry-fitness-pool-routing-honor/src/main.rs`.
- **Acceptance:** `oya gate validate oya-foundry-fitness-pool-routing-honor` exit code 0; CI lane name matches the gate name.

## 2. `oya-foundry-fitness-tos-acknowledgment` (BLOCKER)

- **Owner:** axis-foundry + council-privacy.
- **Severity:** BLOCKER.
- **Trigger:** every PR that modifies a `TenantPoolingPolicy` record, `PoolingPolicyCheck`, or any pool-member set with `len() > 1`.
- **Check:** for every `(tenant_id, provider)` reachable from any pool with `pool_size > 1`, assert a non-revoked `ToSAcknowledgment` row exists in the audit ledger with `evidence_hash` reachable. Any missing ack fails the lane and refuses the PR.
- **Implementation:** `tools/oya-foundry-fitness-tos-acknowledgment/src/main.rs`.
- **Acceptance:** `oya gate validate oya-foundry-fitness-tos-acknowledgment` exit code 0.

## 3. `oya-foundry-fitness-upstream-api-drift` (HIGH)

- **Owner:** axis-foundry.
- **Severity:** HIGH (not blocking; opens auto-PR on BREAKING drift).
- **Cadence:** nightly 02:00 UTC + on every PR touching `contracts/foundry-compat-*.openapi.yaml`.
- **Check:** fetch upstream OpenAPI from each `UpstreamSpec` in `UpstreamRegistry`; run `oasdiff` against the pinned `adapter_contract_path`; classify findings (BREAKING / NON-BREAKING / ADDITIVE). On any BREAKING finding, open a PR with the suggested contract delta and label `upstream-api-drift`; ping `api-shape-reviewer`.
- **Implementation:** `tools/oya-foundry-fitness-upstream-api-drift/src/main.rs` + `.github/workflows/upstream-api-drift.yml`.
- **Acceptance:** lane runs nightly; `oya gate validate oya-foundry-fitness-upstream-api-drift --dry-run` exit code 0.

## 4. `oya-foundry-fitness-compat-api-shape-binding` (BLOCKER)

- **Owner:** axis-foundry.
- **Severity:** BLOCKER.
- **Trigger:** every PR touching `crates/oya-foundry-adapter-*-compat-api/**` or `contracts/foundry-compat-*-v1.openapi.yaml`.
- **Check:** generate a sample request from each contract operation; replay against the running adapter; validate the response shape strictly against the upstream OpenAPI schema (Anthropic Messages v1 or OpenAI Chat Completions v1). Any deviation — extra field, missing field, wrong type, wrong SSE framing — fails the lane.
- **Implementation:** `tools/oya-foundry-fitness-compat-api-shape-binding/src/main.rs` driving `wiremock` + the live adapter.
- **Acceptance:** `oya gate validate oya-foundry-fitness-compat-api-shape-binding` exit code 0; smoke fixtures captured under `crates/oya-foundry-adapter-*-compat-api/tests/fixtures/`.

## 5. Lane interaction matrix

| Lane | Blocks | Notifies |
|---|---|---|
| `-pool-routing-honor` | merge to `main` | axis-foundry + ops-sre-reliability |
| `-tos-acknowledgment` | merge to `main` + tenant onboarding completion | council-privacy + ops-compliance |
| `-upstream-api-drift` | nightly auto-PR scaffold | api-shape-reviewer + axis-foundry |
| `-compat-api-shape-binding` | merge to `main` | axis-foundry |

## 6. Lane ownership + escalation

- **Tier 0** (axis-foundry): default oncall investigates lane reds.
- **Tier 1** (council-architecture): cross-axis coordination if a lane red signals a kernel-level regression.
- **Tier 2** (Founder): regulatory exposure (e.g., ToS-ack lane red in production).

## 7. Lift target

When approved, each row above lands as a file under `.omc/fitness-lanes/<lane>.md` following the canonical lane-file shape; M-CC-P01 owns the lift. This delta-spec is the proposal anchor only.
