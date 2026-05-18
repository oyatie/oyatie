---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-foundation
phase: P01-shorts-foundation
impl_plan_id: IP-005-video-storage-and-cdn-bc
status: pending
execution_unit: ChangeSet
owner: axis-shorts + cloud-secrets
depends_on: [IP-004]
---

# IP-005: video-storage + CDN BC end-to-end

## Intent

Implement `video-storage` BC: S3-compatible blob store with per-tenant prefix
isolation + KMS SSE + Object Lock (WORM) for Professional-tier;
CloudFront-class CDN tier (Cloudflare R2 + Workers); signed-URL with
TTL ≤ 15 min.

## ChangeSet boundary

8 crates: `oya-shorts-video-storage-{kernel,domain,usecase,api,adapter,adapter-s3,adapter-cloudfront,sdk}`.

## Concrete File Targets

| Crate | Key types |
|---|---|
| `oya-shorts-video-storage-kernel` | `BlobRef`, `CdnUrl`, `SignedManifest`, `PrefixScope`; ports: `BlobStore`, `CdnInvalidator` |
| `oya-shorts-video-storage-adapter-s3` | impl `BlobStore`; per-tenant prefix; KMS; Object Lock for Professional |
| `oya-shorts-video-storage-adapter-cloudfront` | impl `CdnInvalidator` against Cloudflare R2 + Workers |

## Acceptance Gates

```bash
cargo build -p oya-shorts-video-storage-rest
cargo nextest run -p oya-shorts-video-storage-{kernel,domain,usecase,adapter-s3,adapter-cloudfront}
```

E2E: signed-URL roundtrip; CDN priming; tier-based Object-Lock activation; KMS encryption verified.

## Halt Conditions

- KMS unavailable.
- Cloudflare R2 outage — fall back to S3 origin (degraded latency).
- Object Lock misconfig — block; engage cloud-secrets.

## Next IP

[`IP-006-thumbnail-and-composition-bc.md`](IP-006-thumbnail-and-composition-bc.md)

## References

- PRD FR-04, FR-29.
- ADR-SHORTS-0001 (CDN choice).
- `threat-model.md` T-I-06, T-T-01.
- `multi-region.md` §CDN topology.
- Cloudflare R2 + Workers; AWS S3 + Object Lock.
