---
doc_class: Runbook
title: Autonomy violation quarantine
microservice: foundry-runtime
severity: "Sev-2 (security signal); Sev-1 if malicious cross-tenant pattern"
status: Accepted
owner_team: ops-security + axis-foundry-runtime
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-06)
  - microservices/intelligence/threat-model.md (T-E-01)
  - microservices/intelligence/policy/runtime-isolation.md (TI-08)
  - ADR-0022
doc_status: published
---

# Runbook: Autonomy violation quarantine

## Trigger

`rate(oya_foundry_runtime_autonomy_violation_total[5m]) > threshold` per tenant.

Per ADR-0022 + threat-model T-E-01 + runtime-isolation.md TI-08: every capability dispatch checks the principal's autonomy tier ceiling against the capability's declared tier. Mismatch refuses dispatch + emits `AutonomyViolationDetected`. A burst of violations from one tenant / one principal is a security signal: misconfig, malicious probing, or genuine ceiling-mismatch needing tenant action.

## Severity

- Single tenant violation surge: Sev-2.
- Cross-tenant pattern OR escalation attempts on `tenant:oya-system` capabilities: Sev-1.

## Pre-checks

1. Verify violations are real, not false-positive: read autonomy ceiling cache; verify `TenantTierCeilingChanged` events recent; confirm ceiling cache signature valid.
2. Identify source: which principal? OIDC subject? Workload SPIFFE identity?
3. Identify pattern: single capability or many? Single session or many?

## Recovery Path A — Misconfig (legitimate tenant; ceiling/declared mismatch)

| Step | Action | Time |
|---|---|---|
| 1 | Engage tenant operator via gtm-customer-success | ≤30min |
| 2 | Walk through tenant's declared tier choice for the capability vs their ceiling | – |
| 3 | If tenant chose tier too high: ask them to lower declared tier OR raise ceiling per their tier-elevation review process | tenant-paced |
| 4 | If ceiling needs to be raised: tenant submits ceiling-elevation request (per ADR-0022 process); council-architecture + ops-security review | per process |
| 5 | After resolution, verify violations cease | ≤24h |

## Recovery Path B — Malicious probing

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1 if cross-tenant or escalation attempts on system capabilities | immediate |
| 2 | Quarantine offending principal: revoke OIDC tokens / API keys at OpenBao | ≤5min |
| 3 | Verify capability descriptor signatures are intact (T-T-04 hasn't co-occurred) | ≤5min |
| 4 | Forensic trace: identify how principal obtained credentials; investigate insider-threat angle if applicable | per investigation |
| 5 | Tenant comms: if a tenant's principal was compromised, notify tenant + force rotation | ≤30min |
| 6 | Breach-notification chain if confirmed data exposure (per `incident-response.md` regulatory timelines) | per pack |

## Recovery Path C — Bug in AutonomyGate (false-positive)

| Step | Action | Time |
|---|---|---|
| 1 | Verify ceiling cache signature, ceiling value, declared tier value | ≤10min |
| 2 | Compare with tenancy µservice source-of-truth ceiling | ≤5min |
| 3 | If ceiling cache is stale / corrupted: force refresh via `cargo run -p oya-intelligence-runtime-capability-executor-app -- refresh-ceilings --tenant <id>` | ≤10min |
| 4 | If AutonomyGate logic is buggy: emergency hotfix PR (review by ops-security + axis-foundry-runtime + ExecSponsor); deploy via Helm | ≤2h |
| 5 | Re-run autonomy gate test set | ≤30min |

## Verification

After recovery:
- `rate(oya_foundry_runtime_autonomy_violation_total[5m])` returns to baseline.
- Affected tenant operator confirms recovery.
- For Sev-1: postmortem includes forensic trace.

## Post-incident updates

- Postmortem within 5 business days.
- For Path A repeated: improve capability descriptor authoring UX in Workflow Studio to surface tier ceiling at author time.
- For Path B: harden credential-issuance + add anomaly-detection on violation patterns.
- For Path C: add regression test for the AutonomyGate logic.

## References

- ADR-0022 (autonomy tiers).
- `microservices/intelligence/failure-modes.md` FM-06.
- `microservices/intelligence/threat-model.md` T-E-01.
- `microservices/intelligence/policy/runtime-isolation.md` TI-08.
- `microservices/intelligence/incident-response.md`.
