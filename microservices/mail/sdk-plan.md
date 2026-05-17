---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: mail
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-mail + gtm-customer-success
deciders: axis-mail, council-architecture
related_adrs: [ADR-0131, ADR-0132, ADR-0133]
related_artifacts:
  - microservices/mail/contracts/openapi/mail.yaml
  - microservices/mail/contracts/proto/mail.proto
  - microservices/mail/contracts/asyncapi/mail-events.yaml
  - microservices/mail/PRD.md
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (mail µservice)

## Purpose

Tenants integrate with mail via three primary surfaces: standard mail protocols (SMTP / IMAP4rev2 / JMAP), the REST facade, and programmatic SDKs. This document specifies the SDK strategy: which languages, generation pipeline, guarantees, sunset policy.

## Surface choice (first decision for tenants)

| Surface | Use when | Authority |
|---|---|---|
| SMTP / IMAP / JMAP wire protocol | Tenant uses Apple Mail, Thunderbird, Outlook, mobile mail clients, off-the-shelf MTAs | RFC standards (5321, 9051, 8620/8621) |
| REST facade (mail.yaml) | Tenant writes a custom mail app or backend pipeline; wants JSON over HTTP | OpenAPI 3.2.0 |
| gRPC (mail.proto) | Tenant runs a backend service; wants strongly-typed contracts + streaming | proto3 |
| Per-language SDK | Tenant wants ergonomic auth + tenant binding + retry | this plan |

## Languages

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M03 (oyatie's own language) | First-party authored (`oya-mail-*-sdk` crates per BC; per PRD §"BC layer mapping") | axis-mail |
| **TypeScript** | M03 (Node + Browser; bundled JMAP-Mail-shaped client) | OpenAPI-generated baseline + first-party ergonomic wrappers; published to npm | axis-mail + gtm |
| **Python** | M03+1 (data-pipeline + scripting tenants) | OpenAPI-generated; published to PyPI; pairs with JMAP `pyjmap` reference lib | axis-mail + gtm |
| **Go** | M04 (backend services + ops tools) | gRPC-generated baseline + ergonomic wrappers; published as go-module | axis-mail + gtm |
| **JVM (Kotlin / Java)** | M04 (enterprise tenants) | gRPC-generated baseline; Maven Central | axis-mail + gtm |
| **Swift / Objective-C** | M03+1 (iOS / macOS partner-app integrators) | thin wrapper over JMAP-Core HTTP; published as Swift Package | axis-mail |
| **C# / .NET** | M05 (Microsoft-ecosystem tenants) | OpenAPI-generated; NuGet | axis-mail + gtm |

Prioritisation: oyatie's languages first (Rust), then tenant developer-population leaders (TS + Python), then mobile (Swift), then enterprise (Go/JVM/C#).

## Generation strategy

### Rust SDKs (first-party)

Per-BC under `microservices/mail/src/crates/oya-mail-<bc>-sdk/`:
- `oya-mail-mailbox-store-sdk`: read mailbox/thread/message; submit outbound; encrypted-token search helpers
- `oya-mail-legal-hold-sdk`: engage/release; eDiscovery export request + verify
- `oya-mail-search-index-sdk`: Cipher-Match HMAC tokenizer (client-side; plaintext never escapes)
- `oya-mail-imap-frontend-sdk`: IMAP / JMAP client over the REST facade for non-mail-client integrators

Common shape:
- `Client::new(opts)` with OIDC token provider closure.
- `Client` bound to tenant + mail-context at construction; `X-Tenant-Id` + `X-Mail-Context` headers automatic.
- Built-in exponential backoff for 5xx + 429.
- gRPC streaming where applicable (e.g., `stream_message_received`).
- Re-exports types from corresponding `-kernel` crate.
- `#![deny(unsafe_code)]`.

### Generated SDKs

Pipeline (lives in `microservices/mail/sdk-generation/`, future IP):

1. Source of truth: `contracts/openapi/mail.yaml` + `contracts/proto/mail.proto` + `contracts/asyncapi/mail-events.yaml`.
2. OpenAPI → language: `openapi-generator-cli` 7.x with language profile.
3. Proto → language: `protoc` + language plugin.
4. AsyncAPI → language: `asyncapi-generator` 2.x for typed event subscription clients.
5. Ergonomic wrapper hand-authored on top: auth helpers, tenant-context binding, retry policy + circuit-breaker matching Rust SDK behavior.
6. Per-language CI lane: build + lint + integration-test against staging mail cluster.

### Standards-compatible JMAP libraries (consume, don't re-author)

For JMAP authoring, we leverage upstream JMAP libraries rather than re-author:
- TypeScript: `jmap-jam` (Fastmail-maintained); wrap in ergonomic shim.
- Python: `pyjmap` (reference impl); wrap.
- Swift: native JMAP-over-HTTP; thin wrapper.
- Apple Mail / Thunderbird: use standard IMAP via `imap-frontend` adapter; no SDK needed.

## Public surface (across SDKs)

All SDKs expose:

| Capability | Method | Returns |
|---|---|---|
| List mailboxes (by context) | `listMailboxes(context)` | `Mailbox[]` |
| Read mailbox | `getMailbox(id)` | `Mailbox` |
| List threads (paginated) | `listThreads(mailbox, folder, cursor)` | `ThreadPage` |
| Read message | `getMessage(mailbox, id)` | `MailMessage` |
| Submit message | `sendMessage(envelope)` | `SubmissionReceipt` |
| Search (encrypted-token) | `searchMail(req)` | `SearchResultPage` |
| Engage legal hold | `engageLegalHold(req)` | `LegalHold` |
| Request eDiscovery export | `createEdiscoveryExport(req)` | `EdiscoveryExportJob` |
| Verify eDiscovery bundle | `verifyEdiscoveryBundle(path)` | `VerificationReport` |
| Subscribe to events | `streamMessageReceived()` | streaming events |

Helper utilities:
- `Cipher-Match` HMAC tokenizer (client-side; for search) — Rust + TS + Python (mandatory before search call)
- MIME builder helper — Rust + TS + Python
- S/MIME + OpenPGP signing helper — Rust + TS (browser+node) + Swift

## Tenant SDK onboarding

| Step | Owner |
|---|---|
| Issue OIDC + per-tenant DEK reference via OpenBao | ops-security |
| Provide tenant onboarding doc + SDK quick-start (per language) | gtm-customer-success |
| Provide sample workflow: how to subscribe to `MessageReceived` in tenant pipeline | axis-mail |
| Provide Cipher-Match tokenizer + search example | axis-mail |
| Quarterly SDK update notifications (breaking changes 6mo advance) | axis-mail |

## Sunset policy

| SDK | Sunset trigger | Window |
|---|---|---|
| Any SDK with < 1% tenant usage for ≥ 12mo | underused | 6mo advance + migration help |
| Generator lib upstream-deprecated | dep-deprecated | 12mo + auto-migrate where possible |
| Breaking API change in mail µservice | per-release | major version bump in SDK; backwards-adapter for 1 prior major |

Per `agent-skills:deprecation-and-migration`: every sunset emits an ADR-shaped notice + deprecation-warning in SDK + tenant comms.

## Versioning

- mail µservice: semver.
- SDK per language: matches mail major.minor; SDK patch independent.
- Compat matrix per language; CI lane verifies SDK against current + 1 prior major.

## Open-source decision

Defer per-SDK OSS decision until API stable in production ≥ 6mo. Default: closed-source until tenant-driven request or competitive consideration. Stripe + Twilio precedent.

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compat test: SDK version N+1 against mail versions N-1, N, N+1.
- Annual SDK telemetry review per language; underused sunsetted.

## References

- `microservices/mail/contracts/openapi/mail.yaml`
- `microservices/mail/contracts/proto/mail.proto`
- `microservices/mail/contracts/asyncapi/mail-events.yaml`
- ADR-0105 (13-layer enum; `sdk` is canonical)
- OpenAPI Generator — `openapi-generator.tech`
- gRPC — `grpc.io`
- jmap-jam — `github.com/fastmailhq/jmap-jam`
- pyjmap — `github.com/python-jmap/pyjmap`
- Stripe SDK precedent — `stripe.com/docs/libraries`
- ProtonMail SDK precedent — `proton.me/business/api`
