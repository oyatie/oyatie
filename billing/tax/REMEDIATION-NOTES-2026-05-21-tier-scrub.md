# cloud-billing-tax tier scrub remediation notes — 2026-05-21

## Scope

Wave 15J-batch-4 BUCKET-07 retired customer-facing Bronze/Silver/Gold/Platinum vocabulary from `billing/tax` and replaced it with ADR-0330 `tenant_class` plus `billing_components` language.

## Files modified

| File | Lines |
|---|---:|
| `README.md` | 23 |
| `benchmarks/cloud-billing-tax-vs-avalara-vs-vertex-vs-stripe-tax-vs-taxjar.md` | 104 |
| `coherence-audit-2026-05-20.md` | 1403 |
| `faqs/tax-engineer-faq.md` | 206 |
| `feature-parity-matrix-2026-05-20.md` | 1634 |
| `migration-playbooks/from-avalara-and-vertex.md` | 188 |
| `onboarding/tax-engineer-first-week.md` | 176 |
| `performance-benchmark-numbers-2026-05-20.md` | 958 |
| `tutorials/calculate-multijurisdiction-tax-and-file-return.md` | 241 |

## Directory retirement

`capability-tiers/` deleted: Y.

## Replacement count

Rough vocabulary replacements in this service: about 135. Corpus-wide assigned bucket replacement count is about 928 outside legacy remediation notes.

## Design decisions

- Added a service README because no README existed in this directory and the Wave 15J assignment requires README adoption language.
- Replaced jurisdiction and tax-code catalog customer ladder references with `tenant_class` posture and compliance-pack gates.
- Preserved tax-engine architecture language where "two-tier" described implementation topology rather than the retired customer ladder.
- Existing shipped docs were amended in place; no SQL migration files were present in this microservice directory.

## Verification

- `rg -i 'bronze|silver|gold|platinum' billing/tax/ | grep -v REMEDIATION-NOTES | grep -v capability-tiers` returns zero matches.
- `rg -i 'capability_tier|max_tier|tier_threshold' billing/tax/ | grep -v REMEDIATION-NOTES` returns zero matches.
- `ls microservices/cloud-billing-tax/capability-tiers/` reports no such file or directory.

## Outstanding follow-ups

None for the Wave 15J required vocabulary gate.

