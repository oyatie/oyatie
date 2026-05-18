---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-drive-foundation
impl_plan_id: IP-007-download
status: pending
execution_unit: ChangeSet
owner: axis-drive
acceptance_lanes: [cargo-build, cargo-nextest]
---

# IP-007: download BC — range + signed URL + CDN steering

## Intent

Stand up `oya-drive-download-*` BC. Implement HTTP Range (RFC 9110), signed-URL minting, CDN steering, byte-range stream optimisation.

## Crates

`oya-drive-download-{kernel,domain,usecase,api,adapter,adapter-s3,rest,app}` (8 crates).

## Acceptance Gates

```bash
cargo nextest run --test e2e_range_download
cargo nextest run --test e2e_signed_url
cargo nextest run -p oya-drive-download-usecase -- first_byte_warm
```

## ChangeSet metadata

```yaml
changeset_id: CS-DRIVE-IP-007-download
depends_on_changesets: [CS-DRIVE-IP-003-file-store-kernel-domain, CS-DRIVE-IP-009-share-link]
parallel_safe_with_changesets: [CS-DRIVE-IP-006-upload, CS-DRIVE-IP-005-folder-hierarchy]
enables: [CS-DRIVE-IP-012-preview]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | First-byte-warm p95 ≤ 100ms via CDN steering | `cargo nextest run -p oya-drive-download-usecase -- first_byte_warm` |
| AC-02 | First-byte-cold p95 ≤ 500ms direct S3 fetch | `cargo nextest run -p oya-drive-download-usecase -- first_byte_cold` |
| AC-03 | RFC 9110 Range request returns 206 with correct `Content-Range` for byte-ranges incl. multi-range | `cargo nextest run --test e2e_range_download` |
| AC-04 | Signed URL bound to single object + expiry rejects after TTL elapsed | `cargo nextest run --test e2e_signed_url` |
| AC-05 | `If-None-Match` / `If-Modified-Since` honoured with 304 (RFC 9110 §13) | `cargo nextest run --test e2e_conditional_get` |

## Build Sequence

1. Kernel ports: `BlobReader`, `RangeStreamer`, `SignedUrlMinter`.
2. Domain: `DownloadGrant`, `RangeSpec`, `ConditionalPrecondition`.
3. Usecase: `OpenRangeStream`, `MintSignedUrl`, `SteerToEdge`.
4. Adapters: `-adapter-s3` (presigned GET) + REST handler.
5. `cargo nextest run --test e2e_range_download`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-drive FR | FR-02 (range download) |
| PRD-drive NFR | NFR perf — download first-byte warm p95 ≤ 100ms |
| PRD-drive AC | AC-02 |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Range-DoS via overlapping byte-range requests | Coalesce or refuse > 16 ranges per RFC 9110 advisory |
| Signed-URL leakage across tenants | URL HMAC bound to `(tenant_id, object_id, expiry)`; constant-time compare |
| CDN cache poisoning via `Vary` mismatch | Pin `Vary: Accept-Encoding, Authorization`; cache-key includes tenant |

## References

- PRD-drive §FR-02; AC-02.
- RFC 9110 (HTTP Semantics; Range Requests §14).
- AWS S3 presigned URL spec (S3 User Guide).
- Cloudflare Range cache behaviour ("Range request handling on Cloudflare").
