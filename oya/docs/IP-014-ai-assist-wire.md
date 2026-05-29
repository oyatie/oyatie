---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-014-ai-assist-wire
status: pending
execution_unit: ChangeSet
owner: axis-docs + foundry-runtime + council-privacy
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-ai-act-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014: AI writing-assist wire (T0/T1/T2 per ADR-DOCS-0005)

## Intent

Wire T0/T1/T2 capabilities from `capabilities/*.yaml` to the editor REST + worker. Tenant-DEK-wrapped prompts via foundry-runtime SDK. Cedar policy gates per capability tier. EU AI Act Annex III §3 HR-context REFUSED at Cedar layer for pack-eu per ADR-DOCS-0005.

## ChangeSet boundary

Capabilities surface crates: shared AI-assist port + per-capability adapters under document-store-usecase + cross-cutting Cedar policy.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-ai-assist-shared/src/{lib,prompt_envelope,capability_dispatcher,reversibility_window}.rs` | create |
| `microservices/docs/src/crates/oya-docs-document-store-usecase/src/{ai_grammar_suggest,ai_auto_summary,ai_expand_rewrite,ai_citation_suggest,ai_grammar_bulk_fix,ai_auto_translate,ai_auto_format,ai_auto_cite}.rs` | create |
| `microservices/docs/policy/ai-act-hr-scope.cedar` | create (NEW; refuses T1/T2 HR-context in pack-eu) |
| `microservices/docs/policy/t1-tenant-class-gate.cedar` | create |
| `microservices/docs/policy/t2-tenant-class-gate.cedar` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-ai-assist-shared -- prompt_envelope_tenant_dek_wrapped
cargo nextest run -p oya-docs-ai-assist-shared -- reversibility_window_audit_emit
cargo run -p oya-dev-cli -- gate validate ai-act-conformance --microservice docs
```

## References

- ADR-DOCS-0005 (AI writing-assist EU AI Act bounds).
- `capabilities/T0-suggest.yaml`, `T1-assist.yaml`, `T2-auto.yaml`.
- EU AI Act Regulation (EU) 2024/1689.

## Wave 15-IP-substance conversion (2026-05-21)
This addendum converts the short implementation note into a buildable docs-service slice for AI assist wire; it does not rely on line count as proof.
Counterpart anchors: Google Docs, Microsoft Word Online, Notion, Coda, Quip, GitHub.

### Problem closed
The gap is not generic documentation infrastructure. `docs` owns collaborative document authoring where rich blocks, CRDT edits, comments, suggestions, version history, sharing, export/import, embeds, and AI assist must work under one tenant and policy model.
For AI assist wire, the implementation must preserve dual-context separation, per-block ACL, legal hold, audit-chain records, and export/import safety described in `microservices/docs/PRD.md` and `microservices/docs/ARCHITECTURE.md`.

### Concrete mechanism
Primary artifact: `microservices/docs/capabilities/T0-suggest.yaml`.
Domain entities or operational surfaces: T0-suggest, T1-assist, T2-auto bounded writing assist.
Contracts and gates: `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/proto/docs.proto`, and Cedar files under `microservices/docs/policy/`.
SLO/runbook evidence: `microservices/docs/slos/*.openslo.yaml`, `microservices/docs/dashboards/*.json`, and `microservices/docs/runbooks/*.md`.

### Implementation steps
1. Bind AI assist wire to the matching bounded context in `microservices/docs/manifest.json` and the catalog row named in this addendum.
2. Confirm every command/event carries tenant_id, principal_id, document context, data_class, traceparent, idempotency key for mutations, and audit_event_class.
3. Extend the contract or catalog artifact only where the cited file already owns that surface; do not invent a sibling µservice or fake Terraform module.
4. Add or update policy checks in `tenant-scope.cedar`, `editor-isolation.md`, `public-read.cedar`, or `auditor-scope.cedar` according to the feature's access path.
5. Add tests around accepted path, cross-tenant denial, stale version/anchor handling, legal-hold or retention interaction, and export/import rollback where applicable.
6. Attach SLO, dashboard, and runbook evidence before promotion so a reviewer can verify more than the presence of this Markdown file.

### Acceptance and counterpart comparison
| Counterpart | Expected behavior | Oyatie closure |
|---|---|---|
| Google Docs / Microsoft Word Online | Native collaborative authoring with comments, sharing, history, and export. | `AI assist wire` must be first-party, tenant-scoped, auditable, and legal-hold aware. |
| Notion / Coda | Block-centric documents with embeds and structured workflows. | `AI assist wire` must preserve block ACL, embed policy checks, and cross-service refresh evidence instead of hidden workspace coupling. |
| Quip / GitHub | Team review and change history expectations. | `AI assist wire` must expose signed audit events, version/revert evidence, and contract-rendered developer docs. |

### Evidence
- `microservices/docs/PRD.md`
- `microservices/docs/ARCHITECTURE.md`
- `microservices/docs/manifest.json`
- `microservices/docs/competitor-parity-matrix.md`
- `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md`
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`
