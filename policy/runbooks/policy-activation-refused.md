---
doc_class: Runbook
shape: Runbook
length_cap: 200
microservice: policy
related_adrs:
  - ADR-0294
inbound_citations:
  - policy/README.md
---

# Runbook — a policy activation was refused

**Symptom:** `ActivatePolicy` or `PublishPolicy` denied for a change that "looks fine".

The decision is attributable: every allow names at least one permit, and a deny names the forbid that
produced it. Read the determining policy id before anything else — it tells you which of these it is.

| Determining rule | Meaning | Correct action |
|---|---|---|
| **F6** | the version is unsigned | get it signed; do not publish unsigned |
| **F5** | the signer is the author | a second principal must sign — this is separation of duties, not a glitch |
| **F7** | the ADR-0294 soak window has not elapsed | wait. 60s minimum. There is no override |
| **F8** | global scope without `platform-policy-engineer` + step-up C | either scope the change to a tenant, or get the right principal with the right step-up |
| **F1** | principal and resource are in different tenants | this is a cross-tenant attempt; treat as a security event until proven a client bug |

## Why there is no break-glass on F7

The soak window is what makes rollback cheap: a policy that has soaked has been observed against real
traffic in shadow before it decides anything. Skipping it trades a 60-second wait for an unbounded
blast radius on a change that widens authorization. If an incident genuinely needs authorization
relaxed faster than the soak, the correct lever is a **narrower, tenant-scoped** policy — which F8
already permits a tenant-policy-admin to publish without step-up.

## Escalation

`axis-policy-engine`.
