---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-005-block-types-kernel-domain
status: pending
execution_unit: ChangeSet
owner: axis-docs + council-design-system
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-wcag-22-aa-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: block-types kernel + domain (block schema + sanitisation per ADR-DOCS-0002)

## Intent

Implement the block-type system per ADR-DOCS-0002 (block-based per Notion). Defines block schema (paragraph, heading_1-3, ordered_list, unordered_list, checklist, table, image, embed, code, math, callout, divider, page_break) + InlineStyle + RenderedBlock. Sanitisation per `ammonia` for HTML; macros refused; XXE prevented.

## ChangeSet boundary

7 crates per layer mapping: kernel + domain + usecase + api + adapter + sdk + app.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-block-types-kernel/src/{lib,block,schema,inline_style,acl}.rs` | create |
| `microservices/docs/src/crates/oya-docs-block-types-domain/src/{lib,sanitiser,heading_hierarchy,alt_text_validator}.rs` | create |
| `microservices/docs/src/crates/oya-docs-block-types-usecase/src/{lib,validate_block_tree,apply_inline_style}.rs` | create |
| `microservices/docs/src/crates/oya-docs-block-types-api/src/lib.rs` | create |
| `microservices/docs/src/crates/oya-docs-block-types-adapter/src/lib.rs` | create |
| `microservices/docs/src/crates/oya-docs-block-types-sdk/src/lib.rs` | create |
| `microservices/docs/src/crates/oya-docs-block-types-app/src/main.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-block-types-domain -- heading_hierarchy
cargo nextest run -p oya-docs-block-types-domain -- alt_text_required
cargo nextest run -p oya-docs-block-types-domain -- ammonia_sanitiser_fuzz
buck2 build //:quality-lane-registry-authority-check # lane=wcag-22-aa-conformance --microservice docs
```

## References

- ADR-DOCS-0002 (block-type system).
- WCAG 2.2 AA — `w3.org/TR/WCAG22/`.
- ammonia HTML sanitiser — `crates.io/crates/ammonia`.

## Wave 15-IP-substance conversion (2026-05-21)
This addendum converts the short implementation note into a buildable docs-service slice for block types; it does not rely on line count as proof.
Counterpart anchors: Google Docs, Microsoft Word Online, Notion, Coda, Quip, GitHub.

### Problem closed
The gap is not generic documentation infrastructure. `docs` owns collaborative document authoring where rich blocks, CRDT edits, comments, suggestions, version history, sharing, export/import, embeds, and AI assist must work under one tenant and policy model.
For block types, the implementation must preserve dual-context separation, per-block ACL, legal hold, audit-chain records, and export/import safety described in `microservices/docs/PRD.md` and `microservices/docs/ARCHITECTURE.md`.

### Concrete mechanism
Primary artifact: `microservices/docs/catalog/oya-docs-block-types-kernel.yaml`.
Domain entities or operational surfaces: BlockKind, InlineStyle, RenderedBlock, sanitisation.
Contracts and gates: `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/proto/docs.proto`, and Cedar files under `microservices/docs/policy/`.
SLO/runbook evidence: `microservices/docs/slos/*.openslo.yaml`, `microservices/docs/dashboards/*.json`, and `microservices/docs/runbooks/*.md`.

### Implementation steps
1. Bind block types to the matching bounded context in `microservices/docs/manifest.json` and the catalog row named in this addendum.
2. Confirm every command/event carries tenant_id, principal_id, document context, data_class, traceparent, idempotency key for mutations, and audit_event_class.
3. Extend the contract or catalog artifact only where the cited file already owns that surface; do not invent a sibling µservice or fake Terraform module.
4. Add or update policy checks in `tenant-scope.cedar`, `editor-isolation.md`, `public-read.cedar`, or `auditor-scope.cedar` according to the feature's access path.
5. Add tests around accepted path, cross-tenant denial, stale version/anchor handling, legal-hold or retention interaction, and export/import rollback where applicable.
6. Attach SLO, dashboard, and runbook evidence before promotion so a reviewer can verify more than the presence of this Markdown file.

### Acceptance and counterpart comparison
| Counterpart | Expected behavior | Oyatie closure |
|---|---|---|
| Google Docs / Microsoft Word Online | Native collaborative authoring with comments, sharing, history, and export. | `block types` must be first-party, tenant-scoped, auditable, and legal-hold aware. |
| Notion / Coda | Block-centric documents with embeds and structured workflows. | `block types` must preserve block ACL, embed policy checks, and cross-service refresh evidence instead of hidden workspace coupling. |
| Quip / GitHub | Team review and change history expectations. | `block types` must expose signed audit events, version/revert evidence, and contract-rendered developer docs. |

### Evidence
- `microservices/docs/PRD.md`
- `microservices/docs/ARCHITECTURE.md`
- `microservices/docs/manifest.json`
- `microservices/docs/competitor-parity-matrix.md`
- `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md`
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`
