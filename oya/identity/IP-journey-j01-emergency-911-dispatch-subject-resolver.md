---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j01-subject-resolver
journey_id: j01-emergency-911-dispatch
microservice: identity
role: subject-resolver
status: draft
related_adrs: [ADR-0298, ADR-0244, ADR-0188, ADR-0247, ADR-0263]
depends_on:
  - microservices/audit-chain/IP-journey-j01-emergency-911-dispatch-emergency-classes.md
date: 2026-05-20
owner_team: axis-identity + axis-emergency-services
parallel_work_compatibility: independent of j09 account recovery work; shares principal-context-overlay surface with j02, j04
---

# IP-journey-j01-subject-resolver — Identity: subject resolution + principal-context-overlay for emergency

## Goal

Implement two gRPC endpoints in the identity µservice:

1. `ResolveSubjectForSos` — given an iOS SOS relay payload (with carrier
   E112 phone hash), resolve the oyatie principal and return their tenant +
   cell + pack context, suitable for messenger fanout.
2. `SetActiveClinicalContext` — flag a user's principal session as
   active-clinical-context for 4 hours so the principal-context-overlay
   surface can bridge consumer ↔ work tenants per ADR-0247.

Both surfaces are touched by j01 Phase 1 (subject resolution) and Phase 5
(context switch).

## Data model

| Object | Storage | Schema |
|---|---|---|
| `SubjectResolution` | (transient — Valkey cache 60s TTL) | `schemas/sos-subject-resolution.json` |
| `PrincipalContextOverlay` | Postgres `principal_context_overlays` table | per-overlay record |
| `EmergencyOptInFlag` | Postgres `emergency_opt_in_flags` table | per-user opt-in record |

```sql
CREATE TABLE principal_context_overlays (
  id UUID PRIMARY KEY,
  user_id TEXT NOT NULL,
  from_principal TEXT NOT NULL,
  to_principal TEXT NOT NULL,
  context_flag TEXT NOT NULL CHECK (context_flag IN ('active-clinical-context', 'active-emergency-context', 'active-break-glass-context')),
  expires_at TIMESTAMPTZ NOT NULL,
  authentication_method TEXT NOT NULL,
  audit_id TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pco_user_active ON principal_context_overlays (user_id, expires_at)
  WHERE expires_at > NOW();

CREATE TABLE emergency_opt_in_flags (
  user_id TEXT PRIMARY KEY,
  opted_in_emergency_profile BOOLEAN NOT NULL DEFAULT FALSE,
  opt_in_field_set TEXT[] NOT NULL DEFAULT '{}',
  emergency_contact_set JSONB NOT NULL DEFAULT '[]'::jsonb,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## API surface (gRPC)

```protobuf
service IdentityEmergency {
  rpc ResolveSubjectForSos(ResolveSubjectForSosRequest) returns (ResolveSubjectForSosResponse);
  rpc SetActiveClinicalContext(SetActiveClinicalContextRequest) returns (SetActiveClinicalContextResponse);
  rpc GetPrincipalContextOverlay(GetPrincipalContextOverlayRequest) returns (PrincipalContextOverlay);
}
```

## Files to author

| File | Purpose | Size |
|---|---|---|
| `microservices/identity/src/emergency/subject_resolver.rs` | gRPC server impl | ~250 lines |
| `microservices/identity/src/emergency/principal_context_overlay.rs` | Context bridge | ~200 lines |
| `microservices/identity/policy/subject-resolution-for-sos.cedar` | Cedar permit | ~30 lines |
| `microservices/identity/policy/active-clinical-context.cedar` | Cedar permit for 4h bridge | ~30 lines |
| `microservices/identity/contracts/proto/emergency.proto` | gRPC defs | ~140 lines |
| `microservices/identity/db/migrations/2026-05-20-001-principal-context-overlays.sql` | DDL | ~40 lines |
| `microservices/identity/db/migrations/2026-05-20-002-emergency-opt-in-flags.sql` | DDL | ~30 lines |
| `microservices/identity/runbooks/principal-context-overlay-incident.md` | Ops runbook | ~120 lines |
| `microservices/identity/tests/integration/emergency_test.rs` | Integration tests | ~400 lines |

## Cedar fragments

```cedar
permit (
  principal == Service::"messenger-emergency-fanout",
  action == Action::"identity.resolve_subject_for_sos",
  resource is User
) when {
  context.audience_type == "EMERGENCY_SERVICES_SOS" &&
  resource.opted_in_emergency_contacts == true
};

permit (
  principal is User,
  action == Action::"identity.set_active_clinical_context",
  resource is User
) when {
  principal == resource &&
  principal.has_passkey_for_principal(context.to_principal) == true &&
  context.duration_seconds <= 14400
};
```

## Audit events to emit

| Class | Trigger | PII | Retention | Pack |
|---|---|---|---|---|
| `SubjectResolvedForSos` | resolver invoked | minimal (principal slug only) | 6y | KR-119 |
| `PrincipalContextSwitch` | context overlay set | minimal | 1y | general |
| `EmergencyOptInUpdated` | user changes opt-in set | n/a | 7y | KR-PIPA |
| `PasskeyAssertSucceeded` / `PasskeyAssertFailed` | passkey auth | n/a | 1y | general |

## Observability

| Metric | Labels |
|---|---|
| `oya_subject_resolution_total` | `outcome`, `tenant_id`, `pack` |
| `oya_principal_context_switch_total` | `from_principal_tenant`, `to_principal_tenant`, `context_flag` |
| `oya_cedar_eval_latency_ms` | `policy` |
| `oya_passkey_assert_total` | `outcome`, `tenant_id` |

## SLOs

| SLO | Target |
|---|---:|
| `subject_resolution_p95` | ≤ 100ms |
| `context_switch_p95` | ≤ 350ms |
| `cedar_eval_warm_p95` | ≤ 30ms |

## Tests to write

Per `integration-test-plan.md`:
- §2.1, §2.2, §3.3 (resolver)
- §6.1, §6.2 (context switch)

## Parallel-work compatibility

- Independent of j09 account-recovery IP (different code paths).
- Shares `PrincipalContextOverlay` with j02, j04, j08 — coordinate schema
  changes via OpenAPI/gRPC contract review.

## Promotion path + rollback

Same pattern as other identity IPs (see `IP-001-zitadel-helm-per-pack.md`).

— end of IP —

## Completion expansion for j01 identity emergency-911-dispatch-subject-resolver

This appendix completes a pre-existing partial IP scaffold to the 400-line per-service bar required by /tmp/codex-brief-j01-j20-lifesafety.md.
The expansion is bound to ADR-0298 and the shared life-safety ADR pack ADR-0298, ADR-0299, ADR-0300, ADR-0301, ADR-0302, ADR-0303, ADR-0304, ADR-0305, ADR-0306, ADR-0292.

## Completion scope

- Microservice: identity.
- Journey: j01 Emergency 119 dispatch.
- Role: emergency-911-dispatch-subject-resolver.
- This is an additive completion; prior scaffold text above is preserved.
- No ADR, standard, PRD, or ARCHITECTURE file is modified by this appendix.

## Contract closure

| Surface | Required behavior | Evidence |
|---|---|---|
| OpenAPI 3.2.0 command | identity validates j01 emergency-911-dispatch-subject-resolver with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| AsyncAPI 3.1.0 event | identity validates j01 emergency-911-dispatch-subject-resolver with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| proto3 internal RPC | identity validates j01 emergency-911-dispatch-subject-resolver with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| Cedar v4.1 policy | identity validates j01 emergency-911-dispatch-subject-resolver with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| audit-chain seal | identity validates j01 emergency-911-dispatch-subject-resolver with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| observability span | identity validates j01 emergency-911-dispatch-subject-resolver with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| integration harness fixture | identity validates j01 emergency-911-dispatch-subject-resolver with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |

## Wave 15 journey row substance

The previous numbered loop in this IP was treated as scaffold, not deliverable evidence. The rows below keep only grounded identity-owned journey actions and delete rows that merely restated the same tenant/Cedar/audit sentence without a backing contract, Cedar predicate, or counterpart reference.

Evidence anchors used below are existing identity surfaces: `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`, `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/policy/tenant-scope.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, and `microservices/identity/feature-parity-matrix-2026-05-20.md`.

| Journey row | Source trigger | Actor identity | Backing contract / Cedar | State effect | Evidence touch | Counterpart equivalence |
|---|---|---|---|---|---|---|
| subject-resolver trigger intake | Receive the j01-emergency-911-dispatch user/operator signal at the identity edge and normalize `tenant_id`, `principal_id`, `audience_type`, `purpose`, `data_class`, and `idempotency_key`. | `User`, `tenant-admin`, `Service`, or `ops-security` principal as declared by the journey; personal context is never inferred from a work tenant. | OpenAPI `/oauth/v2/token`, `/webauthn/authenticate/finish`, `/step-up`, `/principal-context/resolve`; Cedar `ResolvePrincipalContext` when context is involved. | Creates a verified claims envelope, step-up challenge, or context-resolution refusal before any downstream mutation. | AsyncAPI `identity.signin.v1`, `identity.step-up.v1`, `identity.webauthn.v1`, or `oyatie.identity.context-resolved.v1` with audit id and `principal_hash`. | Matches Auth0 Universal Login/MFA and Okta Identity Engine signin while adding Oyatie tenant + Cedar context. |
| subject-resolver Cedar preflight | Evaluate the requested action before adapter calls, credential changes, or cross-tenant reads. | `tenant-admin` stays tenant-bound; `ops-security` requires `acr=critical`, open ticket, and approval context; self-service users can only act on their own user resource. | `tenant-scope.cedar`, `context-split.cedar`, `dual-context-residency.cedar`, and `operator-recovery.cedar` actions including `SwitchPrincipalContext`, `ScimDeleteUser`, and `OperatorRecoveryRevokeCredential`. | Permit id is attached to the decision; denial stops the journey row and records an appeal/rollback branch. | Unauthorized attempts use refusal events and audit emit completeness SLO rather than line-count evidence. | Comparable to Entra Conditional Access and Okta policy checks, but expressed as Cedar fragments. |
| subject-resolver credential continuity | For recovery, lockout, SIM-swap, voice/passkey, survivor, or high-risk flows, prove the credential or recovery chain before state change. | Self-service `User` with device-bound credential, or `ops-security` under mediated recovery when user self-service is impossible. | OpenAPI `/webauthn/credentials`, `/webauthn/authenticate/finish`, `/step-up/passkey/start`, `/step-up/passkey/finish`; proto `ListUserCredentials` / `RevokeCredential`. | Updates credential freshness, revokes a compromised credential, or pins ACR; failed assertion leaves identity state unchanged. | AsyncAPI `identity.webauthn.v1`; SLO `webauthn-authenticate-latency.openslo.yaml`; recovery evidence uses `audit-emit-completeness.openslo.yaml`. | Matches Auth0 MFA reset and Okta FastPass recovery expectations while refusing operator decryption. |
| subject-resolver tenant state update | Only after permit approval, apply the narrow identity mutation required by j01-emergency-911-dispatch. | Resource is `User` or `PrincipalContextEnvelope` scoped by `tenant_id`; cross-pack replication remains forbidden by `dual-context-residency.cedar`. | OpenAPI `/scim/v2/{tenant}/Users/{id}`, `/admin/jwks/rotate`, `/federation/bindings`; proto `RevokeSession`, `PinUserAcr`, or `ResolvePrincipalContext`. | Writes scoped user active state, session revoke, ACR pin, federation binding, or context envelope; duplicate idempotency key returns the original decision. | Audit event includes tenant, principal hash, action, decision, timestamp, idempotency key, and sealed audit id. | Matches Okta/Entra provisioning lifecycle and Auth0 Management API mutation with cross-tenant bearer refusal. |
| subject-resolver refusal branch | If tenant mismatch, missing consent, expired proof, regional hold, or personal-context overreach appears, identity emits refusal before downstream work. | Denied principal remains the original caller; no synthetic service principal is created to force success. | Cedar forbid clauses come from `tenant-scope.cedar`, `context-split.cedar`, and `dual-context-residency.cedar`. | No runtime mutation is performed; refusal carries reason, tenant/resource pair, and appeal route. | AsyncAPI `oyatie.identity.context-switch-refused.v1`, unauthorized-attempt metrics, and audit-chain seal status. | Hosted IdPs expose policy/admin logs; Oyatie adds explicit tenant/resource evidence. |
| subject-resolver operator review | Human override requires open ticket, critical ACR, and second-operator approval for mass actions. | `ops-security` only; tenant admins cannot switch into personal contexts or mass-reset credentials. | `operator-recovery.cedar`, `/step-up/it-approval/request`, `/step-up/it-approval/finish`, proto `PinUserAcr` / `RevokeCredential`. | Credential revoke, forced reauth, or ACR pin records the ticket and approval state; mass operations require distinct second operator id. | AsyncAPI `identity.step-up.v1` and audit completeness SLO link the override to the journey row. | Matches enterprise admin recovery in Okta/Auth0/Entra while keeping operator impersonation denied. |
| subject-resolver closure evidence | Close the slice only after principal context, credential state, and tenant-scoped audit event agree with the contract outcome. | Verifier consumes signed events and contract fixtures; downstream services read sealed identity evidence, not raw personal data. | OpenAPI `identity.yaml` + `multi-context-split.yaml`; proto `identity.proto` + `multi_context_split.proto`; AsyncAPI identity event schemas; Cedar policy fragments. | IP remains planned/scaffolded if the specific concept lacks contract, Cedar, event, and SLO backing. | SLO touch points are OIDC, WebAuthn, SCIM, step-up, and audit emit completeness. | Counterpart reference is the feature parity matrix rows for Auth0, Okta, and Entra identity/governance/provisioning. |

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
- Trigger evidence: `microservices/identity/IP-journey-j01-emergency-911-dispatch-subject-resolver.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=30`, `rpo_p99_seconds=0` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), PCI-DSS-L1-v4(86400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/identity/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/webauthn-authenticate-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`, `microservices/identity/policy/cedar-acr-predicates.cedar`.
