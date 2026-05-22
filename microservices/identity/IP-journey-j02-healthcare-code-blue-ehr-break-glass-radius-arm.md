---
doc_class: IP
ip_id: IP-journey-j02-radius-arm
journey_id: j02-healthcare-code-blue-ehr-break-glass
microservice: identity
role: break-glass-radius-arm
status: draft
related_adrs: [ADR-0247, ADR-0263, ADR-0298]
depends_on: [microservices/audit-chain/IP-journey-j02-healthcare-code-blue-ehr-break-glass-classes.md]
date: 2026-05-20
---

# IP-journey-j02-radius-arm — Identity: break-glass radius arming

## Goal
Implement `ArmBreakGlassRadius` gRPC that, on receipt of a code-blue
event from workflow-engine, arms the Cedar permit cache for all credentialed
clinicians within 30m of the alarm bed for 10 minutes.

## Data model
```sql
CREATE TABLE break_glass_radius_arms (
  id UUID PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  alarm_event_id TEXT NOT NULL,
  bed_id TEXT NOT NULL,
  radius_meters INT NOT NULL DEFAULT 30,
  armed_at TIMESTAMPTZ NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  audit_id TEXT NOT NULL
);
CREATE INDEX idx_bgra_active ON break_glass_radius_arms (tenant_id, bed_id, expires_at) WHERE expires_at > NOW();
```

## API
```protobuf
service IdentityBreakGlass {
  rpc ArmBreakGlassRadius(ArmBreakGlassRadiusRequest) returns (BreakGlassArmResponse);
  rpc CheckBreakGlassEligible(CheckBreakGlassEligibleRequest) returns (BreakGlassEligibleResponse);
}
```

## Files
- `microservices/identity/src/break_glass/radius_arm.rs` (~250 lines)
- `microservices/identity/policy/break-glass-radius-arm.cedar` (~30 lines)
- `microservices/identity/contracts/proto/break_glass.proto` (~80 lines)
- `microservices/identity/db/migrations/2026-05-20-003-break-glass-radius-arms.sql` (~40 lines)
- `microservices/identity/tests/integration/break_glass_test.rs` (~400 lines)
- `microservices/identity/runbooks/break-glass-arm-degraded.md` (~120 lines)

## Cedar
```cedar
permit (
  principal == Workflow::"snuh.org/code-blue-response",
  action == Action::"identity.arm_break_glass_radius",
  resource is Bed
) when {
  context.alarm_active == true &&
  context.radius_meters <= 30 &&
  context.duration_minutes <= 10
};
```

## Audit events
`BreakGlassRadiusArmed`, `BreakGlassRadiusExpired`.

## SLOs
- arm_p95 ≤ 200ms; eligibility_check_p95 ≤ 50ms.

## Tests
Per integration-test-plan §2.

## Parallel
Independent of j02 ontology + workflow-engine IPs. Reused for j04 + j12.

— end of IP —

## Completion expansion for j02 identity clinician-radius-and-acr

This expansion preserves the existing IP scaffold and completes it to the 400-line journey-IP bar for Healthcare code blue EHR break-glass.
# IP - j02 - identity - clinician-radius-and-acr

Goal: implement the identity portion of Healthcare code blue EHR break-glass so Yejin reaches a coding patient and needs immediate chart access under post-hoc break-glass audit.
Binding ADR: ADR-0247. This IP is one independently reviewable slice and must not weaken the common critical-path ADR pack.

## Scope

In scope: clinician-radius-and-acr, JSON Schema validation, Cedar authorization, audit-chain emission, observability, failure branches, and integration tests for j02.
Out of scope: changing ADRs, changing standards, editing existing PRDs, bypassing Cedar, or adding a new microservice directory.

## Data model

| Object | Storage | Schema | Retention |
|---|---|---|---|
| code-blue-intake | identity.clinician-radius-and-acr table or event stream | docs/user-journeys/j02-healthcare-code-blue-ehr-break-glass/schemas/code-blue-intake.json | pack-controlled, minimum audit retention |
| break-glass-access-decision | identity.clinician-radius-and-acr table or event stream | docs/user-journeys/j02-healthcare-code-blue-ehr-break-glass/schemas/break-glass-access-decision.json | pack-controlled, minimum audit retention |
| posthoc-justification | identity.clinician-radius-and-acr table or event stream | docs/user-journeys/j02-healthcare-code-blue-ehr-break-glass/schemas/posthoc-justification.json | pack-controlled, minimum audit retention |

## API and event contract

```yaml
openapi: 3.2.0
info:
  title: identity j02 clinician-radius-and-acr
  version: 1.0.0
paths:
  /journeys/j02/identity/clinician-radius-and-acr:
    post:
      operationId: j02IdentityClinicianRadiusAndAcr
      x-binding-adr: ADR-0247
      x-audit-required: true
      responses:
        "202":
          description: Accepted and audit-sealed
```

```yaml
asyncapi: 3.1.0
info:
  title: identity j02 events
  version: 1.0.0
channels:
  j02.identity.clinician-radius-and-acr.accepted:
    address: j02.identity.clinician-radius-and-acr.accepted
```

## Cedar and policy grammar

The caller-side policy library evaluates before network dispatch where possible. Service-side policy re-evaluates on mutation.

```cedar
permit(principal, action == Action::"j02.execute", resource)
when {
  principal.tenant_id == resource.tenant_id &&
  resource.binding_adr == "ADR-0247" &&
  context.cell_tier >= resource.minimum_cell_tier &&
  context.audit_chain_required == true
};

forbid(principal, action == Action::"j02.execute", resource)
when {
  principal.tenant_id != resource.tenant_id &&
  context.cross_tenant_grant_absent == true
};
```

## Wave 15 journey row substance

The previous numbered loop in this IP was treated as scaffold, not deliverable evidence. The rows below keep only grounded identity-owned journey actions and delete rows that merely restated the same tenant/Cedar/audit sentence without a backing contract, Cedar predicate, or counterpart reference.

Evidence anchors used below are existing identity surfaces: `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`, `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/policy/tenant-scope.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, and `microservices/identity/feature-parity-matrix-2026-05-20.md`.

| Journey row | Source trigger | Actor identity | Backing contract / Cedar | State effect | Evidence touch | Counterpart equivalence |
|---|---|---|---|---|---|---|
| break-glass-radius-arm trigger intake | Receive the j02-healthcare-code-blue-ehr-break-glass user/operator signal at the identity edge and normalize `tenant_id`, `principal_id`, `audience_type`, `purpose`, `data_class`, and `idempotency_key`. | `User`, `tenant-admin`, `Service`, or `ops-security` principal as declared by the journey; personal context is never inferred from a work tenant. | OpenAPI `/oauth/v2/token`, `/webauthn/authenticate/finish`, `/step-up`, `/principal-context/resolve`; Cedar `ResolvePrincipalContext` when context is involved. | Creates a verified claims envelope, step-up challenge, or context-resolution refusal before any downstream mutation. | AsyncAPI `identity.signin.v1`, `identity.step-up.v1`, `identity.webauthn.v1`, or `oyatie.identity.context-resolved.v1` with audit id and `principal_hash`. | Matches Auth0 Universal Login/MFA and Okta Identity Engine signin while adding Oyatie tenant + Cedar context. |
| break-glass-radius-arm Cedar preflight | Evaluate the requested action before adapter calls, credential changes, or cross-tenant reads. | `tenant-admin` stays tenant-bound; `ops-security` requires `acr=critical`, open ticket, and approval context; self-service users can only act on their own user resource. | `tenant-scope.cedar`, `context-split.cedar`, `dual-context-residency.cedar`, and `operator-recovery.cedar` actions including `SwitchPrincipalContext`, `ScimDeleteUser`, and `OperatorRecoveryRevokeCredential`. | Permit id is attached to the decision; denial stops the journey row and records an appeal/rollback branch. | Unauthorized attempts use refusal events and audit emit completeness SLO rather than line-count evidence. | Comparable to Entra Conditional Access and Okta policy checks, but expressed as Cedar fragments. |
| break-glass-radius-arm credential continuity | For recovery, lockout, SIM-swap, voice/passkey, survivor, or high-risk flows, prove the credential or recovery chain before state change. | Self-service `User` with device-bound credential, or `ops-security` under mediated recovery when user self-service is impossible. | OpenAPI `/webauthn/credentials`, `/webauthn/authenticate/finish`, `/step-up/passkey/start`, `/step-up/passkey/finish`; proto `ListUserCredentials` / `RevokeCredential`. | Updates credential freshness, revokes a compromised credential, or pins ACR; failed assertion leaves identity state unchanged. | AsyncAPI `identity.webauthn.v1`; SLO `webauthn-authenticate-latency.openslo.yaml`; recovery evidence uses `audit-emit-completeness.openslo.yaml`. | Matches Auth0 MFA reset and Okta FastPass recovery expectations while refusing operator decryption. |
| break-glass-radius-arm tenant state update | Only after permit approval, apply the narrow identity mutation required by j02-healthcare-code-blue-ehr-break-glass. | Resource is `User` or `PrincipalContextEnvelope` scoped by `tenant_id`; cross-pack replication remains forbidden by `dual-context-residency.cedar`. | OpenAPI `/scim/v2/{tenant}/Users/{id}`, `/admin/jwks/rotate`, `/federation/bindings`; proto `RevokeSession`, `PinUserAcr`, or `ResolvePrincipalContext`. | Writes scoped user active state, session revoke, ACR pin, federation binding, or context envelope; duplicate idempotency key returns the original decision. | Audit event includes tenant, principal hash, action, decision, timestamp, idempotency key, and sealed audit id. | Matches Okta/Entra provisioning lifecycle and Auth0 Management API mutation with cross-tenant bearer refusal. |
| break-glass-radius-arm refusal branch | If tenant mismatch, missing consent, expired proof, regional hold, or personal-context overreach appears, identity emits refusal before downstream work. | Denied principal remains the original caller; no synthetic service principal is created to force success. | Cedar forbid clauses come from `tenant-scope.cedar`, `context-split.cedar`, and `dual-context-residency.cedar`. | No runtime mutation is performed; refusal carries reason, tenant/resource pair, and appeal route. | AsyncAPI `oyatie.identity.context-switch-refused.v1`, unauthorized-attempt metrics, and audit-chain seal status. | Hosted IdPs expose policy/admin logs; Oyatie adds explicit tenant/resource evidence. |
| break-glass-radius-arm operator review | Human override requires open ticket, critical ACR, and second-operator approval for mass actions. | `ops-security` only; tenant admins cannot switch into personal contexts or mass-reset credentials. | `operator-recovery.cedar`, `/step-up/it-approval/request`, `/step-up/it-approval/finish`, proto `PinUserAcr` / `RevokeCredential`. | Credential revoke, forced reauth, or ACR pin records the ticket and approval state; mass operations require distinct second operator id. | AsyncAPI `identity.step-up.v1` and audit completeness SLO link the override to the journey row. | Matches enterprise admin recovery in Okta/Auth0/Entra while keeping operator impersonation denied. |
| break-glass-radius-arm closure evidence | Close the slice only after principal context, credential state, and tenant-scoped audit event agree with the contract outcome. | Verifier consumes signed events and contract fixtures; downstream services read sealed identity evidence, not raw personal data. | OpenAPI `identity.yaml` + `multi-context-split.yaml`; proto `identity.proto` + `multi_context_split.proto`; AsyncAPI identity event schemas; Cedar policy fragments. | IP remains planned/scaffolded if the specific concept lacks contract, Cedar, event, and SLO backing. | SLO touch points are OIDC, WebAuthn, SCIM, step-up, and audit emit completeness. | Counterpart reference is the feature parity matrix rows for Auth0, Okta, and Entra identity/governance/provisioning. |

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
- Trigger evidence: `microservices/identity/IP-journey-j02-healthcare-code-blue-ehr-break-glass-radius-arm.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=30`, `rpo_p99_seconds=0` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), PCI-DSS-L1-v4(86400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/identity/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/webauthn-authenticate-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`, `microservices/identity/policy/cedar-acr-predicates.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/identity/IP-journey-j02-healthcare-code-blue-ehr-break-glass-radius-arm.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/identity/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
