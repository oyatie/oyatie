# sheets remediation notes: 2026-05-21 customer-level vocabulary scrub

## Files modified

- `README.md` - 8 lines
- `IP-007-cell-grid-adapter-postgres-and-materialized-views.md` - 100 lines
- `IP-015-hg-sheets-registration-and-branch-protection.md` - 126 lines
- `PHASE-01-SHEETS-FOUNDATION.md` - 305 lines
- `PRD.md` - 597 lines
- `backfill-replay.md` - 121 lines
- `capabilities/T0-suggest.yaml` - 110 lines
- `capabilities/T1-assist.yaml` - 204 lines
- `capabilities/T2-auto.yaml` - 191 lines
- `competitor-parity-matrix.md` - 214 lines
- `compliance.md` - 1213 lines
- `decisions/ADR-SHEETS-0001-crdt-library-selection.md` - 218 lines
- `decisions/ADR-SHEETS-0007-export-fidelity-policy.md` - 177 lines
- `decisions/README.md` - 74 lines
- `failure-modes.md` - 305 lines
- `manifest.json` - 423 lines
- `runbooks/export-pipeline-failure-xlsx.md` - 137 lines
- Service-local untracked docs with matching retired vocabulary were also scrubbed in place: onboarding, FAQ, benchmark, tutorial, performance, and coherence-audit surfaces.

## Retirement marker

- `capability-tiers/` deleted: Y

## Replacement count

- Rough vocabulary replacements: ~95

## Design decisions

- Reframed spreadsheet scale and availability distinctions as paid tenant_class, demo_trial caps, or cell_topology rather than customer feature ladders.
- Preserved XLSX fidelity semantics while renaming old "best-effort" and validation-corpus wording to avoid the retired vocabulary.
- Preserved T0/T1/T2 autonomy semantics as automation risk classes, not customer classes.

## Outstanding follow-ups

- none

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: PRD now states manifest-aligned 1800s RTO / 120s RPO, `runbooks/dr-failover.md`, active-active multi-AZ/cross-region-warm replication, and the manifest backup substrate (`postgres_wal_g`, `object_storage_versioned`, `valkey`). ADR: ADR-0343. Alternative considered: keeping the older 30s/1s PRD target; rejected because D-2 manifest values are the current contract for this propagation. Cost: hot edit buffers and recalc coordination still need recovery testing.
- Capacity model: PRD now states manifest-aligned 0.18 vCPU / 512Mi / 40Gi storage, 3 Valkey, 3 Postgres, 6 outbound HTTP connections, `per_query` scaling, Tier-3 placement, and visual-grid REST min 4 / max 50. ADR: ADR-0340. Alternative considered: capacity by workbook count only; rejected because query/recalc work and D-2 manifest values govern. Cost: recalc/export queues need explicit back-pressure and pre-warmed worker capacity.
- Sustainability + cost attribution: PRD now requires cost/emission/watt/provider/region on edit/share/formula/session/AI/import/export/connected-refresh audit rows; carbon routing is excluded from EU-AI high-risk, HIPAA, formula-correctness, license, and realtime collab paths. ADR: ADR-0344. Alternative considered: carbon steering AI formulas universally; rejected because high-risk and regulated spreadsheet decisions must keep deterministic provider policy. Cost: cold materialization, AI, and recalc spend must be separately attributed.
- API versioning: PRD now uses YYYY-MM-DD carrier triplet, SDK semver, N=3 / 180d support, tenant pinning, and ADR-0145 internal-mesh exemption for workbook/formula/recalc/import/export/embed contracts. ADR: ADR-0342. Alternative considered: semver formula engine only; rejected because tenant-facing workbook and API compatibility need date-version evidence. Cost: formula and XLSX compatibility suites run across three public versions.
- Frontmatter: added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, and ADR-0345; no OLAP/Iceberg warehouse-write ADR added because Arrow/Parquet large-sheet storage is not the canonical warehouse path.
