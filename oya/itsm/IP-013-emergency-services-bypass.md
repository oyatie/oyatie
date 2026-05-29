---
doc_class: IP
ip_id: IP-013-emergency-services-bypass
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + council-safety
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/policy/emergency-services-bypass.cedar
  - microservices/itsm/src/domain/mod.rs
  - microservices/itsm/src/usecase/mod.rs
  - microservices/itsm/dashboards/local-operator-remediation.json
---

# IP-013 ITSM Emergency Services Bypass

## A. Problem
ITSM has a narrow emergency class: P0/P1 incidents and on-call acknowledgement must stay reachable when ordinary abuse friction, cost admission, or degraded audit backpressure would otherwise slow the operator. The stamped IP did not define the boundary and risked sounding like a policy bypass.

This is not a bypass around tenant scope or Cedar. It is an emergency-path policy that chooses reduced friction while retaining audit evidence.

## B. Approach
Represent emergency behavior as Cedar-permitted break-glass actions:

| Emergency action | Allowed relaxation | Non-negotiable |
|---|---|---|
| open P0/P1 incident | skip nonessential requester friction | tenant id, principal id, audit |
| acknowledge page | no CAPTCHA | signed device/session |
| open incident-room | allow degraded audit buffer | MLS group and later audit seal |
| publish status update | allow pre-approved template | audience/residency gate |

The policy lives in `policy/emergency-services-bypass.cedar` and must be observable in `local-operator-remediation.json`.

## C. Deliverables
- Cedar emergency policy with action names tied to ITSM `Capability` values.
- Domain helper around `Priority::is_major()` to identify major incident path.
- Usecase tests for P0/P1 incident open under degraded nonessential dependencies.
- Audit event classes for break-glass opened, used, expired, and reviewed.
- Operator dashboard for emergency-path usage review.

## D. Implementation
1. Add emergency context fields: `incident_priority`, `break_glass_reason`, `expires_at`, and `reviewer_required`.
2. Gate emergency path through Cedar before state mutation.
3. Use `Priority::is_major()` as the domain trigger, not a free-text title check.
4. Allow degraded audit buffering only for emergency actions; non-emergency high-risk mutations still pause.
5. Emit break-glass audit evidence with principal, tenant, priority, and expiry.
6. Add post-incident review workflow linking the emergency use to postmortem action items.
7. Add tests that P2 incidents cannot claim emergency bypass.
8. Add dashboard panels for emergency count, expiry violations, and review overdue.

## E. Acceptance
- P0/P1 paths remain usable under nonessential friction failure.
- Tenant, Cedar, MLS, and audit evidence are never skipped.
- P2/P3/P4 tickets cannot use the emergency path.
- Every emergency use has a review artifact linked to the incident or postmortem.

## F. Evidence
- `policy/emergency-services-bypass.cedar` exists.
- `Priority::is_major()` exists in `src/domain/mod.rs`.
- `OpenIncident::execute` is the current incident-open usecase.
- `dashboards/local-operator-remediation.json` exists for operator follow-up.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow major incident flows | Break-glass is policy-governed and reviewable |
| Jira Service Management incident escalation | Emergency path preserves tenant and audit checks |
| Freshservice major incident handling | Reduced friction is limited to P0/P1 paths |

## H. Cold-start buildability notes
- Use `Priority::is_major()` as the first domain guard.
- Keep break-glass explicit in Cedar context.
- Require an expiry timestamp for every emergency decision.
- Add P2 negative tests before broadening emergency behavior.
- Buffer audit only where ADR-0263 permits later sealing.
- Link every emergency use to postmortem review.
- Do not allow emergency path to skip tenant id or principal id.
- Keep status-update emergency templates pre-approved.
- Surface emergency usage in operator dashboards.
- Treat missing review as a follow-up finding.

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/itsm/IP-013-emergency-services-bypass.md` matched [`cost`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/itsm/IP-013-emergency-services-bypass.md`, `microservices/itsm/manifest.json`, `microservices/itsm/capacity-model.md`, `microservices/itsm/compliance.md`, `microservices/itsm/ARCHITECTURE.md`].
