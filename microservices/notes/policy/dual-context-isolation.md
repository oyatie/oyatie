---
doc_class: PolicySpec
title: Dual-Context Isolation Specification
microservice: notes
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-security + axis-notes
deciders: council-architecture, ops-security, axis-notes, council-privacy
related_adrs: [ADR-0008, ADR-0028, ADR-0126, ADR-0131, ADR-0132, ADR-NOTES-0001, ADR-NOTES-0003, ADR-NOTES-0005]
related_artifacts:
  - microservices/notes/threat-model.md (T-S-01, T-I-01, T-E-01; cross-context invariant violation)
  - microservices/notes/dpia.md (R-07)
  - microservices/notes/policy/tenant-scope.cedar
  - microservices/notes/policy/e2e-personal-tier-default.md
review_cadence: quarterly + on every BC change
doc_status: published
---

# Dual-Context Isolation Specification (notes µservice)

## Purpose

Define the load-bearing dual-context invariants of the notes substrate. Per parallel ADR-0126 (inheriting Bominal ADR-0208's dual-context model), every notes entity carries a `context_kind: { Personal | Professional }` discriminator that determines:

- which keys encrypt the body (Personal client-derived MLS E2E vs Professional tenant-DEK envelope);
- which retention floor applies (Personal user-policy vs Professional pack-floor);
- which disclosure path is reachable (Personal: never to admin; Professional: four-eyes only);
- which audit-chain seal stream the event lands on (Personal: only sharing events; Professional: every state transition);
- whether AI assist is reachable (Personal: never; Professional: opt-in);
- whether Loro collab is reachable (Personal: never; Professional: opt-in).

The Personal-pillar bar is sharper than docs because notes are first-thought capture: ADR-NOTES-0001 establishes the E2E-default posture, and ADR-NOTES-0005 establishes structural-impossibility of AI on Personal notes.

This document is the authoritative reference for SOC 2 (CC6.1), ISO 27001 (A.5.15, A.8.3), GDPR Art. 25, KR PIPA Art. 28, HIPAA OCR asking *"how does notes keep personal and professional separated?"*

## Context Kind Enumeration

```rust
// oya-notes-note-store-kernel (sealed)
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContextKind {
    Personal,
    Professional,
}
```

Properties:
- Enum is sealed at the kernel layer; only two variants ever exist.
- Cross-variant write is rejected at the domain layer.
- Runtime config CANNOT switch a Personal entity into Professional (or vice versa); this is a compile-time + data-model invariant per parallel ADR-0126.

## Entity Type Invariants

### Invariant DCI-01: Distinct entity reference types

The notes BC introduces **distinct typed references** for direct manipulation:

| Reference type | Context | Backing crate |
|---|---|---|
| `PersonalNoteRef` | only `Personal` | `oya-notes-note-store-kernel` |
| `ProfessionalNoteRef` | only `Professional` | `oya-notes-note-store-kernel` |

There is **no shared trait** that allows generic code to construct either; consumers must match on the ref type. LEAN check `oya-check-dual-context-isolation` asserts there are no trait impls covering both.

### Invariant DCI-02: No cross-type write path

The `NoteRepository::create(note: Note)` port trait requires the `Note` carry the `context_kind` of its parent notebook (where applicable). The domain layer rejects:

- A `Note{context_kind = Personal}` written into a Professional notebook.
- A `Note{context_kind = Professional}` written into a Personal stack.

LEAN check inspects every `NoteRepository` impl + every `VersionHistoryStore` impl + every `SearchIndex` impl and asserts that the type signature forbids cross-type flows.

### Invariant DCI-03: Distinct key material (sharper than docs)

- **Personal**: each `Note` body is **E2E-encrypted with client-derived MLS keys** (ADR-NOTES-0001); oyatie never sees plaintext. Server stores `body_ciphertext`. The `body_client_only: ()` zero-byte marker on `PersonalNoteRef` forbids ever materialising plaintext server-side.
- **Professional**: each `Note` body is **tenant-DEK envelope-encrypted** (Bominal ADR-0111); oyatie can decrypt under four-eyes audit. Server stores `body_ciphertext` and the wrapped DEK.

Key-material types are sealed:

```rust
pub struct PersonalE2EKey(/* zero-byte marker; client-held only */);
pub struct TenantDek(/* OpenBao-bound; never serialised to logs */);
```

The compiler refuses to coerce one into the other.

### Invariant DCI-04: Distinct retention paths

Per `data-residency.md`:

- Personal: retention follows the user's per-user policy (default = no retention floor; tenant cannot override).
- Professional: retention follows the tenant's pack-aware policy (KR-PIPA floor for KR; HIPAA 6y floor for pack-us-healthcare; etc.).

The retention worker reads `context_kind` of every row and routes to the corresponding policy engine. A misroute is a compile-time error in the worker code because the routing function takes typed `Personal | Professional` arms.

### Invariant DCI-05: Distinct audit paths

- Personal: only **sharing events** (`ShareLinkCreated`, `ShareLinkRevoked`, `ShareLinkAccessed`) emit to audit-chain; routine personal capture does NOT emit (per ePrivacy + privacy-by-design + KR PIPA Art. 23).
- Professional: every state transition (create, edit, delete, tag, untag, share, hold, disclosure, AI invocation) emits to audit-chain.

### Invariant DCI-06: Distinct Cedar evaluator

Cedar policy fragments split:

- `policy/tenant-scope.cedar` — covers both tiers but conditions on `context_kind`.
- (no separate `personal-note-scope.cedar` because the type system enforces; the Cedar fragment uses `forbid … when context_kind == "Personal"` for belt-and-suspenders disclosure-refusal and AI-refusal.)

A Cedar query against the wrong entity type returns deny.

### Invariant DCI-07: No runtime config-toggle for context

The `ContextKind` is set at note-creation time and is **immutable**. There is no API, no admin tool, no migration path to convert a Personal note into Professional or vice versa. The user-facing surface offers "create a new Professional note with the same content" instead — explicit migration with consent.

### Invariant DCI-08: AI assist refused on Personal (sharper than docs)

Per ADR-NOTES-0005:

- Type-system: `AssistInvoker::invoke(ProfessionalNoteRef)` — refuses `PersonalNoteRef`.
- Cedar: `forbid … action == Action::"invoke_ai_assist" … when context_kind == "Personal"`.
- CI lane: `oya-check-e2e-ai-refusal` BLOCKS on any `PersonalNoteRef → AssistInvoker::invoke` path.
- Runtime metric: `oya_notes_ai_call_blocked_e2e_total > 0` fires Sev-1.

### Invariant DCI-09: Loro collab refused on Personal

Per ADR-NOTES-0003:

- Type-system: `CollabSessionStore::start_session(ProfessionalNoteRef)` — refuses `PersonalNoteRef`.
- Cedar: `forbid … action == Action::"start_collab_session" … when context_kind == "Personal"`.

## CI-Lane Enforcement

### Lane: `oya-check-dual-context-isolation`

Located at `crates/oya-check-dual-context-isolation/`. Asserts:

1. `ContextKind` enum is sealed at exactly two variants.
2. `PersonalNoteRef` and `ProfessionalNoteRef` are distinct types with no shared inherent impl.
3. Every `NoteRepository::create` impl rejects cross-type writes (verified via UI test on attempt to mix types — compile must fail).
4. Personal key material types cannot be coerced to Professional DEK types.
5. Retention worker routes via exhaustive `match` on `ContextKind`.
6. Audit-chain client distinguishes Personal sharing-only stream vs Professional all-events stream.
7. No `mut context_kind` field exists anywhere; note creation is the only setter.
8. No `into_professional()` / `into_personal()` conversion methods exist.
9. `AssistInvoker::invoke` accepts only `ProfessionalNoteRef`.
10. `CollabSessionStore::start_session` accepts only `ProfessionalNoteRef`.
11. Workflow events for Personal-tier carry opaque `note_id` only; never title or body.

Severity: BLOCKER. Lane is required on `dev` and `staging` per branch-protection.

### Lane: `oya-check-e2e-ai-refusal`

Located at `crates/oya-check-e2e-ai-refusal/`. Asserts:

1. No path from `PersonalNoteRef` to `AssistInvoker::invoke` in the call graph.
2. Cedar `tenant-scope.cedar` carries `forbid … action == Action::"invoke_ai_assist" … when context_kind == "Personal"`.
3. PrometheusRule alarm `MessengerDualContextDenyDetected`-analog exists for AI E2E refusal.
4. Runtime metric `oya_notes_ai_call_blocked_e2e_total` registered.

Severity: BLOCKER.

## Runtime Enforcement

In addition to compile-time + LEAN-lane enforcement, runtime guards:

- Every API request tags with active persona (`Personal` or `Professional`) from JWT claim; the entity-resolution layer refuses requests where persona doesn't match the resource's `context_kind`.
- Postgres rows carry `context_kind` as a non-nullable column with a CHECK constraint; cross-context join queries are rejected at DB layer.
- Search index is partitioned per `context_kind`; Personal-tier search is client-side-only (per ADR-NOTES-0004).

## Operational Procedures

- A `ContextSwitchDeniedAttempt` Prometheus metric is emitted per attempted violation; alert at > 0 over 5min.
- A Sev-1 incident is declared on any confirmed cross-context routing (per `incident-response.md`).
- Periodic chaos test injects a synthetic cross-type write attempt; verifies rejection + alert.

## Verification

- Unit tests: every `NoteRepository` impl has a UI test that fails to compile on cross-type write.
- Integration tests: synthetic Personal-note → Professional-write routing attempt returns 403 + emits metric + writes audit-chain record.
- Pen-test: annual external red-team attempt to break the invariant via API misuse.

## References

- Parallel ADR-0126.
- Bominal ADR-0208 (Connect dual-context unified channel hub; inherited).
- Bominal ADR-0215 (Connect retention legal-hold dual-context; inherited).
- ADR-NOTES-0001 (E2E-default Personal-tier).
- ADR-NOTES-0003 (Loro collab refused on Personal).
- ADR-NOTES-0005 (AI refused on Personal).
- ADR-0008 Data Use Boundary.
- `microservices/notes/threat-model.md`.
- `microservices/notes/dpia.md`.
- `microservices/notes/policy/tenant-scope.cedar`.
- `docs/standards/dual-context-isolation.md` (cross-cutting; this is the notes overlay).
