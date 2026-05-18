---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-guardrails-safety-and-policy-enforcement
impl_plan_id: IP-003-rule-store-postgres-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-guardrails + ops-sre-reliability
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, rule-store-migrations-up-to-date, oya-governance-version-pinning-conformance]
---

# IP-003: Postgres rule-store IaC + migrations

## Intent

Helm chart for per-pack HA Postgres (rule store + Cedar fragment registry + audit-mutation log). Postgres 16 LTS; HA primary + 2 read replicas; pgaudit + Postgres RLS enabled. Migration framework + initial schema migrations.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/foundry/iac/helm/postgres/Chart.yaml` | create | bitnami/postgresql-ha pinned LTS |
| `microservices/foundry/iac/helm/postgres/values.yaml` | create | HA + RLS + pgaudit + TDE |
| `microservices/foundry/iac/helm/postgres/values-pack-kr.yaml` | create | pack-kr |
| `microservices/foundry/iac/postgres/migrations/001-init-schema.sql` | create | tables: rule_definitions, cedar_fragments, audit_mutation_log, classifier_model_versions |
| `microservices/foundry/iac/postgres/migrations/002-rls-policies.sql` | create | Row-level security per tenant + pack |
| `microservices/foundry/iac/postgres/migrations/003-indexes.sql` | create | per `capacity-model.md` access patterns |
| `microservices/foundry/iac/postgres/migrations/manifest.yaml` | create | ordered migration list + checksums |

## Acceptance Gates

```bash
helm lint microservices/foundry/iac/helm/postgres
kubectl --dry-run=client apply -k microservices/foundry/iac/kustomize/overlays/pack-kr
cargo run -p oya-dev-cli -- gate validate rule-store-migrations-up-to-date
```

## Test Plan

- helm-install smoke; verify primary + 2 RR reach Ready.
- Migration test: apply 001 → 002 → 003 against ephemeral Postgres; verify schema matches manifest checksums.
- RLS test: insert rows as tenant-A; query as tenant-B; verify zero rows returned.
- Backup-restore test: pg_dump + pg_restore round-trip via `runbooks/rule-store-restore.md`.

## Halt Conditions

- Postgres version drift from LTS — escalate.
- pgaudit not enabled in values — refuse merge.
- Any migration without explicit rollback step — refuse merge.

## Next IP

[`IP-004-prompt-classifier-kernel.md`](IP-004-prompt-classifier-kernel.md)

## References

- ADR-0131; `policy/tenant-isolation.md`; `capacity-model.md`.
- Postgres 16 LTS — `postgresql.org/docs/16/`.
- pgaudit — `github.com/pgaudit/pgaudit`.
