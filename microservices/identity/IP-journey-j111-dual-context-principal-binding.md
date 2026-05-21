---
doc_class: Implementation-Plan
ip_id: IP-journey-j111-dual-context-principal-binding
journey_ref: docs/user-journeys/j111-staffing-agency-as-tenant-facilitator/
microservice: identity
status: draft
date: 2026-05-20
authority_tier: 3
related_adrs:
  - ADR-0242-oyatie-is-a-tenant-doctrine
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0249-multi-category-marketplace-doctrine
  - ADR-0263-observability-emission-contract
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0313-conglomerate-tenant-hierarchy
  - ADR-0314-marketplace-universal-deal-settlement-substrate
planned_enforcement_ref: oya-governance-doc-rigor
---

# IP - identity role in j111: Staffing agency as tenant facilitator

Role: dual-context-principal-binding.

Journey purpose: A staffing-agency tenant sources workers from Community, places them at KrampusCorp, ConstructionCo,
and HealthcareSystem-Megacorp, and receives Stripe Connect facilitator commissions.

## Scope

identity owns only the dual-context-principal-binding slice for j111. It does not absorb another service responsibility,
does not bypass Cedar, and does not write into another tenant-owned store without an explicit grant.

## Acceptance criteria

1. identity exposes or consumes the typed j111 contract without ad hoc string parsing.
2. Every state-changing path evaluates Cedar and records the permit id.
3. Every mutation emits an ADR-0263 observability event with audit_id linkage.
4. Rollback exists for each reversible state and pause exists for irreversible state.
5. Cross-tenant reads require explicit tenant pair and purpose.
6. Personal-tenant data is default-deny unless the personal tenant owner consents.
7. The implementation maps to one of the ADR-0105 canonical layers.
8. The test plan includes success, expired-permit, outage, and residency-hold cases.

## Atomic deliverables

## Wave 15 journey row substance

The previous numbered loop in this IP was treated as scaffold, not deliverable evidence. The rows below keep only grounded identity-owned journey actions and delete rows that merely restated the same tenant/Cedar/audit sentence without a backing contract, Cedar predicate, or counterpart reference.

Evidence anchors used below are existing identity surfaces: `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`, `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/policy/tenant-scope.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, and `microservices/identity/feature-parity-matrix-2026-05-20.md`.

| Journey row | Source trigger | Actor identity | Backing contract / Cedar | State effect | Evidence touch | Counterpart equivalence |
|---|---|---|---|---|---|---|
| dual-context-principal-binding tenant-pair intake | Receive j111-staffing-agency-as-tenant-facilitator cross-tenant request with explicit source tenant, counterparty tenant, purpose, data class, grant id, and idempotency key. | `Service` principal for workflow handoff or `tenant-admin` principal for source-tenant owned grants; no personal principal is inferred. | OpenAPI `/principal-context/resolve`; proto `ResolvePrincipalContext`; Cedar `ResolvePrincipalContext` plus `tenant-scope.cedar` same-tenant checks. | Creates a principal-context envelope or refusal only; marketplace, tenancy, and workflow stores remain untouched. | AsyncAPI `oyatie.identity.context-resolved.v1` with tenant pair, permit id, audit id, and `principal_hash`. | Matches Auth0 Organizations membership lookup and Okta tenant assignment, with explicit Cedar tenant-pair evidence. |
| dual-context-principal-binding personal-context deny | If j111-staffing-agency-as-tenant-facilitator attempts to use personal tenant data for a work-tenant grant, deny unless personal-owner consent is present. | `tenant-admin`, `Service`, or self-service `User`; tenant admins are forbidden from personal contexts. | Cedar `SwitchPrincipalContext` forbid path in `context-split.cedar`; `dual-context-residency.cedar` pack equality check. | No work-tenant copy of personal attributes is produced; refusal carries reason and appeal route. | AsyncAPI `oyatie.identity.context-switch-refused.v1`; audit completeness SLO records refusal before downstream effects. | Hosted IdPs expose admin/member boundaries, but not Oyatie's hard personal/work tenant split. |
| dual-context-principal-binding counterparty principal bind | Bind a scoped counterparty principal only after tenant grant, KYB/role status, and audience type align. | `tenant-admin` for tenant-owned grants or `Service` principal for workflow-engine handoff; resource remains `User` or context envelope. | OpenAPI `/federation/bindings`; proto `Verify`; AsyncAPI `identity.federation.v1`; Cedar tenant/resource pack checks. | State effect is scoped federation/principal binding, not broad directory sync. | Evidence contains grant id, tenant pair, audience type, permit id, and signed audit event. | Matches Entra B2B guest federation and Auth0 enterprise connections while refusing broad organization membership. |
| dual-context-principal-binding SCIM lifecycle sync | Where work membership is prerequisite, use SCIM tenant paths rather than ad hoc member rows. | SCIM bearer principal bound to exactly one tenant. | OpenAPI `/scim/v2/{tenant}/Users`, `/scim/v2/{tenant}/Users/{id}`, `/scim/v2/{tenant}/Groups`; AsyncAPI `identity.scim.v1`; `tenant-scope.cedar` SCIM bearer refusal. | State effect is create/update/active=false/group membership in the source tenant only. | SLO `scim-availability.openslo.yaml`; audit event class from `identity.user.lifecycle.v1`. | Matches Okta SCIM and Microsoft Entra provisioning breadth; Oyatie adds tenant scope and audit seal. |
| dual-context-principal-binding step-up for grant risk | Before high-risk grant changes, require `/step-up` or passkey step-up to raise ACR. | Self-service `User`, `tenant-admin`, or `ops-security`; `ops-security` must satisfy `operator-recovery.cedar`. | OpenAPI `/step-up`, `/step-up/passkey/start`, `/step-up/passkey/finish`; AsyncAPI `identity.step-up.v1`; proto `PinUserAcr`. | State effect is ACR grant/pin or refusal; no grant is created on failed step-up. | SLO `step-up-grant-latency.openslo.yaml`; step-up audit id links to the cross-tenant grant id. | Comparable to Entra Conditional Access and Okta MFA prompts, but recorded as explicit journey evidence. |
| dual-context-principal-binding token issue and verify | Issue or verify OIDC claims carrying tenant, audience type, purpose, data class, home cell, and recovery epoch. | `User` or service account principal; downstream services consume proto verification result. | OpenAPI `/oauth/v2/token`, `/oauth/v2/userinfo`, `/.well-known/openid-configuration`, `/oauth/v2/keys`; proto `VerifyBatch`. | State effect is signed token issuance or verify result; no cross-tenant write occurs here. | SLOs `oidc-token-issue-latency.openslo.yaml` and `oidc-token-verify-latency.openslo.yaml`; AsyncAPI `identity.oidc.v1`. | Matches Okta/Auth0/Entra OIDC surfaces with Cedar-ready Oyatie claims. |
| dual-context-principal-binding residency and pack hold | If any party crosses pack boundaries, hold the grant until residency and compliance checks pass. | Principal/resource pack pair must match unless a planned cross-pack resolver IP supplies explicit evidence. | Cedar `principal.pack != resource.pack` forbid in `dual-context-residency.cedar`; OpenAPI context resolver carries pack context. | State effect is hold/refusal with no replication; planned cross-pack work remains referenced as an IP. | Audit event includes pack, tenant pair, denial reason, and resolver trace id. | Entra multi-tenant and Auth0 Organizations do not by themselves satisfy Oyatie residency evidence. |
| dual-context-principal-binding audit closure | Close j111-staffing-agency-as-tenant-facilitator identity participation only after contract, Cedar, event, SLO, and counterpart rows line up. | Verifier consumes signed events and contract fixtures; line count alone cannot promote. | Evidence set is OpenAPI + AsyncAPI + proto + Cedar + SLO + feature parity matrix. | State effect is IP promotion readiness, not runtime mutation. | Audit hash links `identity.*.v1` event to the journey row and downstream service handoff. | Counterpart references: Auth0 Organizations/MFA/SCIM, Okta OIE/SCIM/FastPass, and Microsoft Entra provisioning/Conditional Access. |

## Deleted scaffold rows

Rows removed from the prior loop were ungrounded because they repeated generic slice labels, cited no existing per-journey implementation file, or described another microservice's action without an identity contract handoff. Future expansion must add a concrete contract, Cedar action, event class, SLO/evidence check, and counterpart anchor before adding rows back.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/identity/IP-journey-j111-dual-context-principal-binding.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=30`, `rpo_p99_seconds=0` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), PCI-DSS-L1-v4(86400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/identity/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/webauthn-authenticate-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`, `microservices/identity/policy/cedar-acr-predicates.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/identity/IP-journey-j111-dual-context-principal-binding.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/identity/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
