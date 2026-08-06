---
id: ADR-0002
status: Superseded
superseded_by: [ADR-0702]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0002: Establish the Tenant and Identity kernel as the single substrate every axis consumes

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `tenancy-identity`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0003, ADR-0007, ADR-0008, ADR-0009, ADR-0010

---

## Context

The cohesion thesis (ADR-0001) names *single tenancy* and *single identity* as two of the six shared substrates. Without a dedicated kernel that owns the `Tenant` shape and the identity primitives, every axis trends toward an microservice-local tenant struct + a vendor-shaped IdP adapter, and the cohesion moat erodes within months. The contradiction ledger LEDG-009 already records one such regression: client-supplied tenant IDs in the X-Tenant-ID header in healthcare services — exactly the failure mode an microservice-local tenancy model produces.

The kernel must serve all microservices simultaneously: workflow orchestration, connect communications, regulatory bindings, Foundry agent invocation, cloud control-plane mutation, search index segregation, and ads eligibility evaluation. Each consumer reads a different slice of the same Tenant entity — region, residency, regulatory packs, autonomy tier, data-use consent, billing account — and the slice must be authoritative or the cross-microservice contract drifts. Identity is harder still because it spans tenant-issued credentials, federated SSO via regional packs, agent-bound short-lived tokens, and customer-builder service principals; without a single STS-backed kernel, every axis ships a different credential surface.

---

## Decision

We establish two co-located kernel crates that together form the *tenant + identity substrate*:

- `crates/oya-tenancy-kernel` — owns the `Tenant`, `TenantId`, `TenantBinding`, and `TenantPlaneGrants` types.
- `crates/oya-identity-kernel` — owns the `Principal`, `Subject`, `Session`, `Credential`, `Role`, and `Capability-Grant` types, with Cedar-backed RBAC/ABAC and STS-issued short-lived credentials.

### The Tenant entity

```rust
// crates/oya-tenancy-kernel
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TenantId(pub uuid::Uuid);

pub struct Tenant {
    pub id: TenantId,
    pub region_binding: RegionBinding,         // immutable after first commit
    pub residency: ResidencyClass,              // strict_kr | strict_eu | global | ...
    pub regulatory_packs: BTreeSet<RegulatoryPackId>, // PIPA, HIPAA, MFDS, FSC, GDPR, ...
    pub plane_grants: TenantPlaneGrants,        // which planes this tenant may call
    pub autonomy_tier: AutonomyTier,            // T1..T4 ceiling for foundry agents
    pub data_use_consent: DataUseConsent,       // per ADR-0008 purpose-permission matrix
    pub billing_account: BillingAccountId,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub schema_version: u16,
}

pub struct RegionBinding {
    pub primary: RegionCode,                    // KR-Seoul1, JP-Tokyo, US-IAD, EU-FRA, ...
    pub failover: Option<RegionCode>,           // intra-residency-class only
}
```

- **Per-region binding is immutable.** A tenant's `region_binding.primary` is set at creation and may never be mutated by a non-migration path. Cross-region migration is a council-approved, evidence-emitting procedure with a new `TenantId` (the residency class is part of identity).
- **`regulatory_packs` is set-valued and inherited by every axis.** Vertical onboarding adds packs; revocation requires regulator-defined wind-down evidence.
- **Cross-axis change-review class.** Any PR that mutates the `Tenant` struct, its derived types, or the catalog record of `oya-tenancy-kernel` is auto-labeled `cross-microservice-tenant-mutation` and routed for all-axis review per ADR-0011.

### The Identity entity

```rust
// crates/oya-identity-kernel
pub enum Principal {
    Human { user_id: UserId, tenant: TenantId },
    Agent { agent_id: AgentId, tenant: TenantId, on_behalf_of: Option<UserId> },
    ServicePrincipal { sp_id: ServicePrincipalId, tenant: TenantId, owning_capability: CapabilityId },
    External { foreign_subject: ForeignSubjectClaim },
}

pub struct Session {
    pub principal: Principal,
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,    // STS short-lived credentials only
    pub scope: BTreeSet<CapabilityScope>,
    pub constraint: SessionConstraint,                // ip-bound, device-bound, geo-bound, ...
    pub mfa_status: MfaStatus,
}

pub struct AccessDecision {
    pub principal: Principal,
    pub action: ActionId,
    pub resource: ResourceId,
    pub context: AccessContext,                        // request-time attributes for ABAC
}
```

- **Cedar policy is the sole authoritative AuthZ engine** for every cross-microservice decision. Per-axis caches are read-through projections of Cedar's evaluation; they may not author decisions.
- **STS-issued short-lived credentials** are the only credential class accepted at the runtime boundary. Long-lived service-account static secrets are forbidden in product code (per the supply-chain requirements consumed by ADR-0013 license posture and ADR-0007 policy enforcement).
- **Federated identity providers ship as regional-pack seam impls** (ADR-0010): KR `본인확인서비스`, JP `マイナンバーカード`, EU `eIDAS`, US `Login.gov`, IN `Aadhaar`, BR `ICP-Brasil`, KSA `Absher`, UAE `UAEPass`, ANZ `Digital ID`, SG `SingPass`. Adding a region adds a pack, never a kernel patch.

### Validators and CI lanes

- `oya-governance-tenant-isolation` — fails PRs that derive tenancy from request headers or other untrusted inputs (closes LEDG-009).
- `oya-governance-iam-lockstep` — fails PRs where the cloud-IAM surface (ADR consumers in `oya-cloud-iam-*`) drifts from `oya-identity-kernel` (closes LEDG-028).
- `oya-governance-substrate-forking` — fails PRs that introduce a new `Tenant`-shaped struct anywhere outside the kernel.

### Boundary

- Applies to: every cross-microservice call, every regulated capability invocation, every regional-pack identity adapter, every audit emission cite (ADR-0003).
- Does not apply to: per-microservice read-only projections (e.g. a search-axis cache of `tenant.residency` is allowed if the projection is sourced from this kernel and refreshed via the eventing backbone in ADR-0005).

---

## Consequences

### Positive

- Closes LEDG-009 (X-Tenant-ID header derivation) at the substrate level.
- Every axis inherits region, residency, regulatory packs, autonomy tier, and consent atomically — no per-microservice interpretation drift.
- Cedar-backed RBAC/ABAC + STS short-lived creds aligns with KR PIPA Art 29 (security measures), GDPR Art 32, HIPAA §164.312, PCI-DSS Req 8.
- The `Tenant` struct becomes the single most-reviewed kernel in the repository, by design — the cross-microservice review tax is concentrated at the substrate boundary, where it is recoverable.

### Negative

- Tenant-mutation PRs become slower because the cross-microservice review label is mandatory.
- The substrate is a single point of architectural failure — a regression here cascades to all microservices. Mitigation: kernel ships with an exhaustive property-test set + a quarterly rotation of architecture-council review.
- Per-region IdP onboarding requires a regional-pack PR even for adjacent locales; the seam exists to prevent kernel patching, but it is real per-pack work.

### Operational

- On-call: tenant-kernel pages a P1 bridge whenever an `EVT-TENANT-KERNEL-INTEGRITY` event fires.
- CI: the three fitness lanes above run on every PR and emit per-PR evidence records.
- Runbooks: tenant onboarding, region-binding correction (new TenantId path), regulatory-pack add/remove, autonomy-tier uplift, consent-revocation cascade are each separately runbook-anchored under `docs/runbooks/tenant-*.md`.
- Audit: every Tenant mutation emits to the chain (ADR-0003) with the prior+next struct hashes; replayability of regulatory state at any prior `t` is guaranteed.

---

## Alternatives considered

### Alternative A — Per-axis tenant struct + cross-microservice adapter

- **Pros:** microservice-team autonomy; faster per-microservice iteration.
- **Cons:** drift on every axis at every wave; LEDG-009 demonstrated the failure mode in production.
- **Rejected because:** ADR-0001 forbids substrate forking.

### Alternative B — Vendor IdP as authoritative identity (e.g. Auth0, Okta)

- **Pros:** zero day-one identity build cost.
- **Cons:** sovereignty + KCminimum-shippable-tier/CSAP constraints in KR, GAIA-X in EU; vendor lock; per-region failover impossible without rebuilding the surface.
- **Rejected because:** the regulatory packs (esp. KR-pack, EU-pack) require kernel-level control over credential rotation, audit emission, and policy evaluation.

### Alternative C — Cedar but without STS — long-lived service credentials

- **Pros:** simpler agent/capability-binding flow.
- **Cons:** PCI-DSS, HIPAA, and PIPA all penalize long-lived static credentials in regulated paths; auditor evidence requires rotation anyway.
- **Rejected because:** any short-term simplification is paid back at the first regulatory audit.

---

## Open questions

1. **Q1.** Cross-tenant individual identity linking (`cross_tenant_individual` purpose per ADR-0008) — under what evidence threshold does the kernel allow it? Default: founder + privacy-council ratification per request, never automated. → owner: `council-privacy`.
2. **Q2.** Where does the per-tenant HSM partition (ADR-0009) bind to the identity kernel — at session issuance, or at credential decryption? Default: at session issuance. → ADR-0009.
3. **Q3.** Schema-version migration of the `Tenant` struct — what is the maximum supported lag between schema versions in production? Default: two versions; older tenants migrated within one wave. → owner: `tenancy-identity`.
4. **Q4.** Does the customer-builder persona (microservice ISVs) get a distinct `Principal` variant or reuse `ServicePrincipal`? Default: reuse, with `owning_capability` differentiating. → owner: `foundry`.

---

## References

- `docs/DESIGN.md` §5 (unifying tenancy model), §10 (cross-microservice contracts: `Tenant` kernel, `Identity / RBAC / Cedar policy`)
- `docs/PRIVACY-PROGRAM.md` §2.2.7 (KR data residency PIPA Art 17), §2.2.8 (agent-runtime specifics under autonomy ceiling)
- `docs/COMPLIANCE-MATRIX.md` §3.1 (KR PIPA Art 15/17/22/29), §3.2 (GDPR Art 5/6/32), §3.3 (HIPAA §164.312)
- `docs/CONTRADICTION-LEDGER.md` LEDG-009 (X-Tenant-ID header), LEDG-024 (KR identity coverage), LEDG-028 (cloud-IAM lockstep)
- ADR-0001 (cohesion thesis), ADR-0003 (audit chain), ADR-0007 (Cedar + persona tier), ADR-0009 (cell architecture), ADR-0010 (regional pack architecture)
