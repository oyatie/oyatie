---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-004-document-store-adapter-postgres-and-s3
status: pending
execution_unit: ChangeSet
owner: axis-docs
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-rls-coverage]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: document-store -adapter-postgres + -adapter-s3 + RLS schema + tenant-DEK envelope

## Intent

Implement DocumentRepository + AclRepository + LegalHoldStore against Postgres 16 LTS with per-tenant + per-block RLS. Implement BlockBlobStore + AttachmentStorage against OCI Object Storage (S3-compatible). Apply tenant-DEK envelope encryption per Bominal ADR-0111.

## ChangeSet boundary

2 crates: `-adapter-postgres` + `-adapter-s3` + Postgres migrations.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-document-store-adapter-postgres/Cargo.toml` | create |
| `microservices/docs/src/crates/oya-docs-document-store-adapter-postgres/src/{lib,repository,acl,legal_hold,rls_session}.rs` | create |
| `microservices/docs/src/crates/oya-docs-document-store-adapter-s3/Cargo.toml` | create |
| `microservices/docs/src/crates/oya-docs-document-store-adapter-s3/src/{lib,blob_store,attachment_storage,object_lock}.rs` | create |
| `microservices/docs/iac/helm/postgres/migrations/0001-initial-schema.sql` | create |
| `microservices/docs/iac/helm/postgres/migrations/0002-rls-per-tenant.sql` | create |
| `microservices/docs/iac/helm/postgres/migrations/0003-rls-per-block-acl.sql` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-document-store-adapter-postgres -- rls_per_tenant_isolation
cargo nextest run -p oya-docs-document-store-adapter-postgres -- rls_per_block_acl
cargo nextest run -p oya-docs-document-store-adapter-s3 -- object_lock_on_legal_hold
buck2 build //:quality-lane-registry-authority-check # lane=rls-coverage --microservice docs
```

## References

- ADR-0105 Amendment 3 (backend-qualified adapter); ADR-0117 (data residency).
- ADR-DOCS-0004 (per-block ACL).
- Bominal ADR-0111 (envelope encryption).

## Wave 15-IP-substance conversion (2026-05-21)
This addendum converts the short implementation note into a buildable docs-service slice for document-store adapters; it does not rely on line count as proof.
Counterpart anchors: Google Docs, Microsoft Word Online, Notion, Coda, Quip, GitHub.

### Problem closed
The gap is not generic documentation infrastructure. `docs` owns collaborative document authoring where rich blocks, CRDT edits, comments, suggestions, version history, sharing, export/import, embeds, and AI assist must work under one tenant and policy model.
For document-store adapters, the implementation must preserve dual-context separation, per-block ACL, legal hold, audit-chain records, and export/import safety described in `microservices/docs/PRD.md` and `microservices/docs/ARCHITECTURE.md`.

### Concrete mechanism
Primary artifact: `microservices/docs/catalog/oya-docs-document-store-adapter-postgres.yaml`.
Domain entities or operational surfaces: RLS, tenant-DEK envelope, S3 blob keys.
Contracts and gates: `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/proto/docs.proto`, and Cedar files under `microservices/docs/policy/`.
SLO/runbook evidence: `microservices/docs/slos/*.openslo.yaml`, `microservices/docs/dashboards/*.json`, and `microservices/docs/runbooks/*.md`.

### Implementation steps
1. Bind document-store adapters to the matching bounded context in `microservices/docs/manifest.json` and the catalog row named in this addendum.
2. Confirm every command/event carries tenant_id, principal_id, document context, data_class, traceparent, idempotency key for mutations, and audit_event_class.
3. Extend the contract or catalog artifact only where the cited file already owns that surface; do not invent a sibling µservice or fake Terraform module.
4. Add or update policy checks in `tenant-scope.cedar`, `editor-isolation.md`, `public-read.cedar`, or `auditor-scope.cedar` according to the feature's access path.
5. Add tests around accepted path, cross-tenant denial, stale version/anchor handling, legal-hold or retention interaction, and export/import rollback where applicable.
6. Attach SLO, dashboard, and runbook evidence before promotion so a reviewer can verify more than the presence of this Markdown file.

### Acceptance and counterpart comparison
| Counterpart | Expected behavior | Oyatie closure |
|---|---|---|
| Google Docs / Microsoft Word Online | Native collaborative authoring with comments, sharing, history, and export. | `document-store adapters` must be first-party, tenant-scoped, auditable, and legal-hold aware. |
| Notion / Coda | Block-centric documents with embeds and structured workflows. | `document-store adapters` must preserve block ACL, embed policy checks, and cross-service refresh evidence instead of hidden workspace coupling. |
| Quip / GitHub | Team review and change history expectations. | `document-store adapters` must expose signed audit events, version/revert evidence, and contract-rendered developer docs. |

### Evidence
- `microservices/docs/PRD.md`
- `microservices/docs/ARCHITECTURE.md`
- `microservices/docs/manifest.json`
- `microservices/docs/competitor-parity-matrix.md`
- `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md`
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`
