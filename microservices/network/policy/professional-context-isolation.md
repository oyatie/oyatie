---
doc_class: PolicySpec
title: Professional-Context Isolation Specification
microservice: network
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-security + axis-network
deciders: council-architecture, ops-security, axis-network, council-privacy
related_adrs: [ADR-0008, ADR-0028, ADR-0126, ADR-0131, ADR-0132, ADR-0140]
related_artifacts:
  - microservices/network/threat-model.md (T-I-07; Professional-context invariant violation)
  - microservices/network/dpia.md
  - microservices/network/policy/tenant-scope.cedar
  - microservices/network/policy/public-read.cedar
  - microservices/social/policy/dual-context-isolation.md (authoritative sibling pattern; cross-referenced)
review_cadence: quarterly + on every BC change
doc_status: published
---

# Professional-Context Isolation Specification (network µservice)

## Purpose

Define the load-bearing Professional-context invariants of the `network` substrate. Per parallel ADR-0126 (which inherits Bominal ADR-0208's dual-context model) and ADR-0132 (suite-and-bundle dissolution), the `network` µservice is **Professional-tier-only**. The `social` sibling owns Personal/General context. This document is the authoritative reference for SOC 2 examiners (CC6.1), ISO 27001 auditors (A.5.15, A.8.3), GDPR Art. 25 reviewers, KR PIPA Art. 28 reviewers, HIPAA OCR, EU DSA Coordinator, EU AI Act notified body, EEOC examiner, NYC DCWP LL144 auditor asking *"how does network keep Professional and Personal separated?"*

`network` inherits the dual-context-isolation pattern from the sibling `social` µservice (`microservices/social/policy/dual-context-isolation.md`) but adopts the **Professional-only specialisation**: the only valid `ContextKind` for `network` resources is `Professional`. Personal entities never appear in `network` at any layer.

## Context Kind Enumeration

```rust
// oya-network-professional-profile-kernel (sealed)
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NetworkContextKind {
    Professional,
}
// Note: only ONE variant. ProfessionalContextKind is enforced as a type-level singleton.
```

Properties:
- Enum is sealed at the kernel layer with exactly one variant.
- The kernel exposes ZERO methods that produce a Personal-tier value.
- Cross-context coercion from `social::ContextKind::Personal` to `NetworkContextKind::Professional` is rejected at compile-time because no such conversion exists.
- Runtime config CANNOT introduce a Personal entity into `network`; this is a compile-time + data-model invariant per parallel ADR-0126.

## Entity Type Invariants

### Invariant PCI-01: All `network` entity types are Professional-only

| Entity | Context | Backing crate |
|---|---|---|
| `ProfessionalProfile` | only `Professional` | `oya-network-professional-profile-kernel` |
| `ConnectionEdge` | only `Professional` | `oya-network-professional-graph-kernel` |
| `ProfessionalPost` | only `Professional` | `oya-network-post-composition-kernel` |
| `Endorsement` | only `Professional` | `oya-network-endorsement-engine-kernel` |
| `Recommendation` | only `Professional` | `oya-network-endorsement-engine-kernel` |
| `InMail` | only `Professional` | `oya-network-inmail-bridge-kernel` |
| `JobPosting` | only `Professional` | `oya-network-jobs-handoff-kernel` |
| `RecruiterSearchRequest` | only `Professional` | `oya-network-recruiter-stub-kernel` |
| (all other entities) | only `Professional` | (all other `oya-network-*-kernel` crates) |

There is **no shared trait** that bridges to Personal-tier types in `social`. Even if both have a `Post` concept, they are distinct types in distinct kernels with distinct port traits. LEAN check `oya-check-professional-context-isolation` asserts there are no trait impls covering both.

### Invariant PCI-02: No Personal-tier write path

The `PostStore::publish(post: ProfessionalPost)` port trait accepts only `ProfessionalPost`. No `publish_personal` method exists on any `network` repository port. The domain layer rejects:

- Construction of any `network` entity from a `social::PersonalProfile` reference.
- Connection-edge insertion where one side is a `social::PersonalProfile` reference.

The LEAN check inspects every `*Store` impl in `network` and asserts that no signature accepts a Personal-tier type from `social`.

### Invariant PCI-03: Distinct key material

- Professional posts + InMail bodies: tenant-DEK envelope encryption per Bominal ADR-0111; oyatie can decrypt under four-eyes audit.
- Endorsement signatures: per-endorser Ed25519 keypair stored in KMS; public key publishable; private key never exfiltrates the KMS boundary (ADR-NET-0005).
- Recruiter-stub model inputs: tenant-DEK encrypted at rest; surfaces in foundry-runtime classifier under audit-chain seal.

Key-material types are sealed:

```rust
pub struct TenantDek(/* OpenBao-bound; never serialised to logs */);
pub struct EndorserEd25519PrivateKey(/* KMS-bound; never leaves KMS */);
pub struct EndorserEd25519PublicKey(/* publishable; carried in audit-chain seal */);
```

The compiler refuses to coerce one into the other.

### Invariant PCI-04: Pack-aware retention paths

Per `policy/data-residency.md`:

- All `network` data has Professional-tier retention floors per pack (KR PIPA Art. 21 + 근로기준법 work-record 3y for KR; HIPAA 6y for pack-us-healthcare when health-context profile surfaces; etc.).
- The retention worker reads `tenant_id` of every row + pack-overlay; routing function takes typed `Professional` arm only.

### Invariant PCI-05: Distinct audit paths

- Every state transition (profile create, profile update, connection-request, connection-accept, post-create, endorsement-add, endorsement-revoke, recommendation-publish, inmail-send, inmail-disclosure, hold, four-eyes-execute, recruiter-search-invocation, jobs-handoff-event, page-newsletter-send) emits to audit-chain with `context_kind: Professional` label.
- Bias-audit + EU AI Act Art. 15/72 monitoring events emit on a separate `oya.network.audit.recruiter_bias` topic; ingested by `dashboards/recommender-fairness-and-bias.json`.

### Invariant PCI-06: Distinct Cedar evaluator branches

Cedar policy fragments split per-action:

- `policy/tenant-scope.cedar` PERMIT applies only when `resource.context_kind == "Professional"`.
- `policy/auditor-scope.cedar` PERMIT 4 (four-eyes disclosure) applies only when `resource.context_kind == "Professional"`.
- `policy/public-read.cedar` allows anonymous read of public-visibility Professional posts + public Professional profiles (when user opts in).
- `policy/public-read.cedar` FORBID asserts that no Personal-tier resource type can match `network` Cedar action set.

### Invariant PCI-07: No runtime config-toggle from Personal to Professional

A `social::PersonalProfile` is **never** migrated into a `network::ProfessionalProfile`. If a user wishes to maintain both, they create distinct Personal profile in `social` and Professional profile in `network`. There is no "switch context" API, no admin tool, no migration path.

### Invariant PCI-08: `network` NEVER federates in P01

Critical for federation safety. Per the absence of a `federation-gateway` in `network`:

- No outbox port trait exists in `network`. The Helm chart contains no `federation-gateway` deployment.
- LEAN lane `oya-check-professional-context-isolation` validates that no federation egress port exists at compile time.
- If federation is added in a follow-up ADR-NET, the new outbox trait will accept only `ProfessionalPost` and federate only to Professional-tier peers (Personal-tier federation will remain compile-time-impossible because Personal entities never exist in `network`'s type system).

### Invariant PCI-09: `network` never bridges to Personal-tier messenger DM

Per ADR-NET-0003 (InMail bridge):

- The `InMailBridge::send(inmail: InMail)` port trait accepts only `ProfessionalInMail`.
- The bridge routes to the `messenger` µservice's Professional-tier surface only; Personal-tier DM is owned by a distinct `messenger` channel and the bridge cannot reach it.
- Runtime guard belt-and-suspenders: bridge worker checks the messenger response carries `context_kind=Professional` and emits Sev-1 if violated.

### Invariant PCI-10: Recruiter-stub never operates outside tenant scope

Per ADR-NET-0002:

- `RecruiterSearchRequest::tenant_id` is mandatory and rejected at the type level if absent.
- Cedar `tenant-scope.cedar` PERMIT 8 requires `request.tenant_id == principal.tenant_id`; cross-tenant recruiter-search is forbidden.
- The recruiter-stub ranker (when activated) operates only over the tenant's tenant-scoped candidate pool; never across tenants.

## CI-Lane Enforcement

### Lane: `oya-check-professional-context-isolation`

Located at `crates/oya-check-professional-context-isolation/` (extended from sibling `oya-check-dual-context-isolation` pattern). Asserts:

1. `NetworkContextKind` enum is sealed at exactly one variant (`Professional`).
2. All `network` entity types are bound to `Professional` at the type level.
3. No port trait accepts a `social::PersonalProfile` reference or any Personal-tier type.
4. No `from_personal()` / `into_personal()` conversion methods exist between `network` and `social` kernels.
5. No federation outbox port exists in `network` kernel (P01 enforces "no federation").
6. InMail-bridge port accepts only `ProfessionalInMail`.
7. Recruiter-stub request carries mandatory `tenant_id`; never cross-tenant.
8. Postgres migrations carry `context_kind` column with CHECK constraint `context_kind = 'Professional'`.
9. Cedar evaluator branches are exhaustive on Professional-only.
10. Endorsement-engine signing requires per-endorser Ed25519 key bound at the type level; no shared-key code path.

Severity: BLOCKER. Lane is required on `dev` and `staging` per branch-protection.

### Lane: `oya-check-eu-ai-act-employment-conformance`

Asserts EU AI Act Annex III §4 compliance gates:

1. Recruiter-stub OFF by default (Helm values + Cedar entitlement gated).
2. Per-tenant activation of recruiter-stub requires evidence of FRIA (Fundamental Rights Impact Assessment per Art. 27).
3. Per-release model card sealed for recruiter-stub + jobs-ranker + people-you-may-know + endorsement-aggregation.
4. 4/5-rule statistical disparity ratio recorded per release.
5. NYC LL144 annual bias-audit timestamp within rolling 12mo window when NYC tenant active.
6. CA AB-331 + CO SB 24-205 risk-management policy attached when CA / CO tenant active.

### Lane: `oya-check-endorsement-chain-integrity`

Asserts ADR-NET-0005 endorsement-chain integrity gates:

1. Every endorsement record carries a per-endorser Ed25519 signature.
2. Merkle root over a tenant's endorsement-chain is sealed to audit-chain.
3. KMS audit log shows no unauthorised key access.
4. Quarterly drill replay produces matching Merkle root.

## Runtime Enforcement

In addition to compile-time + LEAN-lane enforcement, runtime guards:

- WebSocket gateway tags every connection as `context: Professional`; the entity-resolution layer refuses any inbound payload claiming Personal context (with audit-chain seal of attempt + Sev-2 alert).
- Postgres rows carry `context_kind` as non-nullable column with CHECK constraint `context_kind = 'Professional'`; cross-context join queries against `social` are rejected at DB layer (DB schemas are in separate Postgres clusters per ADR-0131 per-µservice flat layout).
- Search indexes are partitioned per tenant; no cross-context cross-µservice index ever exists.
- InMail-bridge worker has runtime assertion `messenger_response.context_kind == Professional`; violation emits Sev-1.
- Recruiter-stub worker asserts request.tenant_id == principal.tenant_id; violation emits Sev-1.

## Operational Procedures

- A `ProfessionalContextViolation` Prometheus metric is emitted per attempted violation; alert at > 0 over 5min.
- A `RecruiterCrossTenantAttempt` Prometheus metric is emitted per cross-tenant recruiter-search attempt; alert at > 0 over 1min.
- An `EndorsementChainIntegrityFailure` Prometheus metric is emitted per signature-verification failure; alert at any failure.
- A Sev-1 incident is declared on any confirmed cross-context routing (per `incident-response.md` FM-10), recruiter-stub bias-audit failure (FM-15), or endorsement-chain integrity compromise (FM-14).
- Periodic chaos test injects a synthetic Personal-tier-payload-to-network attempt + cross-tenant recruiter-search attempt; verifies rejection + alert.

## Verification

- Unit tests: every `*Store` impl in `network` has a UI test that fails to compile on Personal-tier input.
- Integration tests: synthetic `social::PersonalPost`-→-`network::feed-timeline` routing attempt returns 403 + emits metric + writes audit-chain record.
- Integration tests: synthetic recruiter cross-tenant attempt rejected at Cedar layer.
- Integration tests: endorsement-chain forgery attempt rejected at signature-verification + Merkle-root layer.
- Pen-test: annual external red-team attempt to break the invariants via API misuse.

## References

- Parallel ADR-0126.
- ADR-0132 (suite-and-bundle dissolution; suite-removal makes Professional-only µservice canonical).
- Bominal ADR-0208 (Connect dual-context unified channel hub; inherited).
- Bominal ADR-0215 (Connect retention legal-hold dual-context; inherited).
- ADR-0008 Data Use Boundary.
- ADR-NET-0002 (recruiter-stub EU AI Act + EEOC bounds).
- ADR-NET-0003 (InMail bridge to messenger).
- ADR-NET-0005 (endorsement-chain integrity).
- `microservices/social/policy/dual-context-isolation.md` (authoritative sibling pattern; cross-referenced).
- `microservices/network/threat-model.md` §T-I-07.
- `microservices/network/dpia.md`.
- `microservices/network/policy/tenant-scope.cedar`.
- `microservices/network/policy/public-read.cedar`.
- `docs/standards/dual-context-isolation.md` (cross-cutting; network is a Professional-only specialisation).
