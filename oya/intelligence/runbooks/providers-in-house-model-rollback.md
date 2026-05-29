---
doc_class: Runbook
title: In-house model rollback
microservice: foundry-providers
severity: "Sev-1 (regression observed in production) / Sev-2 (canary regression)"
status: Accepted
owner_team: axis-foundry + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/threat-model.md (T-04 in-house rollout regression)
  - microservices/intelligence/failure-modes.md (FM-FP-04 in-house rollout regression)
  - ADR-0026 (in-house AI model substrate roadmap)
  - microservices/observability/runbooks/rollback.md (release pointer rollback)
doc_status: published
---

# Runbook: In-house model rollback

## Trigger

ONE of:

1. **Automated** — `oya_foundry_providers_provider_quality_score{vendor="in-house"}` drops below the threshold (default 0.95 of incumbent) over a 60 s window; router auto-demotes in-house and burn-rate alert fires.
2. **Tenant report** — tenant operator reports degraded output quality from a workload that recently shifted to in-house.
3. **Manual** — pre-emptive rollback ahead of a model retraining cycle or a known-bad rollout window.

## Severity

- Canary cohort (≤ 10 % traffic) regression, alternate vendor available: **Sev-2**.
- Production cohort regression OR no alternate compliant vendor in pack: **Sev-1**.

## Pre-checks

1. Identify the in-house model version currently serving: `oya_foundry_providers_in_house_model_version{pack="<p>"}` returns the deployed version.
2. Identify the prior-version model that was serving before rollout: `oya_foundry_providers_in_house_model_version_prior{pack="<p>"}`.
3. Confirm prior model is still warm (replica pool has the prior model loaded) or that loading time is acceptable (≤ 5 min cold load).
4. Confirm alternate vendor capacity available if rolling tenants off in-house entirely.

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | If Sev-1: open `#inc-<id>`; IC + OpsLead + axis-foundry SME | ≤ 5 min |
| 2 | Confirm pre-checks above | ≤ 5 min |
| 3 | Demote in-house in the router for affected tenants: `cargo run -p oya-dev-cli -- providers demote --vendor in-house --pack <p> --duration 1h --reason "<id>"`. Router routes affected tenants to next-best vendor per `policy/data-residency.md` | ≤ 1 min |
| 4 | If a prior in-house version is warm + capability-fit: redirect canary traffic back to prior version via `cargo run -p oya-dev-cli -- providers in-house-rollback --pack <p> --to-version <prior-version> --reason "<id>"` | ≤ 5 min |
| 5 | If prior version cold or unavailable: route entirely to alternate vendor for the duration (per `runbooks/provider-outage-failover.md`) | ≤ 5 min |
| 6 | Verify quality score recovers: `oya_foundry_providers_provider_quality_score` returns above threshold within 10 min | ≤ 10 min |
| 7 | Notify tenant operators of the in-house demote per `incident-response.md` template | ≤ 30 min |
| 8 | Postmortem within 5 business days; identify why the regression slipped baseline-set parity tests (per ADR-0026 rollout gates) | – |
| 9 | Fix the regression in a separate PR; re-run baseline-set + canary cohort + ramp per ADR-0026 phase rollout protocol | per priority |

## Rollback (of the rollback)

If reverting to the prior in-house version itself regresses (rare but possible):
1. Route entirely to the alternate vendor per `runbooks/provider-outage-failover.md`.
2. Postmortem + retraining cycle.

## Verification

- `oya_foundry_providers_provider_quality_score` ≥ 0.95 of incumbent sustained 15 min.
- Tenant workload resumes at expected quality (verified via tenant operator + baseline-set re-run).
- `evidence/postmortems/<year>/<incident-id>.md` published.
- `evidence/runbook-drills/in-house-rollback/<unix_ts>.json` recorded for the drill (quarterly).

## Post-incident updates

- Baseline-set is augmented with the regression-pattern that slipped through.
- Canary cohort weighting is adjusted if observed regression escaped at too high a traffic share.
- ADR-0026 rollout-gate criteria are tightened if needed.

## References

- ADR-0026 — in-house AI model substrate roadmap.
- `microservices/intelligence/threat-model.md` T-04.
- `microservices/intelligence/failure-modes.md` FM-FP-04.
- `microservices/observability/runbooks/rollback.md`.
