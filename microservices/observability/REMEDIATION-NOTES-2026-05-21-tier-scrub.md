# Wave 15J-batch-4 Tier Vocabulary Scrub - observability

## Files Modified

- 23 tracked files currently show scrub-related diffs under `microservices/observability/`.
- Representative line counts: `manifest.json` 367, `compliance.md` 1160, `contracts/metric-naming-convention.md` 204, `threat-model.md` 665, `policy/tenant-isolation.md` 278.
- Added README tenant-class surface: `README.md` 32.

## Retirement Actions

- capability-tiers/ dir deleted: Y; the directory is absent.
- Vocabulary replacement count: roughly 180 in this service, dominated by the retired counterpart delta document and verification-regex collisions in reference-signal wording.
- README updated/created: Y, with ADR-0330 tenant_class and billing_components adoption note.

## Design Decisions

- Collapsed customer ladder language into `tenant_class`, usage caps, deployment context, cell topology, and compliance-pack gates.
- Reworded Google SRE `golden signals` and AI `golden-test` occurrences to `canonical signals` / `reference-test` equivalents because the assigned grep pattern matches the substring.
- Preserved Tier-1 OS wording where it describes infrastructure support classification, not customer capability.

## Outstanding Follow-ups

- None for this scrub bucket.
