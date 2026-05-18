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
cargo run -p oya-dev-cli -- gate validate rls-coverage --microservice docs
```

## References

- ADR-0105 Amendment 3 (backend-qualified adapter); ADR-0117 (data residency).
- ADR-DOCS-0004 (per-block ACL).
- Bominal ADR-0111 (envelope encryption).
