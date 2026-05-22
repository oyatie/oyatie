---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-023-byok-credential-rotation
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
related_adrs: [ADR-0255, ADR-0296]
---

# IP-023: provider-BYOK credential rotation flow

## Intent

Implement the full provider-BYOK rotation flow per ADR-0255 §D-4 + ADR-0296. When a tenant
rotates their provider API key, the new credential must be hot-swapped into OpenBao sidecar
without restarting the intelligence pod. Zero downtime. Per `runbooks/byok-rotation-tenant-cascade.md`.

## Concrete file targets

| Path | Action |
|---|---|
| `crates/oya-intelligence-credential-resolver-usecase/src/rotation.rs` | create |
| `crates/oya-intelligence-credential-resolver-adapter/src/openbao_rotation.rs` | create |

## Rotation protocol

```
1. Tenant writes new API key to OpenBao at:
   ${openbao:secret/<tenant_id>/intelligence/provider/<provider>}
2. OpenBao emits key-rotation event → credential-resolver-adapter receives via watch.
3. credential-resolver-usecase invalidates in-flight CredentialHandle cache for that tenant+provider.
4. Next dispatch for tenant+provider issues fresh CredentialHandle from new key.
5. Old CredentialHandle TTL expires (≤ 60 s); all handles drained.
6. Emit audit event: ByokCredentialRotated { tenant_id, provider, rotated_at }.
```

## Key implementation notes

- `provider_credential_mode` per tenant: `platform_default | byok | byok_required_by_pack`.
- `byok_required_by_pack`: HIPAA pack forces BAA-provider-only + tenant provider-BYOK key.
- FedRAMP: `byok_required_by_pack` for `pack-us-federal`; only Bedrock-GovCloud / Azure-OpenAI-Gov allowed.
- Zero active-key-downtime: rotation is atomic at the watch level; no gap window.

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-credential-resolver-usecase -- rotation
cargo run -p oya-dev-cli -- gate validate byok-zero-downtime-rotation --microservice intelligence
```

## References

- `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`.
- ADR-0255 §D-4 (provider-BYOK opt-in — NOT conflated with encryption-BYOK).
- ADR-0296 (sidecar credential handle ≤ 60 s TTL).

## Wave 15 substance conversion — provider credential rotation

### §A Problem

The intelligence substrate advertises provider-BYOK as distinct from encryption BYOK, but stale provider keys
would either break dispatch or tempt adapters to cache raw credentials.
This IP closes the hot-rotation gap between `SecretReference`, OpenBao sidecar watches, `CredentialHandle`, and
the provider adapters.
The success condition is tenant key rotation without pod restart, provider credential leakage, or dispatching with
a stale handle after its bounded TTL.

### §B Approach

Treat OpenBao as the authority for raw provider credentials and keep the intelligence process on opaque handles.
The resolver usecase owns cache invalidation and handle replacement; adapters only receive signed/short-lived
handles during request assembly.
The policy guard is `policy/byok-gating.cedar`, while `runbooks/byok-rotation-tenant-cascade.md` defines operator
response for a failed or cascading rotation.

### §C Deliverables

- Create `crates/oya-intelligence-credential-resolver-usecase/src/rotation.rs`.
- Create `crates/oya-intelligence-credential-resolver-adapter/src/openbao_rotation.rs`.
- Add tests for `CredentialHandle` TTL expiry and cache invalidation.
- Add audit event shape for `ByokCredentialRotated`.
- Add failure-mode runbook links to provider outage and sidecar expiration runbooks.

### §D Implementation

1. Subscribe to OpenBao watch events for `secret/<tenant_id>/intelligence/provider/<provider>`.
2. Validate the tenant/provider tuple against `SecretReference.bound_tenant`.
3. Atomically evict handles for that tuple while preserving handles for other providers.
4. Refuse new dispatch during the tiny swap window only if no fresh handle can be minted.
5. Keep old handles usable only until their existing ≤60 second TTL expires.
6. Emit `ByokCredentialRotated` with tenant, provider, old handle generation, and new generation.
7. Prove regulated packs still deny `platform_default` through `byok-gating.cedar`.

### §E Acceptance

The zero-downtime gate must include an in-flight dispatch, a rotation event, a subsequent dispatch using a new
handle, and a stale-handle deny case.
Evidence must cite `sidecar-credential-handle-expired.md` and the `audit-emission-success` SLO because rotation
without audit is not acceptable.

### §F Evidence

Local anchors: `policy/byok-gating.cedar`, `runbooks/byok-rotation-tenant-cascade.md`,
`runbooks/sidecar-credential-handle-expired.md`, `manifest.json` credential resolver BC.
Doctrine anchors: ADR-0255 §D-4, ADR-0296, ADR-0244, ADR-0263.

### §G Counterparts

| Counterpart | Relevant behaviour | Oyatie closure |
|---|---|---|
| AWS Bedrock IAM | Enterprise-controlled credentials | Keep tenant-controlled provider keys but isolate them in OpenBao handles |
| Azure OpenAI identity | Tenant/cloud identity-mediated access | Match enterprise credential posture without provider-specific leakage |
| OpenAI / Anthropic direct APIs | API key rotation is tenant-side | Add substrate-level hot rotation and audit evidence around direct-provider keys |

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/intelligence/IP-023-byok-credential-rotation.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=300`, `rpo_p99_seconds=60` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), EU-AI-ACT-2024-HIGH-RISK(1800s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-PIPA-2023-amendment(14400s/900s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/intelligence/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml`, `microservices/intelligence/policy/abuse-defence.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-023-byok-credential-rotation.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
