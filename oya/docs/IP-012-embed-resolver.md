---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-012-embed-resolver
status: pending
execution_unit: ChangeSet
owner: axis-docs
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-embed-resolver-acl-passthrough]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: embed-resolver BC (8 crates)

## Intent

Implement cross-µservice embed resolution to workflow-studio (canvases) + sheets (cells) + slides (decks). Cross-µservice mTLS; source-side ACL passthrough; cycle detection bounded at depth 3; stale-fallback. TTL ≤ 5 min with jitter; single-flight coalescing.

## ChangeSet boundary

8 crates: kernel + domain + usecase + api + adapter + rest + worker + app.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-embed-resolver-{kernel,domain,usecase,api,adapter,rest,worker,app}/src/lib.rs` | create |
| `microservices/docs/src/crates/oya-docs-embed-resolver-domain/src/{cycle_detection,acl_passthrough,stale_fallback}.rs` | create |
| `microservices/docs/src/crates/oya-docs-embed-resolver-adapter/src/{lib,workflow_studio_client,sheets_client,slides_client}.rs` | create |
| `microservices/docs/src/crates/oya-docs-embed-resolver-worker/src/{lib,refresh_worker,grant_revocation_listener}.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-embed-resolver-domain -- cycle_detection
cargo nextest run -p oya-docs-embed-resolver-domain -- stale_fallback  # AC-15
cargo nextest run -p oya-docs-embed-resolver-domain -- acl_passthrough_source_side
buck2 build //:quality-lane-registry-authority-check # lane=embed-resolver-acl-passthrough --microservice docs
```

## References

- ADR-DOCS-0004 (per-block ACL; embed-resolver source-side passthrough).
- `policy/data-residency.md` Invariant DR-04 (cross-pack embed snapshot-only).

## Wave 15-IP-substance conversion (2026-05-21)
This addendum converts the short implementation note into a buildable docs-service slice for embed resolver; it does not rely on line count as proof.
Counterpart anchors: Google Docs, Microsoft Word Online, Notion, Coda, Quip, GitHub.

### Problem closed
The gap is not generic documentation infrastructure. `docs` owns collaborative document authoring where rich blocks, CRDT edits, comments, suggestions, version history, sharing, export/import, embeds, and AI assist must work under one tenant and policy model.
For embed resolver, the implementation must preserve dual-context separation, per-block ACL, legal hold, audit-chain records, and export/import safety described in `microservices/docs/PRD.md` and `microservices/docs/ARCHITECTURE.md`.

### Concrete mechanism
Primary artifact: `microservices/docs/catalog/oya-docs-embed-resolver-kernel.yaml`.
Domain entities or operational surfaces: Embed, EmbedSource, RefreshSnapshot.
Contracts and gates: `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/proto/docs.proto`, and Cedar files under `microservices/docs/policy/`.
SLO/runbook evidence: `microservices/docs/slos/*.openslo.yaml`, `microservices/docs/dashboards/*.json`, and `microservices/docs/runbooks/*.md`.

### Implementation steps
1. Bind embed resolver to the matching bounded context in `microservices/docs/manifest.json` and the catalog row named in this addendum.
2. Confirm every command/event carries tenant_id, principal_id, document context, data_class, traceparent, idempotency key for mutations, and audit_event_class.
3. Extend the contract or catalog artifact only where the cited file already owns that surface; do not invent a sibling µservice or fake Terraform module.
4. Add or update policy checks in `tenant-scope.cedar`, `editor-isolation.md`, `public-read.cedar`, or `auditor-scope.cedar` according to the feature's access path.
5. Add tests around accepted path, cross-tenant denial, stale version/anchor handling, legal-hold or retention interaction, and export/import rollback where applicable.
6. Attach SLO, dashboard, and runbook evidence before promotion so a reviewer can verify more than the presence of this Markdown file.

### Acceptance and counterpart comparison
| Counterpart | Expected behavior | Oyatie closure |
|---|---|---|
| Google Docs / Microsoft Word Online | Native collaborative authoring with comments, sharing, history, and export. | `embed resolver` must be first-party, tenant-scoped, auditable, and legal-hold aware. |
| Notion / Coda | Block-centric documents with embeds and structured workflows. | `embed resolver` must preserve block ACL, embed policy checks, and cross-service refresh evidence instead of hidden workspace coupling. |
| Quip / GitHub | Team review and change history expectations. | `embed resolver` must expose signed audit events, version/revert evidence, and contract-rendered developer docs. |

### Evidence
- `microservices/docs/PRD.md`
- `microservices/docs/ARCHITECTURE.md`
- `microservices/docs/manifest.json`
- `microservices/docs/competitor-parity-matrix.md`
- `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md`
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`
