---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-003-document-store-domain-and-usecase
status: pending
execution_unit: ChangeSet
owner: axis-docs
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: document-store domain + usecase

## Intent

Implement pure document-invariant math (block-tree ordering validation, ACL coverage check, hold coverage check) + usecase orchestrators (create-document, update-metadata, archive, apply-legal-hold, expire-retention, restore-from-version).

## ChangeSet boundary

2 crates: `oya-docs-document-store-domain` (pure logic; reads via ports) + `oya-docs-document-store-usecase` (orchestration).

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-document-store-domain/Cargo.toml` | create |
| `microservices/docs/src/crates/oya-docs-document-store-domain/src/{lib,block_tree_validator,acl_coverage,hold_coverage,context_isolation}.rs` | create |
| `microservices/docs/src/crates/oya-docs-document-store-usecase/Cargo.toml` | create |
| `microservices/docs/src/crates/oya-docs-document-store-usecase/src/{lib,create_document,update_metadata,archive,apply_legal_hold,expire_retention,restore_from_version}.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-document-store-domain -- context_isolation
cargo nextest run -p oya-docs-document-store-domain -- per_block_acl
cargo nextest run -p oya-docs-document-store-domain -- legal_hold
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice docs
```

## References

- ADR-0105, ADR-0106.
- PRD AC-01, AC-04, AC-07, AC-13.

## Wave 15-IP-substance conversion (2026-05-21)
This addendum converts the short implementation note into a buildable docs-service slice for document-store domain/usecase; it does not rely on line count as proof.
Counterpart anchors: Google Docs, Microsoft Word Online, Notion, Coda, Quip, GitHub.

### Problem closed
The gap is not generic documentation infrastructure. `docs` owns collaborative document authoring where rich blocks, CRDT edits, comments, suggestions, version history, sharing, export/import, embeds, and AI assist must work under one tenant and policy model.
For document-store domain/usecase, the implementation must preserve dual-context separation, per-block ACL, legal hold, audit-chain records, and export/import safety described in `microservices/docs/PRD.md` and `microservices/docs/ARCHITECTURE.md`.

### Concrete mechanism
Primary artifact: `microservices/docs/catalog/oya-docs-document-store-kernel.yaml`.
Domain entities or operational surfaces: Document, BlockTree, RetentionPolicyRef, LegalHoldRef.
Contracts and gates: `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/proto/docs.proto`, and Cedar files under `microservices/docs/policy/`.
SLO/runbook evidence: `microservices/docs/slos/*.openslo.yaml`, `microservices/docs/dashboards/*.json`, and `microservices/docs/runbooks/*.md`.

### Implementation steps
1. Bind document-store domain/usecase to the matching bounded context in `microservices/docs/manifest.json` and the catalog row named in this addendum.
2. Confirm every command/event carries tenant_id, principal_id, document context, data_class, traceparent, idempotency key for mutations, and audit_event_class.
3. Extend the contract or catalog artifact only where the cited file already owns that surface; do not invent a sibling µservice or fake Terraform module.
4. Add or update policy checks in `tenant-scope.cedar`, `editor-isolation.md`, `public-read.cedar`, or `auditor-scope.cedar` according to the feature's access path.
5. Add tests around accepted path, cross-tenant denial, stale version/anchor handling, legal-hold or retention interaction, and export/import rollback where applicable.
6. Attach SLO, dashboard, and runbook evidence before promotion so a reviewer can verify more than the presence of this Markdown file.

### Acceptance and counterpart comparison
| Counterpart | Expected behavior | Oyatie closure |
|---|---|---|
| Google Docs / Microsoft Word Online | Native collaborative authoring with comments, sharing, history, and export. | `document-store domain/usecase` must be first-party, tenant-scoped, auditable, and legal-hold aware. |
| Notion / Coda | Block-centric documents with embeds and structured workflows. | `document-store domain/usecase` must preserve block ACL, embed policy checks, and cross-service refresh evidence instead of hidden workspace coupling. |
| Quip / GitHub | Team review and change history expectations. | `document-store domain/usecase` must expose signed audit events, version/revert evidence, and contract-rendered developer docs. |

### Evidence
- `microservices/docs/PRD.md`
- `microservices/docs/ARCHITECTURE.md`
- `microservices/docs/manifest.json`
- `microservices/docs/competitor-parity-matrix.md`
- `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md`
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`
