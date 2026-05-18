---
doc_class: SDKPlan
title: notes µservice — Client SDK Plan
microservice: notes
status: Accepted
date: 2026-05-17
owner_team: axis-notes + axis-sdk
doc_status: published
---

# SDK Plan — notes µservice

## SDK Languages

| Language | Target | Use | Status |
|---|---|---|---|
| TypeScript / JavaScript | browser + Node + Workflow Studio shell | web client + web-clipper extension + Workflow Studio plugins | M02 |
| Swift | iOS / macOS native | mobile + macOS client | subsequent-to-M02-completion |
| Kotlin | Android native | mobile client | subsequent-to-M02-completion |
| Rust | desktop (Tauri) + cross-platform | desktop client + Tauri-based mobile-companion | M02 |
| Python | server-side automation + research | tenant-side automation, Notion-migration scripts | subsequent-to-M02-completion |
| Go | server-side automation | tenant-side automation, sysadmin tooling | subsequent-to-M02-completion |

## E2E SDK Considerations

The TypeScript and Rust SDKs MUST support client-side MLS RFC 9420 (via `openmls` Wasm bindings for TS; `openmls 0.6` native for Rust) to derive Personal-tier keys.

Personal-tier search-index is built **client-side** per ADR-NOTES-0004 (encrypted-inverted-index in IndexedDB / SQLite); the SDK exposes:

- `index_personal_note(note_id, plaintext)` — builds token-bloom-filter locally.
- `search_personal(query)` → returns local `note_id[]`.

## API Surfaces

| Surface | Protocol | Versioning | Source of truth |
|---|---|---|---|
| REST | OpenAPI 3.2 | path `/api/v1` | `contracts/openapi/notes.yaml` |
| WebSocket realtime | AsyncAPI 3.0 | path `/ws/v1` | `contracts/asyncapi/notes-events.yaml` |
| Loro CRDT op stream | binary CBOR over WSS | `loro/1` subprotocol | `contracts/proto/notes.proto` |
| Web-clipper REST | OpenAPI 3.2 subset | path `/clip/v1` | `contracts/openapi/notes.yaml` (subpath) |

## SDK Capability Matrix

| Capability | TS | Swift | Kotlin | Rust | Python | Go |
|---|---|---|---|---|---|---|
| Note CRUD | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Tag CRUD + tag-graph | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Backlink resolution | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Daily-note auto-create | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Template materialisation | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Web-clipper bridge | ✓ (browser) | ✓ (Share Sheet) | ✓ (Share) | ✗ | ✗ | ✗ |
| Share-link emit + access | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Embed (drive ref) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Checklist parse + emit | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Version-history | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Search (Professional) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Search (Personal; client-side encrypted) | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ |
| Graph-view data | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Graph-view WebGL render | ✓ | ✓ (Metal) | ✓ (Vulkan) | ✓ (wgpu) | ✗ | ✗ |
| Loro CRDT collab | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ |
| Import (Obsidian/ENEX/etc.) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Export | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| MLS E2E (Personal) | ✓ (openmls-wasm) | ✓ (CryptoKit + openmls bridge) | ✓ (openmls-jvm) | ✓ (openmls 0.6) | ✗ | ✗ |
| AI-assist (Professional) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

## Type Generation

- OpenAPI → TS / Swift / Kotlin / Rust / Python / Go via `openapi-generator` + `oapi-codegen`.
- AsyncAPI → TS / Rust via `@asyncapi/generator`.
- Loro op proto → all languages via `prost` (Rust) + `ts-proto` (TS).

## Authentication

- OIDC bearer token in `Authorization` header.
- Per-tenant token rotation 30d.
- Web-clipper installation token rotation 90d.
- Loro CRDT WebSocket: bearer token in subprotocol parameter `loro/1;token=<jwt>`.

## SDK Release Cadence

- Major version: align with µservice release-pointer advance.
- Minor: monthly.
- Patch: as needed; semver.
- Deprecation: 2-cycle notice + sunset announcement.

## Distribution

| SDK | Channel |
|---|---|
| TS | npm `@oyatie/notes` + jsdelivr CDN |
| Swift | Swift Package Manager (Apple) |
| Kotlin | Maven Central |
| Rust | crates.io `oyatie-notes-sdk` (Rust workspace internal also re-exports) |
| Python | PyPI `oyatie-notes` |
| Go | `go.mod` from `github.com/oyatie/sdk-go-notes` |

## Web-Clipper Browser Extension

| Browser | Manifest | Distribution |
|---|---|---|
| Chrome | MV3 | Chrome Web Store |
| Edge | MV3 (Chromium) | Microsoft Edge Add-ons |
| Firefox | MV3 | Mozilla Add-ons (AMO) |
| Safari | Safari Web Extensions (Xcode + Apple Developer) | Mac App Store |

Per-installation token rotation 90d. CSP strict. Minimum-permission manifest (no broad host_permissions; activeTab only).

## References

- `contracts/openapi/notes.yaml`.
- `contracts/asyncapi/notes-events.yaml`.
- `contracts/proto/notes.proto`.
- ADR-NOTES-0003 (Loro CRDT).
- ADR-NOTES-0004 (search architecture).
