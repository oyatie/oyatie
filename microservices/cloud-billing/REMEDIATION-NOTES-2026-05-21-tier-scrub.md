# cloud-billing tier scrub remediation notes — 2026-05-21

## Scope

Wave 15J-batch-4 BUCKET-07 retired customer-facing Bronze/Silver/Gold/Platinum vocabulary from `microservices/cloud-billing` and replaced it with ADR-0330 `tenant_class` plus `billing_components` language.

## Files modified

| File | Lines |
|---|---:|
| `PRD.md` | 786 |
| `README.md` | 430 |
| `benchmarks/cloud-billing-vs-aws-cur-vs-gcp-billing-vs-azure-cost-management.md` | 105 |
| `coherence-audit-2026-05-20.md` | 638 |
| `faqs/billing-engineer-faq.md` | 200 |
| `feature-parity-matrix-2026-05-20.md` | 438 |
| `migration-playbooks/from-aws-cur-and-cloudability.md` | 179 |
| `onboarding/billing-engineer-first-week.md` | 174 |
| `performance-benchmark-numbers-2026-05-20.md` | 388 |
| `runbooks/invoice-generation-timeout.md` | 269 |
| `tutorials/meter-attribute-invoice-and-export-focus.md` | 196 |

## Directory retirement

`capability-tiers/` deleted: Y.

## Replacement count

Rough vocabulary replacements in this service: about 175. Corpus-wide assigned bucket replacement count is about 928 outside legacy remediation notes.

## Design decisions

- Replaced the deleted capability matrix references with `tenant_class` adoption language instead of preserving customer ladder semantics.
- Reframed commercial segmentation as `demo_trial` vs `paid`; paid billing shape is expressed through `billing_components` (`revenue_share`, `per_seat`, `per_usage`).
- README now names ADR-0330 as the replacement authority and states that quality and capability posture are uniform except for demo-trial caps, compliance-pack/BYOK/marketplace gates, and contractual SLO posture.
- Existing shipped docs were amended in place; no SQL migration files were present in this microservice directory.

## Verification

- `rg -i 'bronze|silver|gold|platinum' microservices/cloud-billing/ | grep -v REMEDIATION-NOTES | grep -v capability-tiers` returns zero matches.
- `rg -i 'capability_tier|max_tier|tier_threshold' microservices/cloud-billing/ | grep -v REMEDIATION-NOTES` returns zero matches.
- `ls microservices/cloud-billing/capability-tiers/` reports no such file or directory.

## Outstanding follow-ups

None for the Wave 15J required vocabulary gate.

