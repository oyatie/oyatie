---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: sheets
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-sheets + council-design-system + gtm-customer-success
deciders: axis-sheets, council-architecture
related_adrs: [ADR-0065, ADR-0105, ADR-0126, ADR-0131]
related_artifacts:
  - microservices/sheets/contracts/openapi/sheets.yaml
  - microservices/sheets/contracts/proto/sheets.proto
  - microservices/sheets/PRD.md
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (sheets µservice)

## Purpose

Tenants integrating with sheets programmatically — agentic-developer role (LLM-emitted workbooks), CI/CD pipelines (git-backed authoring), tenant operator scripts (XLSX import/export, connected-sheets refresh), data-pipeline operators — need first-party SDKs in their workloads' languages.

Sheets's primary user-facing surface is the browser (Leptos WASM cell-grid). SDKs are for programmatic tenant integrations only.

## Languages

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M03 (oyatie's own language) | First-party authored (`oya-sheets-cell-grid-sdk` + per-BC SDK crates) | axis-sheets |
| **TypeScript** | M03+1 (first external-tenant SDK; matches browser-tenant integration) | OpenAPI-generated baseline + first-party ergonomic wrappers; published to npm | axis-sheets + gtm |
| **Python** | M04 (data-pipeline + LLM-orchestrator tenants) | OpenAPI-generated; published to PyPI | axis-sheets + gtm |
| **Go** | M04 (CI/CD agent tenants) | gRPC-generated baseline + ergonomic wrappers; go-module | axis-sheets + gtm |
| **JVM (Kotlin / Java)** | M05+ | gRPC-generated baseline; Maven Central | axis-sheets + gtm |
| **C# / .NET** | M05+ | OpenAPI-generated; NuGet | axis-sheets + gtm |
| **Ruby / PHP** | (none — no demand) | n/a | n/a |

Prioritisation drivers: oyatie's own µservices first; then top tenant-developer-population languages (TypeScript for browser-tenant integrations; Python for data-pipeline + LLM-orchestrator tenants).

## Generation Strategy

### Rust SDK (first-party)

Lives in `microservices/sheets/src/crates/oya-sheets-{cell-grid,formula-engine,recalc-engine,collab-crdt,import-export,sharing-acl,comments,version-history,ai-formula,connected-sheets,trigger-bridge,embed-bridge,license-gate-cedar}-sdk/`.

Per ADR-0105 `-sdk` is a canonical layer. Each BC's SDK crate is first-party authored with:
- `Client::new(opts) -> Client; client.method(...) -> Result<...>`
- OIDC token provider abstraction.
- Tenant-context binding at construction; `X-Scope-OrgID` header automatically populated.
- Retry policy: exponential backoff for 5xx + 429.
- gRPC streaming via `tonic` for collab-crdt + recalc-progress + xlsx-export.
- Re-exports kernel types so consumers see consistent shapes.
- `#![deny(unsafe_code)]`.

### Generated SDKs (TypeScript / Python / Go / JVM / C#)

Generation pipeline at `microservices/sheets/sdk-generation/`:

1. Source of truth: `contracts/openapi/sheets.yaml` (REST) + `contracts/proto/sheets.proto` (gRPC).
2. OpenAPI → language: `openapi-generator-cli` 7.x with language-specific profile.
3. Proto → language: `protoc` + per-language plugin.
4. Ergonomic wrapper: hand-authored thin layer; provides:
   - OIDC auth helpers.
   - Tenant-context binding at client construction.
   - Retry policy + circuit-breaker matching Rust SDK behavior.
   - Per-language idiom.
5. Per-language SDK ships with README + quick-start + versioning + compatibility matrix + license header.
6. Per-language CI lane: build + lint + integration-test against staging Sheets cluster.

## Public Surface (across languages)

All SDKs expose:

| Capability | Method | Returns |
|---|---|---|
| Open workbook session | `openWorkbookSession(workbook_id)` | `WorkbookSession` |
| Get workbook session | `getWorkbookSession(session_id)` | `WorkbookSession` |
| Close workbook session | `closeWorkbookSession(session_id)` | void |
| Load workbook | `loadWorkbook(workbook_id, jurisdiction?)` | `{workbook, jurisdiction_view}` |
| Save workbook | `saveWorkbook(workbook_id, cell_ops, cedar_preview_acknowledged?)` | `SaveResponse` |
| Read cell | `readCell(workbook_id, sheet_id, cell_ref)` | `Cell` |
| Write cell | `writeCell(workbook_id, sheet_id, cell_ref, value, formula?)` | `WriteAck` |
| Read range | `readRange(workbook_id, sheet_id, range)` | `Range` |
| Write range | `writeRange(workbook_id, sheet_id, range, values)` | `WriteAck` |
| Add named range | `addNamedRange(workbook_id, name, scope, range)` | `NamedRange` |
| Apply conditional formatting | `applyConditionalFormat(workbook_id, sheet_id, range, rule)` | `ConditionalFormat` |
| Author pivot table | `authorPivot(workbook_id, sheet_id, config)` | `Pivot` |
| Author chart | `authorChart(workbook_id, sheet_id, config)` | `Chart` |
| Add data-validation rule | `addDataValidation(workbook_id, sheet_id, range, rule)` | `ValidationRule` |
| Share workbook (with per-range ACL) | `shareWorkbook(workbook_id, principal, acl)` | `Share` |
| Import XLSX (async) | `importXlsx(file_url)` | `ImportJob` (poll via `getImportJob`) |
| Export XLSX (async) | `exportXlsx(workbook_id, fidelity_tier?)` | `ExportJob` (poll via `getExportJob`) |
| Stream CRDT ops (bidirectional) | `streamCrdtOps(...)` | stream `CrdtOpAck` (gRPC bidi streaming) |
| Stream recalc progress | `streamRecalcProgress(workbook_id)` | stream `RecalcFrame` (gRPC streaming) |
| Request AI-formula draft | `aiFormulaDraft(workbook_id, sheet_id, cell_ref, prose, consent_acknowledged)` | `AiFormulaDraftResponse` |
| Request smart-fill inference | `smartFillInfer(workbook_id, sheet_id, range, seed_cells)` | `SmartFillResponse` |
| Register connected-query | `registerConnectedQuery(workbook_id, sheet_id, range, source_descriptor, refresh_policy)` | `ConnectedQuery` |
| Refresh connected-query | `refreshConnectedQuery(query_id)` | `RefreshAck` |
| Add comment | `addComment(workbook_id, sheet_id, cell_ref, content)` | `Comment` |
| Restore from version-history | `restoreVersion(workbook_id, version_pointer)` | `RestoreAck` |
| Open embed-bridge (docs / slides) | `openEmbedBridge(workbook_id, target_microservice, target_id, range_or_chart)` | `EmbedDescriptor` |

Schema authoring (function-library upgrade, formula-engine version pin) is via the Sheets binary release + git PR; SDKs do NOT expose a "publish function-library version" method — that requires release-pipeline signoff per ADR-SHEETS-0002.

## Tenant SDK Onboarding

| Step | Owner |
|---|---|
| Issue OIDC client + tenant-scoped SDK API key via OpenBao | ops-security |
| Provide tenant onboarding doc with SDK quick-start per language | gtm-customer-success |
| Sample workflow: workbook round-trip via Rust SDK (developer persona) | axis-sheets |
| Sample workflow: LLM-orchestrator tenant submits AI-formula via Python SDK | axis-sheets + foundry-runtime |
| Sample workflow: data-pipeline tenant ingests via Python SDK | axis-sheets + gtm |
| Quarterly SDK update notification (breaking changes 6mo advance notice) | axis-sheets |

## Sunset Policy

| SDK | Sunset trigger | Sunset window |
|---|---|---|
| Any SDK with < 1% of tenant usage for ≥ 12mo | underused | 6mo advance notice + migration help |
| Any SDK whose generator lib is deprecated upstream | dep-deprecated | 12mo advance notice + auto-migration to replacement |
| Breaking API change in sheets | per-release | major version bump in SDK; backwards-compatible adapter for 1 prior major version |

## Versioning

- sheets µservice version: semver.
- SDK version per language: matches sheets major.minor; SDK patch independent.
- Compatibility matrix: published per-language; CI lane verifies SDK against current + 1 prior major.

## Open-Source Decision

Defer per-SDK open-source decision until API has been stable in production for ≥ 6mo. Default: keep SDKs closed-source until tenant-driven request OR competitive consideration. Same precedent as workflow-studio + observability µservices.

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compatibility test: SDK version N+1 works against sheets versions N-1, N, N+1.
- Annual SDK telemetry review: usage per SDK; underused flagged for sunset review.

## References

- `microservices/sheets/contracts/openapi/sheets.yaml`.
- `microservices/sheets/contracts/proto/sheets.proto`.
- `microservices/sheets/contracts/asyncapi/sheets-events.yaml`.
- ADR-0105 `sdk` canonical layer.
- ADR-0126 Sheets net-new µservice.
- ADR-0131 per-microservice flat layout.
- OpenAPI Generator — `openapi-generator.tech`.
- tonic (Rust gRPC) — `github.com/hyperium/tonic`.
- Stripe SDK precedent — `stripe.com/docs/libraries`.
- Twilio SDK precedent — `twilio.com/docs/libraries`.
