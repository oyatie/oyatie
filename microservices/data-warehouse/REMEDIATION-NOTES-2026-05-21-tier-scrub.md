# Wave 15J Tier Scrub Notes - data-warehouse

## Files Deleted

- `coherence-audit-2026-05-20.md` - stale Wave-4 audit artifact; no longer canonical after ADR-0329/0330/0331.

## Files Retained With Scrub

- `performance-benchmark-numbers-2026-05-20.md` - retained because local docs reference it as the numeric benchmark surface.
- `migration-playbooks/from-bigquery.md`
- `migration-playbooks/from-redshift.md`
- `migration-playbooks/from-snowflake.md`
- IP plans, PRD, README, manifest, and local decision docs under `microservices/data-warehouse/` were scrubbed for retired tier vocabulary, `capability_tier`, and false-positive canonical fixture wording.

## Counterpart-Fact Preservations

- None. Data-lake medallion labels and retired capacity ladder language were rewritten to avoid collision with ADR-0329 tier retirement.
