---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-009-version-history
status: pending
execution_unit: ChangeSet
owner: axis-docs
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-merkle-chain-continuity]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: version-history BC (8 crates)

## Intent

Implement Merkle-chained version snapshots + revert + CRDT op-log compaction per ADR-DOCS-0001 (version-aligned compaction policy).

## ChangeSet boundary

8 crates per layer mapping.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-version-history-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,app}/src/lib.rs` | create |
| `microservices/docs/src/crates/oya-docs-version-history-domain/src/{merkle_chain,compaction_planner,revert_validator}.rs` | create |
| `microservices/docs/src/crates/oya-docs-version-history-worker/src/{lib,compaction_worker,integrity_scan_worker}.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-version-history-domain -- merkle_chain_continuity
cargo nextest run -p oya-docs-version-history-domain -- compaction_byte_equality  # AC-02
cargo run -p oya-dev-cli -- gate validate merkle-chain-continuity --microservice docs
```

## References

- ADR-0028 (Bominal audit-chain).
- ADR-DOCS-0001 (compaction policy).

## Wave 15-IP-substance conversion (2026-05-21)
This addendum converts the short implementation note into a buildable docs-service slice for version history; it does not rely on line count as proof.
Counterpart anchors: Google Docs, Microsoft Word Online, Notion, Coda, Quip, GitHub.

### Problem closed
The gap is not generic documentation infrastructure. `docs` owns collaborative document authoring where rich blocks, CRDT edits, comments, suggestions, version history, sharing, export/import, embeds, and AI assist must work under one tenant and policy model.
For version history, the implementation must preserve dual-context separation, per-block ACL, legal hold, audit-chain records, and export/import safety described in `microservices/docs/PRD.md` and `microservices/docs/ARCHITECTURE.md`.

### Concrete mechanism
Primary artifact: `microservices/docs/catalog/oya-docs-version-history-kernel.yaml`.
Domain entities or operational surfaces: Version, Snapshot, RevertOp.
Contracts and gates: `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/proto/docs.proto`, and Cedar files under `microservices/docs/policy/`.
SLO/runbook evidence: `microservices/docs/slos/*.openslo.yaml`, `microservices/docs/dashboards/*.json`, and `microservices/docs/runbooks/*.md`.

### Implementation steps
1. Bind version history to the matching bounded context in `microservices/docs/manifest.json` and the catalog row named in this addendum.
2. Confirm every command/event carries tenant_id, principal_id, document context, data_class, traceparent, idempotency key for mutations, and audit_event_class.
3. Extend the contract or catalog artifact only where the cited file already owns that surface; do not invent a sibling µservice or fake Terraform module.
4. Add or update policy checks in `tenant-scope.cedar`, `editor-isolation.md`, `public-read.cedar`, or `auditor-scope.cedar` according to the feature's access path.
5. Add tests around accepted path, cross-tenant denial, stale version/anchor handling, legal-hold or retention interaction, and export/import rollback where applicable.
6. Attach SLO, dashboard, and runbook evidence before promotion so a reviewer can verify more than the presence of this Markdown file.

### Acceptance and counterpart comparison
| Counterpart | Expected behavior | Oyatie closure |
|---|---|---|
| Google Docs / Microsoft Word Online | Native collaborative authoring with comments, sharing, history, and export. | `version history` must be first-party, tenant-scoped, auditable, and legal-hold aware. |
| Notion / Coda | Block-centric documents with embeds and structured workflows. | `version history` must preserve block ACL, embed policy checks, and cross-service refresh evidence instead of hidden workspace coupling. |
| Quip / GitHub | Team review and change history expectations. | `version history` must expose signed audit events, version/revert evidence, and contract-rendered developer docs. |

### Evidence
- `microservices/docs/PRD.md`
- `microservices/docs/ARCHITECTURE.md`
- `microservices/docs/manifest.json`
- `microservices/docs/competitor-parity-matrix.md`
- `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md`
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`
