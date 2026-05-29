# Wave 15J-batch-4 Tier Vocabulary Scrub - tasks

## Files Modified

- 20 tracked files currently show scrub-related diffs under `microservices/tasks/`.
- Representative line counts: `PRD.md` 387, `manifest.json` 479, `compliance.md` 1210, `cost-budget.md` 112, `migration-from-connect.md` 495.
- Added README tenant-class surface: `README.md` 30.

## Retirement Actions

- capability-tiers/ dir deleted: Y; the directory is absent.
- Vocabulary replacement count: roughly 35 in this service, plus the old remediation-note-only explanatory residue.
- README updated/created: Y, with ADR-0330 tenant_class and billing_components adoption note.

## Design Decisions

- Replaced customer pricing and workload ladder wording with `tenant_class`, paid billing components, usage caps, deployment context, and cell topology.
- Left T0/T1/T2 autonomy labels intact where they describe AI-assist risk authority, not customer capability.
- Reworded service docs so paid tenants receive uniform product behavior while demo_trial is capped by usage policy.

## Outstanding Follow-ups

- None for this scrub bucket.
