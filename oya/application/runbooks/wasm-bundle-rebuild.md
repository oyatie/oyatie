---
doc_class: Runbook
title: WASM Bundle Rebuild — corrupt or regressed bundle recovery
microservice: application
severity: "Sev-2"
status: Accepted
owner_team: axis-application + axis-foundry
date: 2026-05-17
related_artifacts:
  - microservices/application/failure-modes.md (FM-04, FM-10, FM-16)
  - microservices/application/incident-response.md
doc_status: published
---

# Runbook: WASM Bundle Rebuild

## Trigger

ONE of:

1. **FM-04 WASM corruption** — `oya_application_wasm_instantiate_fail_total > 0`; canary cohort fails on bundle instantiate.
2. **FM-10 Leptos hydration regression** — `oya_application_hydration_error_rate > 1%` for ≥ 1 min.
3. **FM-16 TTI breach** — `oya_application_tti_p99_seconds > 2` for ≥ 5 min (multi-window burn-rate).

## Severity

**Sev-2** — degraded experience; auto-rollback to prior bundle handles
immediate impact; rebuild is the recovery action.

## Pre-checks

1. Confirm the failing bundle version: `oya_application_bundle_version_active{environment="production"}`.
2. Confirm auto-rollback triggered (frontend-bundle-serve worker): `oya_application_bundle_pointer_reverted_total > 0` recently.
3. Identify root cause: review CI build logs for the failing bundle SHA.

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | Confirm auto-rollback already engaged; users on prior bundle | ≤ 2 min |
| 2 | Open `#inc-<id>` (Sev-2; lighter pace than Sev-1) | ≤ 5 min |
| 3 | Open the failing build's CI run; identify the breaking commit | ≤ 15 min |
| 4 | Revert the breaking commit: `git revert <sha>` on `release/application/dev` | ≤ 5 min |
| 5 | Open PR; CI builds new bundle; verify k6 + Lighthouse + canary cohort 1 % passes | ≤ 30 min |
| 6 | SLO gate advances bundle through staging; canary cohort ramps 1 → 10 → 50 → 100 % (per `microservices/observability/IP-015-canary-cohort-weighting.md`) | ≤ 60 min |
| 7 | Confirm TTI / hydration / instantiate metrics return to budget | ≤ 30 min |
| 8 | Update postmortem template; capture root-cause class | per cadence |

## Workspace hygiene (the cause is often)

| Class of cause | Mitigation |
|---|---|
| `wasm-bindgen` version mismatch with Leptos | Pin both in workspace; lane refuses mismatched versions |
| Rust toolchain bump introduces miscompile | Pin toolchain in `rust-toolchain.toml`; CI runs WASM tests |
| Dependency drift via transitive crate | `cargo deny`; pin transitive via `[patch]` |
| Hash mismatch on PR merge (race condition) | CI builds reproducibly; SHA in URL must match in shell HTML |

## Verification

- `oya_application_wasm_instantiate_fail_total == 0` for ≥ 30 min.
- `oya_application_tti_p99_seconds <= 2` for ≥ 30 min on rebuilt bundle.
- `oya_application_hydration_error_rate < 0.1%` for ≥ 30 min.
- Canary cohort reaches 100 % production without SLO breach.

## References

- `failure-modes.md` FM-04, FM-10, FM-16.
- `microservices/observability/IP-015-canary-cohort-weighting.md`.
- ADR-0065 (Leptos as canonical webapp framework).
