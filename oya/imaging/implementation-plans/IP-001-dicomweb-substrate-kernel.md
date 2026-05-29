# IP-001 — DICOMweb substrate kernel

`scope: oya-imaging-kernel + oya-imaging-dicom-domain + oya-imaging-dicomweb-rest + oya-imaging-dicomweb-api`
`wave_target: 16-imaging-substrate`
`adr_binding: ADR-MS-001 + ADR-0105 (13-layer) + ADR-0145 (direct gRPC) + ADR-0253 (HTTP/3)`

## Objective

Stand up the DICOMweb-first PACS substrate kernel. This is the foundation; every subsequent IP depends on it.

## Scope

1. `oya-imaging-dicom-domain` — Rust types for Study Instance UID, Series Instance UID, SOP Instance UID, Transfer Syntax UID, SOP Class UID, AE Title, with parse + display + DICOM PS 3.5 validation.
2. `oya-imaging-kernel` — tenant context propagation port, Cedar evaluation port, audit-chain emission port, `cloud-storage` blob port, `cloud-data` relational port.
3. `oya-imaging-dicomweb-rest` — WADO-RS / QIDO-RS / STOW-RS / UPS-RS codec.
4. `oya-imaging-dicomweb-api` — HTTP/3 + QUIC handler with tenant-scoped + Cedar-gated request lifecycle.

## Acceptance criteria

- `cargo test -p oya-imaging-dicom-domain` covers ≥95% lines including DICOM UID parsing happy + adversarial cases (trailing whitespace / leading zeros / overlength / non-numeric / cross-tag character).
- DICOM PS 3.5 character-set tests pass for ISO_IR 100 (Latin-1), ISO_IR 192 (UTF-8), ISO 2022 IR 13/87 (Japanese).
- `oya-imaging-kernel::TenantContext` propagates via OpenTelemetry baggage on every async boundary.
- QIDO-RS study-level query achieves p95 < 200ms locally with 10k-row index (FR-PACS-002).
- WADO-RS instance retrieval streams multipart/related correctly; integration test verifies byte-exact match against reference DICOM dataset.
- STOW-RS store accepts multipart/related with 1 study × 200 instance × 1 MB and persists end-to-end with audit-chain emission.

## Dependencies

- `cloud-iam` (Cedar evaluation port)
- `cloud-storage` (blob port)
- `cloud-data` (relational port)
- `audit-chain` (emit port)
- `observability` (OTel)

## Risks

- HTTP/3 library maturity in Rust; mitigate by selecting `quinn` (stable since 2024).
- DICOM character-set quirks (Korean ISO 2022 IR 149); mitigate with vendor-quirks library upfront.
- Multipart/related stream parsing performance; mitigate with zero-copy parser.

## Out-of-scope (deferred)

- DIMSE bridge (IP-002).
- AI marketplace (IP-013).
- 3D rendering (deferred to dedicated render-IP).

## Testing strategy

- Unit tests in each crate.
- Integration tests against ephemeral PostgreSQL + S3-compatible (MinIO) substrate.
- DICOM PS 3.4 conformance regression test corpus.
- Multi-tenant isolation tests (deny-default + positive cases).
- Cedar policy tests against `policies/radiologist-can-read.cedar` + `policies/hipaa-deny-default.cedar`.

## Sequencing

1. Layer 5 (domain) types first.
2. Layer 6 (kernel) ports.
3. Layer 7 (adapters) for cloud-storage + cloud-data + audit-chain + cloud-iam.
4. Layer 2 (REST) codec.
5. Layer 1 (API) handler.
6. Smoke test end-to-end.

## Estimated effort

- 12–16 person-weeks for Wave 16-imaging-substrate.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/imaging/implementation-plans/IP-001-dicomweb-substrate-kernel.md:14` - 2. `oya-imaging-kernel` — tenant context propagation port, Cedar evaluation port, audit-chain emission port, `cloud-storage` blob port, `cloud-data` relational port.; `microservices/imaging/implementation-plans/IP-001-dicomweb-substrate-kernel.md:25` - - STOW-RS store accepts multipart/related with 1 study × 200 instance × 1 MB and persists end-to-end with audit-chain emission..
