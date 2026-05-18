---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-foundation
phase: P01-shorts-foundation
impl_plan_id: IP-003-video-upload-bc
status: pending
execution_unit: ChangeSet
owner: axis-shorts
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-port-location, oya-governance-postgres-rls-coverage, oya-governance-dual-context-isolation]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: video-upload BC end-to-end

## Intent

Implement `video-upload` BC across the full clean-architecture stack:
kernel (port traits + entities + data_class), domain (pure logic), usecase
(orchestration), api (Cedar enforcement boundary), adapter (Postgres + S3),
rest (REST endpoints), sdk (client SDK), app (binary).

Scope:
- Multipart resumable upload session lifecycle.
- Scan-first quarantine→clean→production blob lifecycle.
- Per-tenant + per-creator rate limits.
- ULID-keyed upload session + content-hash de-dup.
- Pre-finalize idempotency.
- Audit-chain seal `VideoUploaded` event emission.
- Dual-context invariant enforced at kernel + domain layers (DCI-01, DCI-02).

## ChangeSet boundary

10 crates: `oya-shorts-video-upload-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,rest,sdk,app}`.

Integration with:
- `cell` µservice for per-tenant cell-boundary enforcement.
- `tenancy` µservice for RLS context.
- `audit-chain` µservice for Ed25519 seal.
- `ontology` µservice for `Video` entity write.

## Concrete File Targets

| Crate | Key types |
|---|---|
| `oya-shorts-video-upload-kernel` | `UploadSession`, `VideoBlob`, `ScanVerdict`, `UploadETag`, `PersonalShort`, `ProfessionalShort`, port traits: `VideoBlobStore`, `UploadSessionRepository`, `MalwareScanner` |
| `oya-shorts-video-upload-domain` | pure validation (size, duration, content-type); idempotency-key check; rate-limit check |
| `oya-shorts-video-upload-usecase` | orchestrate initiate→upload→finalize→scan→quarantine→transcode-enqueue |
| `oya-shorts-video-upload-api` | Cedar enforcement boundary |
| `oya-shorts-video-upload-adapter` | Cedar evaluator adapter |
| `oya-shorts-video-upload-adapter-postgres` | impl `UploadSessionRepository`; RLS on every query |
| `oya-shorts-video-upload-adapter-s3` | impl `VideoBlobStore`; per-tenant prefix; KMS SSE; signed multipart upload URL |
| `oya-shorts-video-upload-rest` | POST /upload-sessions; POST /upload-sessions/{id}/finalize |
| `oya-shorts-video-upload-sdk` | client multipart-upload helper with resume |
| `oya-shorts-video-upload-app` | binary launching upload REST service |

## Acceptance Gates

```bash
cargo build -p oya-shorts-video-upload-app
cargo nextest run -p oya-shorts-video-upload-{kernel,domain,usecase,adapter,adapter-postgres,adapter-s3,rest}
cargo run -p oya-dev-cli -- gate validate port-location --microservice shorts --bc video-upload
cargo run -p oya-dev-cli -- gate validate postgres-rls-coverage --microservice shorts
cargo run -p oya-dev-cli -- gate validate dual-context-isolation --microservice shorts
```

E2E: POST /upload-sessions → multipart upload to S3 quarantine → finalize → scan → clean → production → `VideoUploaded` Workflow event emitted.

## Test Plan

- kernel: 90% line / 80% branch on port traits + entities + data_class annotations.
- domain: 90% line / 80% branch on validation logic.
- usecase: 85% line / 75% branch with port mocks; happy + error paths.
- adapter-postgres: integration vs real Postgres 16; RLS verified.
- adapter-s3: integration vs LocalStack or SeaweedFS; signed-URL roundtrip.
- rest: 85% line / 75% branch; per-endpoint 200 / 401 / 403 / 422.

## Halt Conditions

- BNF naming fail.
- RLS coverage gap.
- Dual-context cross-write compiles (must compile-fail).

## Next IP

[`IP-004-video-transcode-bc.md`](IP-004-video-transcode-bc.md)

## References

- PRD FR-01.
- `policy/dual-context-isolation.md` DCI-01, DCI-02.
- `threat-model.md` T-T-01, T-E-04.
- ADR-SHORTS-0001 (transcode pipeline; upload is upstream).
