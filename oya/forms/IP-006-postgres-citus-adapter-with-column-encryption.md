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
- PRD FR-06 and AC-08 / AC-09.
- `microservices/forms/policy/data-residency.md`.
- `microservices/forms/slos/pii-encryption-correctness.openslo.yaml`.
- `microservices/forms/runbooks/pii-leak-incident-p0.md`.
- `microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md`.

## Foundation A-G Substance

- A. Product scope: the response store is the source of truth for form, version, response, audit, webhook, export, and DSR state.
- B. Domain model: storage adapters accept domain records only after tenant, version, data-class, and audit seal are resolved.
- C. Contracts: raw storage never becomes a public contract; REST/proto map through redaction and version loaders.
- D. Policy: RLS is defence-in-depth after Cedar; pack residency and DEK routing are mandatory before insert.
- E. Operations: DEK rotation uses dual-read windows, progress checkpoints, and rollback to prior KEK reference when verification fails.
- F. Observability: track encrypted-column coverage, RLS denial count, shard skew, DEK cache hit ratio, and rotation lag.
- G. Promotion: DBA raw-row inspection, cross-tenant adversarial query, residency probe, and rotation drill all gate done.

## Counterpart Benchmark

- Counterpart: Salesforce Web-to-Lead encrypted fields, ServiceNow encrypted variables, and HubSpot Forms sensitive-data capture controls.
- Defensible parity claim: Oyatie must encrypt PII fields per tenant and keep old responses queryable through the exact form version.
- Differentiator: column encryption is tied to data-class declarations, not a global table toggle.
- Grep counterpart names: Salesforce Web-to-Lead; ServiceNow encrypted variables; HubSpot Forms.

## Remediation Notes

- Expanded storage scope with residency, SLO, benchmark, and incident-response evidence.
- Added A-G substance covering domain handoff, contract redaction, policy, operations, telemetry, and promotion gates.
- Added counterpart names for grep-recognized review.

## Verification Evidence Required

- Raw-row DBA inspection proves PII ciphertext at rest.
- Cross-tenant RLS probe fails before any adapter-level redaction path.
- DEK rotation drill records pre-rotation, dual-key, and post-rotation reads.
- Residency probe proves pack-eu and pack-kr responses stay pack-resident.

## Next IP

[`IP-007-valkey-adapter.md`](IP-007-valkey-adapter.md)
