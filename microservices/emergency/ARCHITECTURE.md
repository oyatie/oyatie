# ARCHITECTURE — Emergency Department Information System (ED-IS)

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

Microservice slug: `emergency`
Layer: L7-application
Authority: ADR-0332 (in flight) | ADR-0131 | ADR-0132 | ADR-0145 | ADR-0248 | ADR-0251 | ADR-0253
Status: scaffold-authored 2026-05-21

---

## 1. Architectural Stance

ED-IS adopts the canonical oyatie µservice architecture under the 13-layer enum (ADR-0105) and the Amazon-shape cellular topology (ADR-0248). It is single-concern + flat per ADR-0131 / 0132 — no internal sub-bundle, no suite. Service code is Rust-strict (no Python, no JS app logic; per `feedback_rust_strict_only_no_python_2026_05_20`), IaC is OpenTofu HCL (not Terraform), policy is Cedar, contracts are OpenAPI + AsyncAPI + proto3, SLOs are OpenSLO.

The architecture is structured around three orthogonal axes:

- **Domain axis** — 17 bounded contexts, each owning one aggregate root + one event prefix + one Cedar policy slice + one SLO slice.
- **Topology axis** — cells of Tier 0..4 per ADR-0248, with shuffle sharding for cross-tenant isolation.
- **Surface axis** — REST / AsyncAPI / gRPC interfaces, with internal-only direct-gRPC peering per ADR-0145.

Each axis is independently versioned and independently observable.

---

## 2. Module / Crate Layout

ED-IS service code lives under `src/`. Per ADR-0131 (per-µservice flat layout), `src/` is the canonical code root with NO intermediate "internal", "core", "app" folder layer. Crates are flat siblings.

```
microservices/emergency/
  src/
    crates/
      emergency-triage/              # ESI 5-level engine + reassessment
      emergency-trackingboard/       # Board projection + fanout
      emergency-protocol/            # Trauma/Stroke/STEMI/Sepsis protocols
      emergency-mci/                 # Mass-casualty mode + START/SALT
      emergency-emshandoff/          # Prehospital + bedside handoff
      emergency-registration/        # Quick-reg + pre-arrival + walk-in
      emergency-orderentry/          # Rapid CPOE + protocol order sets
      emergency-documentation/       # Template-driven note authoring
      emergency-disposition/         # Admit/transfer/discharge/AMA/expired
      emergency-boarding/            # Admitted-but-held tracking
      emergency-lwbs/                # Left-without-being-seen tracking
      emergency-metrics/             # Door-to-X KPIs
      emergency-bedcontrol/          # Bed grid authority
      emergency-communication/       # Multi-disciplinary message board
      emergency-roomassignment/      # Rule + AI-assisted placement
      emergency-traumaregistry/      # TQIP/NTDB feed
      emergency-disasterresponse/    # ICS activation + facility status
      emergency-api-rest/            # REST surface (axum)
      emergency-api-async/           # AsyncAPI surface (NATS/JetStream)
      emergency-api-grpc/            # gRPC surface (tonic)
      emergency-domain/              # Aggregate roots + events
      emergency-application/         # Use-case orchestrators
      emergency-infrastructure/      # Repos, projections, adapters
      emergency-policy/              # Cedar evaluator wiring
      emergency-observability/       # OpenTelemetry + structured logs
      emergency-runtime/             # Binary entry, cell topology, health
      emergency-test-support/        # Test doubles + fixtures
    Cargo.toml
  contracts/
    openapi.yaml
    asyncapi.yaml
    proto/emergency.proto
  policies/
    *.cedar
  slos/
    *.openslo.yaml
  iac/
    aws-guest/
    oci-guest/
    oci-always-free/
    on-prem/
    colo/
    oyatie-cloud/
  decisions/
    ADR-MS-001-triage-engine.md
    ADR-MS-002-mass-casualty-mode.md
  implementation-plans/
    IP-001..IP-010
  PRD.md
  ARCHITECTURE.md
  README.md
  manifest.json
  supported-oses.json
  competitor-parity-matrix.md
```

### 2.1 Crate Dependency Rules

- `emergency-domain` depends on nothing inside ED-IS except shared canonical types (`oya-cell-domain` and friends).
- `emergency-application` depends on `emergency-domain` only.
- `emergency-infrastructure` depends on `emergency-domain` + concrete adapter crates.
- `emergency-api-*` depend on `emergency-application`.
- `emergency-runtime` depends on `emergency-api-*` + `emergency-infrastructure`.
- Per-context crates (`emergency-triage`, `emergency-protocol`, etc.) live as cleanly-bounded slices that wire to `emergency-domain` aggregates.

Inward-only dependency direction is CI-enforced via the canonical dependency-seam check (`oya-check-dependency-seam`).

---

## 3. Layered View (13-layer ADR-0105)

| Layer | Responsibility | Crate / file |
|-------|----------------|--------------|
| L0 — substrate primitives | Time, UUID, error types | shared `oya-*` substrate crates |
| L1 — protocol | HTTP/3, QUIC, gRPC framing | `emergency-runtime` (hyper + quinn + tonic) |
| L2 — auth | mTLS, identity, capability tokens | `emergency-runtime` (mtls bootstrap), `identity` µservice client |
| L3 — gate | Cedar evaluation | `emergency-policy` |
| L4 — domain | Aggregates + events | `emergency-domain` |
| L5 — application | Use-case orchestrators | `emergency-application` |
| L6 — infrastructure | Repos, projections, adapters | `emergency-infrastructure` |
| L7 — api | REST + AsyncAPI + gRPC surfaces | `emergency-api-*` |
| L8 — observability | OpenTelemetry + logs | `emergency-observability` |
| L9 — runtime | Binary, cell topology | `emergency-runtime` |
| L10 — contract | OpenAPI / AsyncAPI / proto3 | `contracts/` |
| L11 — slo | OpenSLO definitions | `slos/` |
| L12 — iac | OpenTofu modules | `iac/` |

---

## 4. Concurrency Model

### 4.1 Tokio Runtime

A single multi-thread Tokio runtime is bootstrapped at `emergency-runtime` start. CPU-bound triage rule evaluation (Triage / Protocol) is offloaded to a Rayon thread pool for predictable latency on board state changes.

### 4.2 Backpressure

- AsyncAPI publishing uses NATS JetStream with bounded-queue backpressure.
- Tracking board fanout uses a per-zone broadcast channel with a `recent_only` policy on slow consumers (preferring the latest state to lossless delivery during peak load).
- Order routing to downstream µservices is at-least-once with idempotency keys.

### 4.3 Critical-section Locks

- BedSlot state transitions take a per-cell write lock via PostgreSQL row-level lock plus a process-level mutex around the projection.
- Protocol activation is single-flight per encounter (idempotent on (encounter_id, protocol_type)).
- MCI activation is single-flight per facility (idempotent on (facility_id, incident_id)).

---

## 5. Data Model

### 5.1 OLTP Storage

PostgreSQL is the system of record for OLTP. Tables (one per aggregate) plus event-projection tables:

- `triage_encounter` — append-only by sequence.
- `board_lane`, `board_cell` — projection of bed grid + patient placement.
- `protocol_activation` — append-only.
- `mci_activation`, `mci_patient` — append-only with tag-number reconciliation columns.
- `ems_handoff` — append-only.
- `registration` — mutable but versioned (history table).
- `order_set`, `order_entry` — append-only.
- `ed_note` — versioned with amendment chain.
- `disposition` — append-only.
- `boarding_hold` — single row per (encounter_id), updated as state changes.
- `lwbs_record` — append-only.
- `metric_snapshot` — rolling window (TTL-pruned).
- `bed_grid`, `bed_slot` — projection.
- `comm_thread`, `comm_message` — append-only.
- `room_assignment_decision` — append-only.
- `trauma_registry_record` — append-only.
- `ics_activation`, `facility_status` — append-only.

### 5.2 Event Store

Per `audit-chain` µservice integration, every domain event is also persisted in the event-stream substrate. The OLTP tables are projections of the events for query convenience, with the event stream as the canonical record.

### 5.3 Search / Read Models

- Live tracking board state is held in Valkey with PostgreSQL as the durable backing store. Read consistency is bounded-staleness ≤ 250 ms.
- Metric snapshots are projected to TimescaleDB / time-series storage in the `observability` substrate.
- Historical / analytic queries go to `data-warehouse`.

### 5.4 PHI Encryption

Per HIPAA + HITRUST, all PHI fields are envelope-encrypted via `cloud-kms`. Tenant-pack-driven BYOK applies — when `provider_credential_mode = byok` for a tenant, the cell uses tenant-supplied KMS keys.

---

## 6. Eventing

### 6.1 Bus Choice

NATS JetStream is the primary inter-µservice eventing substrate. Streams:

- `ed.*` — the ED-IS owned event prefix.
- Consumer durable subscriptions per downstream µservice.

### 6.2 Idempotency

Every published event carries:

- `event_id` (UUIDv7, lexicographically time-ordered)
- `idempotency_key` (per-aggregate, e.g., `<encounter_id>:<sequence>`)
- `tenant_id`
- `cell_id`
- `trace_id` (OpenTelemetry)
- `causation_id` / `correlation_id`

Consumers MUST be idempotent on `idempotency_key`.

### 6.3 Schema Governance

AsyncAPI schemas under `contracts/asyncapi.yaml` are versioned semantically. A breaking change requires an ADR, a version bump, and a sunset window, per `feedback_no_silent_regression`.

---

## 7. Cellular Topology (ADR-0248)

ED-IS is deployable across the Tier 0..4 cell hierarchy:

- **Tier 0 — control cell.** Holds the cell catalog, the cell-promotion gate, and cross-cell coordination. ED-IS is NOT in Tier 0.
- **Tier 1 — substrate cell.** Pre-prod ED-IS instances and integration test cells live here.
- **Tier 2 — single-tenant cell.** Customer hospital tenant gets a dedicated ED-IS cell.
- **Tier 3 — pooled cell.** Smaller tenants share a cell with shuffle sharding for blast-radius control.
- **Tier 4 — edge cell.** Reserved for branch / urgent-care satellite. Not the primary ED-IS surface.

Cells are independently versioned. Promotion from Tier 1 → Tier 2 / 3 requires:

- All OpenSLOs green over 14 days.
- All Cedar policies signed at the current revision.
- Compliance packs (HIPAA + SOC2 minimum) attested.
- Trauma registry export sample passes ACS conformance.

---

## 8. Inter-Microservice Communication

Per ADR-0145 (direct gRPC + 3 invariants):

- ED-IS calls peer µservices directly over gRPC (HTTP/3 + QUIC per ADR-0253).
- No mandatory Workflow / Ontology adapter between µservices.
- 3 invariants: bounded latency budget per call (default 50 ms), idempotency on every mutating call, observability via trace propagation.

Specific peer call patterns:

- `identity` — gRPC (synchronous) for actor lookup; cached in-memory with 60 s TTL.
- `audit-chain` — AsyncAPI (fire-and-forget) for every attestable event.
- `consent-graph` — gRPC (synchronous) for consent-resolved data flows.
- `emr` — gRPC for FHIR ops + AsyncAPI for downstream encounter open.
- `healthcare-integration` — AsyncAPI (HL7 ADT / ORM ingest + outbound).
- `messenger` — gRPC for thread open + AsyncAPI for message delivery.
- `intelligence` — gRPC for room-assignment recommender + voice-to-text.
- `observability` — OpenTelemetry SDK direct (OTLP / gRPC).
- `tenancy`, `compliance`, `governance` — gRPC for context resolution.
- `pharmacy`, `lab`, `imaging` (downstream) — AsyncAPI fan-out.

---

## 9. Compliance Layering

Per ADR-0251 (compliance pack primitive) and `feedback_build_ahead_of_certification`:

- HIPAA + GDPR + SOC2 + HITRUST + ISO-27001 + PCI-DSS + EU-AI-Act + EMTALA + TJC + ACS-Trauma are first-class packs.
- Pack resolution happens at request time via the `compliance` µservice.
- Pack flags drive Cedar policy evaluation, audit log granularity, encryption mode, retention period, and disclosure obligations.

Specific overlays:

- **HIPAA breach evaluation hook.** Any access denial that produces a logged PHI exposure surface routes through a breach-evaluation Cedar policy that decides whether the access counts as a HIPAA breach.
- **EMTALA discharge gate.** Disposition close in `disposition.cedar` enforces EMTALA's screening + stabilization documentation.
- **EU-AI-Act AI flag.** If room-assignment-recommender is enabled, the request path is tagged as `ai_assist=true` and routed through an Annex-III refusal check.
- **TJC PC.01-EM chapter mapping.** Trauma registry export is gated on the TJC element set for the trauma chapter.

---

## 10. Security Model

### 10.1 Identity & Access

- mTLS at every internal endpoint.
- JWT-style provider tokens issued by `identity`.
- Cedar policy evaluation on every gate (no inline policy in code; per `feedback_cedar_as_universal_gate`).
- Charge-nurse / attending / resident / scribe / clerk / EMS / registration roles defined in `identity` and consumed via Cedar entity context.

### 10.2 Encryption

- TLS 1.3 (or HTTP/3 + QUIC TLS 1.3) on every transport.
- AES-256-GCM at rest via `cloud-kms` envelope encryption.
- Tenant BYOK supported and enforced when the compliance pack requires.

### 10.3 Audit

- Every privileged action (protocol activation, MCI activation, disposition close, bed reassignment, ARMA, expired) is audit-stamped via `audit-chain`.
- Audit chain entries are append-only and hash-chained.
- Audit retention is pack-driven (HIPAA: 6 years; EMTALA: 5 years; pack-mandated max wins).

### 10.4 Threat Model

Primary threats:

- T1 — Insider over-access (e.g., curious staff browsing celebrity chart). Mitigation: Cedar least-privilege + access reason logging + auto-flagged anomaly detection via `detection`.
- T2 — Lost device with active session. Mitigation: short-lived tokens + device attestation via `identity`.
- T3 — Compromised EMS uplink. Mitigation: mTLS + per-unit credential rotation + BYOK option for EMS vendors.
- T4 — Ransomware on ED workstation. Mitigation: thin-client architecture + state lives in cell, not workstation.
- T5 — Tampered protocol checklist. Mitigation: protocol templates signed + version-pinned per activation.
- T6 — Forged disposition. Mitigation: attending attestation + audit-chain inclusion-proof.

---

## 11. Failure Modes & Degradation

| Failure | Behavior |
|---------|----------|
| PostgreSQL unavailable | Tracking board enters read-only-from-Valkey; new writes queued in-memory + spilled to local WAL; degraded banner shown |
| Valkey unavailable | Board falls back to direct PostgreSQL reads (slower but correct) |
| NATS unavailable | Events buffered in local outbox; replay on recovery; no message loss |
| `identity` unavailable | Cached roles continue serving; new logins blocked |
| `audit-chain` unavailable | Privileged actions block with "audit unavailable" until restored (chosen over silent compliance violation) |
| `intelligence` unavailable | Voice-to-text disabled; rule-based room assignment continues |
| Cell-level partition | Cell continues serving its tenants; cross-cell coordination paused |
| MCI mode under partition | MCI activations local-only until partition heals; reconciliation event flushed on heal |

Read-only floor: the tracking board must remain readable in every failure scenario except total cell loss. New activations and protocol orders may block but the board never goes dark.

---

## 12. Performance Architecture

### 12.1 Board Fanout

The tracking board is the most performance-sensitive surface. The fanout pipeline:

```
domain mutation
  → publish ed.bed.* / ed.triage.* event
  → in-process projection updater applies to Valkey snapshot
  → broadcast channel publishes diff to per-cell websocket fanout
  → fanout writes to subscribed dashboards (SSE/WebSocket)
```

Target end-to-end: ≤ 500 ms p99 from mutation to dashboard render.

### 12.2 Order Entry

CPOE p95 ≤ 400 ms requires:

- Order set rendering pre-cached per protocol.
- Drug-interaction check is the lookup-side latency budget; uses `intelligence` clinical decision support with a 300 ms timeout and an "advisory-skipped" fallback on timeout.
- Order persistence + downstream publish are pipelined.

### 12.3 Triage Save

Triage p95 ≤ 600 ms requires:

- Vitals + acuity + chief-complaint persistence in a single atomic transaction.
- FHIR Observation projection deferred (async post-commit).

### 12.4 Metric Projection

Door-to-X metrics are computed continuously via materialized projections, not on-read aggregation. The projection updater listens to the event firehose and updates a rolling window every 5 seconds.

---

## 13. Test Architecture

### 13.1 Layers

- **Unit tests** — per crate, in `tests/` adjacent to source.
- **Contract tests** — schema conformance against `contracts/`.
- **Integration tests** — full µservice with NATS + PostgreSQL + Valkey under `testcontainers`.
- **Cell tests** — cell-tier promotion gates run a 14-day SLO replay against staging.
- **MCI drill tests** — drill mode exercises every protocol + MCI mode without touching production metrics.

### 13.2 Fixtures

- `emergency-test-support` provides:
  - `TriageFixture` — synthetic triage records (ESI 1..5).
  - `MciScenarioFixture` — synthetic mass-casualty incident with 20+ tagged patients.
  - `EmsHandoffFixture` — NEMSIS-conformant sample payloads.
  - `BedGridFixture` — 50-bed ED grid with zones + isolation flags.

### 13.3 Coverage Targets

- Domain crates: ≥ 90% line coverage.
- Application crates: ≥ 80%.
- API crates: ≥ 70% (driven by contract tests).

---

## 14. Deployment Architecture

### 14.1 OpenTofu Modules

Per `feedback_zero_handroll_opentofu_only_2026_05_20`, every deployment context is a signed OpenTofu module:

- `iac/aws-guest/` — EKS + Aurora PostgreSQL + ElastiCache for Valkey + MSK / NATS-on-EKS + KMS.
- `iac/oci-guest/` — OKE + Autonomous DB + Valkey + Streaming + Vault.
- `iac/on-prem/` — vSphere / KubeVirt / Talos + Postgres operator + Valkey operator + NATS cluster.
- `iac/colo/` — colocation hardware + Talos + Postgres + Valkey + NATS.
- `iac/oyatie-cloud/` — Cloud Hypervisor + Kata pods (per ADR-0254) on oyatie's own substrate.

### 14.2 K8s Topology (per ADR-0254)

- One Deployment per crate that owns a long-running surface (`emergency-api-rest`, `emergency-api-grpc`, `emergency-api-async`, `emergency-runtime`).
- HorizontalPodAutoscaler driven by p95 latency + CPU.
- PodDisruptionBudget enforces minAvailable for the tracking board surface.
- NetworkPolicy isolates pods to peer µservice subset.

### 14.3 Cell Promotion Gates

- Pre-prod cell → single-tenant cell: 14 days SLO green + Cedar policy signed + compliance packs attested.
- Single-tenant → pooled cell: tenant opt-in plus shuffle-sharding key generation.
- Cell rollback is single-command (OpenTofu state managed).

---

## 15. Observability Architecture

### 15.1 Signals

- **Metrics** — OpenTelemetry meter; namespace `oya.emergency.*`.
- **Traces** — OpenTelemetry tracer; propagated across every peer µservice call.
- **Logs** — structured JSON; canonical schema per `observability` µservice contract.
- **Audit** — `audit-chain` events for every privileged action.

### 15.2 SLO Wiring

Per `feedback_microservice_layout_authority`, OpenSLO authoring at `slos/*.openslo.yaml` is mandatory before any µservice promotes past dev. ED-IS ships with 12 OpenSLO objects (see PRD §7).

### 15.3 Trace Spans

Canonical span names:

- `emergency.triage.complete`
- `emergency.protocol.activate`
- `emergency.mci.activate`
- `emergency.bed.assign`
- `emergency.disposition.set`
- `emergency.metrics.project`

Every span carries `tenant_id`, `cell_id`, `encounter_id`, `actor_id`.

---

## 16. Data Flow — Canonical Happy Path

Patient arrives by ambulance with EMS prehospital report en route:

```
1. EMS uplink → emergency-api-async → ed.ems.report.received → EMSHandoff aggregate
2. Tracking board renders pre-arrival cell with ETA
3. Ambulance arrives → registration clerk opens Quick-reg → ed.patient.registered
4. Triage nurse pulls patient to triage chair → vitals + chief complaint + acuity
5. emergency-application orchestrates TriageEncounter.complete()
6. ed.triage.completed event
7. Tracking board updates within 500 ms
8. If ESI 1 → emergency-protocol auto-suggests trauma alert; attending confirms
9. emergency-application.activate_trauma_alert() → ProtocolActivation
10. Resus team paged via messenger
11. Provider drops protocol order set via emergency-orderentry
12. ed.order.placed × N → pharmacy + lab + imaging
13. Orders complete → tracking board updates badges
14. Provider sets disposition → admit
15. emr µservice opens inpatient encounter
16. BoardingHold opens since no bed yet
17. Bed assigned 90 min later → BoardingHold closes
18. Encounter projected to data-warehouse + analytics
19. If trauma criteria met → TraumaRegistryRecord created
20. End-of-shift handoff publishes ed.shift.handoff
```

---

## 17. Architectural Decisions Held Here vs. Deferred to ADRs

Held in this ARCHITECTURE.md (no separate ADR required):

- Module / crate layout (per ADR-0131 + ADR-0132 — flat).
- Concurrency model (Tokio + Rayon).
- Eventing substrate (NATS JetStream — repo-wide default).

Deferred to dedicated ADRs (in `decisions/`):

- ADR-MS-001 — Triage engine ESI rules, conflict resolution between PEWS overlay and adult overlay, and pack-driven extensions.
- ADR-MS-002 — Mass-casualty mode lifecycle and how it composes with normal Tier 2 cell operation under partition.

Anticipated future ADRs:

- ADR-MS-003 — Boarding threshold schedule per CMS measure (US) vs. EU pack.
- ADR-MS-004 — Trauma registry inclusion criteria pack handling.
- ADR-MS-005 — AI-assisted room assignment governance under EU-AI-Act Annex III.
- ADR-MS-006 — EMS handoff vendor BYOK pattern.

---

## 18. Migration Architecture (incoming tenants)

Every counterpart product (T-System, Wellsoft, FirstNet, Epic ASAP, Picis, Medhost) presents a different exit path for the customer. ED-IS supports three migration archetypes, each with a defined seam:

### 18.1 Archetype A — HL7-bridge migration

The source EDIS continues to operate while ED-IS shadow-runs in parallel. HL7 ADT / ORM / ORU feeds are mirrored from the source into `healthcare-integration`, which projects them into ED-IS. ED-IS becomes the canonical source on a cut-over date. Cut-over rollback is reversible for the first 30 days.

### 18.2 Archetype B — FHIR-bridge migration

The source EDIS exposes a FHIR R4B API. ED-IS reads on demand for historical records. New ED encounters are owned by ED-IS from cut-over day one. The source EDIS read-only archives.

### 18.3 Archetype C — Cold archive

The source EDIS is decommissioned and its historical records are batch-exported into ED-IS through a one-shot importer adapter (vendor-specific mapping). Each migrated record is re-hashed and attestable via `audit-chain`.

Every archetype publishes events with the `migration_source = <vendor>` envelope tag so downstream consumers (analytics, data-warehouse) can distinguish historical from native records.

## 19. Pack Overlays Architecture

Per `feedback_canonical_base_localization` and ADR-0064, ED-IS canonical-base is global-neutral. Locale and regulatory specifics are pack overlays. The shape of each overlay:

- **Seam pack** — additive only, e.g., add a new chief-complaint to the catalog.
- **Adapter pack** — translates a canonical concept to a local one (e.g., CTAS pack maps ESI levels 1-5 to CTAS levels I-V at the projection layer).
- **Replacement pack** — explicitly forbidden against canonical-base rules; the canonical base wins.

Concrete pack examples:

- **US pack**: Joint Commission EM chapter, EMTALA, ACS Trauma Verification, NEMSIS v3.5, NTDB.
- **EU pack**: GDPR data-subject-rights, EU-AI-Act Annex III evaluation, multilingual chief-complaint catalog, EU MDR for any medical-device interface.
- **KR pack**: KISA KSEC compliance overlay, Korean ICD-10 mapping, K-EMS NEMSIS adapter, Hanguel chief-complaint catalog.

Pack composition is multi-layer: a tenant in Germany may load `EU` + `DE-specific` + `customer-corp-001-local` packs. The `compliance` µservice resolves the layered packs at request time.

## 20. Time + Causality Model

Per ADR-0252 (HLC default, TrueTime opt-in for fin-grade) and `feedback_hlc_default_truetime_tier`:

- ED-IS uses a Hybrid Logical Clock for every event timestamp.
- Door-to-X metric computation uses HLC-comparable times across cells.
- TrueTime opt-in applies for cells that need provable global ordering (e.g., a multi-cell drug-error reconstruction).
- HLC handles the protocol bundle timer math; no TrueTime opt-in is required for the regulator window check.

## 21. Workstation / Device Architecture

ED-IS dashboards and clinical surfaces run on thin clients. State lives in the cell — workstations carry only session-scoped UI state.

- **Workstation**: Talos-on-laptop or thin Chromebook-class device, HTTP/3 + QUIC to the ED-IS cell, mTLS provisioned.
- **Mobile**: Swift (iOS) or Kotlin (Android) per the frontend matrix; communicates over MLS-protected channels.
- **Bedside tablet**: surfaces tracking-board excerpt + bedside form entry.
- **Resus dashboard**: large-display SSE consumer with high-contrast overlay.
- **EMS field tablet**: NEMSIS write through `emshandoff` API; supports offline-buffered writes that replay on reconnect.

Device attestation flows through `identity` µservice's device-claim API.

## 22. Offline Behavior

ED workflows happen during disasters when networks fail. ED-IS supports:

- **Workstation offline**: read-only board cached for the last 10 minutes, write attempts queue and surface a "connectivity lost" banner.
- **EMS field offline**: NEMSIS writes buffer locally, replay on reconnect.
- **Cell offline (cross-cell partition)**: cell operates autonomously per ADR-MS-002; cross-cell events buffer and flush on heal.
- **Total cell loss**: rare event; failover to peer cell with bounded-staleness PHI replication. Out of scope for v0; a follow-up ADR addresses cross-cell PHI replication.

## 23. Audit-chain Integration Detail

Every privileged action publishes an attestable event to `audit-chain`. The envelope:

```
{
  "actor_id": "...",
  "action": "ed.protocol.activate",
  "resource_id": "...",
  "tenant_id": "...",
  "cell_id": "...",
  "occurred_at": "<HLC>",
  "trace_id": "...",
  "payload_hash": "sha-256(...)",
  "prior_chain_head": "..."
}
```

`audit-chain` produces inclusion proofs and merkle-roots that can be replayed for regulators. ED-IS exposes `GET /Encounter/{id}/$audit-chain-proof` to fetch the inclusion proof for an encounter.

## 24. Performance Profiling Plan

A baseline profile must be captured before any optimization. The profile measures:

- Cold-start latency for the µservice binary on linux/arm64 and linux/amd64.
- p50/p95/p99 latency for each REST endpoint under a synthetic load of 100 RPS sustained.
- Tracking board fanout latency from event publish to dashboard render.
- Order entry latency under the canonical sepsis bundle order set drop.
- Triage save latency under burst conditions (50 concurrent triages).
- Metric projection lag end-to-end.

The profile is regenerated on every release candidate as part of the cell-promotion gate.

## 25. Capacity Model

A reference 50-bed ED with 60,000 annual visits maps to the following baseline:

- Aggregate write rate: ~5 writes/sec average, ~50 writes/sec peak.
- AsyncAPI publish rate: ~30 events/sec average, ~300 events/sec peak.
- Tracking board fanout: ~3 dashboards × 50 active board cells = ~150 cells in flight per cell.
- Postgres OLTP: ~10 GB / year of structured data (PHI excluded).
- Valkey: ~50 MB working set per ED.
- Object storage: variable based on document attachments (≈ 100 MB / encounter for full-text + scan).

A 10-ED Tier-3 pooled cell scales to ~600,000 annual visits with horizontal Postgres + Valkey + NATS.

## 26. Architectural Decisions Held Here vs. Deferred to ADRs

Held in this ARCHITECTURE.md (no separate ADR required):

- Module / crate layout (per ADR-0131 + ADR-0132 — flat).
- Concurrency model (Tokio + Rayon).
- Eventing substrate (NATS JetStream — repo-wide default).
- Time clock (HLC default per ADR-0252).
- Device thin-client model.
- Migration archetype catalog.

Deferred to dedicated ADRs (in `decisions/`):

- ADR-MS-001 — Triage engine ESI rules, conflict resolution between PEWS overlay and adult overlay, and pack-driven extensions.
- ADR-MS-002 — Mass-casualty mode lifecycle and how it composes with normal Tier 2 cell operation under partition.

Anticipated future ADRs:

- ADR-MS-003 — Boarding threshold schedule per CMS measure (US) vs. EU pack.
- ADR-MS-004 — Trauma registry inclusion criteria pack handling.
- ADR-MS-005 — AI-assisted room assignment governance under EU-AI-Act Annex III.
- ADR-MS-006 — EMS handoff vendor BYOK pattern.
- ADR-MS-007 — Cross-cell PHI replication for total-cell-loss failover.
- ADR-MS-008 — Pediatric severity overlay choice (PEWS vs SREM vs local) per pack.
- ADR-MS-009 — Documentation template versioning + pack overlay rules.
- ADR-MS-010 — EMS NEMSIS schema version pinning + upgrade story.

## 27. Authority Trail

- ADR-0332 (in flight) — Emergency Department Information System µservice.
- ADR-0131 — Per-µservice flat layout.
- ADR-0132 — Suite dissolution.
- ADR-0145 — Direct gRPC + 3 invariants.
- ADR-0248 — Amazon-shape cellular architecture.
- ADR-0251 — Compliance pack primitive.
- ADR-0252 — HLC default, TrueTime opt-in.
- ADR-0253 — HTTP/3 + QUIC default.
- ADR-0254 — K8s + Cloud Hypervisor.
- ADR-0255 — Two-layer intelligence substrate.
- ADR-0105 — 13-layer enum.
- ADR-0064 — Canonical-base neutrality.
- ADR-0244 — Tenant universal scoping.
- ADR-0243 — Cedar universal gate.
- ADR-0130 — OpenSLO authoring mandatory before promotion.
- Constraint memories: Rust-strict, OS matrix, OpenTofu zero-handroll, OCI Always Free, multi-context provider-agnostic, µservice ownership coherence, doc substance, no-silent-regression, build-ahead-of-certification, MLS RFC 9420, BYOK everywhere, intelligence two-layer.
