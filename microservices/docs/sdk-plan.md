---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: docs
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-docs + gtm-customer-success
deciders: axis-docs, council-architecture
related_adrs: [ADR-0131, ADR-0132, ADR-0133, ADR-DOCS-0001, ADR-DOCS-0002, ADR-DOCS-0006]
related_artifacts:
  - microservices/docs/contracts/openapi/docs.yaml
  - microservices/docs/contracts/proto/docs.proto
  - microservices/docs/contracts/asyncapi/docs-events.yaml
  - microservices/docs/PRD.md
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (docs µservice)

## Purpose

Tenants integrate with docs via four primary surfaces: the REST + WebSocket facade, programmatic SDKs, document standards (DOCX / Markdown / HTML / PDF / EPUB import-export), and embed-resolver protocol (cross-µservice references).

## Surface choice (first decision for tenants)

| Surface | Use when | Authority |
|---|---|---|
| REST facade (docs.yaml) | Tenant writes a custom doc app or backend pipeline | OpenAPI 3.2.0 |
| WebSocket gateway | Tenant builds a real-time co-editing client | WebSocket per RFC 6455; CRDT op envelope |
| gRPC (docs.proto) | Tenant runs a backend service; strongly-typed contracts | proto3 |
| OOXML / Markdown / HTML import-export | Tenant migrates from external docs or needs portable backup | ECMA-376 (OOXML) + CommonMark + GFM + HTML5 |
| PDF/A export | Tenant needs archival-grade evidence | PDF/A-1b + PDF/A-2u (ISO 19005) |
| EPUB export | Tenant publishes long-form content | EPUB 3 (W3C) |
| Per-language SDK | Tenant wants ergonomic auth + tenant binding + retry + CRDT client | this plan |

## Launch order (per ADR-CAL-0003-equivalent for docs SDK)

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M03 (oyatie's own language) | First-party authored `oya-docs-<bc>-sdk` crates per BC | axis-docs |
| **TypeScript** | M03 (Node + Browser) | OpenAPI-generated baseline + first-party CRDT client wrapper; published to npm; pairs with Loro WASM bindings | axis-docs + gtm |
| **Python** | M03+1 (data-pipeline + scripting tenants) | OpenAPI-generated; published to PyPI; pairs with `python-docx` for DOCX inspection | axis-docs + gtm |
| **Swift** | M03+1 (iOS / macOS partner-app integrators) | OpenAPI-generated; ergonomic wrapper; CRDT client uses TypeScript-on-WebKit fallback for now | axis-docs |
| **Go** | M04 (backend services + ops tools) | gRPC-generated baseline + ergonomic wrappers | axis-docs + gtm |
| **JVM (Kotlin / Java)** | M04 (enterprise tenants) | gRPC-generated baseline; Maven Central | axis-docs + gtm |
| **C# / .NET** | M05 (Microsoft-ecosystem tenants) | OpenAPI-generated; NuGet | axis-docs + gtm |

CRDT client library availability per ADR-DOCS-0001:
- Rust + TypeScript SDKs ship Loro bindings at M03.
- Python + Swift SDKs ship Loro bindings at M03+1 (WASM-binding via wasmer-python / WebKit-binding).
- Go + JVM + C# SDKs ship server-mediated CRDT (no client-side merge engine) at M04+; merge happens server-side; client receives merged snapshots.

## Generation strategy

### Rust SDKs (first-party)

Per-BC under `microservices/docs/src/crates/oya-docs-<bc>-sdk/`:

- `oya-docs-document-store-sdk`: read docs; write docs; legal-hold; tenant-DEK envelope helper
- `oya-docs-collab-crdt-sdk`: WebSocket client; CRDT op envelope encoding; client-side Loro merge
- `oya-docs-block-types-sdk`: block schema helpers + renderers (read-only client-side)
- `oya-docs-comments-and-suggestions-sdk`: thread + suggestion state machine; anchor stability helpers
- `oya-docs-version-history-sdk`: version listing + revert helpers
- `oya-docs-sharing-and-permissions-sdk`: share grant + share-link redemption
- `oya-docs-export-import-sdk`: client-side import/export pipeline orchestration (server handles heavy lifting in gVisor sandbox)
- `oya-docs-embed-resolver-sdk`: read embedded snapshots; cross-µservice embed binding helpers

Common shape:
- `Client::new(opts)` with OIDC token provider closure.
- `Client` bound to tenant + doc-context at construction; `X-Tenant-Id` + `X-Doc-Context` headers automatic.
- Built-in exponential backoff for 5xx + 429.
- WebSocket streaming for collab; auto-reconnect with op-buffer.
- Re-exports types from corresponding `-kernel` crate.
- `#![deny(unsafe_code)]`.

### Generated SDKs

Pipeline (lives in `microservices/docs/sdk-generation/`, future IP):

1. Source of truth: `contracts/openapi/docs.yaml` + `contracts/proto/docs.proto` + `contracts/asyncapi/docs-events.yaml`.
2. OpenAPI → language: `openapi-generator-cli` 7.x with language profile.
3. Proto → language: `protoc` + language plugin.
4. AsyncAPI → language: `asyncapi-generator` 2.x for typed event subscription clients.
5. Ergonomic wrapper hand-authored on top: auth helpers, tenant-context binding, retry policy + circuit-breaker matching Rust SDK behaviour.
6. CRDT client wrapper hand-authored per language: Rust + TS native; Python + Swift via WASM-binding.
7. Per-language CI lane: build + lint + integration-test against staging docs cluster.

### Document-format libraries (consume, don't re-author)

For document import/export client integration, leverage upstream libraries:

- TypeScript: `docx` (read-write OOXML); `marked` (CommonMark); wrap in ergonomic shim.
- Python: `python-docx`; `markdown`; `weasyprint` (for client-side PDF if needed).
- Swift: native UTIs; `markdown-attributedstring`.

## Public surface (across SDKs)

All SDKs expose:

| Capability | Method | Returns |
|---|---|---|
| List documents (by context) | `listDocuments(context, cursor)` | `DocumentPage` |
| Read document | `getDocument(id)` | `Document` (includes block tree) |
| Create document | `createDocument(req)` | `Document` |
| Edit document (block ops) | `applyBlockOps(doc_id, ops)` | `Document` |
| Stream collab ops (WS) | `streamCollab(doc_id)` | streaming `CrdtOp` |
| Apply local CRDT op | `applyLocalOp(doc_id, op)` | `()` (client-side merge first; server-sync async) |
| List versions | `listVersions(doc_id)` | `Version[]` |
| Revert to version | `revertToVersion(doc_id, version_id)` | `Document` |
| Add comment | `addComment(doc_id, anchor, text)` | `Comment` |
| Resolve comment | `resolveComment(comment_id)` | `Comment` |
| Add suggestion | `addSuggestion(doc_id, anchor, content)` | `Suggestion` |
| Accept / reject suggestion | `respondToSuggestion(suggestion_id, response)` | `Suggestion` |
| Share document | `shareDocument(doc_id, grantee, role)` | `ShareGrant` |
| Revoke share | `revokeShare(grant_id)` | `()` |
| Issue share-link | `issueShareLink(doc_id, role, expires_at)` | `ShareLink` |
| Set per-block ACL | `setBlockAcl(doc_id, block_id, acl)` | `BlockAcl` |
| Export document | `exportDocument(doc_id, format)` | `ExportJob` (async) |
| Import document | `importDocument(blob, format)` | `ImportJob` (async) |
| Upload attachment | `uploadAttachment(doc_id, blob)` | `Attachment` |
| Resolve embed | `resolveEmbed(embed_ref)` | `EmbedSnapshot` |
| Subscribe to events | `streamDocumentLifecycle()` | streaming events |

Helper utilities:
- Client-side Loro CRDT merge — Rust + TS + Python (via WASM) (mirrors server per ADR-DOCS-0001).
- Block-tree canonicalisation helper — Rust + TS + Python (for AC-02 byte-equality computation).
- OOXML round-trip helper — Rust + TS + Python (per ADR-DOCS-0006 fidelity matrix awareness).

## Tenant SDK onboarding

| Step | Owner |
|---|---|
| Issue OIDC + per-tenant DEK reference via OpenBao | ops-security |
| Provide tenant onboarding doc + SDK quick-start (per language) | gtm-customer-success |
| Provide sample workflow: how to subscribe to `DocumentEdited` in tenant pipeline | axis-docs |
| Provide co-editing client tutorial (WS + CRDT) | gtm + axis-docs |
| Quarterly SDK update notifications (breaking changes 6mo advance) | axis-docs |

## Sunset policy

| SDK | Sunset trigger | Window |
|---|---|---|
| Any SDK with < 1% tenant usage for ≥ 12mo | underused | 6mo advance + migration help |
| Generator lib upstream-deprecated | dep-deprecated | 12mo + auto-migrate where possible |
| Breaking API change in docs µservice | per-release | major version bump in SDK; backwards-adapter for 1 prior major |

Per `agent-skills:deprecation-and-migration`: every sunset emits an ADR-shaped notice + deprecation-warning in SDK + tenant comms.

## Versioning

- docs µservice: semver.
- SDK per language: matches docs major.minor; SDK patch independent.
- Compat matrix per language; CI lane verifies SDK against current + 1 prior major.
- CRDT op envelope schema version: pinned per ADR-DOCS-0001; cross-µservice consistency lane validates against workflow-studio op envelope.

## Open-source decision

Defer per-SDK OSS decision until API stable in production ≥ 6mo. Default: closed-source until tenant-driven request or competitive consideration.

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compat test: SDK version N+1 against docs versions N-1, N, N+1.
- Annual SDK telemetry review per language; underused sunsetted.
- CRDT cross-µservice consistency: `oya-governance-crdt-cross-microservice-consistency` lane validates docs-SDK CrdtOp envelope shape matches workflow-studio-SDK.

## References

- `microservices/docs/contracts/openapi/docs.yaml`
- `microservices/docs/contracts/proto/docs.proto`
- `microservices/docs/contracts/asyncapi/docs-events.yaml`
- ADR-0105 (13-layer enum; `sdk` is canonical)
- ADR-DOCS-0001 (CRDT — Loro 1.x; cross-µservice alignment with workflow-studio)
- ADR-DOCS-0002 (block-type system)
- ADR-DOCS-0006 (DOCX import fidelity policy)
- OpenAPI Generator — `openapi-generator.tech`
- gRPC — `grpc.io`
- Loro CRDT — `loro.dev`
- python-docx — `python-docx.readthedocs.io`
- `marked` — `marked.js.org`
- WeasyPrint — `weasyprint.org`
- Stripe SDK precedent — `stripe.com/docs/libraries`
- `microservices/workflow-studio/sdk-plan.md` — sibling reference (CRDT alignment).
- `microservices/calendar/sdk-plan.md` — sibling reference (SDK pattern).
