---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-audit-chain
microservice: audit-chain
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M01-foundation
bominal_source: [ADR-0028, ADR-0003]
related_adrs: [ADR-0028, ADR-0003, ADR-0056, ADR-0105, ADR-0110, ADR-0117, ADR-0123, ADR-0131, ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
related_specs: [/specs/audit-chain-merkle-ed25519.json, /specs/per-microservice-flat-layout.json, microservices/audit-chain/audit-event-class-registry.json]
date: 2026-05-17
owner_team: axis-audit-chain
doc_status: published
---

# PRD-audit-chain: Cryptographic Audit-Chain Substrate

## Purpose

The `audit-chain` microservice is oyatie's substrate for non-repudiation of every state-changing event across every other µservice. It implements the Merkle-tree + Ed25519 model inherited 1:1 from Bominal ADR-0028 and the emission contract inherited from Bominal ADR-0003. It is the **evidence backbone** for every other oyatie µservice: every `EligibilityChanged`, `PromotionExecuted`, `RollbackExecuted`, `DataSubjectRequestExecuted`, `TenantOnboarded`, `OntologyEdgeWritten`, `WorkflowStepCompleted`, `MailDispatched`, etc. terminates in an audit-chain seal.

This µservice is **shared substrate**, not a hero product. It is consumed by every other oyatie µservice (each emits to audit-chain; verifier reads). Its existence is the precondition for oyatie's SOC 2 CC4.x / ISO 27001 A.5.28 + A.8.15 / HIPAA §164.312(b) / GDPR Art. 30 / KR PIPA Art. 29 / KR 전자문서법 Arts. 5–7 compliance posture.

This µservice inherits Bominal ADR-0028 (audit-chain Merkle + Ed25519) and ADR-0003 (audit emission contract) 1:1; oyatie-specific decisions overlay only where local jurisdiction or master-plan principles diverge (per `feedback_bominal_inheritance_precedence.md`).

## Tenant Value

- **Tenant Outcome 1 — Cryptographic non-repudiation.** Every state-changing event tied to a tenant has an Ed25519 signature + Merkle inclusion proof bound to a published periodic root; tenant can independently verify "this happened, was authored by this principal, has not been tampered with."
- **Tenant Outcome 2 — Regulator-ready evidence export.** Tenant raises a DPA / PIPC / OCR audit request; oyatie exports a signed, Merkle-rooted, time-bounded subset of the audit-chain scoped to the tenant + framework + engagement window; the auditor independently verifies without trusting oyatie.
- **Tenant Outcome 3 — Self-service forensic queries.** Tenant queries "show me every action against entity X in the last 90 days, with who/when/what" through the audit-query API; reads are themselves audit-emitted (audit-of-audits).
- **Internal Outcome 4 — Tamper detection at platform scale.** Internal mass-deletion attempts, key-compromise blast radius, or storage corruption are detected at the Merkle-root validation cadence, not at next-quarter-audit cadence.
- **Internal Outcome 5 — Substrate uniformity.** Every oyatie µservice writes via the same `AuditEmitter` port-trait; eliminates per-team divergence in "what counts as an auditable event."

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | workload µservice emitter | to call `emit(event)` and receive `{event_id, period_id, pack, tenant_partition, accepted_at, sealed=false}` synchronously within ≤100ms p99 | the calling business operation can commit + return without blocking on the asynchronous durable seal | emission | Must |
| FR-02 | sealing engine | to batch emitted events into a Merkle tree per `(tenant, period)` at a configurable cadence ≤1s | per Bominal ADR-0028 seal-latency target; tenants see roots in near-real-time | sealing | Must |
| FR-03 | sealing engine | to sign the Merkle root with the pack-resident HSM-backed Ed25519 key | the root carries an advanced electronic signature satisfying eIDAS Art. 26 + KR 전자문서법 Art. 5 | sealing | Must |
| FR-04 | verifier | to read an `(event_id, claimed_root, claimed_signature, claimed_inclusion_proof)` and return `verified: true | false` with structured failure reason | tenants + auditors + CI lanes can independently verify chain integrity | verification | Must |
| FR-05 | query API | to read `audit_records(tenant, time_range, event_type?, principal?, entity?)` with pagination | tenants run forensic queries on their own audit history | query | Must |
| FR-06 | retention cascade | to enforce per-(tenant, data_class, pack) retention windows (e.g., HIPAA 6y; KR PIPA 3y; default 2y) | regulatory retention obligations honoured without manual operator action | retention-cascade | Must |
| FR-07 | retention cascade | to soft-delete (mark for redaction) on DSR cascade from `cloud-secrets`/`tenancy` while preserving the Merkle proof of "this was redacted at <ts> on subject DSR" | GDPR Art. 17 / KR PIPA Art. 36 honoured without breaking chain integrity | retention-cascade | Must |
| FR-08 | every µservice integration | to consume a stable `oya-audit-chain-sdk` client with idiomatic Rust + future TS / Python bindings | uniform integration across oyatie + tenant workloads | emission, query | Must |
| FR-09 | HSM operator | to rotate the pack signing key with explicit overlap window + chain-of-trust transition record | scheduled key rotation (90d cadence) does not break verification of pre-rotation events | sealing | Must |
| FR-10 | auditor | to receive a frozen, Merkle-rooted evidence bundle scoped to (tenant, framework, time-range) signed by the pack key | external audit engagements (SOC 2, ISO 27001, HIPAA, PIPC, DPA) are answerable without granting raw production access | query | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| `emit(event)` synchronous latency | ≤20ms | ≤100ms | ≤500ms | Bominal ADR-0028 §"Emission latency target"; covers durable receipt + assignment of `event_id` and the period bucket; full Merkle seal completes asynchronously |
| Seal latency per `(tenant, period)` | ≤200ms | ≤1s | ≤3s | Bominal ADR-0028 target; period = 1s by default; configurable per-tenant |
| `verify(event_id, proof)` latency | ≤50ms | ≤200ms | ≤1s | proof-only verification; no full-tree replay |
| Query latency (single-tenant, 30d window) | ≤500ms | ≤2s | ≤5s | Postgres-indexed range scan |
| HSM signing throughput | — | ≥500 ops/s per HSM partition | — | per pack HSM envelope; verified at deploy |
| Sustained emission rate per cluster | — | ≥50k events/s | — | per Bominal ADR-0028 scale target; horizontally shardable per tenant |
| Retention-cascade scan completion | — | ≤24h for full per-pack sweep | — | runs daily; doesn't block emission |

### Security

- Signing keys are **HSM-backed** (OCI Cloud-HSM partition per pack); the private key never appears in process memory; signing calls are remote via PKCS#11 or KMIP.
- Every `emit` is authenticated via the caller's SPIFFE identity; SPIFFE → tenant_id binding validated server-side per `policy/tenant-scope.cedar`.
- Object-storage tier is WORM-locked (S3-compatible Object Lock in Compliance mode) for raw event records + sealed roots.
- Postgres index is read-replicated; primary writes are append-only at the SQL level (no UPDATE, no DELETE except via retention cascade RPC).
- `verify` is a pure-function read; never mutates state.
- Per Bominal ADR-0028 §"Key separation": pack-specific signing keys; no global key.
- The Merkle root for every period is itself published to a tamper-evident store (`tenant:oya-aggregate` Mimir series + GitHub-pinned manifest) so external observers detect chain forks without trusting oyatie.

### Audit + Compliance

- This µservice IS the audit. Every emission, sealing, verification, and retention-cascade action is itself audit-emitted (recursive; bootstrapped via pack-scoped `oya-self` chain).
- Retention defaults per pack:
  - pack-us-healthcare: 6y per HIPAA §164.316(b)(2).
  - pack-kr: 3y per KR PIPA + KR-FSS sector guidance (5y for financial-services tenants).
  - pack-eu: 2y default; bounded by purpose per ROPA.
  - pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa: per pack's local-law minimum.
- HSM key rotation cadence: 90d per ISO 27001 A.5.17 + Bominal ADR-0028.
- Per Bominal ADR-0028 §"Chain-of-trust on rotation": every key-rotation event itself signed by both the outgoing and incoming key during a 24h overlap window; chain-of-custody recorded on-chain.

### Availability + SLO

- Availability target: 99.99% monthly for the `emit` path (writes MUST never silently fail; if HSM/seal infra is unavailable, the write is queued to a durable WAL with degraded-mode emission that still produces an event_id + later-sealed receipt; the caller never blocks indefinitely).
- Availability target: 99.95% monthly for `verify` and query paths.
- RTO: ≤15 min. RPO: ≤1s (period-aligned).
- Self-observability: the audit-chain emits SLI for `emit_latency`, `seal_latency`, `verify_latency`, `hsm_avail`, `cross_period_root_publication_lag`; the SLO engine (`observability` µservice) gates this µservice's own promotion identically to every other µservice.

### DR posture (ADR-0343)

- Target: RTO ≤300s and RPO 0s for `emit`, sealing backlog replay, verifier root lookup, and regulator export metadata, matching manifest `dr.rto_p99_seconds=300` and `dr.rpo_p99_seconds=0`.
- Compliance-pack floors considered: EU-AI-ACT-2024-HIGH-RISK (RTO 1800s/RPO 300s, multi-region), HIPAA-2024 (3600s/300s, multi-region), KR-CSAP-v3.1 (3600s/900s, multi-region), SOC2-T2 (14400s/900s), PCI-DSS-L1-v4 (86400s/3600s), ISO27001-2022/SOX-404 (14400s/3600s), and KR-PIPA-2023-amendment (14400s/900s). Effective target is the stricter service posture: RTO 900s, RPO 1s, multi-region required.
- Failover runbook: `microservices/audit-chain/runbooks/chain-replay-from-snapshot-protocol.md`, matching manifest `dr.failover_runbook`; HSM/signature-specific recovery uses `microservices/audit-chain/runbooks/merkle-seal-recovery.md` and `microservices/audit-chain/runbooks/signature-verification-failure.md`.
- Multi-region active-active: yes, but chain locality remains per `(pack, tenant_partition)`; cross-pack chain merge remains forbidden.
- WHY: tenants and regulators keep independent Merkle proof verification through a region loss, and callers receive durable event IDs without waiting for regional seal infrastructure to recover.

### Capacity model (ADR-0340)

- Per-tenant baseline: 0.22 vCPU, 384 MiB RAM, 8 GiB storage allowance, 2 Valkey connections, 4 Postgres connections, and 4 outbound HTTP/HSM slots, matching manifest `capacity_model`.
- Scaling dimension: `per_message`, because manifest doctrine treats Merkle sealing, evidence writes, and retention obligations as emitted-audit-message shaped.
- Cell placement class: Tier-0 evidence substrate, matching manifest `capacity_model.cell_placement_class`; runtime placement maps to pod runtime Tier 1 because manifest `pod_runtime_tier=1`.
- Autoscaling boundary: minimum 2 emit receivers, 1 sealer, and 1 verifier per active pack/cell; maximum 12 emit receivers, 6 sealers, and 8 verifier/export workers per hot tenant partition before shard split is required.
- WHY: the model absorbs bursty fleet-wide state changes while preserving per-pack cryptographic continuity and preventing one tenant's audit flood from starving another tenant's seal cadence.

### Sustainability + cost attribution (ADR-0344)

- Every `emit`, `seal`, `verify`, query, retention-cascade, and export audit row emits `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, and `carbon_intensity_source` alongside the existing audit payload.
- Provider-routing affected by carbon: no for synchronous emit/seal/verify and regulator export deadlines; yes only for asynchronous replay, cold export assembly, and historical query jobs when the tenant's compliance pack has no realtime mandate.
- Per-tenant cost surface: the tenant FinOps dashboard rolls audit-chain spend and carbon by tenant, capability, provider, cell, and compliance_pack; regulator export bundles include the same attribution for the exported window.
- WHY: the audit substrate itself must prove CSRD, SB-253, and SEC climate-disclosure cost/carbon attribution without weakening the legal evidence path.

### API versioning posture (ADR-0342)

- Public API version model: verifier, query, export, and non-mesh emitter surfaces use the YYYY-MM-DD carrier triplet: `Oyatie-Version` header, `/v/<YYYY-MM-DD>/...` URL prefix, and `oyatie_version` proto3 field.
- SDK semver model: `oya-audit-chain-sdk` ships as major.minor.patch; date-versioned wire contracts can be supported by multiple SDK minor releases.
- Support window: last 3 public API versions for at least 180 days.
- Per-tenant pinning: yes for verifier/query/export APIs; emitters may pin only on north-south/non-mesh paths.
- Internal-mesh exemption: yes; direct gRPC between oyatie services remains governed by ADR-0145 mesh compatibility and does not carry public API date routing.

### Data residency

- Audit records inherit the source tenant's `jurisdiction_code` per ADR-0117 and `policy/data-residency.md`. Audit data **strictly stays in the source pack**; cross-pack replication is **forbidden** for cryptographic continuity (each pack has its own chain).
- Per Bominal ADR-0028 §"Chain locality": one chain per `(pack, tenant_partition)`; chains never merge across packs.
- Cross-pack export to a tenant-controlled archive is permitted only via the tenant-initiated DPA-recorded export RPC + receiving-tenant SCC.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`usecase` for new crates), layers used by this µservice: `kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-postgres`, `adapter-s3`, `adapter-hsm`, `rest`, `worker`, `sdk`, `app`.

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `emission` | `oya-audit-chain-emission-{kernel,domain,usecase,api,adapter,rest,sdk,app}` | Receive `AuditEvent` from any µservice; assign `event_id`; durable-write to WAL; emit `AuditEmitted` event; return `(event_id, period_id)` synchronously. | `AuditEvent`, `EventEnvelope`, `Period`, `Principal`, `EventClass` |
| `sealing` | `oya-audit-chain-sealing-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,adapter-hsm,worker,app}` | Batch events per `(tenant, period)`; build Merkle tree; sign root with HSM-backed Ed25519 key; publish root to Mimir + GitHub-pinned manifest; emit `SealMinted` event. | `MerkleTree`, `MerkleRoot`, `SealRecord`, `SigningKey`, `PackEpoch` |
| `verification` | `oya-audit-chain-verification-{kernel,domain,usecase,api,adapter,rest,sdk}` | Given `(event_id, claimed_proof)`, verify Merkle inclusion + Ed25519 root signature; emit `VerificationFailed` on tamper. | `MerkleProof`, `Verdict`, `RootRegistry`, `KeyResolver` |
| `query` | `oya-audit-chain-query-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,sdk}` | Tenant-scoped + Cedar-gated forensic queries; pagination; auditor JIT-token-scoped exports; every read is itself audit-emitted. | `AuditQuery`, `QueryResult`, `ExportBundle`, `AuditorEngagement` |
| `retention-cascade` | `oya-audit-chain-retention-cascade-{kernel,domain,usecase,api,adapter,worker}` | Periodic sweep per `(tenant, data_class, pack)` retention windows; soft-delete (redaction-while-keeping-proof) on DSR cascade; hard-delete after grace; emit `RetentionApplied`. | `RetentionPolicy`, `DsrCascade`, `RedactionToken`, `RetentionRun` |

Naming justification — `emission`:

```
NAME: oya-audit-chain-emission-<layer>
JUSTIFICATION:
- microservice = audit-chain: this µservice; ADR-0056 v4.1 flat BNF + ADR-0131 per-microservice
  folder; mirrors Bominal audit-chain BC token.
- bc-tokens = emission: primary BC for accepting external AuditEvent submissions, assigning
  event_id, durable-WAL'ing, returning receipt. ADR-0056 v4.1 BC-optionality rule honoured
  (sibling BCs sealing/verification/query/retention-cascade exist; explicit BC token required).
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port-traits + sealed-trait + entity types (AuditEvent, EventEnvelope, Period,
    Principal, EventClass). Zero I/O. Carries data_class annotations per Bominal ADR-0028.
  - domain: pure event-classification, envelope construction, period bucketing math.
  - usecase (per ADR-0106; replaces legacy 'application'): orchestrator accepting the inbound
    AuditEvent, deriving the EventEnvelope, invoking the WAL-write port, returning receipt.
  - api: protocol-neutral typed I/O contracts; consumed by rest/sdk.
  - adapter: WAL writer (Postgres + S3 hybrid), event-id minter.
  - rest: HTTP endpoint for non-mesh callers (tenant-side workloads).
  - sdk: Rust + future TS/Python client library shipped with idiomatic AuditEmitter wrapper.
  - app: composition-root binary.
- exemptions claimed: none.
```

Naming justification — `sealing`:

```
NAME: oya-audit-chain-sealing-<layer>
JUSTIFICATION:
- microservice = audit-chain.
- bc-tokens = sealing: BC for Merkle-tree construction + HSM-backed Ed25519 signing.
- layer = <layer>: 10-crate set with three backend-qualified adapters per ADR-0105 Amendment 3
  (*-adapter-<backend> pattern).
  - kernel: port-traits (MerkleEngine, SignerPort, RootPublisher, ObjectStoreWriter, IndexWriter);
    entity types (MerkleTree, MerkleRoot, SealRecord, SigningKey, PackEpoch).
  - domain: Merkle-tree math (RFC 6962-shaped SHA-256 Merkle tree with leaf-index + proof
    extraction); root chaining (each period's root commits to prior period's root, per
    Bominal ADR-0028 chain-of-roots).
  - usecase: per-(tenant, period) sealing orchestrator; bounded-batch builder.
  - api: typed I/O for worker + verification.
  - adapter: protocol-neutral; in-process Merkle build.
  - adapter-postgres: SealRecord index writer + integrity manifest table.
  - adapter-s3: WORM-locked Merkle-tree + signed-root blob writer.
  - adapter-hsm: PKCS#11 / KMIP integration to OCI Cloud-HSM partition; per-pack key handle.
  - worker: long-lived sealing-cycle daemon (period-driven).
  - app: composition root.
- exemptions claimed: none. -adapter-<backend> pattern matches ADR-0105.
```

Layer mapping per BC:

| BC | kernel | domain | usecase | api | adapter | adapter-postgres | adapter-s3 | adapter-hsm | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `emission` | ✅ | ✅ | ✅ | ✅ | ✅ | — | — | — | ✅ | — | ✅ | ✅ |
| `sealing` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — | ✅ | — | ✅ |
| `verification` | ✅ | ✅ | ✅ | ✅ | ✅ | — | — | — | ✅ | — | ✅ | — |
| `query` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — | — | ✅ | — | ✅ | — |
| `retention-cascade` | ✅ | ✅ | ✅ | ✅ | ✅ | — | — | — | — | ✅ | — | — |

Total crates introduced: **38** across the 5 BCs.

Existing crates `oya-audit-chain-domain`, `oya-audit-chain-file-adapter`, `oya-audit-chain-usecase` (referenced; not physically moved per task brief) are mapped into the new layout via a thin re-export shim under the `emission` + `sealing` BCs; they remain at their current paths until the M01-MIGR-audit-chain phase formally moves them.

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `AuditEmitter` | `oya-audit-chain-emission-kernel` | `-adapter` (Postgres WAL + S3 stage) | `AUDIT` (every emission) + variable (per source event payload) |
| `WalWriter` | `oya-audit-chain-emission-kernel` | `-adapter-postgres` (via sealing BC's `-adapter-postgres`) | `AUDIT` |
| `PrincipalResolver` | `oya-audit-chain-emission-kernel` | `-adapter` (SPIFFE → Principal) | `SENSITIVE_PIPA_ART23` (principal identity) |
| `MerkleEngine` | `oya-audit-chain-sealing-kernel` | `-adapter` (in-process Merkle build) | `AUDIT` |
| `SignerPort` | `oya-audit-chain-sealing-kernel` | `-adapter-hsm` (PKCS#11/KMIP → OCI Cloud-HSM) | `SECRET` (signing-key handle); `AUDIT` (signature emission) |
| `RootPublisher` | `oya-audit-chain-sealing-kernel` | `-adapter-s3` + Mimir publisher | `AUDIT` |
| `MerkleVerifier` | `oya-audit-chain-verification-kernel` | `-adapter` (pure-function proof verification) | `AUDIT` |
| `RootRegistry` | `oya-audit-chain-verification-kernel` | `-adapter` (reads published roots from Mimir + S3) | `AUDIT` |
| `KeyResolver` | `oya-audit-chain-verification-kernel` | `-adapter` (resolves period → public key from rotation history) | `INTERNAL_ONLY` (public key material) |
| `AuditQueryRepository` | `oya-audit-chain-query-kernel` | `-adapter-postgres` | `AUDIT` + variable per record |
| `ExportBuilder` | `oya-audit-chain-query-kernel` | `-adapter` (builds signed bundles) | `AUDIT` |
| `RetentionPolicyStore` | `oya-audit-chain-retention-cascade-kernel` | `-adapter` (reads per-pack retention matrix) | `INTERNAL_ONLY` |
| `DsrCascadeReceiver` | `oya-audit-chain-retention-cascade-kernel` | `-adapter` (consumes DSR events from `tenancy`/`cloud-secrets`) | `AUDIT` + `PII_IDENTIFYING` (subject identifier) |

Data-class enforcement: every kernel struct field carries `#[data_class(...)]`; the `oya-check-data-class` LEAN lane refuses unannotated fields at PR-time.

Cross-product rule: `audit-chain` MUST NOT import any other product µservice crate at any layer. All inbound flows are via the `AuditEmitter` port (SDK) or REST/gRPC; all outbound flows are events (`AuditEmitted`, `SealMinted`, `VerificationFailed`, `RetentionApplied`). LEAN-A2 CI lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice audit-chain` — dependency-direction
- `oya gate validate lean-a2 --microservice audit-chain` — cross-product-refusal
- `oya gate validate port-location --microservice audit-chain` — ports in kernel
- `oya gate validate layer-correctness --microservice audit-chain` — layer enum match
- `oya gate validate per-microservice-layout --microservice audit-chain` — ADR-0131 conformance
- `oya gate validate statelessness --microservice audit-chain`
- `oya gate validate shardability --microservice audit-chain`
- `oya gate validate authority-cohesion` — HG-AUDIT registers here

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine / DAG |
|---|---|---|---|
| `AuditEmitted` | every `emit(event)` call | sealing BC's batcher | seal-state-machine per `/specs/audit-chain-merkle-ed25519.json` |
| `SealMinted` | per-(tenant, period) Merkle root signed | `tenancy` (residency confirmation), `observability` (self-SLO), tenant-facing dashboards | — |
| `VerificationFailed` | `verify(event, proof)` returns `false` for a record claimed to be sealed | `grafana-oncall`, `ops-security` Sev-1 paging, `audit-chain`'s own incident channel | tamper-detected state machine |
| `RetentionApplied` | retention-cascade worker applies a soft/hard deletion | `tenancy` (DSR confirmation back to tenant), `audit-chain` (recursive audit-of-audit) | — |
| `KeyRotated` | HSM key rotated; new pack epoch | `verification` (refresh KeyResolver), every other µservice with a cached pack-key copy | key-rotation-overlap state machine |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `DataSubjectRequestRaised` | `tenancy` | `retention-cascade` | identify all chain entries for `(tenant, subject)`; mark for redaction; emit `RetentionApplied` |
| `TenantOnboarded` | `tenancy` | `emission` (config) | provision tenant partition in audit-chain index; initialise per-tenant pack epoch |
| `TenantOffboarded` | `tenancy` | `retention-cascade` | trigger retention-statutory final hold + post-statutory cleanup |
| `KeyRotationScheduled` | `cloud-secrets` | `sealing` | begin 24h overlap window; co-sign with outgoing + incoming keys |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `AuditEvent{event_id, tenant, principal, event_class, payload_sha, emitted_at}` | `audit_for→<source-entity>` | `emission` | self (recursive) |
| `SealRecord{period_id, tenant, root_hash, signature, signer_key_handle, signed_at}` | `seals→AuditEvent[]` | `sealing` | self |
| `RedactionToken{event_id, redacted_at, subject_hash, dsr_id}` | `redacts→AuditEvent` | `retention-cascade` | self |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `Microservice` (catalog) | `emission` (entity validation) | `filter(active=true)` to validate `event.source_microservice` is a known µservice |
| `Tenant` (catalog) | every BC | resolve tenant partition + retention overlay |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| AWS | CloudTrail + CloudTrail Lake | Per-account event capture; immutable log; integrity validation via digest | `docs.aws.amazon.com/awscloudtrail/` |
| Google Cloud | Cloud Audit Logs | Admin/Data/Policy/System audit logs; tamper-evident via Cloud KMS | `cloud.google.com/logging/docs/audit` |
| Azure | Activity Log + Microsoft Defender for Cloud | Resource + management plane audit | `learn.microsoft.com/en-us/azure/azure-monitor/essentials/activity-log` |
| IBM | IBM Cloud Activity Tracker + IBM Verify Trust | Application + identity audit | `cloud.ibm.com/docs/activity-tracker` |
| Splunk | Splunk Enterprise + Splunk Security Cloud | SIEM with cryptographic hash-chain audit option | `docs.splunk.com` |
| Datadog | Audit Logs (Datadog) | Resource + user-action audit | `docs.datadoghq.com/account_management/audit_logs/` |
| Sumo Logic | Audit Index | Audit-log retention + correlation | `help.sumologic.com` |

Key parity gaps to close (priority order):

1. **Multi-pack residency** — none of the cloud-native incumbents offer pack-pinned chains with cross-pack replication forbidden as a default; oyatie's advantage is tenant-controlled jurisdictional posture.
2. **Cryptographic verifiability open to tenants** — CloudTrail digests are AWS-validated; Cloud Audit Logs integrity is Google-attested; the tenant cannot independently verify without AWS/Google trust. oyatie publishes Merkle roots to a tamper-evident store; tenants verify with public keys.
3. **HSM-rooted Ed25519 signing** — most incumbents use SHA-256 digest chains; Ed25519 + HSM is the eIDAS-AdES posture for EU and the KR 전자문서법 posture for KR.
4. **Per-event Merkle inclusion proofs returned synchronously to tenants** — incumbents offer integrity over batches, not per-event proofs.

## Performance Targets

(See "Non-Functional Requirements / Performance" above; reiterated for hyperscaler-maturity gate registration.)

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| `emit` synchronous latency | ≤20ms | ≤100ms | ≤500ms | Bominal ADR-0028 |
| Seal latency per (tenant, period) | ≤200ms | ≤1s | ≤3s | Bominal ADR-0028 |
| `verify` latency | ≤50ms | ≤200ms | ≤1s | per published proof |
| Sustained emission rate | — | ≥50k events/s/cluster | — | horizontal shard scale |

Error budget:
- Monthly error budget for emission: 0.01% (≈4 min/month) — tighter than observability because emission is the load-bearing primitive for every other µservice's compliance posture.
- Burn-rate alarm on the emission path itself: 14.4× burn over 1h triggers Sev-1 page (audit-chain outage = compliance debt accruing).
- Error budget evidence and incident execution use the published SLO manifests in `microservices/audit-chain/slos/` plus `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/merkle-root-discrepancy-investigation.md`, and `microservices/audit-chain/runbooks/signature-verification-failure.md`.

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Emission rest is stateless; sealing-worker is leader-elected per-pack-partition; storage is `postgres` (index) + `object-storage` (raw events + sealed bundles).

**Active-active compatibility**: stateless-compatible for emission-rest + verification-rest + query-rest. Sealing-worker is leader-active-warm-standby per `(pack, tenant_partition)` shard.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Emission rate | 50k events/s | 500k events/s | emission-rest queue depth > 100ms p99 |
| Sealing throughput | 1k periods/s | 10k periods/s | sealing-worker backlog > 5s |
| Postgres index size | 200 GB hot | 2 TB hot (per shard) | shard split when single-shard exceeds 80% |
| Object-storage write | 100 MB/s | 1 GB/s | bucket-throughput SLO alert |

Scale-out policy:
- Kubernetes HPA: emission-rest + verification-rest + query-rest scale on CPU `>70%`; min 3 replicas, max 100 replicas.
- Sealing-worker: scale by tenant-partition shard count; one leader per shard; warm replicas via lease-based election.
- Postgres: per-pack primary + replica; tenant-partition shards via Citus or vanilla logical sharding when single-Postgres exceeds 2 TB.
- Pre-warmed pool: 3 standby emission-rest pods; cold-start budget ≤500 ms.

Cross-region story:
- Audit-chain is **strictly per-pack**. No cross-region replication of chain state. Each pack has its own HSM partition + Postgres + object-storage bucket + chain epochs.
- Cross-pack export (tenant-initiated, DPA-recorded) writes a signed bundle to a tenant-controlled receiving bucket; receiving bucket is NOT part of oyatie's chain.

Sharding:
- Tenant-partition: `hashed_tenant_id` mod `shard_count`; each shard owns its own period sequence + Merkle root chain.
- Period: 1-second buckets by default; tenant can request larger buckets via DPA (trades emit-to-seal lag for batch efficiency).
- `oya-check-shardability` CI lane verifies partition key presence.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | `emit(event)` returns a receipt within ≤100ms p99 under 50k events/s load | load-test plan in `microservices/audit-chain/test-plans/integration-test-strategy.md` |
| AC-02 | Sealing produces a Merkle root with valid Ed25519 signature within ≤1s of last event in the period | seal-cycle SLO in `microservices/audit-chain/slos/seal-cycle-latency.openslo.yaml` plus integration strategy |
| AC-03 | `verify(event, proof)` correctly classifies tampered events (mutated payload, mutated proof, mutated signature) | property-test plan in `microservices/audit-chain/test-plans/unit-test-strategy.md` |
| AC-04 | DSR cascade redacts target event within 30 days while preserving Merkle proof of redaction | timed e2e drill |
| AC-05 | HSM key rotation overlap window allows verification of both pre- and post-rotation events | rotation drill |
| AC-06 | Auditor export bundle is independently verifiable by a third-party tool using only public artifacts (root manifest + public key) | external verifier reference implementation |
| AC-07 | Cross-pack export is forbidden by default; allowed only with tenant SCC + receiving-bucket attestation | pack-routing CI lane |
| AC-08 | `buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice audit-chain` exit 0 | ADR-0131 lane |
| AC-09 | `buck2 build //:quality-lane-registry-authority-check # lane=authority-cohesion` exit 0; HG-AUDIT registered | ADR-0123 lane |
| AC-10 | HIPAA pack 6y retention verified by automated retention-cascade dry-run | per-pack retention-conformance lane |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Period bucket size default (1s vs 5s vs adaptive) — final landing | axis-audit-chain | resolved in IP-005 (1s default with adaptive opt-in) |
| 2 | Postgres index sharding (Citus vs vanilla logical) — pick at S-tier crossover | axis-audit-chain | TBA at S-tier capacity-model trigger |
| 3 | Tenant-side bundle verifier as open-source reference implementation | council-architecture | subsequent-to-M01-completion successor-IP ADR |
| 4 | Should sealing key rotation re-sign historical roots (re-seal) or rely on KeyResolver for time-bound public keys? | axis-audit-chain + ops-security | resolved in IP-011 (KeyResolver; do NOT re-sign — historical roots remain signed by their epoch's key) |
| 5 | What level of audit-chain self-observability is the right balance vs recursion explosion? | axis-audit-chain + axis-observability | resolved in IP-013 (selective: only state transitions + outage SLI emit; not every internal trace) |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0028 (Bominal) | Audit chain (Merkle + Ed25519) | inherited 1:1 |
| ADR-0003 (Bominal) | Audit emission contract | inherited 1:1 |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0110 | ChangeSet state machine | each IP is one ChangeSet |
| ADR-0117 | Cloud-native infrastructure | storage backend (Postgres + S3-compatible + Cloud-HSM) |
| ADR-0123 | Hyperscaler maturity claim gate | HG-AUDIT registers here |
| ADR-0131 | Per-microservice flat layout | this PRD authored natively under it |
| ADR-0140 (retired per ADR-0145) | Cedar policy enforcement | tenant + auditor + CI + public-read scopes |

## ADR-0162 / ADR-AUD-001 Update — Per-Cell Authoritative Slicing

ADR-0162 defines the tenant-slicing requirement; local ADR-AUD-001 refines the operational shape for audit-chain. The current µservice contract is:

- **Authoritative roots are per cell and per tenant partition.** A cell owns the append-only tree for the tenants pinned to that cell; no global tree is an authority for writes or disputes.
- **Per-pack shared shards** hold multi-tenant pack traffic through tenant-partition leaves; sovereign tenants may receive dedicated shards when the pack overlay explicitly declares it.
- **Sealing cadence:** hot leaf append p99 ≤100ms, full root completion p99 ≤1s, regional summary every minute, daily fleet witness for transparency only.
- **Current API projection:** tenant retrieval is implemented through `contracts/openapi/audit-chain.yaml` (`POST /query`, `GET /events/{event_id}/proof`, `POST /verify`, `GET /roots/{pack}/{period_id}`, and `GET /keys/{pack}/{epoch_id}`), not through a separate `/tenant/{tenant_id}/seals` endpoint.
- **No fleet-wide authoritative merge:** regional summaries and daily fleet witnesses are observability/transparency artifacts; sovereign-pinned tenants do not leak chain state across pack boundaries.

`oya gate validate audit-chain-per-tenant-slicing` enforces Cedar-gated retrieval, declared sovereign shard overlays, and tenant-only leaves inside tenant-partition proofs. See `/specs/per-tenant-audit-log-slicing-canonical.json` for the canonical declaration.

## ADR-0158 Update — Active-Active Disposition

Per ADR-0158 and ADR-AUD-001, audit-chain is active-active at the edge for emission/query/verification, while sealing authority is per `(pack, cell, tenant_partition)` leader shard. Append-only cells do not merge into an authoritative global root; failover transfers shard leadership with an audit-chain authority-transfer event. See `multi-region.md` for the full disposition statement.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `audit-chain` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `audit-chain` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 4 module pin(s) across 4 context(s).
- Scaling input: `per_message` with cell placement `Tier-0` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
