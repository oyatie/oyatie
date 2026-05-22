---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-006-collab-crdt-kernel-domain
status: pending
execution_unit: ChangeSet
owner: axis-docs
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-crdt-no-silent-loss, oya-governance-crdt-cross-microservice-consistency]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: collab-crdt kernel + domain + adapter (Loro 1.x integration; cross-µservice consistent per ADR-DOCS-0001 + ADR-WS-0001)

## Intent

Implement the CRDT substrate per ADR-DOCS-0001 (Loro 1.x). Cross-µservice consistent CrdtOp envelope shape with workflow-studio per ADR-WS-0001. Property tests for AC-06 never-silent-loss invariant. Deterministic projection to canonical block tree for AC-02 byte-equality.

## ChangeSet boundary

4 crates: kernel + domain + usecase + adapter (Loro wrapping).

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-collab-crdt-kernel/src/{lib,merge_engine,state,op,conflict}.rs` | create |
| `microservices/docs/src/crates/oya-docs-collab-crdt-domain/src/{lib,no_silent_loss,canonicalisation,conflict_surfacing}.rs` | create |
| `microservices/docs/src/crates/oya-docs-collab-crdt-usecase/src/{lib,apply_op,project_to_canonical,resolve_conflict}.rs` | create |
| `microservices/docs/src/crates/oya-docs-collab-crdt-adapter/src/{lib,loro_engine,projection}.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-collab-crdt-domain -- never_silent_loss  # AC-06
cargo nextest run -p oya-docs-collab-crdt-domain -- round_trip_byte_equality  # AC-02
cargo run -p oya-dev-cli -- gate validate crdt-no-silent-loss --microservice docs
cargo run -p oya-dev-cli -- gate validate crdt-cross-microservice-consistency
```

## References

- ADR-DOCS-0001 (Loro 1.x; cross-µservice consistent with workflow-studio).
- ADR-WS-0001 (CRDT library selection; primary cross-µservice authority).
- Loro CRDT — `loro.dev`.

## Wave 15-IP-substance conversion (2026-05-21)
This addendum converts the short implementation note into a buildable docs-service slice for collab CRDT; it does not rely on line count as proof.
Counterpart anchors: Google Docs, Microsoft Word Online, Notion, Coda, Quip, GitHub.

### Problem closed
The gap is not generic documentation infrastructure. `docs` owns collaborative document authoring where rich blocks, CRDT edits, comments, suggestions, version history, sharing, export/import, embeds, and AI assist must work under one tenant and policy model.
For collab CRDT, the implementation must preserve dual-context separation, per-block ACL, legal hold, audit-chain records, and export/import safety described in `microservices/docs/PRD.md` and `microservices/docs/ARCHITECTURE.md`.

### Concrete mechanism
Primary artifact: `microservices/docs/catalog/oya-docs-collab-crdt-kernel.yaml`.
Domain entities or operational surfaces: Loro CRDT op stream and no silent edit loss.
Contracts and gates: `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/proto/docs.proto`, and Cedar files under `microservices/docs/policy/`.
SLO/runbook evidence: `microservices/docs/slos/*.openslo.yaml`, `microservices/docs/dashboards/*.json`, and `microservices/docs/runbooks/*.md`.

### Implementation steps
1. Bind collab CRDT to the matching bounded context in `microservices/docs/manifest.json` and the catalog row named in this addendum.
2. Confirm every command/event carries tenant_id, principal_id, document context, data_class, traceparent, idempotency key for mutations, and audit_event_class.
3. Extend the contract or catalog artifact only where the cited file already owns that surface; do not invent a sibling µservice or fake Terraform module.
4. Add or update policy checks in `tenant-scope.cedar`, `editor-isolation.md`, `public-read.cedar`, or `auditor-scope.cedar` according to the feature's access path.
5. Add tests around accepted path, cross-tenant denial, stale version/anchor handling, legal-hold or retention interaction, and export/import rollback where applicable.
6. Attach SLO, dashboard, and runbook evidence before promotion so a reviewer can verify more than the presence of this Markdown file.

### Acceptance and counterpart comparison
| Counterpart | Expected behavior | Oyatie closure |
|---|---|---|
| Google Docs / Microsoft Word Online | Native collaborative authoring with comments, sharing, history, and export. | `collab CRDT` must be first-party, tenant-scoped, auditable, and legal-hold aware. |
| Notion / Coda | Block-centric documents with embeds and structured workflows. | `collab CRDT` must preserve block ACL, embed policy checks, and cross-service refresh evidence instead of hidden workspace coupling. |
| Quip / GitHub | Team review and change history expectations. | `collab CRDT` must expose signed audit events, version/revert evidence, and contract-rendered developer docs. |

### Evidence
- `microservices/docs/PRD.md`
- `microservices/docs/ARCHITECTURE.md`
- `microservices/docs/manifest.json`
- `microservices/docs/competitor-parity-matrix.md`
- `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md`
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`
