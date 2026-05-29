---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-013-rest-websocket-protocol
status: pending
execution_unit: ChangeSet
owner: axis-docs
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: REST + WebSocket frame protocol

## Intent

Author the document-store-rest + sharing-rest + comments-rest + export-import-rest endpoints per OpenAPI 3.2.0 + WebSocket gateway frame protocol for CRDT op streaming. OIDC + per-tenant API key + share-link token (Ed25519) auth surfaces.

## ChangeSet boundary

Per-BC `-rest` crates already scaffolded in earlier IPs; this IP wires the cross-cutting auth + protocol layer.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-shared-protocol/src/{lib,oidc,api_key,share_link_verify,ws_frame}.rs` | create (shared protocol crate) |
| `microservices/docs/src/crates/oya-docs-document-store-rest/src/lib.rs` | extend |
| `microservices/docs/src/crates/oya-docs-collab-crdt-worker/src/ws_gateway.rs` | extend (WebSocket protocol) |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-shared-protocol -- oidc_tenant_claim_extraction
cargo nextest run -p oya-docs-shared-protocol -- share_link_constant_time_verify
cargo nextest run -p oya-docs-shared-protocol -- ws_frame_signature_required
```

## References

- OpenAPI 3.2.0 spec — `spec.openapis.org/oas/v3.2.0`.
- RFC 6455 (WebSocket).
- `contracts/openapi/docs.yaml`.
- `contracts/asyncapi/docs-events.yaml`.

## Wave 15-IP-substance conversion (2026-05-21)
This addendum converts the short implementation note into a buildable docs-service slice for REST/WebSocket protocol; it does not rely on line count as proof.
Counterpart anchors: Google Docs, Microsoft Word Online, Notion, Coda, Quip, GitHub.

### Problem closed
The gap is not generic documentation infrastructure. `docs` owns collaborative document authoring where rich blocks, CRDT edits, comments, suggestions, version history, sharing, export/import, embeds, and AI assist must work under one tenant and policy model.
For REST/WebSocket protocol, the implementation must preserve dual-context separation, per-block ACL, legal hold, audit-chain records, and export/import safety described in `microservices/docs/PRD.md` and `microservices/docs/ARCHITECTURE.md`.

### Concrete mechanism
Primary artifact: `microservices/docs/contracts/openapi/docs.yaml`.
Domain entities or operational surfaces: HTTP commands, WebSocket CRDT stream, async events.
Contracts and gates: `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/proto/docs.proto`, and Cedar files under `microservices/docs/policy/`.
SLO/runbook evidence: `microservices/docs/slos/*.openslo.yaml`, `microservices/docs/dashboards/*.json`, and `microservices/docs/runbooks/*.md`.

### Implementation steps
1. Bind REST/WebSocket protocol to the matching bounded context in `microservices/docs/manifest.json` and the catalog row named in this addendum.
2. Confirm every command/event carries tenant_id, principal_id, document context, data_class, traceparent, idempotency key for mutations, and audit_event_class.
3. Extend the contract or catalog artifact only where the cited file already owns that surface; do not invent a sibling µservice or fake Terraform module.
4. Add or update policy checks in `tenant-scope.cedar`, `editor-isolation.md`, `public-read.cedar`, or `auditor-scope.cedar` according to the feature's access path.
5. Add tests around accepted path, cross-tenant denial, stale version/anchor handling, legal-hold or retention interaction, and export/import rollback where applicable.
6. Attach SLO, dashboard, and runbook evidence before promotion so a reviewer can verify more than the presence of this Markdown file.

### Acceptance and counterpart comparison
| Counterpart | Expected behavior | Oyatie closure |
|---|---|---|
| Google Docs / Microsoft Word Online | Native collaborative authoring with comments, sharing, history, and export. | `REST/WebSocket protocol` must be first-party, tenant-scoped, auditable, and legal-hold aware. |
| Notion / Coda | Block-centric documents with embeds and structured workflows. | `REST/WebSocket protocol` must preserve block ACL, embed policy checks, and cross-service refresh evidence instead of hidden workspace coupling. |
| Quip / GitHub | Team review and change history expectations. | `REST/WebSocket protocol` must expose signed audit events, version/revert evidence, and contract-rendered developer docs. |

### Evidence
- `microservices/docs/PRD.md`
- `microservices/docs/ARCHITECTURE.md`
- `microservices/docs/manifest.json`
- `microservices/docs/competitor-parity-matrix.md`
- `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md`
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`
