---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-drive-foundation
impl_plan_id: IP-006-upload
status: pending
execution_unit: ChangeSet
owner: axis-drive
acceptance_lanes: [cargo-build, cargo-nextest, s3-sigv4-conformance, tus-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: upload BC — multipart resumable + S3 + tus 1.0

## Intent

Stand up `oya-drive-upload-*` BC. Implement S3-multipart, tus 1.0, FastCDC chunking (per ADR-DRIVE-0002), virus-scan pipeline integration (per ADR-DRIVE-0005), staging-to-durable promotion, abandonment sweep.

## Crates

`oya-drive-upload-{kernel,domain,usecase,api,adapter,adapter-redis,adapter-s3,rest,worker,app}` (10 crates).

## Acceptance Gates

```bash
cargo nextest run --test e2e_s3_sigv4
cargo nextest run --test e2e_tus_1_0
cargo nextest run -p oya-drive-upload-domain -- fastcdc_parameters_pinned
cargo nextest run --test e2e_upload_multipart_1gb
```

## ChangeSet metadata

```yaml
changeset_id: CS-DRIVE-IP-006-upload
depends_on_changesets: [CS-DRIVE-IP-003-file-store-kernel-domain, CS-DRIVE-IP-013-dlp-virus-scan]
parallel_safe_with_changesets: [CS-DRIVE-IP-007-download, CS-DRIVE-IP-005-folder-hierarchy]
enables: [CS-DRIVE-IP-008-sync, CS-DRIVE-IP-012-preview]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | S3 multipart 1GB upload completes p95 ≤ 90s under ≥ 4 concurrent parts | `cargo nextest run --test e2e_upload_multipart_1gb` |
| AC-02 | AWS SigV4 conformance suite passes (all canonical-request fixtures) | `cargo nextest run --test e2e_s3_sigv4` |
| AC-03 | tus.io 1.0 conformance suite passes (HEAD, PATCH, OPTIONS, expired upload) | `cargo nextest run --test e2e_tus_1_0` |
| AC-04 | FastCDC chunk parameters pinned (min 4KiB, avg 8KiB, max 64KiB) and stable | `cargo nextest run -p oya-drive-upload-domain -- fastcdc_parameters_pinned` |
| AC-05 | Staging-to-durable promotion gated by clean ClamAV + OPSWAT verdict | `cargo nextest run --test e2e_upload_virus_quarantine` |

## Build Sequence

1. Stand up `oya-drive-upload-kernel` port traits (`UploadSession`, `ChunkStore`, `ScanGate`).
2. Implement FastCDC in `-domain` (Xia 2016 paper params); pin via `fastcdc_parameters_pinned` test.
3. Implement S3 multipart adapter (`-adapter-s3`) + tus 1.0 adapter (`-adapter-tus`).
4. Wire scan handoff to `dlp-virus-scan` BC via internal Workflow event.
5. `cargo nextest run -p oya-drive-upload-*`.
6. `cargo run -p oya-dev-cli -- gate validate s3-sigv4-conformance --microservice drive`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-drive FR | FR-01 (multipart resumable upload) |
| PRD-drive NFR | NFR perf — multipart 1GB p95 ≤ 90s |
| PRD-drive AC | AC-01, AC-02 |
| ADR | ADR-DRIVE-0002 (FastCDC), ADR-DRIVE-0005 (preview/DLP sandbox) |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Abandoned upload sessions consuming staging quota | Sweeper worker reaps sessions older than 24h (per `domain`) |
| Slow-loris-style multipart attack | Per-tenant SigV4 rate limit + max-in-flight cap |
| Chunk-boundary tampering pre-scan | Content-addressed `BLAKE3` of each chunk; durable promotion only after match |

## References

- PRD-drive §FR-01; AC-01; AC-02; ADR-DRIVE-0002; ADR-DRIVE-0005.
- tus.io 1.0 specification (`tus.io/protocols/resumable-upload`).
- AWS S3 multipart upload (S3 User Guide — "Uploading objects in parts using multipart upload").
- Xia, W. et al. "FastCDC: A Fast and Efficient Content-Defined Chunking Approach for Data Deduplication" (USENIX ATC 2016).
- RFC 7233 / RFC 9110 (HTTP Range / Conditional Requests).
