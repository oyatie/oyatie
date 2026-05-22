---
doc_class: IP
ip_id: IP-022-chaos-drill-pack
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + sre
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/ARCHITECTURE.md
  - microservices/itsm/dashboards/local-operator-remediation.json
  - microservices/itsm/dashboards/compliance-pack-health.json
  - microservices/itsm/src/usecase/mod.rs
---

# IP-022 ITSM Chaos Drill Pack

## A. Problem
ITSM is the system operators use when other systems fail. Its own failure modes must be drilled: audit-chain outage, Cedar mismatch, regional outage, credential lease failure, notification provider failure, and CMDB drift. The stamped IP did not define scenarios.

This IP creates a chaos drill pack that validates ITSM's degraded behavior without corrupting tenant data.

## B. Approach
Drill against named architecture failure modes:

| Drill | Expected behavior |
|---|---|
| audit-chain unavailable | high-risk mutation pauses; emergency path buffers |
| Cedar fragment mismatch | mutations fail closed; safe reads degraded |
| regional outage | home-cell writes queue; no residency violation |
| credential lease expired | provider action denied; core tickets continue |
| notification provider down | escalation marks partial delivery and retries |
| CMDB drift | reconciliation emits relation-drift evidence |

## C. Deliverables
- Chaos scenario catalog for ITSM with tenant, cell, pack, and expected outcome.
- Drill fixtures that exercise `OpenIncident`, `RecomputeSla`, and `ApproveChange` paths.
- Dashboard evidence in `local-operator-remediation.json` and `compliance-pack-health.json`.
- Runbook links for each failure scenario.
- Rollback criteria for stopping a destructive or noisy drill.

## D. Implementation
1. Define synthetic tenant and pack fixture for each chaos drill.
2. Inject audit-chain outage and assert high-risk mutation behavior matches architecture.
3. Inject Cedar deny/mismatch and assert no repository write occurs.
4. Inject regional outage and assert no pack residency boundary is crossed.
5. Expire credential lease and assert provider actions fail without blocking incident open.
6. Simulate notification failure and assert escalation evidence records retry state.
7. Simulate CMDB drift and assert reconciliation event is emitted.
8. Add dashboard checks for drill result and operator remediation state.

## E. Acceptance
- Each drill has a named expected outcome and rollback condition.
- Chaos drills use synthetic tenants and cannot mutate production tenant records.
- Emergency path behavior from IP-013 is tested under audit outage.
- Drill output includes dashboard and audit evidence.

## F. Evidence
- `ARCHITECTURE.md` names source import drift, cross-tenant reference, duplicate command, regional outage, audit-chain outage, and pack conflict.
- `src/usecase/mod.rs` exposes usecases suitable for drill fixtures.
- `dashboards/local-operator-remediation.json` and `compliance-pack-health.json` exist.
- ADR-0244, ADR-0248, and ADR-0263 govern tenant/cell/evidence behavior.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow resiliency / operational drills | Oyatie drills failure modes against tenant/cell policy |
| Jira Service Management incident tooling | ITSM remains usable when dependencies degrade |
| Freshservice operations maturity | Drill results feed operator remediation evidence |

## H. Cold-start buildability notes
- Use synthetic tenants for every drill.
- Start with Cedar mismatch and audit outage drills.
- Keep provider outage drills separate from credential sidecar tests.
- Never run chaos against production tenant ids.
- Record expected behavior before injecting failure.
- Stop drills on unexpected cross-tenant access.
- Link each drill to dashboard evidence.
- Keep rollback command in the drill definition.
- Verify emergency behavior under audit outage.
- Preserve raw drill output for closeout evidence.
