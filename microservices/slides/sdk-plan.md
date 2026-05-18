---
doc_class: SdkPlan
template_id: TPL-SDK-PLAN
microservice: slides
status: Accepted
date: 2026-05-17
owner_team: axis-workspace + axis-developer-experience
doc_status: published
---

# SDK plan — slides µservice

## Surface

Per-BC SDK crates under `oya-slides-<bc>-sdk` (Rust, semver-pinned to µservice major version), with a top-level façade `oya-slides-sdk` re-exporting common types.

## Languages

| Language | Status | Source |
|---|---|---|
| Rust | first-class; published to internal registry | `src/crates/oya-slides-*-sdk/` |
| TypeScript | subsequent-to-GA-tier-promotion; generated from OpenAPI 3.2.0 + protobuf | `sdk/typescript/` (subsequent-to-M03-completion) |
| Python | subsequent-to-GA-tier-promotion; generated from OpenAPI 3.2.0 | `sdk/python/` (subsequent-to-M03-completion) |
| Go | subsequent-to-GA-tier-promotion; generated from protobuf | `sdk/go/` (subsequent-to-M03-completion) |

## API surfaces exposed

| BC | Public surface (SDK) | Internal-only |
|---|---|---|
| presentation | deck CRUD (create / get / list / delete); deck-level ACL CRUD; version-history navigation | Postgres adapter |
| slide | slide CRUD; reorder; per-slide ACL | adapter-postgres |
| real-time-collaboration | `CrdtOp` envelope submission + subscription via WS-bridge; `EditorSession` lease query | Loro types (never exposed) |
| chart | bind chart to sheets cell-range; refresh policy CRUD; revocation observation | sheets-SDK consumer logic |
| broadcast-mode | session start/stop; viewer count subscription | LiveKit credentials |
| ai-design + ai-content-generation | suggestion-request + decision callback | foundry-runtime invocation details |
| themes + templates | gallery list + apply + custom theme upload | signing key |
| import-export | submit job; subscribe to job result | gVisor worker internals |
| accessibility | alt-text suggest + contrast check + reduced-motion-policy CRUD | foundry-runtime invocation |
| embed-bridge | bind / unbind embed; revocation observation | per-target SDK details |
| acl | deck-level + per-slide ACL CRUD | Cedar evaluator internals |

## SDK quality bar

- Every SDK exposes typed errors mapped from OpenAPI 3.2.0 error schemas (no anyhow leaks).
- Every SDK respects per-tenant OIDC; no SDK consumer can spoof tenant binding.
- Pagination via cursor (opaque tokens) per `contracts/openapi/slides.yaml`.
- WS streaming uses the typed AsyncAPI 3.1.0 event envelope.
- Backward compatibility per semver; breaking changes require ADR-0140 (retired per ADR-0145) deprecation cycle.

## Tenant SDK consumers

- Tenant Rust apps (workspace admin tooling).
- Cross-µservice oyatie consumers (docs / sheets / forms / drive / messenger / mail / social / observability / foundry-runtime).
- Future TypeScript / Python / Go SDKs for external integrators (subsequent-to-GA-tier-promotion).

## Cross-µservice consumption pattern

Per `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md`: all cross-µservice flows MUST route through SDK boundaries. Slides consumes:

| Target | Slides SDK consumer call site |
|---|---|
| sheets | `chart` BC — bind cell range + refresh listen |
| docs | `embed-bridge` BC — quote bind |
| forms | `embed-bridge` BC — poll bind |
| drive | `presentation` BC — asset storage hierarchy |
| messenger | `broadcast-mode` BC — LiveKit signaling reuse |
| social | `embed-bridge` BC — publish-as-shorts |
| mail | `embed-bridge` BC — share-via-email |
| ontology | every BC — read object-type descriptors |
| foundry-runtime | `ai-design` + `ai-content-generation` BCs — T0/T1/T2 invocation |
| tenancy | `acl` + `presentation` BCs — per-pack residency + per-seat licensing |
| audit-chain | every BC — Ed25519 seal |
| observability | every BC — SLI + metric emission |

## SDK release lifecycle

- ChangeSet IP per SDK breaking change.
- semver versioning.
- Backward compatibility period: 6 months for major bumps.
- ADR per breaking change.

## SDK testing

- Each SDK crate has integration tests against a mock slides-rest server (`tests/sdk-integration/`).
- Round-trip property tests (e.g., op envelope serialize/deserialize byte-equal).
- WS streaming reliability tests (reconnect, backpressure, ordering).

## SDK documentation

- Per-crate `cargo doc`.
- Top-level `microservices/slides/src/crates/oya-slides-sdk/README.md` orientation.
- Code examples for the 10 most-common consumer flows.

## SDK security

- All SDKs require OIDC tenant token.
- SDKs reject server responses that don't match expected schema (defensive deserialization).
- SDKs propagate audit-chain trace IDs.

## Migration / deprecation

- Slides is net-new per ADR-0135; no legacy SDK migration.
- Future deprecations: ADR-required per ADR-0140; tenant comms 90d before removal.

## References

- ADR-0140 Cedar policy enforcement.
- ADR-0135 Connect dissolution (no legacy migration).
- `feedback_workflow_objectgraph_adapter_layer.md` (cross-µservice SDK-only rule).
