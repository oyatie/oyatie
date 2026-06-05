---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-003-recording-ingest-bc
status: pending
execution_unit: ChangeSet
owner: axis-recordings
acceptance_lanes: [port-location, lean-a1, ingest-contract-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: Recording-Ingest BC — kernel + domain + usecase + ingest contract

## Intent

Land the multi-source durable ingest contract per ADR-RECORDINGS-0007:
`RecordingIngestRequest` + `RecordingIngestResponse` typed contract; SPIFFE
identity allowlist + Cedar PERMIT 7; parent-chain audit-chain linkage;
idempotency + ordering + backpressure protocol.

## Concrete crates

- `oya-recordings-recording-ingest-kernel`: `RecordingIngestRequest`,
  `IngestSession`, `IngestSourceKind`, port traits.
- `oya-recordings-recording-ingest-domain`: validation rules + idempotency
  semantics.
- `oya-recordings-recording-ingest-usecase`: orchestration; calls
  RecordingRepository + AuditChainClient.
- `oya-recordings-recording-ingest-api`: shared proto bindings.
- `oya-recordings-recording-ingest-adapter`: legacy-Strangler shim.
- `oya-recordings-recording-ingest-adapter-s3`: presigned-URL generator for
  manual upload (PRD FR-17).
- `oya-recordings-recording-ingest-rest`: REST endpoints `POST /v1/ingest/presign`
  + `POST /v1/ingest/finalize`.
- `oya-recordings-recording-ingest-worker`: consumes
  `meet.session.ended.v1` + `messenger.huddle.ended.v1` events.
- `oya-recordings-recording-ingest-sdk`: client surface for producers.
- `oya-recordings-recording-ingest-app`: deployable composition.

## Acceptance Gates

```bash
cargo nextest run -p oya-recordings-recording-ingest-kernel
buck2 build //:quality-lane-registry-authority-check # lane=ingest-contract-conformance
buck2 build //:quality-lane-registry-authority-check # lane=port-location --microservice recordings
buck2 build //:quality-lane-registry-authority-check # lane=lean-a1 --microservice recordings
```

## Next IP

[`IP-004-recording-bc.md`](IP-004-recording-bc.md)
