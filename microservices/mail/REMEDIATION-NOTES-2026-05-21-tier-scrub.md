# Wave 15J-batch-4 Tier Vocabulary Scrub - mail

## Files Modified

- 13 tracked files currently show scrub-related diffs under `microservices/mail/`.
- Representative line counts: `PRD.md` 1545, `manifest.json` 501, `compliance.md` 1345, `migration-from-connect.md` 335, `README.md` 48.
- Deleted tracked retired ADR file: `decisions/ADR-MAIL-0002-backend-tenant-tier-policy.md`.

## Retirement Actions

- capability-tiers/ dir deleted: Y; the directory is absent.
- Vocabulary replacement count: roughly 70 in this service, including verification-regex collisions in reference dashboard wording.
- README updated/created: Y, with ADR-0330 tenant_class and billing_components adoption note.

## Design Decisions

- Replaced backend workload policy wording with `tenant_class`, paid billing components, usage caps, compliance-pack gates, and cell topology.
- Reworded customer ladder references in manifest, PRD, and migration material without changing mail protocol scope.
- Preserved Tier-0/1/2 only where replaced by cell-topology language in README status.

## Outstanding Follow-ups

- None for this scrub bucket.
