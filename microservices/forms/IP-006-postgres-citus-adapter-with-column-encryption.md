---
doc_class: ImplementationPlan
milestone: M03-workspace-tier-foundation
phase: P01-forms-foundation
impl_plan_id: IP-006-postgres-citus-adapter-with-column-encryption
status: pending
execution_unit: ChangeSet
owner: axis-forms + ops-security
acceptance_lanes: [cargo-test, oya-forms-pii-column-encryption-correctness, oya-governance-citus-rls-enforced, oya-forms-dek-rotation-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: Postgres + Citus adapter with column-level envelope encryption

## Intent

Implement the Postgres + Citus adapter per ADR-FORMS-0003. Per-tenant DEK with envelope encryption (OpenBao-rooted KEK). RLS + Citus tenant_id shard key. Quarterly DEK rotation with rolling re-encryption.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/forms/src/adapter/postgres/client.rs` | create |
| `microservices/forms/src/adapter/postgres/citus_shard.rs` | create |
| `microservices/forms/src/adapter/postgres/encryption.rs` | create — AES-256-GCM column encryption |
| `microservices/forms/src/adapter/postgres/dek_cache.rs` | create — ≤ 5min DEK cache |
| `microservices/forms/src/adapter/postgres/dek_rotation.rs` | create — rolling re-encryption worker |
| `microservices/forms/src/adapter/postgres/rls_policies.sql` | create |
| `microservices/forms/migrations/0001_create_forms_tables.sql` | create |
| `microservices/forms/migrations/0002_enable_rls.sql` | create |
| `microservices/forms/migrations/0003_enable_pgaudit.sql` | create |
| `microservices/forms/tests/postgres_column_encryption.rs` | create |
| `microservices/forms/tests/postgres_rls_cross_tenant.rs` | create |

## Acceptance Gates

- Every PII column write encrypted; verify by reading raw row at the DBA level.
- RLS rejects cross-tenant query.
- DEK rotation drill: pre-rotation reads + post-rotation reads both succeed during dual-key window.
- Forms tables defined: `forms_forms`, `forms_responses`, `forms_versions`, `forms_dek_metadata`, `forms_audit_chain`, `forms_webhooks`, `forms_bulk_distribute_jobs`, `forms_dsr_ledger`.

## References

- ADR-FORMS-0003 PII column encryption.
- ADR-0140 (retired per ADR-0145) Cedar.
- Citus docs.
- OpenBao docs.

## Next IP

[`IP-007-redis-adapter.md`](IP-007-redis-adapter.md)
