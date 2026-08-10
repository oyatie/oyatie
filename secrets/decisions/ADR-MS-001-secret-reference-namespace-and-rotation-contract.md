---
id: ADR-MS-001
title: SecretReference namespace and rotation contract for cloud-secrets
status: Proposed
date: 2026-05-20
microservice: cloud-secrets
related_oyatie_adrs:
  - ADR-0003-audit-chain-and-evidence-emission
  - ADR-0007-cedar-authorization-policy-and-persona-tier
  - ADR-0008-data-use-boundary
  - ADR-0009-cell-architecture-per-tenant-per-region
  - ADR-0037-public-api-stability-tiers-and-deprecation
  - ADR-0043-secrets-management-openbao-and-hsm-per-cell
  - ADR-0128-hyperscaler-architecture-invariants
  - ADR-0131-per-microservice-flat-layout
decision_owner: axis-cloud-secrets + ops-security
---

# ADR-MS-001: SecretReference namespace and rotation contract for cloud-secrets

## Context

- Pressure name: raw-secret blast-radius pressure.
- Every microservice needs credentials, signing keys, OAuth client secrets, and webhook signing material.
- The service PRD makes `cloud-secrets` the secret-manager substrate rather than another product surface.
- The service already exposes `SecretReference` read, list, rotate, revoke, encryption-key BYOK (ADR-0251 §D-10), attestation, audit, health, ready, and metrics endpoints.
- The local OpenAPI contract names `/secrets/{tenant}/{microservice}/{secret_name}/reference` as the read path.
- The local OpenAPI contract names `/secrets/{tenant}/{microservice}/references` as the inventory path.
- The local OpenAPI contract names `/secrets/{tenant}/{microservice}/{secret_name}/rotate` as the rotation path.
- The local OpenAPI contract names `/secrets/{tenant}/{microservice}/{secret_name}/revoke` as the revocation path.
- The local OpenAPI contract names `/tenants/{tenant}/namespace` as the namespace discovery path.
- The local OpenAPI contract names `/tenants/{tenant}/byok` as the tenant-owned key upload path.
- The local OpenAPI contract names `/attestation/{pack}/reports` as the compliance evidence path.
- The local OpenAPI contract names `/audit/query` as the signed evidence lookup path.
- The local AsyncAPI contract already emits `SecretCreated`, `SecretRotated`, `SecretRevoked`, `SecretAccessed`, `NamespaceProvisioned`, `NamespaceSealed`, `KekAttested`, `RotationOverdue`, and `RevocationPush`.
- Constraint name: no raw secret egress.
- Human principals must never receive SECRET-class material directly.
- Workload identities may resolve only references within matching tenant and microservice scope.
- Tenant operators may upload encryption-key BYOK after MFA and JIT elevation (ADR-0251 §D-10), but may not resolve raw secret values.
- CI may read sandbox metadata and run leak scans, but may not resolve real tenant secrets.
- Auditors may inspect signed events and pack assignment evidence, but may not resolve secret references.
- Constraint name: cell-local HSM custody.
- ADR-0043 requires OpenBao plus HSM per cell for custody, not central shared key storage.
- ADR-0009 requires tenant and region cell boundaries to remain visible at every resolver step.
- ADR-0008 requires data class annotation so secret metadata cannot drift into ordinary operational data.
- ADR-0003 requires every create, rotate, revoke, access, and attestation decision to emit audit-chain evidence.
- Constraint name: dependency fan-out pressure.
- `api-gateway`, `connector`, `feature-flags`, and payment or webhook surfaces depend on low-latency reference resolution.
- A slow resolver can turn a routine deploy or key rotation into a shared outage.
- The existing SLO set names p99 secret resolve latency <=100ms and p95 secret write latency <=200ms.
- The existing SLO set names HSM availability >=99.99% and audit completeness 100%.
- This ADR decides the service-level contract for references, namespace shape, policy denial, and rotation cadence.

## Decision

- Decision name: cell-scoped SecretReference custody.
- `cloud-secrets` will expose only typed `SecretReference` handles to callers.
- The handle format is `secretref:v1:{tenant_id}:{home_cell}:{microservice}:{purpose}:{secret_name}:{version}`.
- The handle must not contain ciphertext, token material, private key material, or reversible tenant identity.
- The resolver maps each handle to an OpenBao path `secret/{tenant_id}/{microservice}/{purpose}/{secret_name}`.
- The resolver maps each handle to an HSM-backed KEK id `kek/{home_cell}/{tenant_id}/{purpose}`.
- The resolver requires SPIFFE workload identity for machine resolution.
- The resolver requires `principal.tenant_id == resource.tenant_id`.
- The resolver requires `principal.microservice == resource.microservice` unless the reference is explicitly shared.
- Shared references require `resource.scope == "shared"` and `principal.microservice in resource.reader_allowlist`.
- Shared references are limited to public signing-key metadata, webhook verification material, and configured cross-service sidecars.
- Shared references may not carry database credentials, OAuth refresh tokens, private keys, or tenant-owned encryption-key BYOK material (ADR-0251 §D-10).
- Tenant namespace provisioning creates OpenBao mount, HSM KEK slot, rotation policy, audit sink, and revocation topic in one transaction.
- Namespace provisioning fails closed when any of the four custody surfaces is unavailable.
- Reference resolution TTL is <=60 seconds for dynamic secrets and <=300 seconds for signing-key public metadata.
- Rotation cadence is 30 days for high-risk OAuth or payment credentials.
- Rotation cadence is 90 days for ordinary service credentials.
- Rotation cadence is 180 days for public signing metadata.
- Emergency revocation push must reach subscribed sidecars within 30 seconds at p99.
- Any rotation overdue by more than 24 hours emits `RotationOverdue` and blocks new deploy promotion for the owning microservice.
- Any revocation failure emits `RevocationPush` with `outcome=failed` and opens Sev-2 when the secret remains live after 5 minutes.
- Any denied read emits an audit event with reason, tenant hash, microservice, data class, and policy fragment version.
- The service will use OpenBao Transit and PKI engines for envelope operations and certificate issuance.
- The service will use PKCS#11 HSM integration for KEK custody in regulated cells.
- The service will keep rotation policy ids stable across migrations so audit history remains queryable.
- Numeric threshold: p99 reference resolve latency <=100ms for warm cache.
- Numeric threshold: p95 secret write latency <=200ms excluding HSM cold recovery.
- Numeric threshold: PKI certificate issuance p95 <=1s.
- Numeric threshold: audit completeness target is exactly 1.0 for secret operations.
- Numeric threshold: HSM availability target is >=0.9999 over 30 days.

## Alternatives Considered

### Alternative 1: Store raw secret values in each consuming service

- Pros: lowest resolver latency for a single service.
- Pros: simple local boot path.
- Cons: every service becomes a secret custodian.
- Cons: rotation requires broad redeploy coordination.
- Cons: audit evidence fragments across service boundaries.
- Cons: violates ADR-0043 and the local tenant-scope Cedar policy.
- Rejected because raw-secret fan-out creates an unacceptable blast radius.

### Alternative 2: Central OpenBao cluster without per-cell HSM custody

- Pros: simpler operations for early development.
- Pros: fewer namespaces and fewer attestation reports.
- Cons: central outage blocks unrelated cells.
- Cons: residency boundaries become policy comments instead of runtime topology.
- Cons: regulated packs cannot prove local KEK custody.
- Cons: cross-cell requests increase tail latency during boot storms.
- Rejected because ADR-0009 and ADR-0043 require cell-aware custody.

### Alternative 3: Vendor secret managers per cloud provider

- Pros: mature managed services for AWS, Azure, and Google Cloud.
- Pros: lower direct HSM operations burden.
- Cons: provider-specific APIs leak into product code.
- Cons: multi-cloud and on-prem cells lose one contract.
- Cons: encryption-key BYOK (ADR-0251 §D-10) and evidence semantics differ per provider.
- Cons: tenant migration would require vendor-specific rewrites.
- Rejected because Oyatie needs one SecretReference contract across all cells.

### Alternative 4: Kubernetes Secrets as the canonical substrate

- Pros: available in clusters and easy for operators.
- Pros: simple deployment mechanics for non-regulated demos.
- Cons: not a sufficient audit, HSM, rotation, or encryption-key BYOK system (ADR-0251 §D-10).
- Cons: secrets are too close to runtime scheduling boundaries.
- Cons: namespace escape or controller compromise has high impact.
- Rejected because Kubernetes Secrets are delivery plumbing, not the custody authority.

### Alternative 5: Encrypted environment variables at deploy time

- Pros: fast process startup once deployed.
- Pros: no network resolver call during steady state.
- Cons: rotation waits for process restart.
- Cons: leaked crash dumps or env snapshots expose material.
- Cons: audit cannot distinguish read intent from process existence.
- Rejected because the service must support live revoke and live rotation.

## Consequences

### Positive

- Consumers get one typed reference format across Rust, TypeScript, Python, and operator tooling.
- Tenants get pack-aware custody without every microservice inventing secret policy.
- Operators can rotate, revoke, attest, and audit from one surface.
- Sidecars can cache short-lived material without becoming authoritative.
- Policy denials become signed evidence instead of silent boot or timeout failures.
- The service can support OpenBao, HSM, and future custody adapters behind one contract.
- Secret metadata can be indexed without storing raw values in search or analytics.
- The audit chain can reconstruct who requested resolution and which policy version decided it.

### Negative

- Boot and rotation paths now depend on `cloud-secrets` SLOs.
- HSM integration adds supply, certification, and failure-mode complexity.
- Incident response requires secret-specific runbooks, not generic application restart steps.
- Shared references require careful allowlist review to avoid accidental cross-service authority.
- encryption-key BYOK upload creates tenant-facing support burden for MFA, JIT elevation, and failed attestation (ADR-0251 §D-10).
- Cold-start cells need local namespace bootstrap before dependent services can become ready.
- Long-running batch jobs must tolerate revocation push while processing.

### Neutral

- Kubernetes Secret or CSI projection remains allowed as an ephemeral delivery mechanism.
- Vendor secret managers remain allowed as adapter implementation details in non-authoritative cells.
- Public certificate chains may be mirrored to cache, but private keys remain under this decision.
- Read-only health, ready, metrics, and public contract paths stay anonymous only when data class permits.
- Audit export may show salted tenant ids and pack assignments without showing raw tenant mapping.

### Follow-up work

- Add conformance tests for every `SecretReference` parser in SDK bindings.
- Add OpenBao namespace drift detection to `secrets/scorecards/`.
- Add revocation push replay fixtures for sidecars that are offline for more than 30 seconds.
- Add dashboard panels for `RotationOverdue` by tenant pack and microservice.
- Add evidence correlation from `/audit/query` to downstream service deploy promotions.
- Add a pack-specific attestation export for KR, EU, FedRAMP, and healthcare overlays.

## Implementation Notes

### Data Shapes

- `SecretReference` fields: `reference_id`, `tenant_id_hash`, `home_cell`, `microservice`, `purpose`, `secret_name`, `version`, `data_class`, `rotation_policy_id`, `created_at`, `expires_at`.
- `SecretReference.reference_id` uses the `secretref:v1` prefix and is opaque to consumers after parsing.
- `TenantNamespace` fields: `tenant_id_hash`, `home_cell`, `pack`, `openbao_mount`, `hsm_kek_id`, `audit_sink`, `sealed_state`, `created_at`.
- `RotationPolicy` fields: `policy_id`, `credential_class`, `cadence_days`, `max_overdue_hours`, `dual_publish_window_seconds`, `emergency_revoke_seconds`.
- `ByokUpload` fields: `tenant_id_hash`, `operator_principal_id`, `mfa_verified`, `jit_elevation_id`, `wrapped_kek`, `attestation_report_id`, `requested_pack`.
- `AuditQuery` filters: `tenant_id_hash`, `microservice`, `purpose`, `event_type`, `policy_version`, `from`, `to`, `pack`.
- `SecretAccessed` event fields: `reference_id`, `tenant_id_hash`, `principal_id`, `spiffe_id`, `microservice`, `decision`, `policy_fragment`, `latency_ms`.
- `SecretRotated` event fields: `reference_id`, `old_version`, `new_version`, `rotation_policy_id`, `dual_publish_until`, `initiator`, `evidence_id`.
- `RevocationPush` event fields: `reference_id`, `version`, `reason`, `subscribers`, `deadline_seconds`, `outcome`, `evidence_id`.
- `KekAttested` event fields: `hsm_kek_id`, `home_cell`, `pack`, `attestation_report_id`, `fips_level`, `valid_until`.

### API Endpoints

- `GET /secrets/{tenant}/{microservice}/{secret_name}/reference` returns a typed handle and metadata, never raw value.
- `GET /secrets/{tenant}/{microservice}/references` returns inventory filtered by Cedar tenant and auditor scope.
- `POST /secrets/{tenant}/{microservice}/{secret_name}/rotate` starts dual-publish rotation.
- `POST /secrets/{tenant}/{microservice}/{secret_name}/revoke` pushes revocation and blocks further resolution.
- `GET /tenants/{tenant}/namespace` returns namespace metadata and sealed state.
- `POST /tenants/{tenant}/byok` accepts tenant-owned wrapped key material after MFA and JIT validation.
- `GET /attestation/{pack}/reports` returns HSM and namespace evidence for auditors.
- `GET /audit/query` returns signed evidence windows for secret operations.
- `GET /health` checks process health only.
- `GET /ready` requires OpenBao, HSM, audit sink, and revocation topic readiness.
- `GET /metrics` exports counters and histograms without raw tenant ids.

### Cedar Policies

- `policy/tenant-scope.cedar` permits workload resolution only when tenant and microservice match.
- `policy/tenant-scope.cedar` permits shared references only when reader allowlist matches.
- `policy/tenant-scope.cedar` forbids human principals from receiving SECRET-class resources.
- `policy/auditor-scope.cedar` permits audit-chain export inside an authorized audit window.
- `policy/auditor-scope.cedar` forbids auditor resolution of secret references.
- `policy/ci-scope.cedar` permits sandbox reads and scan finding emission.
- `policy/ci-scope.cedar` forbids CI from resolving tenant secret references.
- `policy/public-read.cedar` permits health, ready, metrics, public contracts, and public docs only under safe data class.
- `policy/secret-isolation.md` remains the readable control map for tenant and microservice isolation.
- Deny decisions must emit `oya_cloud_secrets_tenant_unauthorized_read_attempt_total`.

### SLO Targets

- `secret-resolve-latency.openslo.yaml`: p99 <=100ms and target 0.99.
- `secret-write-latency.openslo.yaml`: p95 <=200ms and target 0.95.
- `pki-cert-issuance-latency.openslo.yaml`: p95 <=1s and target 0.95.
- `hsm-availability.openslo.yaml`: 30-day HSM availability >=99.99%.
- `audit-log-completeness.openslo.yaml`: target 1.0 for every operation sealed.
- `key-rotation-correctness.openslo.yaml`: target 1.0 for required rotations completed.

## Verification

- Unit test `secret_reference_parser_rejects_raw_material`.
- Unit test `secret_reference_parser_accepts_v1_handle`.
- Unit test `rotation_policy_rejects_overdue_threshold_above_24h`.
- Unit test `shared_reference_requires_reader_allowlist`.
- Cedar test `workload_can_resolve_same_tenant_same_microservice`.
- Cedar test `workload_cannot_resolve_other_microservice_private_reference`.
- Cedar test `tenant_operator_can_upload_byok_after_mfa_and_jit`.
- Cedar test `tenant_operator_cannot_resolve_secret_reference`.
- Cedar test `auditor_can_read_audit_export_inside_window`.
- Cedar test `auditor_cannot_resolve_secret_reference`.
- Integration test `get_secret_reference_returns_reference_only`.
- Integration test `rotate_secret_dual_publishes_versions`.
- Integration test `revoke_secret_pushes_subscriber_event`.
- Integration test `tenant_namespace_requires_openbao_hsm_audit_and_topic`.
- Integration test `byok_upload_requires_attestation_report`.
- Contract test `cloud-secrets.openapi.yaml_paths_match_router`.
- Contract test `cloud-secrets-events.yaml_messages_match_event_codec`.
- Load test `resolve_secret_reference_p99_under_100ms_warm_cache`.
- Load test `secret_write_p95_under_200ms_without_hsm_cold_recovery`.
- Load test `revocation_push_p99_under_30s`.
- Chaos test `hsm_unavailable_fails_ready_and_blocks_namespace_create`.
- Chaos test `audit_sink_backpressure_blocks_high_risk_mutation`.
- Replay test `offline_sidecar_replays_revocation_push_before_ready`.
- Metric `cloud-secrets-resolve-latency-p99`.
- Metric `cloud-secrets-write-latency-p95`.
- Metric `cloud-secrets-audit-completeness-ratio`.
- Metric `oya_cloud_secrets_rotation_overdue_total`.
- Metric `oya_cloud_secrets_tenant_unauthorized_read_attempt_total`.
- Metric `oya_cloud_secrets_hsm_availability_ratio`.
- Dashboard `dashboards/secret-resolution-rate.json`.
- Dashboard `dashboards/rotation-compliance.json`.
- Dashboard `dashboards/audit-emission-completeness.json`.
- Dashboard panel `SecretAccessed deny reasons by policy fragment`.
- Dashboard panel `RotationOverdue by tenant pack`.
- Dashboard panel `HSM availability by home cell`.
- Runbook check `runbooks/secret-rotation-failed.md` must cover rollback and dual-publish.
- Runbook check `runbooks/hsm-partition.md` must cover cell-local degraded mode.
- Runbook check `runbooks/openbao-seal-event.md` must cover namespace sealed state.
- Promotion gate blocks if audit completeness is below 1.0 for secret operations.
- Promotion gate blocks if any critical credential is overdue by more than 24 hours.

## References

- Oyatie ADR-0003: Audit chain and evidence emission.
- Oyatie ADR-0007: Cedar authorization policy and persona tier.
- Oyatie ADR-0008: Data use boundary.
- Oyatie ADR-0009: Cell architecture per tenant per region.
- Oyatie ADR-0037: Public API stability tiers and deprecation.
- Oyatie ADR-0043: Secrets management OpenBao and HSM per cell.
- Oyatie ADR-0128: Hyperscaler architecture invariants.
- Oyatie ADR-0131: Per-microservice flat layout.
- OpenBao documentation: Secrets engines, Transit engine, PKI engine, namespaces, and audit devices.
- OASIS PKCS#11 Cryptographic Token Interface Base Specification.
- NIST FIPS 140-3: Security Requirements for Cryptographic Modules.
- NIST SP 800-57 Part 1: Recommendation for Key Management.
- NIST SP 800-63B: Digital Identity Guidelines Authentication and Lifecycle Management.
- RFC 5280: Internet X.509 Public Key Infrastructure Certificate and CRL Profile.
- RFC 5869: HMAC-based Extract-and-Expand Key Derivation Function.
- RFC 8446: The Transport Layer Security Protocol Version 1.3.
- RFC 8941: Structured Field Values for HTTP.
- SPIFFE ID and SPIRE workload API documentation.
- Cedar policy language documentation.
- Google SRE Workbook: Alerting on SLOs and multi-window burn rates.
