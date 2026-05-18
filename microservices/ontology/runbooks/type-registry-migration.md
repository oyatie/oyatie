---
doc_class: Runbook
title: Type registry migration (Object Type / Link Type / Action Type / Function Type schema evolution)
microservice: ontology
severity: "Sev-2 (operational schema-evolution) / Sev-3 (additive migration only)"
status: Accepted
owner_team: axis-ontology
date: 2026-05-17
related_artifacts:
  - microservices/ontology/failure-modes.md (FM-04 schema corruption, FM-11 deprecation broke tenant)
  - microservices/ontology/PRD.md §"Functional Requirements" FR-01, FR-07
doc_status: published
---

# Runbook: Type registry migration

## Trigger

Any of:
- A new Object Type / Link Type / Action Type / Function Type registration is being shipped.
- An existing schema is being amended (add property, deprecate property, change property tier — see "Tier loosening" below).
- A failed migration left the schema registry in an inconsistent state (FM-04).

## Severity

- **Sev-3** for additive-only migrations (new property; new schema; new Link Type endpoint pair).
- **Sev-2** if migration touches existing tenant data (deprecation, tier change, type rename).
- **Sev-1** if schema registry corruption detected at runtime (FM-04).

## Pre-checks

1. **Schema diff reviewed**: PR carries the diff between current registry state and target state; CODEOWNERS = `axis-ontology + council-privacy + workload-µservice-owner`.
2. **Tier-classification validation**: every new property declares `data_class` + `property_tier`; LEAN lane `oya-foundry-fitness-ontology-tier-enforcement` exit 0.
3. **Cedar coverage**: every new Action Type carries a Cedar permit fragment + default-deny; `cedar-coverage` lane exit 0.
4. **Tenant-impact assessment**: which tenants currently rely on the deprecated property/type; PR notes the count + email-list.
5. **Rollback plan documented in the PR**.

## Steps — Additive migration (Sev-3)

| Step | Action | Time |
|---|---|---|
| 1 | Merge PR with the new schema definition under `microservices/ontology/specs/object-types/<name>.json` | – |
| 2 | `schema-propagation-worker` picks up the new schema; emits `ObjectTypeRegistered` event | ≤ 60 s |
| 3 | All `*-rest` + `function-engine` + `action-engine` pods hot-reload via Valkey cache invalidation + Kafka event subscription | ≤ 30 s |
| 4 | Validate by writing a sample instance: `oya-ontology-sdk create-object --type <new-name> --tenant <test-tenant>` | ≤ 2 min |
| 5 | Confirm Function evaluator returns expected shape | ≤ 5 min |
| 6 | Update tenant-facing changelog at status portal | ≤ 10 min |

## Steps — Migration with existing data (Sev-2)

For schema amendments touching existing tenant data:

| Step | Action | Time |
|---|---|---|
| 1 | Open `#inc-<id>` Slack channel; declare Sev-2; assign IC | ≤ 5 min |
| 2 | Verify pre-checks above | ≤ 10 min |
| 3 | Apply migration in **shadow mode**: new schema active for new writes; old schema preserved for existing rows | ≤ 30 min |
| 4 | Run backfill: `oya-ontology-sdk backfill --type <name> --from-schema <prev> --to-schema <new> --tenant <tenant>` (batched per Citus shard; rate-limited; monitor Postgres load) | varies; hours per million instances |
| 5 | Confirm 100 % backfill complete per Object Type table: `SELECT count(*) FROM <table> WHERE schema_version = <new>` matches expected | ≤ 30 min |
| 6 | Decommission old schema: register deprecation timestamp; alert any tenant still reading via deprecated property path | ≤ 1 h |
| 7 | After 30-day deprecation window (or longer per tenant SLA), drop deprecated columns | per timeline |
| 8 | Audit-chain emit `SchemaMigrationCompleted{schema_id, prev_version, new_version, backfilled_count}` | automatic |

## Tier loosening (special case)

Per Bominal ADR-0008 §2.2.10: changing a property's tier to a less-restrictive value (Tier1 → Tier2) requires explicit human approval. The migration runbook:

1. PR carries the proposed tier change + justification.
2. CODEOWNERS = `council-privacy chair + ops-security chair` (2-person rule).
3. DPO impact assessment recorded.
4. Tenant DPA addendum required if the property has been published to tenants in the prior tier.
5. Migration proceeds only after sign-off.
6. Audit-chain emit `PropertyTierChanged{property_id, prev_tier, new_tier, approved_by[2]}`.

## Failure mode: schema registry corruption (FM-04)

Symptoms: Function evaluator fails on schema lookup; `oya_ontology_schema_registry_validation_failures_total > 0`.

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-2 if a single tenant's writes affected; Sev-1 if all tenants | – |
| 2 | Roll back schema registry transaction via `BEGIN; SELECT schema_rollback(<txn_id>); COMMIT;` | ≤ 5 min |
| 3 | Reload from git-versioned schemas: `oya-ontology-sdk schema-reload --from git` | ≤ 5 min |
| 4 | Flush Valkey cache: `oya-ontology-sdk valkey-flush --keyspace ontology:schema` | ≤ 1 min |
| 5 | Verify Function evaluator + Action engine pick up restored schemas | ≤ 5 min |
| 6 | If corruption originated from a bad PR: revert via `git revert` + ArgoCD apply | ≤ 10 min |
| 7 | Postmortem | ≤ 5 business days |

## Verification

After migration:
- `oya gate validate ontology-tier-enforcement --microservice ontology` — exit 0.
- `oya gate validate cedar-coverage --microservice ontology` — exit 0.
- `oya gate validate schema-registry-conformance --microservice ontology` — exit 0.
- Sample Object Type read returns expected shape across affected tenants.
- ClickHouse history-mirror tables include the new schema; new partitions created.
- Audit-chain seal includes `SchemaMigrationCompleted` event for the migration.

## Post-incident updates

- Postmortem within 5 business days for Sev-2/Sev-1 migrations.
- If FM-04 (schema corruption) recurs ≥ 2 in 12 months: investigate registry transaction patterns; consider stronger transactional boundary.

## References

- `microservices/ontology/PRD.md` §"Functional Requirements" FR-01, FR-07.
- `microservices/ontology/failure-modes.md` FM-04, FM-11.
- Bominal ADR-0008 §2.2.10 (tier loosening).
- ADR-0028 (audit-chain).
- ADR-0132 (Bominal pillars).
