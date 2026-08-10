---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-cloud-secrets
microservice: cloud-secrets
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M01-foundation
bominal_source: []
related_adrs: [ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0120, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
related_specs: [/specs/per-microservice-flat-layout.json]
date: 2026-05-17
owner_team: axis-cloud-secrets + ops-security
doc_status: published
---

# PRD-cloud-secrets: OpenBao Secret Manager Substrate

## Purpose

The `cloud-secrets` microservice is oyatie's secret-manager substrate. It owns the OpenBao operator (HashiCorp Vault open-source fork at v2.x LTS), the **SecretReference** contract that every other µservice uses to consume secrets without raw exposure, the key-rotation scheduler, HSM integration, audit emitter (to the `audit-chain` µservice), and per-tenant namespace controller. It is the enforcement origin of the durable user directive (2026-05-12): **raw secrets must NEVER enter the repo, chat, or checkpoints; every secret reference uses the form `${openbao:secret/<path>}`, resolved at runtime via the SecretReference SDK.**

This µservice is **shared substrate**, not a hero product. It is consumed by every other oyatie µservice that needs a secret (provider credentials, API keys, signing keys, encryption keys, OTel tokens, mTLS material) and emits to `audit-chain` on every access. Its existence is the precondition for every other µservice meeting the security posture in `feedback_quality_performance_scalability_bar.md` and the residency posture in `microservices/observability/policy/data-residency.md` §"Per-Pack Overlay Sections".

Per ADR-0131 Cloud split, the umbrella `microservices/cloud/` is dissolved into focused µservices; `cloud-secrets` owns the OpenBao operator + SecretReference contract. Sibling cloud µservices (`cloud-iac`, `cloud-k8s`, `cloud-mesh`, `cloud-cdn`, `cloud-edge`) consume this substrate but do not duplicate its responsibilities.

This µservice has no Bominal equivalent; it originates in oyatie.

## Tenant Value

- **Tenant Outcome 1 — Zero raw-secret exposure.** Tenants' OAuth client secrets, signing keys, encryption keys, encryption-BYOK material live in tenant-namespaced OpenBao paths; no µservice ever holds a raw value at rest in code, config, or telemetry.
- **Tenant Outcome 2 — Auditable secret access.** Every secret read, rotation, revocation, and access attempt is sealed in the `audit-chain` µservice within ≤1s; tenants receive `secret_access_audit_export` per pack legal cadence.
- **Tenant Outcome 3 — Per-pack residency for secrets.** Each pack runs its own OpenBao instance + HSM partition; cross-pack secret replication is forbidden, satisfying KR PIPA Art. 28, GDPR Art. 44, HIPAA §164.312(a)(2)(iv), DPDPA §10.
- **Tenant Outcome 4 — encryption-BYOK + HSM signing.** Tenants in regulated packs can bring their own KEK or pin DEKs to a hardware HSM partition (OCI Cloud-HSM or Thales Luna); `cloud-secrets` orchestrates rotation, attestation, and emergency-revoke.
- **Internal Outcome 5 — SecretReference uniformity.** Every internal µservice consumes secrets via the SDK; no µservice ever sees a raw secret in code paths; the LEAN-A11 `oya-check-raw-secret-emission` lane refuses any commit that emits a credential-shaped string.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | µservice author | to reference a secret as `${openbao:secret/<path>}` in config | the raw value is never in the repo, image, or chat transcript | secret-reference-resolver | Must |
| FR-02 | SecretReference SDK consumer | to resolve a SecretReference at runtime to its current value | my µservice receives the live secret without persisting it | secret-reference-resolver | Must |
| FR-03 | key-rotation scheduler | to rotate a credential on its declared cadence (30d API keys, 90d signing keys, 365d KEK) | rotation conformance per ISO 27001 A.5.17 holds without operator action | key-rotation-scheduler | Must |
| FR-04 | per-tenant namespace controller | to provision an isolated OpenBao namespace per tenant on tenant-onboarding | per-tenant blast radius is bounded to the tenant's own namespace | per-tenant-namespace-controller | Must |
| FR-05 | HSM integration | to delegate KEK signing operations to OCI Cloud-HSM or Thales Luna partition | KEK material never exists in software memory | hsm-integration | Must |
| FR-06 | audit emitter | to emit a sealed `SecretAccessed` event for every secret-read | every access is auditable in `audit-chain` and queryable by auditors | audit-emitter | Must |
| FR-07 | openbao-operator | to manage OpenBao cluster lifecycle (HA, unseal, upgrade) via Kubernetes operator pattern | OpenBao runs as a managed substrate rather than a bespoke install | openbao-operator | Must |
| FR-08 | revocation API | to immediately revoke a leaked or compromised credential and trigger cascade rotation of derived secrets | leaked credentials cease to be valid within ≤5s of declaration | secret-reference-resolver + key-rotation-scheduler | Must |
| FR-09 | secret-leak detector | to scan PR diffs, chat logs, and commit history for credential-shaped strings and BLOCK before merge | the durable user directive (no raw secrets in repo/chat/checkpoints) is mechanically enforced | (cross-cutting; consumed by governance) | Must |
| FR-10 | external auditor (read-only, time-boxed) | to query the audit-chain for all `SecretAccessed` records over a window | SOC 2 / ISO 27001 / PIPA examiners can verify access patterns | audit-emitter | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Secret resolution latency (cache hit) | ≤2ms | ≤10ms | ≤25ms | SDK-local in-process cache with TTL ≤60s; OpenBao authoritative |
| Secret resolution latency (cache miss → OpenBao) | ≤8ms | ≤25ms | ≤80ms | mTLS + tenant-scoped policy eval |
| Secret rotation (per secret end-to-end) | ≤5s | ≤30s | ≤60s | rotate + propagate + cascade-rotate dependents |
| HSM signing operation | ≤15ms | ≤50ms | ≤200ms | OCI Cloud-HSM HSM partition; Thales Luna in pack-kr |
| Audit emission (SecretAccessed → audit-chain seal) | ≤300ms | ≤1s | ≤3s | per Bominal ADR-0028 audit-chain posture |
| Per-tenant namespace provisioning | ≤2s | ≤10s | ≤30s | initial namespace + per-µservice scope policies |

### Security

- **No raw secrets in repo/chat/checkpoints.** Mechanically enforced by the LEAN-A11 `oya-check-raw-secret-emission` lane (gitleaks + tartufo + custom oyatie patterns). PR-time BLOCKER.
- **SecretReference contract** is the only sanctioned mode of consuming a secret. Form: `${openbao:secret/<path>}`. The SDK resolves at runtime, holds in memory for the consumer's process lifetime only, and never logs the resolved value.
- **mTLS required** for every OpenBao client; per-µservice SPIFFE identity bound to a tenant-scoped policy.
- **HSM-backed KEK** for every pack in `regulated` posture (pack-kr, pack-eu, pack-us-healthcare, pack-ksa, pack-ae); software-only KEK only in non-regulated sandbox.
- **Per-tenant namespaces** in OpenBao isolate tenant credentials; cross-tenant reads are explicitly forbidden by both Cedar policy and OpenBao policy.
- **Emergency revoke** path: a credential can be revoked + cascade-rotated within ≤5s end-to-end; consumers' in-process caches honour OpenBao revocation push (server-sent invalidation).
- Tenant identifiers, secret paths, and access patterns themselves are `BEHAVIORAL_TENANT_PRODUCT` and `SENSITIVE_PIPA_ART23`-class; never logged in plaintext, never exposed across tenants.

### Audit + Compliance

- Every `SecretCreated`, `SecretRotated`, `SecretRevoked`, `SecretAccessed`, `NamespaceProvisioned`, `KekAttested` event emits to `audit-chain` (Merkle / Ed25519 per Bominal ADR-0028).
- Audit-chain seal latency ≤1s per event.
- Retention of audit events per pack: KR PIPA Enforcement Decree Art. 30 (≥1y; KR-FSS sector ≥5y), HIPAA §164.316(b)(2) (6y), GDPR Art. 30 (purpose-bounded), PCI-DSS v4.0 §10.5.1 (≥1y, 3mo immediately available).

### Availability + SLO

- Availability target for `secret-resolution` hot path: **99.99 % monthly** (≈4.3 min/month error budget). Resolution must remain available even when adjacent µservices are degraded — failure cascades through every consumer.
- Availability target for `secret-rotation` and `audit-emission`: 99.95 % monthly.
- RTO: ≤2 min (hot-path resolution). RPO: ≤1s (audit emission backlog acceptable; rotation schedule re-derivable from KV state).
- Self-observability: OpenSLO manifest at `microservices/cloud-secrets/slos/{secret-resolution,rotation-completeness,audit-emission-completeness}.openslo.yaml` (authored under the `observability` µservice gate per ADR-0139).

### DR posture (ADR-0343)

- Target: RTO ≤120s for hot-path secret resolution and RPO ≤1s for secret access/rotation audit emission, matching manifest `dr.rto_p99_seconds=120` and `dr.rpo_p99_seconds=1`.
- Compliance-pack floors considered: EU-AI-ACT-2024-HIGH-RISK (1800s/300s, multi-region), HIPAA-2024 (3600s/300s, multi-region), KR-CSAP-v3.1 (3600s/900s, multi-region), SOC2-T2 (14400s/900s), PCI-DSS-L1-v4 (86400s/3600s), ISO27001-2022/SOX-404 (14400s/3600s), and KR-PIPA-2023-amendment (14400s/900s). Effective target is the stricter cloud-secrets posture: RTO 120s, RPO 1s, multi-region required.
- Failover runbook: `secrets/runbooks/secret-substrate-failover.md`, matching manifest `dr.failover_runbook`; operational recovery fans into `secrets/runbooks/openbao-restart.md`, `secrets/runbooks/namespace-controller-restart.md`, and `secrets/runbooks/audit-emission-backlog.md`.
- Multi-region active-active: yes for tenant namespace control, audit bridge backlog, and read-side SecretReference resolution; pack-pinned HSM material never leaves its owning partition.
- WHY: tenant workloads can continue resolving secrets during regional degradation without exposing raw secret material or losing the audit trail for secret reads and rotations.

### Capacity model (ADR-0340)

- Per-tenant baseline: 0.16 vCPU, 512 MiB RAM, 4 GiB OpenBao metadata/log storage per namespace, 4 Valkey connections, 3 Postgres connections, and 6 outbound OpenBao/HSM/audit-chain slots, matching manifest `capacity_model`.
- Scaling dimension: `per_request`, because the dominant load driver is secret resolution, rotation, revocation, and audit emission volume.
- Cell placement class: Tier-1 substrate, matching manifest `capacity_model.cell_placement_class`; runtime placement maps to pod runtime Tier 1 because manifest `pod_runtime_tier=1`.
- Autoscaling boundary: minimum 2 resolver replicas, 1 namespace controller, 1 rotation worker, and 1 audit bridge per pack/cell; maximum 16 resolvers, 8 rotation workers, and 8 audit bridges per tenant namespace before namespace sharding is required.
- WHY: the model serves latency-sensitive secret reads while leaving enough worker headroom for emergency revoke and cascade rotation bursts.

### Sustainability + cost attribution (ADR-0344)

- Every `SecretCreated`, `SecretAccessed`, `SecretRotated`, `SecretRevoked`, `NamespaceProvisioned`, and `KekAttested` audit row emits `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, and `carbon_intensity_source`.
- Provider-routing affected by carbon: no for realtime secret resolution, emergency revoke, HSM signing, HIPAA emergency, or PCI realtime-fraud contexts; yes for off-peak attestation export and non-urgent rotation-report jobs.
- Per-tenant cost surface: the tenant FinOps dashboard exposes cloud-secrets cost/carbon by tenant, product, capability, provider, cell, and compliance_pack; the security admin surface links those aggregates to namespace and rotation cadence.
- WHY: regulated tenants need transparent secret-management cost and carbon evidence, but the resolver path cannot be delayed for carbon optimization.

### API versioning posture (ADR-0342)

- Public API version model: SecretReference admin, namespace, revocation, and auditor-access APIs use the YYYY-MM-DD carrier triplet: `Oyatie-Version` header, `/v/<YYYY-MM-DD>/...` URL prefix, and `oyatie_version` proto3 field.
- SDK semver model: SecretReference SDKs ship as major.minor.patch while preserving compatibility with date-versioned wire contracts.
- Support window: last 3 public API versions for at least 180 days.
- Per-tenant pinning: yes for public admin/revocation APIs and tenant SDK configuration.
- Internal-mesh exemption: yes; direct in-mesh gRPC resolution remains exempt so existing service-to-service SecretReference flows keep ADR-0145 behavior.

### Data residency

- Each pack runs its own OpenBao instance + HSM partition + Postgres-HA backend. Cross-pack secret replication is forbidden by default (mirrors the residency contract in `microservices/observability/policy/data-residency.md`). KEK material never leaves its HSM partition. Tenant DEKs encrypt with pack-pinned KEKs.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`usecase` rename), layers used by this µservice: `kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-openbao`, `adapter-hsm`, `adapter-postgres`, `adapter-audit-chain-bridge`, `rest`, `worker`, `sdk`, `app`.

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `secret-reference-resolver` | `oya-cloud-secrets-secret-reference-resolver-{kernel,domain,usecase,api,adapter,adapter-openbao,rest,sdk,app}` | parse `${openbao:secret/<path>}` references; resolve via OpenBao; in-process cache; revocation push consumer | `SecretReference`, `ResolvedSecret`, `CacheEntry`, `RevocationEvent` |
| `openbao-operator` | `oya-cloud-secrets-openbao-operator-{kernel,domain,usecase,api,adapter,app}` | Kubernetes operator pattern; manages OpenBao cluster lifecycle (deploy/unseal/upgrade/HA) | `OpenBaoCluster`, `UnsealState`, `RaftPeer`, `UpgradePlan` |
| `key-rotation-scheduler` | `oya-cloud-secrets-key-rotation-scheduler-{kernel,domain,usecase,api,adapter,worker,app}` | cron-driven rotation; cascade rotation; rotation-stuck detection | `RotationPolicy`, `RotationJob`, `CascadeDependency` |
| `hsm-integration` | `oya-cloud-secrets-hsm-integration-{kernel,usecase,api,adapter-hsm,app}` | PKCS#11 + KMIP integration with OCI Cloud-HSM / Thales Luna; KEK signing operations | `HsmPartition`, `KekHandle`, `AttestationReport` |
| `per-tenant-namespace-controller` | `oya-cloud-secrets-per-tenant-namespace-controller-{kernel,domain,usecase,api,adapter,app}` | OpenBao namespace per tenant; per-µservice scope policies; namespace lifecycle | `TenantNamespace`, `MicroserviceScope`, `NamespacePolicy` |
| `audit-emitter` | `oya-cloud-secrets-audit-emitter-{kernel,usecase,api,adapter-audit-chain-bridge,app}` | bridge OpenBao audit log → `audit-chain` µservice (Ed25519 seal) | `SecretAuditEvent`, `AuditChainBridgeMessage` |

Naming justification — `secret-reference-resolver`:

```
NAME: oya-cloud-secrets-secret-reference-resolver-<layer>
JUSTIFICATION:
- microservice = cloud-secrets: per ADR-0131 Cloud split.
- bc-tokens = secret-reference-resolver: primary BC; the SDK's lookup engine.
  Sibling BCs (openbao-operator, key-rotation-scheduler, hsm-integration,
  per-tenant-namespace-controller, audit-emitter) justify explicit BC token
  per ADR-0056 v4.1.
- layer = <layer>: one crate per layer per ADR-0105 13-value enum.
  - kernel: port-trait + sealed-trait + entity types (SecretReference,
    ResolvedSecret, CacheEntry, RevocationEvent). Zero I/O. Carries
    data_class annotations on every field (Bominal ADR-0028 + oya-check-data-class).
  - domain: pure SecretReference URI parsing, cache-TTL arithmetic, revocation
    invalidation logic.
  - usecase (per ADR-0106): orchestrators resolving a reference (parse → policy-eval →
    OpenBao → audit-emit → cache).
  - api: typed I/O contracts; consumed by rest/sdk.
  - adapter: protocol-neutral implementations.
  - adapter-openbao: backend-qualified adapter (per ADR-0105 Amendment 3
    *-adapter-<backend> pattern); implements OpenBaoClient against OpenBao 2.x HTTP API.
  - rest: HTTP handler/route layer (admin endpoints only; resolution is SDK-side).
  - sdk: client library (Rust + TS bindings via napi-rs + Python bindings via pyo3).
    THE primary integration surface — every µservice imports.
  - app: composition root (admin REST binary).
- exemptions claimed: none.
```

Naming justification — `openbao-operator`:

```
NAME: oya-cloud-secrets-openbao-operator-<layer>
JUSTIFICATION:
- microservice = cloud-secrets.
- bc-tokens = openbao-operator: sibling BC for cluster lifecycle.
- layer = <layer>: trimmed because lifecycle ops don't need rest/sdk
  (operator works in-cluster via controller-runtime).
  - kernel: port-trait + entity types (OpenBaoCluster, UnsealState, RaftPeer,
    UpgradePlan).
  - domain: pure raft-peer-election + unseal-quorum arithmetic.
  - usecase: orchestrators driving cluster lifecycle.
  - api: typed contracts for the CRD spec (OpenBaoCluster CRD).
  - adapter: kube-rs CRD reader + writer; Helm template emission for the OpenBao
    chart.
  - app: composition root (controller binary).
- exemptions claimed: none.
```

Layer mapping per BC (13-layer canonical enum):

| BC | kernel | domain | usecase | api | adapter | adapter-openbao | adapter-hsm | adapter-postgres | adapter-audit-chain-bridge | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `secret-reference-resolver` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | ✓ | — | ✓ | ✓ |
| `openbao-operator` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | ✓ |
| `key-rotation-scheduler` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | ✓ | — | ✓ |
| `hsm-integration` | ✓ | — | ✓ | ✓ | — | — | ✓ | — | — | — | — | — | ✓ |
| `per-tenant-namespace-controller` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | ✓ |
| `audit-emitter` | ✓ | — | ✓ | ✓ | — | — | — | — | ✓ | — | — | — | ✓ |

Total crates introduced: **34** (9 + 6 + 7 + 5 + 6 + 5; with shared `adapter-postgres` consumed by openbao-operator's `app`).

Port traits (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `OpenBaoClient` | `oya-cloud-secrets-secret-reference-resolver-kernel` | `-adapter-openbao` | `SECRET` (raw value at the moment of resolve only) |
| `SecretCache` | `oya-cloud-secrets-secret-reference-resolver-kernel` | `-adapter` (in-process LRU + TTL) | `SECRET` (transient in-memory) |
| `RevocationConsumer` | `oya-cloud-secrets-secret-reference-resolver-kernel` | `-adapter` (server-sent-events from OpenBao) | `AUDIT` (revocation events) |
| `OpenBaoClusterRepository` | `oya-cloud-secrets-openbao-operator-kernel` | `-adapter` (kube-rs CRD) | `INTERNAL_ONLY` |
| `RotationPolicyRepository` | `oya-cloud-secrets-key-rotation-scheduler-kernel` | `-adapter` | `INTERNAL_ONLY` |
| `RotationExecutor` | `oya-cloud-secrets-key-rotation-scheduler-kernel` | `-usecase` (uses `OpenBaoClient` + `HsmPartitionClient`) | `SECRET` |
| `HsmPartitionClient` | `oya-cloud-secrets-hsm-integration-kernel` | `-adapter-hsm` (PKCS#11 / KMIP) | `SECRET` (KEK never leaves HSM; signing op crosses port) |
| `NamespaceProvisioner` | `oya-cloud-secrets-per-tenant-namespace-controller-kernel` | `-usecase` | `BEHAVIORAL_TENANT_PRODUCT` (tenant identifiers) |
| `AuditChainBridgeClient` | `oya-cloud-secrets-audit-emitter-kernel` | `-adapter-audit-chain-bridge` | `AUDIT` |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane refuses unannotated fields at PR-time.

Cross-product rule: `cloud-secrets` MUST NOT import any other product µservice crate at any layer. Cross-µservice flows go through Workflow (events) or Ontology (entity reads). `audit-emitter` writes to `audit-chain` via the `audit-chain-bridge` port (not a direct crate import). LEAN-A2 enforces.

CI lanes that must green:

- Branch-protected `oya-ci-required` gate packet `lean-a1` — dependency-direction
- Branch-protected `oya-ci-required` gate packet `lean-a2` — cross-product-refusal
- Branch-protected `oya-ci-required` gate packet `lean-a11` — raw-secret-emission (BLOCKER)
- Branch-protected `oya-ci-required` gate packet `port-location`
- Branch-protected `oya-ci-required` gate packet `layer-correctness`
- Branch-protected `oya-ci-required` gate packet `per-microservice-layout`
- Branch-protected `oya-ci-required` gate packet `statelessness` (resolver only; operator is stateful by design)
- Branch-protected `oya-ci-required` gate packet `shardability`
- Branch-protected `oya-ci-required` gate packet `authority-cohesion` — HG-CLOUD-SECRETS registers here

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine / DAG |
|---|---|---|---|
| `SecretCreated` | new secret written to OpenBao via SDK | `audit-chain`, `tenancy` (per-tenant inventory) | secret-lifecycle FSM |
| `SecretRotated` | rotation scheduler completes a rotate | `audit-chain`, downstream consumers' cache-invalidation | secret-lifecycle FSM |
| `SecretRevoked` | revoke API invoked | `audit-chain`, downstream consumers (cache-flush + cascade-rotate) | secret-lifecycle FSM |
| `SecretAccessed` | every resolve | `audit-chain` (sealed) | — |
| `NamespaceProvisioned` | tenant onboarding → namespace created | `audit-chain`, `tenancy` | tenant onboarding FSM |
| `KekAttested` | HSM attestation snapshot every 24h | `audit-chain`, `governance` (compliance evidence) | — |
| `RotationOverdue` | scheduler detects a rotation past SLA | `grafana-oncall` (page), `governance` | — |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `TenantRegistered` | `tenancy` | `per-tenant-namespace-controller` | provision OpenBao namespace + scope policies |
| `TenantDeprovisioned` | `tenancy` | `per-tenant-namespace-controller` | seal namespace; preserve audit; schedule cryptographic erasure of DEKs (≤30d) |
| `MicroserviceRegistered` | `tenancy` | `per-tenant-namespace-controller` | provision per-µservice scope policy |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `Secret{path, version, data_class, rotation_policy_id}` | `belongs_to→TenantNamespace` | `secret-reference-resolver` | Ed25519 via `audit-emitter` |
| `RotationPolicy{secret_path, cadence_days, cascade_dependents}` | `governs→Secret` | `key-rotation-scheduler` | Ed25519 |
| `TenantNamespace{tenant_id, pack, openbao_namespace_path}` | `provisioned_for→Tenant` | `per-tenant-namespace-controller` | Ed25519 |
| `KekHandle{partition, alias, attestation_sha}` | `protects→TenantNamespace` | `hsm-integration` | Ed25519 |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `Tenant{tenant_id, pack, jurisdiction}` | `per-tenant-namespace-controller`, `secret-reference-resolver` | by tenant_id |
| `Microservice` | `per-tenant-namespace-controller` | per-µservice scope policy generation |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| HashiCorp | Vault Enterprise + HCP Vault | secret KV v2; transit; PKI; KMIP; namespace; auto-unseal; performance replication | `developer.hashicorp.com/vault/docs` |
| AWS | Secrets Manager + KMS + CloudHSM | per-region secret store; auto-rotation; KMS-CMK; CloudHSM PKCS#11 | `docs.aws.amazon.com/secretsmanager` |
| GCP | Secret Manager + Cloud KMS + Cloud HSM | per-region; replication policy; CMEK; HSM-protected keys | `cloud.google.com/secret-manager/docs` |
| Microsoft | Azure Key Vault (Standard + Premium + Managed HSM) | secret vault; CMK; FIPS 140-3 Managed HSM | `learn.microsoft.com/en-us/azure/key-vault` |
| Oracle | OCI Vault + Cloud HSM | per-region vault; HSM-backed master encryption keys | `docs.oracle.com/en-us/iaas/Content/KeyManagement` |
| 1Password | Secrets Automation + | developer-secret distribution; per-service tokens; SCIM | `developer.1password.com/docs/connect` |
| Doppler | Doppler workplace + service tokens | per-environment secret config; CLI + SDK; webhook rotation | `docs.doppler.com` |
| Infisical | Infisical OSS | open-source KV; per-environment; integrations | `infisical.com/docs` |
| Akeyless | Akeyless Vault + DFC | DFC (Distributed Fragments Cryptography); zero-knowledge | `docs.akeyless.io` |

Key parity gaps to close (ordered by priority):

1. **SecretReference contract uniformity** — none of the competitors mandate a single SDK-resolved reference form across all internal consumers; this is oyatie's differentiator (mechanical enforcement via LEAN-A11 raw-secret-emission BLOCKER).
2. **Per-pack OpenBao + HSM partition** — HashiCorp Performance Replication is the closest analogue but ships SaaS-tinged; oyatie's per-pack isolation is invariant for residency.
3. **Audit-chain integration** — competitors emit audit to CloudTrail/Cloud Audit Logs/AWS CloudWatch/Azure Monitor; oyatie emits to `audit-chain` (Merkle + Ed25519 sealed) for cryptographic non-repudiation.
4. **OSS-first, no vendor coupling** — HashiCorp's BSL re-license (2023) is the rationale for the OpenBao fork; oyatie commits to OpenBao + OSS-only.

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Secret resolution (cache hit) | ≤2ms | ≤10ms | ≤25ms | SDK-local LRU + TTL ≤60s |
| Secret resolution (cache miss) | ≤8ms | ≤25ms | ≤80ms | mTLS + tenant policy eval + OpenBao KV read |
| Secret rotation | ≤5s | ≤30s | ≤60s | rotate + propagate revocation + cascade-rotate |
| HSM signing | ≤15ms | ≤50ms | ≤200ms | OCI Cloud-HSM partition |
| Audit emission | ≤300ms | ≤1s | ≤3s | SecretAccessed → audit-chain Ed25519 seal |
| Namespace provisioning | ≤2s | ≤10s | ≤30s | tenant onboarding |
| Revocation cascade end-to-end | ≤2s | ≤5s | ≤10s | revoke → consumer cache flush |

Error budget:
- Monthly error budget for hot-path resolution: 0.01 % (≈4.3 min/month).
- Monthly error budget for rotation + audit: 0.05 % (≈22 min/month).
- Burn-rate alarm on resolver: 14.4× burn over 1h triggers page.

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `stateless | postgres | object-storage | persistent-volume | mixed` → **`mixed`**. Rationale: `secret-reference-resolver` SDK is stateless (in-process cache derives from OpenBao); OpenBao itself is stateful (Raft + Postgres backend); HSM is stateful (per-partition keys).

**Active-active compatibility**: `stateless-compatible` for resolver SDK; OpenBao uses Raft consensus (5-node HA per pack); Postgres uses Patroni-HA.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Resolver SDK qps (per-µservice process) | 1k qps | 10k qps | cache-hit-rate < 95 % |
| OpenBao read qps (per cluster) | 5k qps | 50k qps | OpenBao p99 > 25ms |
| OpenBao write qps (per cluster) | 500 qps | 5k qps | Raft leader CPU > 70 % |
| HSM signing qps | 100/s | 1000/s | HSM partition queue depth |
| Audit emission throughput | 10k events/s | 100k events/s | audit-chain backlog > 1s |
| Active tenants per OpenBao cluster | 1k tenants | 10k tenants | namespace count |

Scale-out policy:
- Kubernetes HPA: resolver SDK is in-process (no scale-out; scales with the consumer µservice).
- OpenBao: Raft cluster fixed at 5 nodes per pack; add per-pack clusters as tenants grow.
- HSM: add HSM partitions per pack as KEK count grows.
- Audit emission worker: scales on event-bus backlog.

Cross-region story:
- M01 launch: pack-kr only (OCI ap-seoul-1).
- Post-M01 expansion: per-pack OpenBao + HSM + Postgres-HA; cross-pack replication forbidden.

Sharding:
- OpenBao namespaces partition by tenant.
- HSM partitions partition by pack.
- `oya-check-shardability-cli` lane verifies partition-key presence.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | A `${openbao:secret/<path>}` reference resolves to the live value via the SDK within p99 ≤25ms | bench under `microservices/cloud-secrets/tests/bench/resolution-latency.rs` |
| AC-02 | A rotated secret invalidates every consumer's cache within p99 ≤2s | drill under `tests/e2e/cascade-rotation.rs` |
| AC-03 | A revoked secret is unresolvable from every consumer within p99 ≤5s | drill under `tests/e2e/emergency-revoke.rs` |
| AC-04 | The LEAN-A11 raw-secret-emission packet refuses a PR introducing a credential-shaped string | Branch-protected `oya-ci-required` evidence shows the `lean-a11` fixture check exits non-zero for a credential-shaped fixture |
| AC-05 | An HSM-signing operation completes within p99 ≤50ms | bench against OCI Cloud-HSM in pack-kr |
| AC-06 | A new tenant's namespace provisions within p99 ≤10s on `TenantRegistered` | event-driven e2e test |
| AC-07 | Every `SecretAccessed` event seals in `audit-chain` within p99 ≤1s | end-to-end audit emission drill |
| AC-08 | `helm install` of pack-kr overlay reaches `Ready` on a kind cluster within 10 min | CI lane `oya-cloud-secrets-iac-smoke` |
| AC-09 | Branch-protected `oya-ci-required` evidence shows the `per-microservice-layout` packet passes for `cloud-secrets` | ADR-0131 lane |
| AC-10 | Branch-protected `oya-ci-required` evidence shows the `authority-cohesion` packet passes with HG-CLOUD-SECRETS registered | ADR-0123 lane; HG-CLOUD-SECRETS registered |
| AC-11 | Emergency-revoke drill propagates revocation across 100 consumers within p99 ≤5s end-to-end | timed chaos drill |
| AC-12 | A KEK attestation report is produced every 24h and sealed in audit-chain | attestation cron evidence |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Per-pack HSM vendor — OCI Cloud-HSM universal vs Thales Luna per regulated pack | ops-security + ops-finance | resolved in IP-005 |
| 2 | Cache TTL ceiling — 60s default vs per-secret-class override | axis-cloud-secrets | resolved in IP-003 |
| 3 | Revocation push transport — OpenBao server-sent-events vs WebSocket vs pub/sub | axis-cloud-secrets | resolved in IP-004 |
| 4 | KEK rotation cadence — 365d default vs per-pack regulatory ceiling | privacy-reviewer + ops-security | per-pack overlay, see policy/data-residency.md |
| 5 | encryption-BYOK material acceptance — accept tenant-supplied KEK wrapped under our KEK-of-KEKs, or only HSM-generated? | architecture-reviewer | ADR successor-IP |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | usecase rename | new crates use `usecase` |
| ADR-0117 | Cloud-native infrastructure | residency posture |
| ADR-0120 | Rust-first on-prem tooling | OpenBao operator Rust-first |
| ADR-0139 | Agentic SLO-gated promotion | observability gates apply |
| ADR-0131 | Per-microservice flat layout | this PRD authored under it; Cloud split origin |
| ADR-0132 | Industry-vertical unbundle policy | sibling cloud µservices |
| ADR-0133 | (Cloud split formalisation) | this µservice scaffolds under it |
| ADR-0123 | Hyperscaler maturity claim gate | HG-CLOUD-SECRETS registers here |
| ADR-0116 | Retire external agent-coordination tooling | oya vcs primitives throughout |

## ADR-0164 Update — Sovereign Cloud / Air-Gapped Deployment Variant

Per ADR-0164 (2026-05-18), the cloud-secrets µservice ships a per-pack air-gap variant. See `multi-region.md` for the full statement.

Highlights:
- **No external KMS dependency** — all KMS code paths replaced by OpenBao Transit secrets-engine. Cloud KMS adapter ABSENT from air-gap pack image builds.
- **HSM-backed OpenBao seal** — OpenBao auto-unseal uses in-cell HSM partition (PKCS#11). Quorum recovery (Shamir 5-of-9 default; per-pack overlay).
- **key-custody-BYOK + sovereign-tenant key custody** — sovereign tenant may bring its own HSM-generated KEK; cloud-secrets accepts the KEK wrapped under cell KEK-of-KEKs. encryption-BYOK material is HSM-stored; never exported.
- **In-cell HSM partition** — per-pack: Thales Luna (KSA / EU-sovereign), financial-grade HSM (KR FSC), FIPS 140-3 L4 (US-Gov).

Branch-protected `oya-ci-required` gate packet `air-gap-overlay` enforces (a) air-gap packs contain no external KMS adapter binary, (b) OpenBao auto-unseal binds to in-cell HSM, (c) encryption-BYOK paths use HSM-wrapped material only.

## ADR-0158 Update — Single-Region Disposition

Per ADR-0158 (2026-05-18), the cloud-secrets µservice is declared `single_region`. Secret material does not cross region. Cross-region replication is forbidden by construction; failover is intra-region only via OpenBao Raft + Patroni HA. See `multi-region.md` for the full disposition statement.

## Current authority

- Canon source: `registry/stores/design-store.json` and
  `registry/stores/instructions-store.json` (`D-SSOT-CURRENT-TRUTH`,
  `D-AUTHORITY-CONVERSATION`, `D-CLOUD-NATIVE`, `D-CICD-AUTHORITY`, and
  `D-GOVERNANCE-CENTRAL`) plus `specs/masterplan.json` for planning projection.
- Merge/gate authority: branch-protected GitHub Actions required context
  `oya-ci-required` is the live blocker until the owned `oya-ci` cutover reuses
  the same shared Rust gate logic. Retired local verifier/gate wrappers, dev-entrypoint flows, Cargo-only
  checks, shell scripts, and legacy build-server mirrors are
  non-authoritative unless explicitly re-homed through the cloud-ci pipeline.
- Delivery authority: Kubernetes/cloud-native services, controllers, APIs, and
  declarative manifests are canonical. ArgoCD/GitOps consumes signed
  declarative state; manual `kubectl apply`, Helm CLI deploys, and local
  operator scripts are break-glass diagnostics only, not canonical procedure.
- ADR-0346/ADR-0349 text below preserves historical control intent only where
  it does not conflict with the current authority chain above.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — legacy CI-mirror control intent only. The former local verifier authority wording is superseded for `cloud-secrets`; the branch-protected `oya-ci-required` context is the live required gate, and reusable Rust gate logic must be re-homed into cloud-ci / owned `oya-ci` rather than revived as local CLI authority.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-retired-vocab-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — legacy self-hostable substrate control intent only. The retired build-server bridge is not a parallel merge authority for `cloud-secrets`; GitHub Actions `oya-ci-required` remains the live required context until owned `oya-ci` cutover. ArgoCD/GitOps remains the declarative CD direction and replaces manual `kubectl apply` or Helm CLI deploys as canonical procedure.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `cloud-secrets` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `cloud-secrets` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 6 module pin(s) across 4 context(s).
- Scaling input: `per_request` with cell placement `Tier-1` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
