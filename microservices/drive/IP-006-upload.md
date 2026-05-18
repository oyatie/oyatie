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

## References

- PRD-drive §FR-01; AC-01; AC-02; ADR-DRIVE-0002; ADR-DRIVE-0005.
- tus.io 1.0 spec; AWS S3 multipart spec.
