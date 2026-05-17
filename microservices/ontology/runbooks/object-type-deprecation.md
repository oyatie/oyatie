---
doc_class: Runbook
title: Object Type / Link Type / Action Type / Function Type deprecation
microservice: ontology
severity: "Sev-3 (planned deprecation) / Sev-2 (deprecation broke tenant)"
status: Accepted
owner_team: axis-ontology
date: 2026-05-17
related_artifacts:
  - microservices/ontology/failure-modes.md (FM-11 deprecation broke tenant)
  - microservices/ontology/runbooks/type-registry-migration.md
doc_status: published
---

# Runbook: Object Type deprecation

## Trigger

- A schema author proposes deprecating an existing Object Type, Link Type, Action Type, or Function Type.
- A breaking change requires migration off a current type to a successor type.

## Severity

- Sev-3 for planned deprecation with adequate notice (≥ 90 days).
- Sev-2 if deprecation broke a tenant (FM-11).

## Pre-checks

1. **Tenant impact assessment**: query the audit chain to find every tenant actively writing or reading the deprecated type within the last 30 days.
2. **Successor designation**: which type should consumers migrate to? Provide a JSON-IR mapping if shapes differ.
3. **Deprecation window**: ≥ 90 days default; ≥ 180 days if HIPAA-tagged or has > 100 active tenant consumers.
4. **Tenant notification plan**: email list + status page + per-tenant onboarding contact.

## Phase 1 — Deprecation announcement (Day 0)

| Step | Action |
|---|---|
| 1 | Merge PR adding `"deprecated": true, "deprecated_at": "<ISO8601>", "successor": "<type-id>"` to the schema entry. |
| 2 | Schema-propagation-worker emits `ObjectTypeDeprecated` event; downstream subscribers logged. |
| 3 | Active consumers' Function reads continue to work; deprecation warning header `Sunset: <date>` added to responses. |
| 4 | Email blast to affected tenants with migration guide + successor mapping. |
| 5 | Status page entry under "Deprecations". |
| 6 | Document the rationale + timeline in `microservices/ontology/decisions/ADR-<NNN>-deprecate-<type>.md`. |

## Phase 2 — Migration window (Day 1 – Day 89)

| Step | Action |
|---|---|
| 1 | Monitor `oya_ontology_deprecated_type_read_total{type="<id>"}` metric weekly. |
| 2 | Reach out to laggard tenants (still reading deprecated type at 60+ days). |
| 3 | If a tenant cannot migrate by deadline: offer extension OR migrate them on a best-effort basis. |
| 4 | Final reminder at Day 75. |

## Phase 3 — Sunset (Day 90+)

| Step | Action |
|---|---|
| 1 | If `deprecated_type_read_total` > 0 (anyone still using): hold; engage active tenants individually. |
| 2 | If all clear: merge PR that removes type from schema registry + drops associated Postgres table (with PITR backup retained for ≥ 1 year). |
| 3 | Schema-propagation-worker emits `ObjectTypeRemoved` event. |
| 4 | Postgres table renamed `_archived_<table>_<timestamp>`; physical drop after 1 year per audit retention. |
| 5 | Audit-chain emit `ObjectTypeSunset{type_id, sunset_at, residual_consumer_count}`. |

## Failure mode: deprecation broke a tenant (FM-11)

Symptoms: tenant reports broken read after sunset; `oya_ontology_deprecated_property_read_total > 0` after sunset day.

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-2; engage IC | ≤ 5 min |
| 2 | Restore deprecated type: `git revert <sunset-PR-sha>` + ArgoCD apply | ≤ 1 h |
| 3 | Schema-propagation-worker restores the type; Postgres table un-archived | ≤ 30 min |
| 4 | Tenant confirms reads work again | ≤ 30 min |
| 5 | Extend deprecation timeline + work directly with tenant on migration | per timeline |
| 6 | Postmortem | ≤ 5 business days |

## Tier loosening as deprecation special case

When deprecating because of tier-loosening (e.g., Tier1 → Tier2 to lift compliance constraints):

- 2-person rule per `runbooks/type-registry-migration.md` §"Tier loosening".
- DPO impact assessment.
- Tenant DPA addendum required.

## Verification

After deprecation cycle:
- `oya gate validate ontology-deprecation-conformance --microservice ontology` — exit 0.
- Audit chain emits `ObjectTypeDeprecated` + `ObjectTypeSunset` events with timestamps + consumer counts.
- ClickHouse partition for deprecated type tagged `_archived`.
- No active tenant reads on sunset type after Day 90.

## Post-incident updates

- Postmortem if FM-11 triggered.
- Action items: improve tenant-notification reach; tighten deprecation tooling.

## References

- `microservices/ontology/failure-modes.md` FM-11.
- `microservices/ontology/runbooks/type-registry-migration.md`.
- Bominal ADR-0019 (deprecation policy).
- ADR-0149 (Bominal — schema evolution; inherited).
