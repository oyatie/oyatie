# Payments Performance Benchmark Numbers — Target Baseline — 2026-05-20

## Citation Anchor Block
1. Canonical audit requirement: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3829-4235`.
2. Benchmark disclosure requirement: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4202-4228` and deployment context taxonomy in `specs/master-plan-sequencing.json:704-868`.
3. Payments PRD performance source: `microservices/payments/PRD.md:781-815`, `microservices/payments/PRD.md:1556-1568`.
4. Payments capacity and current benchmark source: `microservices/payments/capacity-model.md:24-256` and `microservices/payments/benchmarks/stripe-vs-adyen-vs-checkout-vs-oyatie.md:1-112`.
5. Documentation rigor source: `docs/standards/documentation-rigor.md:143-156` for capacity math, observability, rollback, multi-region, and versioning evidence.

## Methodology Disclosure
- These are target benchmark numbers and planning numbers, not measured Oyatie production results.
- Measured benchmarks must be added in a later build phase with source-controlled harnesses, raw run artifacts, OS/arch/context disclosure, and repeatability evidence.
- The local benchmark file is treated as an unverified planning artifact because it includes public-looking figures without enough public-source provenance at `microservices/payments/benchmarks/stripe-vs-adyen-vs-checkout-vs-oyatie.md:21-38`.
- Stripe numeric limits are sourced from current official Stripe rate-limit documentation: https://docs.stripe.com/rate-limits.
- Adyen public docs expose feature surfaces but not one universal public API-latency or account-wide RPS number; Adyen rows below are estimated from Oyatie's current capacity model and Adyen platform feature shape.
- Braintree public docs expose feature surfaces but not one universal public API-latency or account-wide RPS number; Braintree rows below are estimated from Oyatie's current capacity model and Braintree feature shape.
- Any row marked "estimated from Oyatie model" is not a counterpart-measured public benchmark.
- Deployment contexts use the six canonical IDs from `specs/master-plan-sequencing.json:704-746`.
- OCI Always Free demo_trial rows explicitly honor the Always Free profile from `specs/master-plan-sequencing.json:857-868`.

## §1 Methodology
| # | Benchmark dimension | Workload definition | Disclosure axis |
|---:|---|---|---|
| M-01 | Charge authorization p50 latency | `POST /v1/charges` with idempotency key, single PSP happy path, no 3DS challenge. | latency |
| M-02 | Charge authorization p95 latency | Same workload under steady-state tier load. | latency |
| M-03 | Charge authorization p99 latency | Same workload under target tier burst. | latency |
| M-04 | Capture p99 latency | `POST /v1/charges/{id}/capture` with ledger write and audit event. | latency |
| M-05 | Refund p99 latency | `POST /v1/refunds` with original charge lookup and ledger reversal. | latency |
| M-06 | Payout API p99 latency | `POST /v1/payouts` with sub-merchant eligibility check. | latency |
| M-07 | Webhook acknowledge p99 latency | PSP webhook receiver validates signature and persists normalized event. | latency |
| M-08 | Webhook delivery success | Normalized event reaches consumer topic within SLO window. | reliability |
| M-09 | Sustained charge throughput | 15-minute steady-state successful charge attempts per second. | throughput |
| M-10 | Burst charge throughput | 60-second burst before shedding/non-critical degradation. | throughput |
| M-11 | Webhook ingest throughput | Normalized webhook deliveries per second. | throughput |
| M-12 | Concurrent in-flight operations | Unique active payment workflows without queue breach. | concurrency |
| M-13 | Tenant count ceiling | Active tenants with isolated PSP credentials and policy scope. | scale |
| M-14 | Sub-merchant count ceiling | Active connected sellers/sub-merchants. | scale |
| M-15 | Daily transaction ceiling | Successful payment operations per day. | scale |
| M-16 | Ledger write ceiling | Durable ledger entries per second. | storage/write |
| M-17 | Event fanout ceiling | Audit/payment events emitted per second. | eventing |
| M-18 | RPO target | Maximum data loss target for committed payment event. | resilience |
| M-19 | RTO target | Time to restore tier service after cell failure. | resilience |
| M-20 | Cost ceiling | Payment-platform infra cost excluding PSP fees. | cost |
| M-21 | OS/arch disclosure | Every measured run must name OS, architecture, package/container, and kernel/runtime. | ADR-0328 §D-20.152 |
| M-22 | Deployment-context disclosure | Every measured run must name canonical deployment context. | ADR-0328 §D-15 |
| M-23 | Tenant-class disclosure | Every measured run must name tenant_class and whether OCI Always Free applies. | ADR-0328 §D-19 |
| M-24 | PSP-mode disclosure | Every measured run must name mocked PSP, sandbox PSP, or live PSP and rate-limit model. | benchmark provenance |
| M-25 | Measurement stop condition | Claims remain target-only until raw harness output and dashboards exist. | anti-overclaim |

## §2 Counterpart Numbers
### §2.1 Stripe Public/Estimated Numbers
| # | Number | Source/provenance | Interpretation |
|---:|---|---|---|
| ST-01 | 100 operations/second global live-mode API limiter. | Official Stripe rate-limit docs. | Hard public planning ceiling before approved increase. |
| ST-02 | 25 operations/second global sandbox API limiter. | Official Stripe rate-limit docs. | Sandbox load tests cannot represent live capacity. |
| ST-03 | 25 requests/second default endpoint limit. | Official Stripe rate-limit docs. | Endpoint-level target for client throttling. |
| ST-04 | 1,000 PaymentIntent update operations/hour per PaymentIntent. | Official Stripe rate-limit docs. | Resource-specific mutation ceiling. |
| ST-05 | 20 read operations/second Files API. | Official Stripe rate-limit docs. | Non-payment API read cap. |
| ST-06 | 20 write operations/second Files API. | Official Stripe rate-limit docs. | Non-payment API write cap. |
| ST-07 | 20 read operations/second Search API. | Official Stripe rate-limit docs. | Search/reporting cap. |
| ST-08 | 10 new invoices/subscription/minute. | Official Stripe rate-limit docs. | Billing mutation cap. |
| ST-09 | 20 new invoices/subscription/day. | Official Stripe rate-limit docs. | Billing daily cap. |
| ST-10 | 200 quantity updates/subscription/hour. | Official Stripe rate-limit docs. | Billing update cap. |
| ST-11 | 15 Create Payout API requests/second. | Official Stripe rate-limit docs. | Payout endpoint cap. |
| ST-12 | 30 concurrent payout requests/business. | Official Stripe rate-limit docs. | Payout concurrency cap. |
| ST-13 | 30 Connect accounts/second live mode. | Official Stripe rate-limit docs. | Platform onboarding cap. |
| ST-14 | 5 Connect accounts/second sandbox. | Official Stripe rate-limit docs. | Sandbox onboarding cap. |
| ST-15 | 1,000 meter events/second/account live mode. | Official Stripe rate-limit docs. | Usage billing ingestion cap. |
| ST-16 | 500 read API requests per transaction average over rolling 30 days. | Official Stripe rate-limit docs. | Read allocation planning. |
| ST-17 | Minimum 10,000 read requests/month/account. | Official Stripe rate-limit docs. | Low-volume read allocation. |
| ST-18 | Payment gateway live latency differs materially from sandbox latency. | Official Stripe rate-limit load-test guidance. | Oyatie should mock PSP for load tests. |

### §2.2 Adyen Public/Estimated Numbers
| # | Number | Source/provenance | Interpretation |
|---:|---|---|---|
| AD-01 | 200 authorizations/second PSP ceiling used in current Oyatie capacity model. | Estimated from `capacity-model.md:80`; not an Adyen public universal cap. | Planning input only. |
| AD-02 | 2,000 authorizations/second Paid aggregate target across PSP/account shards. | Estimated from Oyatie target model. | Requires account-specific approval and sharding. |
| AD-03 | 300ms p99 external PSP round-trip target. | Estimated from Oyatie charge budget and local benchmark doc. | Target, not public Adyen measurement. |
| AD-04 | 1,000 webhooks/second ingest target for paid tenant_class Adyen event load. | Estimated from `capacity-model.md:181-188`. | Normalized webhook target. |
| AD-05 | 5,000 webhooks/second burst target for paid tenant_class. | Estimated from `capacity-model.md:181-188`. | Burst capacity planning. |
| AD-06 | 99.95% charge API availability target at Paid. | Derived from local OpenSLO availability target `slos/charge-api-availability.openslo.yaml`. | Oyatie target. |
| AD-07 | 99.99% webhook delivery target at Paid. | Derived from local webhook SLO `slos/webhook-delivery-success.openslo.yaml`. | Oyatie target. |
| AD-08 | 48-hour dispute evidence response operational target. | Estimated from dispute/runbook class; counterpart docs support dispute management but no universal metric. | Operational target. |
| AD-09 | 15-minute PSP failover detection target. | Estimated from Oyatie runbook expectations. | Needs measured monitoring. |
| AD-10 | 1-minute high-severity alert target. | Estimated from incident-response severity timing. | Needs alert rule. |
| AD-11 | 0 committed ledger-entry data loss target. | Derived from payment ledger invariants. | RPO planning target. |
| AD-12 | 50ms internal routing decision p99 target. | Estimated from adapter/routing budget. | Internal target. |
| AD-13 | 20,000 sub-merchants Paid planning ceiling. | Estimated from tier model and marketplace scope. | Not public Adyen cap. |
| AD-14 | 1,000,000 sub-merchants Paid planning ceiling. | Estimated from hyperscaler target. | Requires sharded onboarding architecture. |
| AD-15 | 10,000 daily settlement batches Paid planning target. | Estimated from settlement journey family. | Needs measured worker benchmark. |

### §2.3 Braintree Public/Estimated Numbers
| # | Number | Source/provenance | Interpretation |
|---:|---|---|---|
| BT-01 | 100 authorizations/second planning ceiling. | Estimated from Oyatie model and common PSP throttling posture; no universal public Braintree cap found in official docs. | Planning input only. |
| BT-02 | 25 sandbox operations/second planning ceiling. | Estimated using Stripe sandbox-public cap as conservative sandbox analogue. | Must be replaced with Braintree account guidance. |
| BT-03 | 300ms p99 external PSP round-trip target. | Estimated from local benchmark table and capacity budget. | Target only. |
| BT-04 | 500ms p99 charge API wrapper target at Paid. | Derived from local OpenSLO latency file. | Oyatie target. |
| BT-05 | 1,000ms p99 3DS challenge initiation target. | Estimated from 3DS flow complexity. | Needs measured harness. |
| BT-06 | 99.9% subscription renewal success target. | Derived from local subscription renewal SLO class. | Oyatie target. |
| BT-07 | 99.9% webhook processing success target at Paid. | Derived from webhook SLO direction. | Oyatie target. |
| BT-08 | 5,000 active subscriptions DemoTrial planning ceiling. | Estimated from tier capacity. | Target only. |
| BT-09 | 100,000 active subscriptions Paid planning ceiling. | Estimated from tier capacity. | Target only. |
| BT-10 | 5,000,000 active subscriptions Paid planning ceiling. | Estimated from hyperscale marketplace target. | Target only. |
| BT-11 | 50,000,000 active subscriptions Paid planning ceiling. | Estimated from hyperscaler target. | Requires sharded billing engine. |
| BT-12 | 1,000 webhooks/second Paid target. | Estimated from capacity model. | Needs benchmark. |
| BT-13 | 2,500 transactions/second Paid aggregate target. | Estimated from tenant_class target. | Requires PSP/account sharding. |
| BT-14 | 10,000 transactions/second Paid aggregate target. | Estimated from tenant_class target. | Requires Braintree adapter absent today. |
| BT-15 | 0 duplicate charge tolerance. | Derived from `failure-modes.md:40-51`. | Correctness target, not throughput metric. |

## §3 Oyatie Target Numbers by Tier and Deployment Context
### §3.1 `oyatie-public-cloud`
| Tier | Metric | Target | Provenance |
|---|---|---:|---|
| DemoTrial | Charge p50 | 80 ms | target |
| DemoTrial | Charge p95 | 180 ms | target |
| DemoTrial | Charge p99 | 350 ms | target |
| DemoTrial | Sustained charges | 50 rps | tenant_class target |
| DemoTrial | Burst charges | 100 rps | tenant_class target |
| DemoTrial | Webhook ingest | 250 rps | tenant_class target |
| DemoTrial | Concurrent operations | 1,000 | tenant_class target |
| DemoTrial | Tenant ceiling | 25 | tenant_class target |
| DemoTrial | Daily tx ceiling | 1,000,000 | tenant_class target |
| DemoTrial | RPO | 0 committed events | ledger invariant |
| Paid | Charge p50 | 60 ms | target |
| Paid | Charge p95 | 140 ms | target |
| Paid | Charge p99 | 250 ms | target |
| Paid | Sustained charges | 1,000 rps | tenant_class target |
| Paid | Burst charges | 2,500 rps | tenant_class target |
| Paid | Webhook ingest | 5,000 rps | tenant_class target |
| Paid | Concurrent operations | 50,000 | tenant_class target |
| Paid | Tenant ceiling | 5,000 | `tenant_class adoption record:41-71` |
| Paid | Daily tx ceiling | 50,000,000 | target |
| Paid | RPO | 0 committed events | ledger invariant |
| Paid | Charge p50 | 45 ms | target |
| Paid | Charge p95 | 110 ms | target |
| Paid | Charge p99 | 200 ms | PRD bar |
| Paid | Sustained charges | 5,000 rps | target |
| Paid | Burst charges | 12,000 rps | target |
| Paid | Webhook ingest | 25,000 rps | target |
| Paid | Concurrent operations | 250,000 | target |
| Paid | Tenant ceiling | 50,000 | target |
| Paid | Daily tx ceiling | 250,000,000 | target |
| Paid | RTO | 15 minutes | target |
| Paid | Charge p50 | 35 ms | hyperscaler target |
| Paid | Charge p95 | 90 ms | hyperscaler target |
| Paid | Charge p99 | 150 ms | hyperscaler target |
| Paid | Sustained charges | 25,000 rps | hyperscaler target |
| Paid | Burst charges | 60,000 rps | hyperscaler target |
| Paid | Webhook ingest | 100,000 rps | hyperscaler target |
| Paid | Concurrent operations | 1,000,000 | hyperscaler target |
| Paid | Tenant ceiling | 1,000,000 | hyperscaler target |
| Paid | Daily tx ceiling | 1,000,000,000 | hyperscaler target |
| Paid | RTO | 5 minutes | hyperscaler target |

### §3.2 `guest-on-aws`
| Tier | Metric | Target | Provenance |
|---|---|---:|---|
| DemoTrial | Charge p99 | 380 ms | target with guest cloud overhead |
| DemoTrial | Sustained charges | 40 rps | target |
| DemoTrial | Webhook ingest | 200 rps | target |
| DemoTrial | Tenant ceiling | 20 | target |
| Paid | Charge p99 | 270 ms | target |
| Paid | Sustained charges | 800 rps | target |
| Paid | Webhook ingest | 4,000 rps | target |
| Paid | Tenant ceiling | 4,000 | target |
| Paid | Charge p99 | 220 ms | target |
| Paid | Sustained charges | 4,000 rps | target |
| Paid | Webhook ingest | 20,000 rps | target |
| Paid | Tenant ceiling | 40,000 | target |
| Paid | Charge p99 | 170 ms | target |
| Paid | Sustained charges | 20,000 rps | target |
| Paid | Webhook ingest | 80,000 rps | target |
| Paid | Tenant ceiling | 800,000 | target |

### §3.3 `guest-on-oci`
| Tier | Metric | Target | Provenance |
|---|---|---:|---|
| DemoTrial | Compute envelope | 4 OCPU / 24 GB RAM | `ADR-0328:3514-3527` |
| DemoTrial | Block storage envelope | 200 GB | `ADR-0328:3532-3549` |
| DemoTrial | Object/archive envelope | 10 GB + 10 GB | `ADR-0328:3532-3549` |
| DemoTrial | Load balancer bandwidth | 10 Mbps | `ADR-0328:3565-3577` |
| DemoTrial | Egress budget | 10 TB/month with 8 TB alert | `ADR-0328:3565-3577` |
| DemoTrial | Charge p50 | 120 ms | target constrained to Always Free |
| DemoTrial | Charge p95 | 300 ms | target constrained to Always Free |
| DemoTrial | Charge p99 | 650 ms | target constrained to Always Free |
| DemoTrial | Sustained charges | 15 rps | target constrained to Always Free |
| DemoTrial | Burst charges | 30 rps | target constrained to Always Free |
| DemoTrial | Webhook ingest | 75 rps | target constrained to Always Free |
| DemoTrial | Tenant ceiling | 5 | target constrained to Always Free |
| DemoTrial | Daily tx ceiling | 250,000 | target constrained to Always Free |
| Paid | Charge p99 | 290 ms | paid OCI target |
| Paid | Sustained charges | 700 rps | paid OCI target |
| Paid | Webhook ingest | 3,500 rps | paid OCI target |
| Paid | Tenant ceiling | 3,500 | paid OCI target |
| Paid | Charge p99 | 230 ms | paid OCI target |
| Paid | Sustained charges | 3,500 rps | paid OCI target |
| Paid | Webhook ingest | 17,500 rps | paid OCI target |
| Paid | Tenant ceiling | 35,000 | paid OCI target |
| Paid | Charge p99 | 180 ms | paid OCI target |
| Paid | Sustained charges | 18,000 rps | paid OCI target |
| Paid | Webhook ingest | 72,000 rps | paid OCI target |
| Paid | Tenant ceiling | 720,000 | paid OCI target |

### §3.4 `on-prem`
| Tier | Metric | Target | Provenance |
|---|---|---:|---|
| DemoTrial | Charge p99 | 450 ms | target; customer hardware variability |
| DemoTrial | Sustained charges | 25 rps | target |
| DemoTrial | Webhook ingest | 125 rps | target |
| DemoTrial | Tenant ceiling | 10 | target |
| Paid | Charge p99 | 320 ms | target |
| Paid | Sustained charges | 500 rps | target |
| Paid | Webhook ingest | 2,500 rps | target |
| Paid | Tenant ceiling | 2,500 | target |
| Paid | Charge p99 | 250 ms | target |
| Paid | Sustained charges | 2,500 rps | target |
| Paid | Webhook ingest | 12,500 rps | target |
| Paid | Tenant ceiling | 25,000 | target |
| Paid | Charge p99 | 200 ms | target |
| Paid | Sustained charges | 12,000 rps | target |
| Paid | Webhook ingest | 50,000 rps | target |
| Paid | Tenant ceiling | 500,000 | target |

### §3.5 `colo`
| Tier | Metric | Target | Provenance |
|---|---|---:|---|
| DemoTrial | Charge p99 | 420 ms | target |
| DemoTrial | Sustained charges | 30 rps | target |
| DemoTrial | Webhook ingest | 150 rps | target |
| DemoTrial | Tenant ceiling | 15 | target |
| Paid | Charge p99 | 300 ms | target |
| Paid | Sustained charges | 600 rps | target |
| Paid | Webhook ingest | 3,000 rps | target |
| Paid | Tenant ceiling | 3,000 | target |
| Paid | Charge p99 | 235 ms | target |
| Paid | Sustained charges | 3,000 rps | target |
| Paid | Webhook ingest | 15,000 rps | target |
| Paid | Tenant ceiling | 30,000 | target |
| Paid | Charge p99 | 190 ms | target |
| Paid | Sustained charges | 15,000 rps | target |
| Paid | Webhook ingest | 60,000 rps | target |
| Paid | Tenant ceiling | 600,000 | target |

### §3.6 `oyatie-as-cloud-provider`
| Tier | Metric | Target | Provenance |
|---|---|---:|---|
| DemoTrial | Charge p99 | 330 ms | target |
| DemoTrial | Sustained charges | 75 rps | target |
| DemoTrial | Webhook ingest | 375 rps | target |
| DemoTrial | Tenant ceiling | 50 | target |
| Paid | Charge p99 | 230 ms | target |
| Paid | Sustained charges | 1,500 rps | target |
| Paid | Webhook ingest | 7,500 rps | target |
| Paid | Tenant ceiling | 7,500 | target |
| Paid | Charge p99 | 180 ms | target |
| Paid | Sustained charges | 7,500 rps | target |
| Paid | Webhook ingest | 37,500 rps | target |
| Paid | Tenant ceiling | 75,000 | target |
| Paid | Charge p99 | 130 ms | hyperscaler target |
| Paid | Sustained charges | 40,000 rps | hyperscaler target |
| Paid | Webhook ingest | 150,000 rps | hyperscaler target |
| Paid | Tenant ceiling | 1,500,000 | hyperscaler target |

## §4 Per-Context Overlay
| Context | Latency adjustment | Throughput adjustment | Primary reason | Required missing proof |
|---|---:|---:|---|---|
| `oyatie-public-cloud` | baseline | baseline | Oyatie controls substrate and topology. | OpenTofu context root and measured run. |
| `guest-on-aws` | +10-20 ms p99 | -20% | Guest cloud account control-plane and PSP egress variability. | `iac/guest-on-aws/` and AWS guest harness. |
| `guest-on-oci` DemoTrial | +250-450 ms p99 | severe cap | Always Free compute/LB envelope. | `iac/oci-guest/always-free/` and capacity smoke. |
| `guest-on-oci` paid | +20-40 ms p99 | -30% | OCI guest topology and regional service variability. | paid OCI context plan and measured run. |
| `on-prem` | +70-150 ms p99 | -50% | Customer hardware/network variance and local PSP egress. | on-prem hardware class manifest. |
| `colo` | +50-100 ms p99 | -40% | Remote-hands/network edge variability. | colo OpenTofu and bare-metal profile. |
| `oyatie-as-cloud-provider` | -20-40 ms p99 | +50% | Native substrate, cell placement, and integrated observability. | `iac/oyatie-iaas/` and provider harness. |

## §5 Comparison Narrative
| Headline number | Oyatie target vs counterparts | Classification | Evidence |
|---|---|---|---|
| Charge p99 150-200 ms at Paid/Paid | Ahead of Stripe public rate-limit docs because Stripe publishes limits, not a latency SLO; not proven measured. | target-ahead | `PRD.md:96-105`; target table |
| Charge p99 500-650 ms at OCI Always Free demo_trial | Catch-up; intentionally constrained by Always Free resources. | catch-up | ADR OCI limits and target table |
| Sustained 25,000-40,000 rps Paid | Above public Stripe default limits, requires account sharding and multi-PSP routing. | target-ahead but unmeasured | Stripe rate-limit docs; target table |
| Stripe live 100 ops/s global default | Oyatie paid target exceeds single-account default. | requires sharding/approval | Stripe official rate-limit docs |
| Stripe sandbox 25 ops/s | Oyatie DemoTrial test targets must mock PSP instead of direct sandbox saturation. | parity/caution | Stripe official load-test guidance |
| Stripe payout 15 rps and 30 concurrent requests/business | Oyatie payout targets must throttle per PSP. | catch-up | Stripe official rate-limit docs |
| Adyen platform split/payout throughput | Oyatie targets use model estimates, not Adyen public numbers. | unproven | Adyen docs plus local capacity model |
| Braintree throughput | Oyatie targets are blocked by missing adapter and no public universal cap. | catch-up | Braintree docs plus local adapter gap |
| Webhook ingest 100,000-150,000 rps Paid | Ahead of counterpart public docs if achieved, but no measured harness. | target-ahead | `capacity-model.md:181-198` and target table |
| Tenant ceiling 1M+ Paid | Ahead of current local docs, requires sub-merchant/account sharding proof. | target-ahead | target table; contract gap noted |
| RPO zero committed events | Stronger than public payment-provider user-facing docs; depends on ledger/audit-chain implementation. | parity/ahead target | `failure-modes.md:157-194` |
| RTO 5-15 minutes | Competitive target; no measured failover evidence. | target parity | `multi-region.md:1-210` |
| OCI Always Free demo_trial 15 rps | Below counterpart default public ceilings; acceptable only for sandbox/evaluation. | intentionally below | ADR OCI profile |
| OpenSLO p99 500 ms vs PRD 200 ms | Current local docs conflict; benchmark target must reconcile. | local incoherence | `slos/charge-api-latency.openslo.yaml`; `PRD.md:96-105` |
| Current benchmark file numbers | Not accepted as measured facts. | evidence gap | `benchmarks/stripe-vs-adyen-vs-checkout-vs-oyatie.md:21-38` |

## §6 Build-Phase Measurement Requirements
| Requirement | Required artifact | Why it is needed |
|---|---|---|
| R-01 | Rust benchmark harness committed under payments or shared perf crate. | Prevents target-only claims from hardening into facts. |
| R-02 | Mock PSP mode with latency distributions. | Stripe docs warn sandbox latency differs from live gateway behavior. |
| R-03 | Live PSP smoke mode with throttling. | Validates real adapter behavior without load-test abuse. |
| R-04 | Per-context OpenTofu plan artifact. | Confirms the deployment context exists before measurement. |
| R-05 | OS/arch matrix output. | Satisfies ADR-0328 §D-17 and §D-20.152. |
| R-06 | Raw latency histograms. | Needed for p50/p95/p99 claims. |
| R-07 | Throughput saturation curve. | Needed for sustained and burst RPS claims. |
| R-08 | Webhook replay harness. | Needed for webhook ingest and delivery success. |
| R-09 | Ledger consistency checker. | Needed for zero duplicate-charge and RPO claims. |
| R-10 | Cost telemetry capture. | Needed for tenant_class cost ceilings. |
| R-11 | OCI Always Free quota monitor. | Needed to prove DemoTrial does not spill into paid resources. |
| R-12 | PSP-rate limiter verification. | Needed to avoid violating counterpart API limits. |
| R-13 | Multi-tenant noisy-neighbor test. | Needed for Paid tenant isolation claims. |
| R-14 | Failover drill transcript. | Needed for RTO claims. |
| R-15 | Regression dashboard snapshot. | Needed before any public benchmark claim. |
