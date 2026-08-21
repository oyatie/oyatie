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

> **Status: not yet operable.** This capability has no crate, no running PDP and no metric emitter —
> `PROMOTION.md` §2 records why. The identifiers below (the Cedar rules below) are real and tested
> (`policy/cedar/CONFORMANCE.md`), but nothing evaluates them at runtime yet. This is the procedure the design commits to, written before the code so it can be
> reviewed against the design; it is not a description of a system in production.

**Symptom:** `ActivatePolicy` or `PublishPolicy` denied for a change that "looks fine".

Read the determining policy ids first. **Two shapes of deny exist and they are not the same:**

- **An explicit deny** names the forbid that produced it. Match it in the table below.
- **A default deny names nothing** — the determining-policy set is *empty*. Cedar denies when no
  permit matches, so an empty set means "no rule granted this", not "a rule refused it". If you see
  no determining policy, the actor is missing a role, a tenant membership or a step-up class; go to
  the permits in `authoring-grants.cedar`, not to the table.

Every allow names at least one permit, so an allow is always attributable.

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
