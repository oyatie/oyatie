---
id: ADR-RECORDINGS-0007
status: Accepted
date: 2026-05-17
microservice: recordings
deciders: axis-recordings, axis-meet, axis-messenger, council-architecture
owner: axis-recordings
supersedes: []
superseded_by: []
related: [ADR-0008, ADR-0028, ADR-0131, ADR-RECORDINGS-0002]
related_artifacts:
  - microservices/recordings/PRD.md (FR-01 multi-source ingest, FR-17 manual upload)
  - microservices/recordings/contracts/proto/recordings.proto
  - microservices/recordings/contracts/asyncapi/recordings-events.yaml
  - microservices/recordings/contracts/openapi/recordings.yaml
  - microservices/recordings/policy/cedar/tenant-scope.cedar
purpose: |
  Fix the durable contract through which producer µservices (meet,
  messenger huddles, live-broadcast, manual upload, workflow screen-capture)
  hand off a recording to the recordings µservice. Idempotent + ordering-
  preserving + backpressure-aware. Conforms to ADR-0131 cross-µservice
  refusal rule + Bominal ADR-0028 audit-chain Merkle parent-chain.
---

# ADR-RECORDINGS-0007: Multi-source ingest contract — durable cross-µservice handoff with idempotency + ordering + backpressure

## Status

Accepted — 2026-05-17.

## Context

PRD-recordings FR-01 mandates a durable contract through which each
producer µservice emits to the recordings archive:

- `meet`: meeting session ended → recording finalised → emit to recordings
- `messenger` huddles: huddle ended → recording finalised → emit
- `live-broadcast` (future): live stream ended → recording finalised → emit
- `manual upload` (FR-17): user uploads audio/video via presigned-URL → emit
- `workflow-engine screen-capture`: agentic capture → emit

ADR-0131 cross-µservice rule says µservices don't call each other directly;
all cross-µservice flow goes through Workflow events. But for a recording
ingest, three properties are critical:

1. **Idempotency**: a meet session that emits twice (network retry) must
   produce one recording, not two.
2. **Ordering**: a session that emits a "session-ended" event after several
   "segment-uploaded" events must enforce that the segments precede the
   ended-event in the archive's view.
3. **Backpressure**: when recordings is queue-pressured (e.g., Whisper
   queue overflow), producers must back off (not retry-storm).

The right shape is a **typed durable contract** (proto + AsyncAPI + REST)
emitted via the canonical Workflow event bus, with idempotency-key and
parent-chain audit semantics.

## Decision

oyatie recordings ships a **durable multi-source ingest contract** with
these properties:

### 1. Contract surface

- **proto** (`contracts/proto/recordings.proto`): `RecordingsIngest`
  service exposing `Ingest(RecordingIngestRequest) → RecordingIngestResponse`.
  Used for direct gRPC ingest (preferred low-latency path).
- **REST** (`contracts/openapi/recordings.yaml`): `POST /v1/ingest/presign`
  + `POST /v1/ingest/finalize` for manual upload via presigned-URL.
- **AsyncAPI** (`contracts/asyncapi/recordings-events.yaml`): consumed
  events `meet.session.ended.v1` + `messenger.huddle.ended.v1` +
  `live-broadcast.session.ended.v1`. These events do NOT carry the media
  blob — only a `recording_target_id` reference that recordings then
  fetches via the gRPC ingest service.

### 2. Idempotency

- Every ingest request carries `idempotency_key` (per-producer-µservice
  deterministic based on source_ref + ingest-cycle).
- recordings de-duplicates on `(idempotency_key, tenant_id)`; second
  request returns the existing `recording_id` (200 OK; not 409).

### 3. Ordering

- Recording-segment uploads must complete before the ingest-finalize call.
- Producer side enforces this via local sequencing.
- recordings side validates: refuses ingest-finalize if any expected
  segment is missing (per ingest-manifest declared in the
  RecordingIngestRequest).

### 4. Backpressure

- recordings publishes `ingest_queue_depth_minutes` metric.
- Producer µservices subscribe to the metric (Mimir query or direct
  ingest-rate-limit response header from recordings-rest).
- When queue depth > 60 min: producers fall back to local-disk + delayed
  re-emit pattern.

### 5. SPIFFE identity + Cedar PERMIT

- Per `policy/cedar/tenant-scope.cedar` PERMIT 7: only producer SPIFFE
  identities on the allowlist can ingest.
- mTLS-terminated gRPC + REST.

### 6. Parent-chain audit (per Bominal ADR-0028)

- Each ingest request carries `parent_audit_chain_ref` (the producer's
  last audit-chain seal for the source).
- recordings emits `RecordingIngested` event with `audit_chain_parent_ref:
  <producer's last seal>` → chain continuity.

### 7. Consent-banner gating

- Every ingest request carries `consent_banner_confirmed: bool`.
- recordings refuses ingest with `consent_banner_confirmed == false` for
  packs that require it (KR / AU / EU); pass-through for others (ingest
  still records the flag for audit).

### 8. Per-source-kind shape

The `SourceKind` enum is non-exhaustive; future producers add a variant
+ recordings updates the consumer.

Current variants:
- `meet` — produced by axis-meet
- `messenger_huddle` — produced by axis-messenger
- `live_broadcast` — future axis-live-broadcast
- `manual_upload` — user via REST presign + finalize
- `workflow_screen_capture` — workflow-engine agentic capture
- `legacy_workspace_recording` — Strangler replay of the legacy
  `oya-connect-recordings-domain` shape (per `migration-from-connect.md`).

### 9. Cross-µservice rule alignment

- Recordings calls **nothing** in producer µservices.
- Producers call recordings via the contract (this is the **one** cross-
  µservice direct-call exception, justified by the centralisation
  architecture and audited via SPIFFE + Cedar + ingest-contract conformance
  CI lane).
- All other cross-µservice flow remains via Workflow events.

## Alternatives Considered

### A. Producer µservices store their own recordings; recordings reads-through

- Pros: producer µservices have full ownership of their session media.
- Cons: defeats centralisation (which is the whole point of recordings as
  a µservice); transcript + redaction + retention + legal-hold all have to
  be duplicated; ediscovery requires cross-µservice scatter-gather; cost
  + ops complexity explode.
- Rejected: ADR-RECORDINGS-0002 + the recordings PRD positioning depend on
  centralisation.

### B. Recordings pulls from each producer (poll-based)

- Pros: recordings has full control over ingest timing.
- Cons: ingest latency p99 grows; producers don't know when their
  recording is durable.
- Rejected; push-based is the right pattern.

### C. Pure event-bus (no gRPC; only AsyncAPI events with embedded blobs)

- Pros: maximal decoupling.
- Cons: media blobs (multi-GB) can't ride the event bus efficiently; the
  natural pattern is event-with-reference + side-channel media fetch.
- Rejected; hybrid (event reference + gRPC blob fetch / presigned-URL) is
  canonical.

### D. No idempotency (producers retry blindly; recordings de-dupes by
content_hash)

- Pros: simpler interface.
- Cons: content_hash de-dupe is fragile (a single byte change makes it a
  new recording); producers genuinely need idempotency_key.
- Rejected; idempotency_key is the canonical pattern.

### E. Auto-detect source-kind from request shape

- Pros: less typing per producer.
- Cons: ambiguous; new source kinds require recordings code change anyway;
  explicit enum is clearer.
- Rejected; explicit `SourceKind` enum.

## Consequences

### Positive

- Centralised archive across every recording source.
- Idempotency-key + parent-chain-audit give end-to-end forensic continuity.
- Backpressure protocol prevents cascade failures.
- Manual upload via presigned-URL gives external API consumers a clean
  path (per PRD FR-17).
- Strangler-migration replay shape (legacy_workspace_recording) gives a
  natural path for the migration adapter.

### Negative

- One direct cross-µservice call shape (the ingest gRPC) sits outside the
  pure-Workflow-events rule; documented as an explicit, audited exception
  per LEAN-A2 CI lane.
- Per-source-kind variants accumulate; future addition requires recordings
  + producer co-changes.

### Operational

- Cargo workspace adds `oya-recordings-recording-ingest-*` (9 crates) +
  shared proto bindings via `oya-recordings-recording-ingest-api` crate.
- CI lane `ingest-contract-conformance` validates producer SPIFFE identity
  allowlist + idempotency-key behaviour + parent-chain audit linkage.
- IaC: gRPC ingest endpoint registered at `recordings-grpc-ingest.internal`
  with mTLS + Envoy ingress.

### Regulatory

- **KR 통신비밀보호법 / AU TIA Act / ePrivacy Art. 5(3)**: producer-side
  consent_banner_confirmed required at ingest.
- **Bominal ADR-0028**: parent-chain audit-chain Merkle linkage.
- **SEC 17a-4 / FINRA 4511 / MiFID II 16(7)**: ingest contract records
  source-µservice provenance for recorded-communications.

## References

- gRPC documentation, Protocol Buffers.
- AsyncAPI 3.0 specification.
- OpenAPI 3.1 specification.
- RFC 9457 (Problem Details for HTTP APIs).
- SPIFFE identity standard.
- Cedar v4.2 policy language.
- ADR-0008 (data use boundary).
- Bominal ADR-0028 (audit-chain Merkle + Ed25519).
- ADR-0131 — per-µservice flat layout + cross-µservice rule.
- ADR-RECORDINGS-0002 (retention + legal hold context for ingest).
- microservices/recordings/contracts/proto/recordings.proto.
- microservices/recordings/contracts/openapi/recordings.yaml.
- microservices/recordings/contracts/asyncapi/recordings-events.yaml.
- microservices/recordings/policy/cedar/tenant-scope.cedar PERMIT 7.
- microservices/recordings/migration-from-connect.md.
