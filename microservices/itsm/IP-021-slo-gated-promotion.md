---
doc_class: IP
ip_id: IP-021-slo-gated-promotion
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + sre
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/dashboards/slo-and-error-budget.json
  - microservices/itsm/dashboards/local-slo-burn.json
  - microservices/itsm/tests/integration.rs
  - microservices/itsm/manifest.json
---

# IP-021 ITSM SLO-Gated Promotion

## A. Problem
ITSM cannot promote a new incident, change, portal, or mobile path just because docs exist. ServiceNow/Jira/Freshservice customers judge the product on uptime, latency, and escalation reliability. Oyatie needs promotion gates tied to SLO evidence and test results.

The stamped IP did not name which SLOs gate promotion.

## B. Approach
Define promotion lanes by capability:

| Capability | Gate |
|---|---|
| incident open | p95 command latency and audit event completeness |
| SLA recompute | p99 breach detection under 15s target from PRD |
| change approve | freeze-window policy latency and denial correctness |
| portal | self-service latency + deflection audit |
| mobile ack | end-to-end ack under 12s target |
| CMDB sync | relation drift freshness |

Promotion consumes tests, dashboards, and audit evidence; it does not use line count.

## C. Deliverables
- SLO gate matrix for each ITSM capability and tenant class.
- Dashboard references in `slo-and-error-budget.json` and `local-slo-burn.json`.
- Integration tests that feed promotion evidence for scaffold/usecase paths.
- Failure policy: promotion blocks on missing evidence, not just failed evidence.
- Rollback policy for SLO regression after promotion.

## D. Implementation
1. Define capability-to-SLO mapping in a machine-readable gate record when schema exists.
2. Add promotion checks for latency, availability, audit completeness, and policy-denial correctness.
3. Require separate demo_trial and paid evidence where tenant class changes behavior.
4. Pull usecase test evidence from `tests/integration.rs`.
5. Pull operational evidence from dashboards and OpenSLO manifests when present.
6. Treat missing SLO files as BLOCKER for GA promotion but not as invented pass claims.
7. Add rollback trigger when burn rate crosses threshold or audit completeness drops below target.
8. Add counterpart benchmark notes for ServiceNow/Jira/Freshservice expectations.

## E. Acceptance
- Every promoted ITSM capability has named SLO, test, dashboard, and rollback evidence.
- Missing evidence blocks promotion.
- Demo_trial evidence cannot promote paid behavior.
- ServiceNow/Jira/Freshservice parity claims cite specific ITSM capability metrics.

## F. Evidence
- `dashboards/slo-and-error-budget.json` and `dashboards/local-slo-burn.json` exist.
- `tests/integration.rs` validates current scaffold, REST, and AsyncAPI behavior.
- `manifest.json` distinguishes demo_trial from paid.
- ADR-0328 rejects line count as proof; ADR-0263 supplies evidence discipline.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow availability expectations | Promotion waits for SLO evidence |
| Jira Service Management Cloud reliability | Tenant-class-specific readiness gates |
| Freshservice enterprise operations | Rollback tied to burn and audit evidence |

## H. Cold-start buildability notes
- Define the promotion claim before running checks.
- Treat missing evidence as a failed gate.
- Separate demo_trial and paid evidence.
- Use existing integration tests as scaffold proof only.
- Add dashboard links for each capability gate.
- Keep SLA breach target aligned with PRD wording.
- Do not promote AI deflection based on incident tests.
- Record rollback threshold with every promoted capability.
- Include counterpart benchmark only when evidence exists.
- Preserve failed-gate output in remediation notes.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-021-slo-gated-promotion.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/itsm/IP-021-slo-gated-promotion.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].
