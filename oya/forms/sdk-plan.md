---
doc_class: SdkPlan
microservice: forms
status: Accepted
date: 2026-05-17
owner_team: axis-forms + council-design-system + axis-sdk
doc_status: published
---

# Forms — SDK Plan

## Public SDK Surface

Forms exposes three public SDK surfaces:

### 1. forms-rest-sdk (server-side)

For tenant backend integration; speaks REST 1:1 with `contracts/openapi/forms.openapi.yaml`.

- **Rust**: `oya-forms-sdk-rest` — uses `reqwest` + tokio.
- **TypeScript**: `@oyatie/forms-sdk` — uses `fetch`; bundled for browser + Node.
- **Python**: `oyatie-forms` — uses `httpx`; supports sync + async.
- **Go**: `github.com/oyatie/forms-sdk-go`.
- **Java**: `com.oyatie:forms-sdk:1.0.0`.

### 2. forms-embed-sdk (browser-side; iframe + JS widget)

For embedding a form on a tenant-owned page; CSP-aware; postMessage protocol for parent-window integration.

- **embed.js** — single-bundle ESM + UMD.
- Per-tenant CSP `frame-ancestors` configuration mandatory.
- postMessage events: `formLoaded`, `formSubmitted`, `formError`, `formResized`.

### 3. forms-async-sdk (event-stream)

For tenants subscribing to submission events; speaks AsyncAPI 3.0 from `contracts/asyncapi/forms.asyncapi.yaml`.

- **Rust + tokio + rdkafka** for high-volume tenants.
- **TypeScript** (Server-Sent Events fallback for low-volume).
- HMAC-signed payloads; tenant verifies signature.

## SDK Versioning

- Semver; major versions held LTS for 24 months.
- Breaking changes require ADR + sunset window per `feedback_no_silent_regression.md`.
- SDKs include `User-Agent: oyatie-forms-sdk/<version>` automatically; observability bridges this label.

## BYO-LLM SDK Hook

For AI-form-build BYO-LLM tenants:
- Tenant configures their LLM endpoint via tenancy entitlement.
- foundry-providers SDK routes calls; tenant LLM never sees other tenants' data.
- Zero-retention contract attestation required.

## SDK Compatibility Matrix

| SDK | Min runtime | Max runtime tested |
|---|---|---|
| oya-forms-sdk-rest (Rust) | 1.82 | 1.88 |
| @oyatie/forms-sdk (TS) | Node 20 LTS / browsers ES2022 | Node 22 LTS |
| oyatie-forms (Python) | 3.11 | 3.13 |
| forms-sdk-go | go 1.22 | go 1.24 |
| forms-sdk-java | JDK 17 LTS | JDK 21 LTS |

## SDK CI Gates

- Contract conformance: every SDK passes `oya-governance-sdk-contract-conformance` against `contracts/openapi/forms.openapi.yaml`.
- Backward compatibility: every minor release passes `oya-governance-sdk-backward-compat` (replays last-major test corpus).
- Doc coverage: every SDK ships full API reference + examples for the 14 industry competitors' canonical use cases.

## References

- `contracts/openapi/forms.openapi.yaml`.
- `contracts/asyncapi/forms.asyncapi.yaml`.
- ADR-0131 per-microservice flat layout.
- `feedback_no_silent_regression.md`.
- Semver 2.0.
