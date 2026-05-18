---
doc_class: SdkPlan
template_id: TPL-SDK-PLAN
microservice: drive
status: Accepted
date: 2026-05-17
owner_team: axis-drive + axis-developer-experience
related_adrs: [ADR-0131, ADR-0133]
doc_status: published
---

# SDK Plan — drive µservice

## Purpose

Define the SDK surface, language coverage, version cadence, semver guarantees, and parity matrix for the drive µservice.

## Surface

Three protocol surfaces; SDK clients can use any:

1. **oya-native REST** — `oya-drive-*-sdk` crates per BC; idiomatic + opinionated + first-class.
2. **S3-compat REST** — works with any AWS SDK / `aws s3` / `mc` / `s3cmd`; SigV4-bound.
3. **WebDAV (RFC 4918)** — works with macOS Finder / Windows Explorer / davfs2 / Cyberduck / Nextcloud client; per-tenant credential.
4. **tus 1.0** — resumable-upload spec; works with any tus client (web / desktop / mobile).

## Language coverage

| Language | Primary use-case | Source |
|---|---|---|
| Rust | first-party µservice consumers (workflow, mail attachment-bridge, messenger embed) | `microservices/drive/src/crates/oya-drive-*-sdk` |
| TypeScript/JavaScript | web client + Node tooling | `microservices/drive/src/packages/oya-drive-sdk-ts` (M03) |
| Python | Foundry-runtime ML + analytics | `microservices/drive/src/packages/oya-drive-sdk-py` (M03) |
| Go | infrastructure tooling + CLI | `microservices/drive/src/packages/oya-drive-sdk-go` (M04) |
| Java/Kotlin | enterprise integration + Android | `microservices/drive/src/packages/oya-drive-sdk-jvm` (M04) |
| Swift | macOS / iOS desktop+mobile sync client | `microservices/drive/src/packages/oya-drive-sdk-swift` (M04) |
| C# / .NET | Windows tooling + enterprise integration | `microservices/drive/src/packages/oya-drive-sdk-dotnet` (M05) |

## Semver guarantees

- Major: breaking change to public API; requires ADR + sunset notice per `feedback_no_silent_regression`.
- Minor: additive change; no breaking.
- Patch: bug fix; no public surface change.

## Migration path

For consumers of legacy `oya-connect-drive-*` crates: see `microservices/drive/migration-from-connect.md`. Strangler-pattern adapter shim preserves legacy symbol surface verbatim during the 6-month sunset window per ADR-0134.

## SDK features

- Multipart resumable upload (parallel chunks + per-chunk retry).
- Range download (HTTP Range RFC 9110).
- Delta-sync (FastCDC + LBFS).
- Share-link mint + revoke.
- Permission management (per-folder / per-file).
- Search (filename + full-text).
- Preview fetch (thumbnail + first-page render).
- Webhook subscription for file-change events.
- E2E client-side encryption (libsodium secretstream) — opt-in for Personal pillar.
- Audit-chain receipt verification (verify Ed25519 seal on download).

## Idempotency

All write APIs accept `idempotency_key`; the SDK auto-generates per-call UUIDs unless caller provides.

## Errors

| Class | When | SDK behaviour |
|---|---|---|
| Network | transport error | retry with exponential backoff + jitter; max 5 retries |
| 4xx (client) | bad request / forbidden / not found | surface to caller; do NOT retry |
| 429 (rate limit) | rate limit hit | obey `Retry-After`; retry with backoff |
| 5xx (server) | server error | retry with backoff |
| Concurrency conflict | optimistic concurrency on file version | surface; allow caller to refresh + retry |

## Telemetry

SDK emits per-call OpenTelemetry spans + per-call request_id; telemetry opt-out per tenant.

## Documentation

- per-language quick-start.
- per-feature recipes.
- per-protocol conformance matrix (oya-native + S3 + WebDAV + tus).
- migration guide from competitor SDKs (AWS S3, Dropbox, Google Drive, OneDrive, Box).

## Compliance

- All SDKs vetted via supply-chain review (SLSA L3) before publish.
- SDK release artifacts signed; signature verification documented per language.
- SDK sources include LICENSE + NOTICE matching OSS-policy.

## Roadmap

| Milestone | Deliverable |
|---|---|
| M02 | Rust SDK GA |
| M03 | TypeScript + Python SDK GA; S3-compat + WebDAV + tus conformance |
| M04 | Go + JVM + Swift SDK GA |
| M05 | .NET SDK GA |

## References

- ADR-0131 — per-µservice flat layout (SDK folder placement).
- ADR-0133 — industry-conformance program (SDK compliance bar).
- AWS S3 SDK reference patterns.
- Dropbox SDK reference patterns.
- Google Drive SDK reference patterns.
- tus.io 1.0 spec.
- RFC 9110 (HTTP); RFC 4918 (WebDAV).
