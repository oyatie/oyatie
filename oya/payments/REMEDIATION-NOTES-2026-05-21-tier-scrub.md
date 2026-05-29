# payments tier scrub remediation notes — 2026-05-21

## Scope

Wave 15J-batch-4 BUCKET-07 retired customer-facing Bronze/Silver/Gold/Platinum vocabulary from `microservices/payments` and replaced it with ADR-0330 `tenant_class` plus `billing_components` language.

## Files modified

| File | Lines |
|---|---:|
| `ARCHITECTURE.md` | 1720 |
| `README.md` | 123 |
| `benchmarks/stripe-vs-adyen-vs-checkout-vs-oyatie.md` | 112 |
| `capabilities/charge.yaml` | 101 |
| `capabilities/payout.yaml` | 93 |
| `tenant-class-counterpart-deltas-2026-05-20.md` | 357 |
| `coherence-audit-2026-05-20.md` | 795 |
| `competitor-parity-matrix.md` | 141 |
| `faqs/payments-engineer-faq.md` | 152 |
| `feature-parity-matrix-2026-05-20.md` | 412 |
| `manifest.json` | 378 |
| `migration-playbooks/from-adyen.md` | 584 |
| `migration-playbooks/from-braintree.md` | 584 |
| `migration-playbooks/from-checkout-com.md` | 584 |
| `onboarding/payments-engineer-first-week.md` | 239 |
| `performance-benchmark-numbers-2026-05-20.md` | 312 |
| `reference-implementations/charge-and-refund-rust-sdk.md` | 248 |
| `test-plans/contract-test-strategy.md` | 362 |
| `test-plans/integration-test-strategy.md` | 360 |
| `tutorials/process-cross-currency-charge.md` | 233 |

## Directory retirement

`capability-tiers/` deleted: Y.

## Replacement count

Rough vocabulary replacements in this service: about 618. Corpus-wide assigned bucket replacement count is about 928 outside legacy remediation notes.

## Design decisions

- Renamed `capability-tier-deltas-vs-counterparts-2026-05-20.md` to `tenant-class-counterpart-deltas-2026-05-20.md` so the artifact path no longer carries the retired customer ladder.
- Reframed sandbox/evaluation payment flows as `demo_trial` and production money movement as `paid`.
- Kept ADR-0248 cell criticality language (`Tier-0`..`Tier-3`) and KYC verification-level language where it describes infrastructure or regulatory classification rather than the retired customer capability ladder.
- README now links payments capability posture to `tenant_class`, active `billing_components`, and `compliance_pack` gates.
- Existing shipped docs were amended in place; no SQL migration files were present in this microservice directory.

## Verification

- `rg -i 'bronze|silver|gold|platinum' microservices/payments/ | grep -v REMEDIATION-NOTES | grep -v capability-tiers` returns zero matches.
- `rg -i 'capability_tier|max_tier|tier_threshold' microservices/payments/ | grep -v REMEDIATION-NOTES` returns zero matches.
- `ls microservices/payments/capability-tiers/` reports no such file or directory.

## Outstanding follow-ups

None for the Wave 15J required vocabulary gate.

## Wave 15-IP-substance scrub (2026-05-21)

Scope: IP-BUCKET-F / `payments`.

Inventory: 88 `IP-*.md` files found under `microservices/payments`; 18 are base implementation-plan IPs and the rest are journey/overlay plans. The base IPs already referenced concrete payment artifacts such as `contracts/openapi-v1.yaml`, `contracts/payments-v1.proto`, `policy/charge-authorization.cedar`, `policy/refund-authorization.cedar`, `policy/dispute-authorization.cedar`, `capabilities/subscription-lifecycle.yaml`, and the PSP adapter pattern from `ARCHITECTURE.md`.

Preserved as already-substantive: IP-001 through IP-018. They contain specific crates, value objects, state machines, ports, Cedar files, routes, PSP APIs, runbooks, and test commands rather than placeholder scaffold.

Rewritten in place: none for payments. This service's short base IPs were compact but not stamped shells.

Counterpart-reference repairs: added explicit counterpart rows to IP-002, IP-005, IP-009, IP-010, IP-011, IP-012, and IP-016 where the content was specific but lacked a literal Stripe/Adyen/Chargebee comparison row.

Deleted as duplicative: none.

Follow-up: many 400+ line journey IPs still use non-Big-8 or domain-specific evidence language and do not all match the literal Wave grep pattern. They were not rewritten in this bucket because they are not the 55-line stamped implementation-plan cluster.

## Wave 15-IMPL-truth-up (2026-05-21)

Scaffolded 20 payment-domain crates per IP-substance declarations. Workspace updated, cargo check run.

### Crates scaffolded

- `oya-payments-adapter-adyen`
- `oya-payments-adapter-stripe`
- `oya-payments-charge-app`
- `oya-payments-charge-domain`
- `oya-payments-charge-grpc`
- `oya-payments-charge-kernel`
- `oya-payments-charge-rest`
- `oya-payments-charge-usecase`
- `oya-payments-dispute-domain`
- `oya-payments-dispute-usecase`
- `oya-payments-kyc-kyb-domain`
- `oya-payments-kyc-kyb-usecase`
- `oya-payments-payout-domain`
- `oya-payments-payout-usecase`
- `oya-payments-refund-domain`
- `oya-payments-refund-usecase`
- `oya-payments-settlement-domain`
- `oya-payments-settlement-worker`
- `oya-payments-subscription-domain`
- `oya-payments-subscription-usecase`

### Cargo check status

PASS. Focused command completed with exit code 0:

`cargo check -p oya-payments-charge-domain -p oya-payments-charge-kernel -p oya-payments-charge-usecase -p oya-payments-charge-app -p oya-payments-charge-rest -p oya-payments-charge-grpc 2>&1 | tail -40`

Observed tail:

`Finished dev profile [unoptimized + debuginfo] target(s) in 0.16s`

### Follow-ups

None for the focused charge crate workspace check. Broader payment crate validation beyond the requested six charge crates was not run in this finisher slice.
