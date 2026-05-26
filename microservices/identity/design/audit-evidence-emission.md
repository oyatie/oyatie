---
doc_class: DesignNote
title: Workload-Identity Audit Evidence Emission
microservice: identity
bounded_context: workload-identity
status: Proposed
date: 2026-05-26
owner_team: axis-identity + ops-security
related_adrs: [ADR-0002, ADR-0162]
research_brief: microservices/identity/design/hyperscaler-best-practice-brief.md
---

# Workload-Identity — Audit Evidence Emission

## Why we emit our own decision log

The cited brief's load-bearing flag #3: AWS Verified Permissions `IsAuthorized`
is **not** logged to CloudTrail by default — when enabled it lands in
`additionalEventData` as opt-in data events. The lesson (brief §9): an
authorization substrate must emit its **own** immutable decision log, because the
managed analog is opt-in and easy to leave dark. So this context emits
**unconditionally**, on every decision.

## The canonical record (brief §9)

One immutable record per authorize call, carrying exactly:

```json
{
  "timestamp": "2026-05-26T12:00:00Z",
  "tenant_id": "tenant_acme",
  "trust_domain": "acme.oyatie.dev",
  "principal_id": "spiffe://acme.oyatie.dev/ns/payments/sa/ledger-writer",
  "action": "ledger:Write",
  "resource": "tenant_acme/ledger/main",
  "context": { "mfa_present": false, "source_cell": "eu-frankfurt-1" },
  "decision": "DENY",
  "determining_policies": [],
  "errors": [],
  "correlation_id": "0d3b...-uuid",
  "decision_id": "evt_01J..."
}
```

Field rationale (brief §9):

- **`decision` + `determining_policies` + `errors`** — the AVP decision core.
  An empty `determining_policies` on a `DENY` is an **implicit deny**; non-empty
  is an explicit `forbid`. The distinction is preserved — never collapsed — for
  forensics.
- **`principal/action/resource/context`** — the AVP request context (PARC).
- **`tenant_id` / `trust_domain`** — the isolation key (brief §6).
- **`correlation_id`** — ties the decision to the originating request across PEP
  and PDP.
- **`decision_id`** — stable, never-reused id (`evt_<ULID>`), the AsyncAPI
  subject (brief §3, §5).

## Two distinct event types

Per the brief (§9, "capture validation outcomes AND authorization outcomes as
distinct event types"):

1. `IdentityWorkloadTokenValidated` on `identity.authz.decision.v1` — the
   validation outcome (valid / typed failure). Carries `sub` + `jti`, never the
   token body.
2. `IdentityWorkloadAuthorizationDecided` on `identity.authz.decision.v1` — the
   authorization decision (the record above).

Lifecycle transitions emit on `identity.principal.lifecycle.v1`
(`IdentityWorkloadPrincipalSuspended`, `IdentityWorkloadPrincipalRetired`).

## Binding + integrity

- **Bind to the immutable subject id** (brief §9). The `principal_id` is the
  SPIFFE-shaped, never-reused id; retired ids are tombstoned, so a subject id
  uniquely identifies one principal for all time — the GCP one-to-one subject
  mapping property for non-repudiation (brief §3, §9).
- **Never log token bodies** (brief §9). For replay forensics we log `sub` +
  `jti` (or a token hash), never the JWS.
- **Emit into the audit chain** (`evidence/audit-chain.jsonl`-style, per
  ADR-0162 per-tenant slicing) — Merkle-sealed, append-only, immutable.

## Completeness as an SLI

Emission is unconditional, so emission *completeness* is itself an operational
guarantee: a stalled or dropped seal is alerted and back-filled, never silently
lost (`design/failure-modes.md` F14). The decision-correctness SLO
(`slos/decision-correctness.openslo.yaml`) consumes this log to verify decisions
against the golden corpus.

## References

Brief load-bearing flag #3 + §9 (AVP/CloudTrail fields, implicit-vs-explicit
deny, one-to-one subject mapping); ADR-0162. Schemas:
`contracts/identity.asyncapi.yaml`.
