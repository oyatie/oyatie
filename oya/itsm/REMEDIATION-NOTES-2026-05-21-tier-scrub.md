# Wave 15J Tier Scrub Notes - itsm

## Files Deleted

- `coherence-audit-2026-05-20.md` - stale Wave-4 audit artifact; no longer canonical after ADR-0329/0330/0331.

## Files Retained With Scrub

- `performance-benchmark-numbers-2026-05-20.md` - retained because local docs reference it as the numeric benchmark surface; scrubbed retired named-tier vocabulary.
- `benchmarks/servicenow-vs-jsm-vs-freshservice-vs-oyatie.md` - scrubbed retired Oyatie tier labels into post-ADR-0330 tenant-class/context language.
- `migration-playbooks/from-bmc-helix-itsm.md`
- `migration-playbooks/from-jira-service-management.md`
- `migration-playbooks/from-servicenow-itsm.md`
- Additional local FAQ/tutorial/IP/feature-parity references under `microservices/itsm/` were scrubbed where they carried retired Oyatie tier labels or false-positive canonical fixture wording.

## Counterpart-Fact Preservations

- None. Remaining references were either Oyatie tier residue or false-positive validation vocabulary, not counterpart product tier facts.
