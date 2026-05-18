---
doc_class: PolicySpec
title: Workflow Spec Integrity Contract
microservice: workflow-engine
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-workflow + ops-security
deciders: axis-workflow, ops-security, council-architecture
related_adrs: [ADR-0028, ADR-0035, ADR-0103, ADR-0131]
related_artifacts:
  - microservices/workflow-engine/threat-model.md (T-T-01, T-T-04, T-T-06)
  - microservices/workflow-engine/PRD.md
review_cadence: annually + on any spec format version bump
doc_status: published
---

# Workflow Spec Integrity Contract (workflow-engine µservice)

## Purpose

Define the integrity contract for workflow specs: versioning, signature, immutability, revocation, replay-safety. This document is the canonical reference for `oya-governance-workflow-spec-signature-verification` LEAN lane and the engine's spec-store BC.

## Spec Versioning

### Version model

Each workflow spec is identified by:

```
(tenant_id, spec_id, version_sha)
```

- `tenant_id`: per-tenant namespace (set by engine from authenticated tenant; client cannot supply).
- `spec_id`: tenant-chosen stable identifier (e.g., `leave-request-approval`).
- `version_sha`: SHA-256 over the canonicalized spec body + signing metadata; deterministically derivable from the spec content; immutable.

Properties:
- Two distinct spec versions for the same `(tenant_id, spec_id)` have different `version_sha`.
- Bit-identical spec bodies under the same `(tenant_id, spec_id)` produce identical `version_sha` (idempotent submission).
- Re-submission of an already-existing version is a no-op (returns the existing version row).

### Version lifecycle

```text
draft → published → deprecated → retired
```

- `draft`: in-progress; Studio may save without engine registration.
- `published`: submitted to engine; available for run-starts; immutable.
- `deprecated`: still loadable for in-flight runs; new runs forbidden.
- `retired`: forbidden from any run-start; replay still permitted for audit.

State transitions are recorded with actor identity in audit-chain.

## Signature

### Signing scheme

Every published spec carries an Ed25519 signature over a canonicalized representation:

```
signed_message = canonical_json(spec_body)
              || version_sha
              || tenant_id
              || spec_id
              || signer_identity
              || timestamp
```

- Signing keys are OpenBao-managed (per-tenant); rotated 90d.
- Signer identity is bound at OpenBao issuance; SDK auto-populates on submit.
- Engine verifies the signature against the OpenBao-published public-key set at every spec read.

### Verification path

Every spec read (run-start, replay, debugger) verifies the signature. Cached verification:
- Successful verification cached per `version_sha` for 60s.
- Cache eviction on revocation event.

### Revocation

Per ops-security policy, signing keys can be revoked (key compromise, signer offboarding, etc.). Revocation propagation:

```text
ops-security revokes key in OpenBao
    ↓
OpenBao publishes revocation list (CRL)
    ↓
Engine's spec-store cache invalidated within ≤ 60s
    ↓
In-flight runs using a spec signed by the revoked key are PAUSED with audit-chain emission
    ↓
Operator must re-sign or migrate to a non-revoked spec version before resume
```

Revocation propagation lag SLI: `oya_workflow_engine_revocation_propagation_lag` target ≤ 60s p99.

## Immutability

### Storage immutability

Published spec rows in Postgres are append-only:
- INSERT-only on `spec_versions` table.
- UPDATE/DELETE refused by Postgres trigger.
- Soft-deprecate via separate `spec_lifecycle` table (which IS UPDATE-able for lifecycle transitions).

### Replay safety

Runs pin to a specific `version_sha` at run-start time. Mid-run version migration is forbidden:
- A long-running workflow started against v1 continues against v1 even after v2 is published.
- Operator-initiated migration requires explicit `oya vcs migrate-workflow-version` command with 2-person rule.

## Spec Body Canonicalization

The spec body MUST be canonicalized before hashing/signing:
- JSON keys sorted lexicographically.
- Whitespace normalized.
- Unicode normalized to NFC.
- Floating-point excluded (use rationals / fixed-point integers).
- No external references (no `$ref` to external URLs; inline-only).

Canonical form is reversible by the engine for human reading; original form preserved alongside canonical for tenant-side display.

## Forbidden Spec Constructs

Specs that contain any of these are refused at submit:

| Construct | Rationale |
|---|---|
| `eval` / `exec` step bodies | Non-deterministic; breaks replay invariant |
| System-time access in step body | Non-deterministic; use engine-provided clock |
| Non-deterministic RNG | Non-deterministic; use engine-provided seeded RNG |
| Uncached I/O in step body | Non-deterministic; side effects must go through engine retry/idempotency |
| Plain-text secrets in step input | Use OpenBao SecretReference |
| Circular sub-workflow references | Would cause deadlock |
| Step body > 1MB | Bounded payload size; large payloads use object-storage reference |
| Unrecognized event type in trigger | Must be a registered workflow event |

LEAN check `oya-governance-spec-construct-conformance` enforces.

## Audit Trail

Every spec lifecycle transition emits an audit-chain record:

```json
{
  "kind": "WorkflowSpecLifecycle",
  "tenant_id": "<hash>",
  "spec_id": "<id>",
  "version_sha": "<sha>",
  "lifecycle_from": "draft|published|deprecated|retired",
  "lifecycle_to": "published|deprecated|retired",
  "signer": "<spiffe-or-oidc>",
  "timestamp": "ISO8601",
  "merkle_seal": "<ed25519>"
}
```

Retention: append-only; immutable; replicated within-pack only.

## Verification

- `oya gate validate workflow-spec-signature-verification` — exits 0; signature verification path is exercised in unit tests for every spec read path.
- `oya gate validate spec-construct-conformance` — exits 0; LEAN parser validates all submitted specs.
- Annual security audit of OpenBao-managed signing keys.

## References

- ADR-0028 (audit-chain).
- ADR-0035 (workflow engine).
- ADR-0103 (workflow hexagonal).
- `microservices/workflow-engine/threat-model.md` T-T-01, T-T-04, T-T-06.
- `microservices/workflow-engine/PRD.md` FR-01, FR-02.
- OpenBao key management — `openbao.org/docs/secrets/transit/`.
- Ed25519 RFC 8032.
