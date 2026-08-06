---
id: ADR-702
title: "Live identity, tenancy, authz, secrets, and control-plane fail-closed posture"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-08-06
door: two-way
owner: council-architecture
supersedes: [ADR-2, ADR-7, ADR-43, ADR-95, ADR-155, ADR-163, ADR-191, ADR-214, ADR-242, ADR-244, ADR-294, ADR-311, ADR-326, ADR-329, ADR-330, ADR-543, ADR-553, ADR-572, ADR-573, ADR-589, ADR-592, ADR-593, ADR-603, ADR-607]
superseded_by: []
amends: []
amended_by: []
depends_on: [ADR-515, ADR-363, ADR-562]
related: []
milestone: W0
---
# ADR-702: Live identity, tenancy, authz, secrets, and control-plane fail-closed posture

## Status

**Accepted** — live consolidated source-of-truth entry for topic `identity_authz` (E5 2026-08-06).

## Context

Oyatie ADR corpus cleanup: agents must not treat every historical Accepted file as equal live law.
This apex consolidates **24** Accepted ADRs in the `identity_authz` topic. Member files are
**Superseded** by this apex and then archived; full text remains in git history.

Live resolution: prefer this apex; follow `supersedes` for provenance.

## Decision

1. **This ADR is the live reading entry** for topic `identity_authz` under the end-state ADR policy.
2. **Member ADRs listed in `supersedes`** are historical; normative gist is preserved below.
3. **Contradictions** among members are resolved by later higher-number members and by
   ADR-0515 / ADR-0363 / ADR-0562 / ADR-0615 / ADR-0635 / ADR-0637–0639 when applicable.
4. **Activation-sensitive** items (warm CAS, RE workers) remain fail-closed until explicit go-gate.

## Preserved member gists

- **ADR-2** (ADR-0002-tenant-and-identity-kernel): We establish two co-located kernel crates that together form the *tenant + identity substrate*: - `crates/oya-tenancy-kernel` — owns the `Tenant`, `TenantId`, `TenantBinding`, and `TenantPlaneGrants` types. - `crates/oya-identity-kernel` — owns the `Principal`, `Subject`, `Session`, `Credential`, `Role`, and `Capability-Grant` types, with Cedar-bac
- **ADR-7** (ADR-0007-cedar-authorization-policy-and-persona-tier): We adopt **Cedar** as the sole authorization policy engine for RBAC/ABAC across all axes, **persona tiers T1–T4** as the autonomy-ceiling scale, and **per-capability runtime enforcement** that consults both Cedar and the autonomy ceiling on every invocation. ### Cedar surface - Engine: Cedar (Apache-2.0; in-house Rust binding under `crates/oya-poli
- **ADR-43** (ADR-0043-secrets-management-openbao-and-hsm-per-cell): We adopt **OpenBao** (MPL-2) as the canonical secrets store; **per-tenant per-cell HSM partition** with **KCminimum-shippable-tier for KR cells + FIPS 140-3 globally**; a **rotating session-token vault** for Foundry subscription-mode adapters; a **per-capability `SecretProvider` trait** so axes never read raw secrets; **quarterly key-rotation drill
- **ADR-95** (ADR-0095-tenant-slug-in-tenancy-kernel): Add a SECOND newtype to `oya-tenancy-kernel`: ```rust pub const TENANT_SLUG_MAX_LEN: usize = 128; pub struct TenantSlug(String); impl TenantSlug { pub fn try_new(value: impl Into<String>) -> Result<Self, TenantKernelError> { /* … */ } pub fn as_str(&self) -> &str { /* … */ } pub fn into_inner(self) -> String { /* … */ } } impl TryFrom<&str> for Ten
- **ADR-155** (ADR-0155-per-tenant-resource-quotas): Adopt per-tenant quotas on five canonical axes (rate, concurrent, memory, storage, connections) as MANDATORY across every µservice. 1. The canonical spec is `docs/standards/per-tenant-resource-quotas-canonical.md`. 2. The trait surface lives in `crates/oya-shared-tenant-quota-kernel/`. 3. The tenancy µservice OWNS canonical quota definitions; runti
- **ADR-163** (ADR-0163-tenant-environment-tiers): Every oyatie tenant has three environment tiers. Each tier is a logically isolated dataset within the tenant's cell: ### Tier definitions - **`test`** — sandbox environment. Tenant integrations land here first. Data is ephemeral (90-day TTL default; per-pack overlay). Outbound side effects (email send, SMS send, webhook dispatch, billing event) are
- **ADR-191** (ADR-0191-edge-authz-tier-vs-origin-cedar-pdp): **Two tiers; one concern per tier. Neither tier reimplements the other.** ### Edge tier — Envoy Gateway (ADR-0182, ADR-0157) Owned by `api-gateway` µservice. Enforces: | Concern | Mechanism | Source of truth | Failure response | |---|---|---|---| | IP block (geo deny-list) | MaxMind GeoIP + per-pack policy | residency policy + abuse ledger | 403 wi
- **ADR-214** (ADR-0214-cross-tenant-real-time-visibility): # ADR-0214: Cross-Tenant Real-Time Visibility (Consent-Graph + Ontology Projection Extension)
- **ADR-242** (ADR-0242-oyatie-is-a-tenant-doctrine): ### D-1. `oyatie` is the canonical org-tenant slug The tenant ID `oyatie` (exactly that spelling, lowercase, ASCII) is **reserved at platform genesis** for the org operating the platform. Justification for the literal slug: - Matches AWS's `aws` IAM principal pattern (AWS uses `aws` for its internal control-plane principals; AWS docs refer to this 
- **ADR-244** (ADR-0244-tenant-as-universal-scoping-primitive): The platform adopts **tenant ID + dotted hierarchical sub-scope** as the universal scoping primitive. The following twelve decisions are locked. ### D-1. Tenant ID format Every tenant has a globally unique slug ID conforming to: - **Character set.** Lowercase ASCII letters `a-z`, digits `0-9`, hyphen `-`, dot `.`. - **Anchored regex.** `^[a-z][a-z0
- **ADR-294** (ADR-0294-cedar-fragment-soak-anomaly-rollback): The keystone establishes nine decision sub-sections, D-1 through D-9. ### D-1. Fragment lifecycle gains a `Soaking` stage ADR-0243 §D-2 currently defines five fragment lifecycle stages: `Authored → Reviewed → Signed → Published → Activated → Audited`. ADR-0294 inserts a sixth stage **between Published and Activated**: ``` Authored │ ▼ Reviewed (mul
- **ADR-311** (ADR-0311-dual-tenant-identity-personal-vs-work-boundary): Bundled with the keystone-bundle 2026-05-20 foundational doctrine synthesis as the **dual-tenant-identity-personal-vs-work-boundary** ADR, surfaced by the Wave-3-E ecosystem journey catalog (j126-j150). The catalog introduced four new persona archetypes (Inspector Diana Reyes, Priya Krishnan, Sam Okafor, Chris Volkov) and 25 journeys whose load-bea
- **ADR-326** (ADR-0326-per-tenant-data-residency-attestation): Residency is a first-class tenant attribute with four named tiers: - **R-1 `multi_region`** — tenant accepts the global default cell topology; data may move freely across regions; no cross-border bar. - **R-2 `single_region`** — tenant's data must stay within a named region (e.g. `region: eu_west`, `region: kr_central`). The cell-placement enforcer
- **ADR-329** (Tier system retired; replaced by tenant-class model): ### B.1 Decision statement The Bronze/Silver/Gold/Platinum capability-tier doctrine codified by ADR-0316 is retired in full. The capability-tier-grant primitive, the per-microservice `capability-tiers/tier-matrix.md` artifact, the centralised `registry/capability-tiers/` directory, the `capability-tier-deltas-vs-counterparts-*.md` audit deliverable
- **ADR-330** (Tenant Class — demo_trial vs paid with Composable Billing Components): The decision is recorded as a numbered set of normative clauses. Every clause is a load-bearing commitment; downstream microservice work, governance lanes, and CI checks bind to clause numbers. Numbering is immutable once accepted. ### B.1 The tenant_class enum 1. **B.1.1** The tenant_class field on every oyatie tenant principal is a closed enum wi
- **ADR-543** (Commission the cloud-kms K8s operator (G002 slice 2)): Ship the cloud-kms operator as three single-concern crates plus GitOps surfaces: - `oya-cloud-kms-operator-kernel` — pure reconciler kernel (typed desired-state for KeyRings, SealingRoots, KeyVersion rotation; `reconcile(observed, desired) -> Vec<Action>`; injected clock; ZERO kube dependencies — the cutover-stable seam). - `oya-cloud-kms-operator-
- **ADR-553** (Commission the oya-identity runnable workload-identity service (G005 slice 1)): Promote `oya-identity` to a runnable workload-identity service binary that composes the existing workload-identity crates (domain, usecase, Cedar adapter, OIDC validation adapter, REST/gRPC delivery) behind one boot path: - `iam/facade/identity-service/src/server.rs` — the single composition root used by both `main` and the E2E suite: fail-fast con
- **ADR-572** (Fail-closed authz for the Cedar policy publish control plane (AUTH-005 remediati): Make the publish surface fail-closed, with the authorization decision modelled as **ports** owned by the boundary crate (clean architecture per ADR-0131; ports model the owned W5 destination so they do not change at cutover; the concrete cloud-iam PDP client + credential store are **adapters** that live outside this crate). The new source file `iam
- **ADR-573** (Fail-closed authz for the Cloud KMS crypto control plane (AUTH-005 / C5 remediat): Make the Cloud KMS crypto surfaces fail-closed, with the authorization decision modelled as **ports** owned by the boundary crate (clean architecture per ADR-0131; ports model the owned W5 destination so they do not change at cutover; the concrete cloud-iam PDP client + credential store are **adapters** that live outside this crate). The new source
- **ADR-589** (Fail-closed authz for the DSR erasure cascade (AUTH-005 / Wave-2b remediation)): The erasure cascade is UNREACHABLE without (1) a verified principal and (2) a passing server-side PDP decision. The caller-supplied `allowed_surfaces` field is removed entirely. 1. **Unforgeable verified principal.** A new `compliance/ports/dsr-usecase/src/authz.rs` introduces `VerifiedDsrPrincipal` (private fields, `pub(crate)` constructor, public
- **ADR-592** (Tenant-scoped, body-fingerprinted accounting idempotency keys (cross-tenant coll): 1. **Tenant-scope every accounting idempotency key, tenant-id first.** Introduce a single-sourced builder `scoped_idempotency_key(tenant_id, scope, primary_ref)` in the core crate that emits the *logical* key `idem-v2:<tenant_id>:<scope>:<primary_ref>`. The tenant id is the leading keyed component, so two tenants can never collide on a shared calle
- **ADR-593** (Fail-closed authz for the Accounting + Payroll money-mutation control planes (AU): Wire a **fail-closed, verified-principal + cloud-iam-PDP** authz seam onto the money-mutation routes of both crates, mirroring the proven doctrine that landed for the Cloud KMS crypto control plane (ADR-0573), the Cedar policy publish control plane (ADR-0572), `intelligence/adapters/rest` (`constant_time_eq` bearer compare + a PDP `decide` port), t
- **ADR-603** (Fail-closed authz for the CRM revenue control plane (AUTH-005 remediation)): Install the established unforgeable-authz seam (mirroring ADR-0572 / #815 and the `intelligence/adapters/rest` doctrine) in a new `src/authz.rs` owned by this crate: 1. **Unforgeable verified identity.** A `VerifiedPrincipal { principal_id, tenant_id }` with **private fields**, a `pub(crate)` constructor, and a `cfg(test)` constructor only. Externa
- **ADR-607** (Fail-closed Cedar authz on the managed-K8s control-plane facades (cluster-lifecy): Make all three facades fail-closed against a SERVER-VERIFIED principal and a consulted Cedar PDP, mirroring the merged `tenant-quota-adapter-cedar` (ADR-0243, Cedar as the universal gate) and the clean-arch ports/adapters layering (ADR-0131): - A `VerifiedCaller` is bound from a constant-time bearer check; the `x-oya-tenant-id` header compare is de

## Consequences

- Agent default read path: `docs/decisions/ADR-0xxx` apex files + this topic.
- Citations to member numbers remain valid via `docs/decisions/_disposition/adr-redirect.v1.json`.
- Further body merge refinements may land as amendments to this apex only.

### ADR-294 residual

**ADR-0294-cedar-fragment-soak-anomaly-rollback** — The keystone establishes nine decision sub-sections, D-1 through D-9. ### D-1. Fragment lifecycle gains a `Soaking` stage ADR-0243 §D-2 currently defines five fragment lifecycle stages: `Authored → Reviewed → Signed → Published → Activated → Audited`. ADR-0294 inserts a sixth stage **between Published and Activated**: ``` Authored │ ▼ Reviewed (multispectrum-review v2.4.0 per ADR-0243 §D-2) │ ▼ Si

### ADR-214 residual

**ADR-0214-cross-tenant-real-time-visibility** — # ADR-0214: Cross-Tenant Real-Time Visibility (Consent-Graph + Ontology Projection Extension)

### ADR-43 residual

**ADR-0043-secrets-management-openbao-and-hsm-per-cell** — We adopt **OpenBao** (MPL-2) as the canonical secrets store; **per-tenant per-cell HSM partition** with **KCminimum-shippable-tier for KR cells + FIPS 140-3 globally**; a **rotating session-token vault** for Foundry subscription-mode adapters; a **per-capability `SecretProvider` trait** so axes never read raw secrets; **quarterly key-rotation drill** per cell; an **emergency rotation runbook** for

### ADR-607 residual

**Fail-closed Cedar authz on the managed-K8s control-plane facades (cluster-lifecycle / control-plane-host / tenant-quota)** — Make all three facades fail-closed against a SERVER-VERIFIED principal and a consulted Cedar PDP, mirroring the merged `tenant-quota-adapter-cedar` (ADR-0243, Cedar as the universal gate) and the clean-arch ports/adapters layering (ADR-0131): - A `VerifiedCaller` is bound from a constant-time bearer check; the `x-oya-tenant-id` header compare is deleted — identity is never caller-asserted. authn r

### ADR-553 residual

**Commission the oya-identity runnable workload-identity service (G005 slice 1)** — Promote `oya-identity` to a runnable workload-identity service binary that composes the existing workload-identity crates (domain, usecase, Cedar adapter, OIDC validation adapter, REST/gRPC delivery) behind one boot path: - `iam/facade/identity-service/src/server.rs` — the single composition root used by both `main` and the E2E suite: fail-fast config -> JWKS/Cedar/seed load -> independently bound

### ADR-329 residual

**Tier system retired; replaced by tenant-class model** — ### B.1 Decision statement The Bronze/Silver/Gold/Platinum capability-tier doctrine codified by ADR-0316 is retired in full. The capability-tier-grant primitive, the per-microservice `capability-tiers/tier-matrix.md` artifact, the centralised `registry/capability-tiers/` directory, the `capability-tier-deltas-vs-counterparts-*.md` audit deliverable, the N-014 and N-015 naming forms that suffix cap

### ADR-592 residual

**Tenant-scoped, body-fingerprinted accounting idempotency keys (cross-tenant collision fix)** — 1. **Tenant-scope every accounting idempotency key, tenant-id first.** Introduce a single-sourced builder `scoped_idempotency_key(tenant_id, scope, primary_ref)` in the core crate that emits the *logical* key `idem-v2:<tenant_id>:<scope>:<primary_ref>`. The tenant id is the leading keyed component, so two tenants can never collide on a shared caller-chosen `primary_ref`. All three builders (journa

### ADR-2 residual

**ADR-0002-tenant-and-identity-kernel** — We establish two co-located kernel crates that together form the *tenant + identity substrate*: - `crates/oya-tenancy-kernel` — owns the `Tenant`, `TenantId`, `TenantBinding`, and `TenantPlaneGrants` types. - `crates/oya-identity-kernel` — owns the `Principal`, `Subject`, `Session`, `Credential`, `Role`, and `Capability-Grant` types, with Cedar-backed RBAC/ABAC and STS-issued short-lived credentia

### ADR-572 residual

**Fail-closed authz for the Cedar policy publish control plane (AUTH-005 remediation)** — Make the publish surface fail-closed, with the authorization decision modelled as **ports** owned by the boundary crate (clean architecture per ADR-0131; ports model the owned W5 destination so they do not change at cutover; the concrete cloud-iam PDP client + credential store are **adapters** that live outside this crate). The new source file `iam/ports/policy-cedar-api/src/authz.rs` defines the

### ADR-543 residual

**Commission the cloud-kms K8s operator (G002 slice 2)** — Ship the cloud-kms operator as three single-concern crates plus GitOps surfaces: - `oya-cloud-kms-operator-kernel` — pure reconciler kernel (typed desired-state for KeyRings, SealingRoots, KeyVersion rotation; `reconcile(observed, desired) -> Vec<Action>`; injected clock; ZERO kube dependencies — the cutover-stable seam). - `oya-cloud-kms-operator-k8s-adapter` — ADR-0510 transient adapter: kube-rs

### ADR-326 residual

**ADR-0326-per-tenant-data-residency-attestation** — Residency is a first-class tenant attribute with four named tiers: - **R-1 `multi_region`** — tenant accepts the global default cell topology; data may move freely across regions; no cross-border bar. - **R-2 `single_region`** — tenant's data must stay within a named region (e.g. `region: eu_west`, `region: kr_central`). The cell-placement enforcer admits the tenant only to cells in the named regi

### ADR-242 residual

**ADR-0242-oyatie-is-a-tenant-doctrine** — ### D-1. `oyatie` is the canonical org-tenant slug The tenant ID `oyatie` (exactly that spelling, lowercase, ASCII) is **reserved at platform genesis** for the org operating the platform. Justification for the literal slug: - Matches AWS's `aws` IAM principal pattern (AWS uses `aws` for its internal control-plane principals; AWS docs refer to this in `arn:aws:iam::aws:` patterns for AWS-owned mana

### ADR-95 residual

**ADR-0095-tenant-slug-in-tenancy-kernel** — Add a SECOND newtype to `oya-tenancy-kernel`: ```rust pub const TENANT_SLUG_MAX_LEN: usize = 128; pub struct TenantSlug(String); impl TenantSlug { pub fn try_new(value: impl Into<String>) -> Result<Self, TenantKernelError> { /* … */ } pub fn as_str(&self) -> &str { /* … */ } pub fn into_inner(self) -> String { /* … */ } } impl TryFrom<&str> for TenantSlug { /* delegates to try_new */ } impl FromSt

### ADR-311 residual

**ADR-0311-dual-tenant-identity-personal-vs-work-boundary** — Bundled with the keystone-bundle 2026-05-20 foundational doctrine synthesis as the **dual-tenant-identity-personal-vs-work-boundary** ADR, surfaced by the Wave-3-E ecosystem journey catalog (j126-j150). The catalog introduced four new persona archetypes (Inspector Diana Reyes, Priya Krishnan, Sam Okafor, Chris Volkov) and 25 journeys whose load-bearing constraint is that a single human MUST be abl

### ADR-573 residual

**Fail-closed authz for the Cloud KMS crypto control plane (AUTH-005 / C5 remediation)** — Make the Cloud KMS crypto surfaces fail-closed, with the authorization decision modelled as **ports** owned by the boundary crate (clean architecture per ADR-0131; ports model the owned W5 destination so they do not change at cutover; the concrete cloud-iam PDP client + credential store are **adapters** that live outside this crate). The new source file `secrets/ports/kms-api/src/authz.rs` defines

### ADR-191 residual

**ADR-0191-edge-authz-tier-vs-origin-cedar-pdp** — **Two tiers; one concern per tier. Neither tier reimplements the other.** ### Edge tier — Envoy Gateway (ADR-0182, ADR-0157) Owned by `api-gateway` µservice. Enforces: | Concern | Mechanism | Source of truth | Failure response | |---|---|---|---| | IP block (geo deny-list) | MaxMind GeoIP + per-pack policy | residency policy + abuse ledger | 403 with `X-Block-Reason: geo` | | IP block (ASN deny-li

### ADR-330 residual

**Tenant Class — demo_trial vs paid with Composable Billing Components** — The decision is recorded as a numbered set of normative clauses. Every clause is a load-bearing commitment; downstream microservice work, governance lanes, and CI checks bind to clause numbers. Numbering is immutable once accepted. ### B.1 The tenant_class enum 1. **B.1.1** The tenant_class field on every oyatie tenant principal is a closed enum with exactly two members: `demo_trial` and `paid`. 2

### ADR-603 residual

**Fail-closed authz for the CRM revenue control plane (AUTH-005 remediation)** — Install the established unforgeable-authz seam (mirroring ADR-0572 / #815 and the `intelligence/adapters/rest` doctrine) in a new `src/authz.rs` owned by this crate: 1. **Unforgeable verified identity.** A `VerifiedPrincipal { principal_id, tenant_id }` with **private fields**, a `pub(crate)` constructor, and a `cfg(test)` constructor only. External crates cannot struct-literal one; they must run

### ADR-244 residual

**ADR-0244-tenant-as-universal-scoping-primitive** — The platform adopts **tenant ID + dotted hierarchical sub-scope** as the universal scoping primitive. The following twelve decisions are locked. ### D-1. Tenant ID format Every tenant has a globally unique slug ID conforming to: - **Character set.** Lowercase ASCII letters `a-z`, digits `0-9`, hyphen `-`, dot `.`. - **Anchored regex.** `^[a-z][a-z0-9-]{0,62}(\.[a-z0-9-]{1,62}){0,4}$` — first segme

### ADR-155 residual

**ADR-0155-per-tenant-resource-quotas** — Adopt per-tenant quotas on five canonical axes (rate, concurrent, memory, storage, connections) as MANDATORY across every µservice. 1. The canonical spec is `docs/standards/per-tenant-resource-quotas-canonical.md`. 2. The trait surface lives in `crates/oya-shared-tenant-quota-kernel/`. 3. The tenancy µservice OWNS canonical quota definitions; runtime µservices query it. 4. Exceeded quota → `429 To

### ADR-593 residual

**Fail-closed authz for the Accounting + Payroll money-mutation control planes (AUTH-005 / Wave-2b money-CRIT remediation)** — Wire a **fail-closed, verified-principal + cloud-iam-PDP** authz seam onto the money-mutation routes of both crates, mirroring the proven doctrine that landed for the Cloud KMS crypto control plane (ADR-0573), the Cedar policy publish control plane (ADR-0572), `intelligence/adapters/rest` (`constant_time_eq` bearer compare + a PDP `decide` port), the cloud-iam PDP caller-authn precedent (ADR-0561

### ADR-7 residual

**ADR-0007-cedar-authorization-policy-and-persona-tier** — We adopt **Cedar** as the sole authorization policy engine for RBAC/ABAC across all axes, **persona tiers T1–T4** as the autonomy-ceiling scale, and **per-capability runtime enforcement** that consults both Cedar and the autonomy ceiling on every invocation. ### Cedar surface - Engine: Cedar (Apache-2.0; in-house Rust binding under `crates/oya-policy-cedar-*`). - Per-tenant scope: tenant admins au

### ADR-163 residual

**ADR-0163-tenant-environment-tiers** — Every oyatie tenant has three environment tiers. Each tier is a logically isolated dataset within the tenant's cell: ### Tier definitions - **`test`** — sandbox environment. Tenant integrations land here first. Data is ephemeral (90-day TTL default; per-pack overlay). Outbound side effects (email send, SMS send, webhook dispatch, billing event) are *intercepted and logged* but not delivered to ext

### ADR-589 residual

**Fail-closed authz for the DSR erasure cascade (AUTH-005 / Wave-2b remediation)** — The erasure cascade is UNREACHABLE without (1) a verified principal and (2) a passing server-side PDP decision. The caller-supplied `allowed_surfaces` field is removed entirely. 1. **Unforgeable verified principal.** A new `compliance/ports/dsr-usecase/src/authz.rs` introduces `VerifiedDsrPrincipal` (private fields, `pub(crate)` constructor, public accessors, `cfg(test)` test-only constructor). Ex
