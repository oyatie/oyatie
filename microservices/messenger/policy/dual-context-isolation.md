---
doc_class: PolicySpec
title: Dual-Context Isolation Specification
microservice: messenger
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-security + axis-messenger
deciders: council-architecture, ops-security, axis-messenger, council-privacy
related_adrs: [ADR-0008, ADR-0028, ADR-0135, ADR-0131, ADR-0132, ADR-0140]
related_artifacts:
  - microservices/messenger/threat-model.md (T-I-07; cross-context invariant violation)
  - microservices/messenger/dpia.md (R-07)
  - microservices/messenger/policy/channel-scope.cedar
  - microservices/messenger/policy/tenant-scope.cedar
review_cadence: quarterly + on every BC change
doc_status: published
---

# Dual-Context Isolation Specification (messenger µservice)

## Purpose

Define the load-bearing dual-context invariants of the messenger substrate. Per parallel ADR-0135 (which inherits Bominal ADR-0208's dual-context model), every messenger entity carries a `context_kind: { Personal | Professional }` discriminator that determines:

- which keys encrypt the body (personal E2E vs tenant-DEK);
- which retention floor applies (personal user-policy vs professional pack-floor);
- which disclosure path is reachable (personal: never to admin; professional: four-eyes only);
- which audit-chain seal stream the event lands on.

This document is the authoritative reference for SOC 2 examiners (CC6.1), ISO 27001 auditors (A.5.15, A.8.3), GDPR Art. 25 reviewers, KR PIPA Art. 28 reviewers, and HIPAA OCR asking *"how does messenger keep personal and professional separated?"*

## Context Kind Enumeration

```rust
// oya-messenger-channel-store-kernel (sealed)
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContextKind {
    Personal,
    Professional,
}
```

Properties:
- Enum is sealed at the kernel layer; only two variants ever exist.
- Cross-variant write is rejected at the domain layer.
- Runtime config CANNOT switch a Personal entity into Professional (or vice versa); this is a compile-time + data-model invariant per parallel ADR-0135.

## Entity Type Invariants

### Invariant DCI-01: Distinct entity types

The messenger BC introduces **two distinct top-level entities** for direct interaction:

| Entity | Context | Backing crate |
|---|---|---|
| `DirectConversation` | only `Personal` | `oya-messenger-channel-store-kernel` |
| `Channel` | always `Professional` (DMs in professional context are still `Channel` with member-count == 2 + `kind = DM`) | `oya-messenger-channel-store-kernel` |

There is **no shared trait** that allows generic code to construct either; consumers must match on the entity type. LEAN check `oya-check-dual-context-isolation` asserts there are no trait impls covering both.

### Invariant DCI-02: No cross-type write path

The `MessageStore::append(msg: Message)` port trait requires the `Message` carry the `context_kind` of its parent. The domain layer rejects:

- A `Message{context_kind = Personal}` written against a `Channel` (which is Professional by construction).
- A `Message{context_kind = Professional}` written against a `DirectConversation` (which is Personal by construction).

The LEAN check inspects every `MessageStore` impl + every `MessageRepository` impl + every `RealtimeBroadcaster` impl and asserts that the type signature forbids cross-type flows.

### Invariant DCI-03: Distinct key material

- Personal: each `DirectConversation` body is **E2E-encrypted with client-derived keys**; oyatie never sees plaintext. Server stores `body_ciphertext`. The `body_client_only: ()` zero-byte marker on `Message` forbids ever materialising plaintext server-side.
- Professional: each `Channel` body is **tenant-DEK encrypted** (envelope encryption per Bominal ADR-0111); oyatie can decrypt under four-eyes audit. Server stores `body_ciphertext` and the wrapped DEK.

Key-material types are sealed:

```rust
pub struct PersonalE2EKey(/* zero-byte marker; client-held only */);
pub struct TenantDek(/* OpenBao-bound; never serialised to logs */);
```

The compiler refuses to coerce one into the other.

### Invariant DCI-04: Distinct retention paths

Per `policy/data-residency.md`:

- Personal: retention follows the user's per-user policy (default = no retention floor; tenant cannot override).
- Professional: retention follows the tenant's pack-aware policy (KR-PIPA floor for KR; HIPAA 6y floor for pack-us-healthcare; etc.).

The retention worker reads `context_kind` of every row and routes to the corresponding policy engine. A misroute is a compile-time error in the worker code because the routing function takes typed `Personal | Professional` arms.

### Invariant DCI-05: Distinct audit paths

- Personal: only `PersonalDmAdminDecryptAttempt` events (which MUST be zero in normal operations) emit to audit-chain; routine personal operations do NOT emit (per ePrivacy + privacy-by-design).
- Professional: every state transition (create, post, edit, delete, member-grant, hold, disclosure) emits to audit-chain.

### Invariant DCI-06: Distinct Cedar evaluator

Cedar policy fragments are split:

- `policy/channel-scope.cedar` — Professional channel evaluation.
- `policy/personal-dm-scope.cedar` — Personal DM evaluation.

Both fragments load into the evaluator but operate on disjoint principal/resource types (`Channel` vs `DirectConversation`). A Cedar query against the wrong entity type returns deny.

### Invariant DCI-07: No runtime config-toggle for context

The `ContextKind` is set at entity-creation time and is **immutable**. There is no API, no admin tool, no migration path to convert a Personal entity into Professional or vice versa. The user-facing surface offers "create a new Professional channel and invite the same people" instead — explicit migration with consent.

## CI-Lane Enforcement

### Lane: `oya-check-dual-context-isolation`

Located at `crates/oya-check-dual-context-isolation/`. Asserts:

1. `ContextKind` enum is sealed at exactly two variants.
2. `DirectConversation` and `Channel` are distinct types with no shared inherent impl.
3. Every `MessageStore::append` impl rejects cross-type writes (verified via UI test on attempt to mix types — compile must fail).
4. Personal-DM key material types cannot be coerced to professional DEK types.
5. Retention worker routes via exhaustive `match` on `ContextKind`.
6. Audit-chain client distinguishes personal vs professional event streams.
7. No `mut context_kind` field exists anywhere; entity creation is the only setter.
8. No `into_professional()` / `into_personal()` conversion methods exist.

Severity: BLOCKER. Lane is required on `dev` and `staging` per branch-protection.

## Runtime Enforcement

In addition to compile-time + LEAN-lane enforcement, runtime guards:

- WebSocket gateway tags every connection with the active persona (`Personal` or `Professional`); the entity-resolution layer refuses requests where the persona doesn't match the resource's `context_kind`.
- Postgres rows carry `context_kind` as a non-nullable column with a CHECK constraint; cross-context join queries are rejected at DB layer.
- Search index is partitioned per `context_kind`; cross-partition queries are not supported by the search API.

## Operational Procedures

- A `ContextSwitchDeniedAttempt` Prometheus metric is emitted per attempted violation; alert at > 0 over 5min.
- A Sev-1 incident is declared on any confirmed cross-context routing (per `incident-response.md`).
- Periodic chaos test injects a synthetic cross-type write attempt; verifies rejection + alert.

## Verification

- Unit tests: every `MessageStore` impl has a UI test that fails to compile on cross-type write.
- Integration tests: synthetic personal-DM → professional-channel routing attempt returns 403 + emits metric + writes audit-chain record.
- Pen-test: annual external red-team attempt to break the invariant via API misuse.

## References

- Parallel ADR-0135.
- Bominal ADR-0208 (Connect dual-context unified channel hub; inherited).
- Bominal ADR-0215 (Connect retention legal-hold dual-context; inherited).
- ADR-0008 Data Use Boundary.
- `microservices/messenger/threat-model.md` §T-I-07.
- `microservices/messenger/dpia.md` §R-07.
- `microservices/messenger/policy/channel-scope.cedar`.
- `microservices/messenger/policy/personal-dm-scope.cedar`.
- `docs/standards/dual-context-isolation.md` (cross-cutting; this is the messenger overlay).
