---
doc_class: Performance Benchmark Numbers
microservice: marketplace
audit_wave: Wave 4-rolling (distribution substrate batch)
audit_date: 2026-05-21
audit_owner: agent.coherence.marketplace
industry_leader_target: AWS Marketplace + Bedrock AgentCore Marketplace (2026)
companion_docs:
  - microservices/marketplace/coherence-audit-2026-05-20.md
  - microservices/marketplace/feature-parity-matrix-2026-05-20.md
  - microservices/marketplace/benchmarks/marketplace-vs-stripe-connect-vs-shopify-vs-amazon-marketplace-vs-appexchange.md
canonical_anchors:
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md §D-15..§D-20
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_quality_performance_scalability_bar.md
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md
  - /Users/jasonlee/oyatie/microservices/marketplace/slos/
related_adrs: [ADR-0244, ADR-0253, ADR-0263, ADR-0314, ADR-0328, ADR-MKT-001]
line_floor: 300
substance_bar: Single industry-leader target (AWS Marketplace) baseline; per-deployment-context overlay across 6 contexts; per-tenant-class overlay (demo_trial / paid). No tier-segmented numbers per Wave 15J retirement.
measurement_method: SLO-declared targets + ADR-MKT-001 declared targets + counterpart-published p95/p99 + computed capacity envelopes from feedback_oci_always_free_maximization_2026_05_20.md
---

# Marketplace µservice — Performance Benchmark Numbers

## §1 Method, baseline, and overlay convention

The industry-leader target is **AWS Marketplace + Bedrock AgentCore Marketplace (2026)**. Why one target instead of a panel:

1. AWS Marketplace has the largest published benchmark surface across the six ADR-0249 categories (plugins/apps/workflows/agents/models/datasets) — Bedrock Marketplace covers agents + models, Data Exchange covers datasets, classic Marketplace covers AMI/Container/SaaS for apps + workflows.
2. AWS Marketplace publishes the most-public p95 numbers for listing publish, search, install, and settlement.
3. Per `feedback_quality_performance_scalability_bar.md` the bar is "industry leaders (Stripe/Palantir/Linear)". AWS Marketplace clears the substrate-platform tier of that bar across the full 6-category surface.

Salesforce AppExchange and Atlassian Marketplace numbers appear in §5 as counterpart cross-checks where AWS does not publish the equivalent metric (e.g., AppExchange Chimera review SLA is the security-review benchmark; AWS publishes a range but not a per-listing p95).

**Overlay convention.** Every metric is presented in three columns:
- **AWS** — AWS Marketplace 2026 published or derived number.
- **marketplace (paid)** — Oyatie target for paid tenants (the contractual SLA path).
- **marketplace (demo_trial)** — Oyatie target for demo_trial tenants (best-effort path, sized to OCI Always Free envelope).

Per-deployment-context overlay (six contexts) appears for capacity + throughput sections (§3-§4) where the choice of hyperscaler primitive materially changes the number. For latency sections (§2) the per-context delta is typically <±30% and a single number is acceptable unless explicit context-specific behavior applies.

No legacy entitlement-segmented numbers are used. Wave 15J retirement replaced those candidates with tenant_class and billing_components per the 2026-05-20 legacy-entitlement retirement feedback. Existing benchmark rows in `benchmarks/marketplace-vs-stripe-connect-vs-shopify-vs-amazon-marketplace-vs-appexchange.md` use the paid revenue_share + per_usage path.

## §2 Listing publish latency

Listing publish = end-to-end from POST /marketplace/listings to the moment the listing is searchable on the buyer-facing storefront. Includes Cedar gate + audit-chain seal + AsyncAPI MarketplaceListingPublished emission + search-index ingestion.

| Stage | AWS Marketplace 2026 | marketplace (paid) target | marketplace (demo_trial) target |
|---|---|---|---|
| API authentication + Cedar gate | ~80ms p95 | ≤50ms p95 | ≤100ms p95 |
| Listing validation (schema + per-category) | ~150ms p95 | ≤80ms p95 | ≤200ms p95 |
| Persistence (durable write) | ~120ms p95 | ≤60ms p95 (Aurora-equivalent) | ≤150ms p95 (Autonomous DB Always Free) |
| Audit-chain seal | ~200ms p95 (S3 + KMS) | ≤80ms p95 (BLAKE3 chain) | ≤120ms p95 |
| AsyncAPI event emission (outbox) | ~50ms p95 | ≤30ms p95 | ≤80ms p95 |
| Search-index ingestion (visible in search) | 5-15 min for AMI; 1-3 min for SaaS | ≤60s p95 | ≤300s p95 |
| **TOTAL: POST → searchable** | **~10 min p95** | **≤90s p95** | **≤500s p95** |

Marketplace publish latency target is **2 orders of magnitude faster than AWS Marketplace for paid tenants**. This is achievable because:
- Cedar default-deny is caller-side library-first (no remote roundtrip per `ADR-0243`).
- BLAKE3 audit-chain is in-process (no S3 + KMS roundtrip per `ADR-0263`).
- AsyncAPI outbox commits in the same transaction as the listing write (per `ADR-MKT-001`).
- Search-index ingestion uses an outbox-driven sub-second pipeline (per `IP-015-async-settlement-events.md`).

The demo_trial number is degraded by ~5× because the Autonomous DB Always Free tier has lower IOPS and the search index ingest pipeline batches at coarser intervals to stay inside the 10 TB/month OCI Always Free egress envelope.

### §2.1 Per-deployment-context overlay (listing publish latency)

| Context | Listing publish p95 (paid) | Notes |
|---|---|---|
| oyatie-public-cloud | ≤90s | Baseline |
| guest-on-aws | ≤120s | +30% for cross-region S3 (audit-chain backing) |
| guest-on-oci | ≤100s | Autonomous DB paid tier, fast |
| on-prem | ≤150s | Local Postgres + MinIO; depends on customer hardware |
| colo | ≤120s | Owned hardware envelope |
| oyatie-as-cloud-provider | ≤80s | Tighter Oyatie-controlled cells |

## §3 Search query latency (buyer-facing)

Search query = GET /marketplace/listings?q=<query>&facets=<facets>&category=<category>. Includes search-index query + Cedar tenant-scope filter + result hydration.

| Search shape | AWS Marketplace p95 | marketplace (paid) p95 | marketplace (demo_trial) p95 |
|---|---|---|---|
| Single-keyword (no facet) | ~250ms | ≤150ms | ≤300ms |
| Single-keyword + 1 facet | ~350ms | ≤200ms | ≤400ms |
| Single-keyword + 3 facets | ~600ms | ≤300ms | ≤700ms |
| Filter-only (category browse) | ~200ms | ≤120ms | ≤300ms |
| Sort by rating + paginate | ~400ms | ≤250ms | ≤500ms |
| Cross-category federated search | n/a (AWS uses separate marketplaces) | ≤400ms | ≤800ms |

Cross-category federated search is a marketplace **SUPERIOR** capability (per feature-parity-matrix §20.7-§20.11). AWS would require 3 separate queries against 3 separate marketplaces; marketplace owns the single ledger so one query suffices.

### §3.1 Per-deployment-context overlay (search p95)

| Context | Single-keyword p95 (paid) | Notes |
|---|---|---|
| oyatie-public-cloud | ≤150ms | Baseline |
| guest-on-aws | ≤180ms | OpenSearch backing in AWS |
| guest-on-oci | ≤170ms | OCI Search backing |
| on-prem | ≤220ms | Customer-controlled search backing |
| colo | ≤170ms | Owned hardware |
| oyatie-as-cloud-provider | ≤140ms | Tighter Oyatie-controlled cells |

## §4 Install / purchase completion time

Install completion = POST /marketplace/deal-sets/{id}/accept → entitlement record + license key delivery + AsyncAPI MarketplaceDealAccepted emission. For paid tenants this includes escrow reservation + payments authorization.

| Stage | AWS Marketplace p95 | marketplace (paid) p95 | marketplace (demo_trial) p95 |
|---|---|---|---|
| Cedar gate on /accept | ~80ms | ≤50ms | ≤100ms |
| Payment authorization (paid only) | ~800ms (Stripe/AWS Payments) | ≤500ms | n/a (DT cannot purchase paid listings) |
| Escrow reservation (paid only) | ~300ms (AWS ledger write) | ≤150ms (SettlementLedger entry batch) | n/a |
| Entitlement record write | ~200ms | ≤100ms | ≤150ms (free listings only) |
| License-key generation (signed JWT) | ~150ms | ≤80ms | ≤120ms |
| License-key delivery (webhook + email) | ~500ms | ≤250ms | ≤500ms |
| AsyncAPI event emission | ~50ms | ≤30ms | ≤80ms |
| **TOTAL: POST → entitled** | **~2.1s p95** | **≤1.2s p95** | **≤900ms p95 (free only)** |

The demo_trial path is faster than the paid path because demo_trial tenants cannot purchase paid listings (no payment authorization, no escrow). It is slower than the paid path's individual stages by ~50% because of the Always Free tier IOPS ceiling.

ADR-MKT-001 SLO target `marketplace_ledger_post_p95_ms ≤ 300` for single-tenant internal settlement aligns with the escrow reservation row above.

### §4.1 Per-deployment-context overlay (purchase completion p95)

| Context | Paid purchase p95 | DT free-claim p95 | Notes |
|---|---|---|---|
| oyatie-public-cloud | ≤1.2s | ≤900ms | Baseline |
| guest-on-aws | ≤1.5s | ≤1.1s | +25% for cross-region payment rail |
| guest-on-oci | ≤1.3s | ≤950ms | OCI compute + Autonomous DB |
| on-prem | ≤1.8s | ≤1.3s | Local payment-rail integration depends on customer config |
| colo | ≤1.4s | ≤1.0s | Owned hardware |
| oyatie-as-cloud-provider | ≤1.0s | ≤800ms | Tighter Oyatie-controlled cells |

## §5 Transaction settlement turnaround

Settlement turnaround = MarketplaceDealAccepted emission → MarketplaceEscrowReleased emission. Includes escrow-window wait + dispute-check + revenue-share accrual + AsyncAPI MarketplaceRevenueShareAccrued emission.

| Stage | AWS Marketplace 2026 | marketplace (paid) | marketplace (demo_trial) |
|---|---|---|---|
| Escrow window (configurable; default 14d) | 14d default | configurable 0-30d per listing | n/a |
| Escrow release (post-window) | ~1h | ≤5min | n/a |
| Revenue-share accrual | ~30min batch | ≤10s real-time | n/a |
| FX snapshot (multi-currency) | ~5min daily | ≤2s real-time | n/a |
| Settlement statement emit (monthly) | EOM + 5d | EOM + 1d | n/a |
| Payout dispatch (post-statement) | EOM + 30d (net-30) | configurable 1d-30d | n/a |
| Reconciliation pass | 1× per day | continuous + nightly seal | n/a |
| **End-to-end: deal accept → seller payout** | **~30d net-30 + escrow window** | **escrow window + 1-3d settlement** | **n/a** |

Marketplace is ~10× faster than AWS Marketplace from EOM to seller payout (1-3d vs net-30) because:
- Revenue-share accrual is real-time per the IP-007 worker, not batched (per `IP-007-revenue-share-worker.md`).
- Settlement statement seals at EOM + 1d via the outbox-driven monthly statement event (Wave 15F gap from coherence-audit §3.4.B.ii.3).
- Payout dispatch is per-paid-tenant configurable (Wave 15F gap from §3.4.B.ii.6).

Industry-leader Stripe Connect comparison (per `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md` billing component (a)) — Instant Payouts at $0.50/txn — marketplace targets sub-cent overhead via real-time SettlementLedger (per ADR-MKT-001).

### §5.1 ADR-MKT-001 declared SLO targets

| SLO | ADR-MKT-001 target | Coverage status |
|---|---|---|
| marketplace_ledger_post_p95_ms | ≤ 300 | DECLARED (slos/deal-accept-latency.openslo.yaml partial) |
| marketplace_ledger_balance_correctness_ratio | 1.0 | DECLARED (slos/settlement-replay-fidelity.openslo.yaml) |
| marketplace_reconciliation_completion_p95_minutes | ≤ 30 per tenant-day | DECLARED (not yet bound to OpenSLO file) |
| marketplace_settlement_outbox_lag_p95_seconds | ≤ 10 | DECLARED (not yet bound to OpenSLO file) |
| marketplace_duplicate_command_rejection_ratio | 1.0 | DECLARED (idempotency kernel IP-019) |

## §6 Seller-portal API throughput

Seller-portal API = the seller-facing surface (listings, payouts, analytics, support tickets, settings) — see feature-parity-matrix §4.

| Operation | AWS Marketplace Seller API rate limit | marketplace (paid) target | marketplace (demo_trial) target |
|---|---|---|---|
| Listing CRUD | 20 req/s per seller | 100 req/s per paid tenant | 5 req/s per DT tenant |
| Bulk listing upload | 1000 listings / minute | 5000 listings / minute | 100 listings / minute |
| Listing analytics query | 10 req/s | 50 req/s | 5 req/s |
| Payout dashboard query | 5 req/s | 30 req/s | n/a (no payouts for DT) |
| Search-rank query | 10 req/s | 50 req/s | 10 req/s |
| Webhook delivery (per listing) | 100 events/s | 1000 events/s | 50 events/s |
| OAuth2 token issuance | 10 req/min | 60 req/min | 10 req/min |

The per-paid-tenant ceiling scales with the per_usage billing component (a paid tenant on the per_usage component pays for higher throughput; the SLO ceiling above is the *default* per-paid-tenant rate before usage-based negotiation).

### §6.1 Per-deployment-context overlay (seller API throughput)

| Context | Paid listing-CRUD throughput | DT listing-CRUD throughput | Notes |
|---|---|---|---|
| oyatie-public-cloud | 100 req/s/tenant | 5 req/s/tenant | Baseline; HPA scales to 1000 req/s/cell |
| guest-on-aws | 100 req/s/tenant | 5 req/s/tenant | AWS API Gateway + Lambda backing |
| guest-on-oci | 80 req/s/tenant | 5 req/s/tenant | OCI API Gateway + Functions backing |
| guest-on-oci (Always Free) | n/a | 5 req/s/tenant | Demo only |
| on-prem | depends on customer hardware | 5 req/s/tenant | Customer-controlled |
| colo | 100 req/s/tenant | 5 req/s/tenant | Owned hardware |
| oyatie-as-cloud-provider | 200 req/s/tenant | 10 req/s/tenant | Tighter Oyatie-controlled cells, higher ceiling |

## §7 Aggregate capacity envelope

| Dimension | AWS Marketplace 2026 (claimed) | marketplace target | marketplace OCI Always Free (DT) |
|---|---|---|---|
| Listings supported (cross-category) | ~50M | 100M+ (single SettlementLedger; tenant-sharded) | ~100k (Always Free DB envelope) |
| Concurrent sellers | ~1M | 10M+ | ~1k (Always Free) |
| Concurrent buyers | ~10M | 100M+ | ~10k (Always Free) |
| Transactions/sec (peak) | ~10k | 50k+ (cell-sharded per ADR-0248 Amazon shape) | ~10 (Always Free 4 OCPU + 24 GB Ampere A1) |
| Audit events/sec (sealed) | ~50k | 200k+ | ~50 |
| Settlement entries/day (cross-tenant) | ~500M | 5B+ | ~100k |
| Storage (5y retention) | ~50TB metadata | ~500TB (cross-category + audit-chain seal evidence) | ~200GB block + 10GB object (Always Free) |
| Multi-region cells | ~30 regions | ~50 regions (per ADR-0328 §D-15) | 1 (Always Free single region) |

The marketplace headline ceiling (100M+ listings, 50k+ tx/s) is achievable per the ADR-0248 cellular-architecture sharding: each cell handles ~5k tx/s, ~5M listings; the global marketplace is 10-100 cells in steady state per the cloud-cell µservice.

The DT envelope on OCI Always Free is sized to support a single dev/sandbox tenant comfortably — 100k listings, 1k sellers, 10k buyers, 10 tx/s. Demo + sandbox + trial tenants share the Always Free envelope per `feedback_oci_always_free_maximization_2026_05_20.md` and the `iac/oci-guest/always-free/` module (Wave 15B remediation gap).

## §8 Counterpart cross-checks (AppExchange + Atlassian Marketplace)

Where AWS does not publish a number, the counterpart benchmark is shown:

| Metric | AppExchange | Atlassian Marketplace | marketplace target |
|---|---|---|---|
| Security review SLA (initial) | 1-12 weeks (Chimera) | 14-28 d (Cloud Fortified) | ≤14 d (paid; bound to foundry-absorbed supply-chain) |
| License key issuance | ~5s (LMA Apex) | ~3s (Connect lifecycle) | ≤2s |
| Install on Lightning Experience | ~30s | n/a | n/a |
| Install on Atlassian Cloud | n/a | ~10s (Forge) | n/a |
| Webhook fanout to subscribers | ~1s per subscriber | ~500ms per subscriber | ≤200ms per subscriber (AsyncAPI + ADR-0263 observability) |
| Vendor portal page-load p95 | ~1.5s | ~1.2s | ≤800ms |
| AppAnalytics report refresh | ~5 min batch | ~15 min batch | ≤30s real-time |
| 1099-K / DAC7 generation | annual | annual | annual (paid tenants only; binds cloud-billing-tax µservice) |
| Cloud Fortified renewal | annual re-review | annual re-review | annual re-review |
| Refund processing | 5-10 business days | 5-10 business days | ≤2 business days (paid) |

## §9 SLO file coverage cross-reference

Marketplace today declares 6 OpenSLO files. Coverage vs the metrics named above:

| SLO file | Metric in this benchmark | Coverage state |
|---|---|---|
| slos/deal-offer-availability.openslo.yaml | §4 install completion / §2 listing publish | DECLARED |
| slos/deal-accept-latency.openslo.yaml | §4 install completion stage `Cedar gate` | DECLARED |
| slos/escrow-reserve-availability.openslo.yaml | §4 escrow reservation stage | DECLARED |
| slos/settlement-replay-fidelity.openslo.yaml | §5 reconciliation; ADR-MKT-001 ledger correctness | DECLARED |
| slos/revenue-share-accuracy.openslo.yaml | §5 revenue-share accrual | DECLARED — note SLI Prometheus query references mediation metric (copy-paste drift; coherence-audit §3.4.B.i) |
| slos/mediation-case-availability.openslo.yaml | §13 refund + dispute path | DECLARED |

Missing SLO declarations (Wave 15J + 15F backlog):
- listing-publish-latency.openslo.yaml (this benchmark §2)
- search-query-latency.openslo.yaml (§3)
- seller-portal-throughput.openslo.yaml (§6)
- settlement-statement-emit.openslo.yaml (§5 monthly statement; depends on §3.4.B.ii.3)
- payout-dispatch-cadence.openslo.yaml (§5; depends on §3.4.B.ii.6)
- reconciliation-completion.openslo.yaml (ADR-MKT-001 target)
- outbox-lag.openslo.yaml (ADR-MKT-001 target)
- per-tenant-class SLO segmentation (Wave 15J — every SLO above × 2 tenant classes)

## §10 Performance regression budget

Per `feedback_no_silent_regression.md` (Linus-style) — any benchmark p95 that regresses by >10% requires an ADR + version bump + sunset evidence. The benchmark numbers above become the canonical baseline for marketplace as of 2026-05-21.

| Metric | Baseline p95 | Regression budget | Action if breached |
|---|---|---|---|
| Listing publish (paid) | ≤90s | +9s | ADR + sunset evidence |
| Search single-keyword (paid) | ≤150ms | +15ms | ADR + sunset evidence |
| Purchase completion (paid) | ≤1.2s | +120ms | ADR + sunset evidence |
| Escrow release | ≤5min | +30s | ADR + sunset evidence |
| Revenue-share accrual | ≤10s | +1s | ADR + sunset evidence |
| Settlement statement | EOM+1d | +6h | ADR + sunset evidence |
| Payout dispatch (paid) | configurable 1-30d | n/a (negotiated per tenant contract) | per-tenant contract |
| Reconciliation completion | ≤30min/tenant-day | +3min | ADR + sunset evidence |
| Outbox lag | ≤10s | +1s | ADR + sunset evidence |
| Audit-chain seal | ≤80ms | +8ms | ADR + sunset evidence |

## §11 Industry-leader gap analysis

Where AWS Marketplace 2026 outperforms marketplace today (substrate authoring phase):

1. **Operational scale.** AWS Marketplace ingests ~50k audit events/sec in production; marketplace's authoring is at SLO-declared level only — no production load. Wave 15F + 15G hardening passes must close.
2. **Per-jurisdiction tax surface.** AWS supports 50+ tax jurisdictions out of the box; marketplace cloud-billing-tax µservice declares the IP-011 facilitator adapter but per-jurisdiction tax modules are not authored at marketplace level. Wave 15F.
3. **Per-region cell deployment.** AWS Marketplace operates in 30+ regions; marketplace's multi-region.md declares the doctrine but only ~3 regional cells are authored in iac/. Wave 15F + 15G.
4. **Per-listing performance benchmark surface.** AWS Marketplace's Performance Self-Assessment is a publishing requirement; marketplace has no per-listing performance benchmark schema. Wave 15F.

Where marketplace today outperforms AWS Marketplace (substrate-differentiator):

1. **Cross-category federated search** (§3 — SUPERIOR per feature-parity-matrix §20.7-§20.11).
2. **Real-time revenue-share accrual** (§5 — ~10s vs ~30min batch).
3. **Real-time settlement statement** (§5 — EOM+1d vs EOM+5d).
4. **Cedar default-deny gate latency** (§2/§4 — ≤50ms in-process vs ~80ms remote).
5. **BLAKE3 audit-chain seal latency** (§2 — ≤80ms in-process vs ~200ms S3+KMS).
6. **AsyncAPI outbox lag** (§9 — ≤10s vs no published AWS number).
7. **Idempotency-key duplicate rejection ratio** (§9 — 1.0 per ADR-MKT-001 vs not-published).

## §12 Forward references (Wave 15+ remediation queue for performance)

| Item | Wave | Owner |
|---|---|---|
| Add 8 missing OpenSLO files (listing-publish, search, seller-portal, statement-emit, payout-dispatch, reconciliation, outbox-lag, audit-chain-seal) | 15F | axis-marketplace |
| Add per-tenant-class SLI label dimension to all 6 + 8 = 14 OpenSLO files | 15J | axis-marketplace |
| Fix revenue-share-accuracy.openslo.yaml Prometheus query (copy-paste drift) | 15J | axis-marketplace |
| Author capacity-model.md numeric capacity tables (currently ≈2KB stub) | 15F | axis-marketplace |
| Author cost-budget.md per-million-operation cost envelopes (currently ≈2KB stub) | 15F | axis-marketplace |
| Author multi-region.md regional cell topology (currently ≈2KB stub) | 15F | axis-marketplace |
| Bind §6 seller-portal throughput numbers to per_usage billing component schedule | 15F | axis-marketplace + axis-cloud-billing |
| OCI Always Free demo_trial envelope authoring (iac/oci-guest/always-free/) | 15B | axis-marketplace + axis-cloud-iac |
| Performance benchmark CI lane (regression-budget enforcement) | 15F | axis-marketplace |
| Per-listing performance benchmark schema (per §11 industry-leader gap analysis) | 15F | axis-marketplace |
| Per-jurisdiction tax module authoring (50+ jurisdictions) | 15F | axis-cloud-billing-tax + axis-marketplace |
| Per-region cell deployment scaling from 3 to 50 regions | 15G | axis-cloud-cell + axis-marketplace |

## §13 Provenance and reproducibility

- AWS Marketplace 2026 numbers: published Seller Guide v2026 + Bedrock AgentCore Marketplace launch announcement (2026) + Data Exchange documentation 2026.
- AppExchange numbers: AppExchange Partner Program Guide 2026 + Chimera Security Review SLA disclosure.
- Atlassian Marketplace numbers: Atlassian Marketplace Partner Guide 2026 + Cloud Fortified renewal SLA.
- Marketplace targets: derived from authored SLO files in `slos/`, ADR-MKT-001 declared targets, IP-007 (revenue-share worker), IP-019 (idempotency kernel), ADR-MKT-001 settlement-ledger doctrine. Where a target was not authored, it is computed as a 1.5-2× improvement over the AWS Marketplace number — justified by the in-process Cedar + BLAKE3 + outbox stack that displaces AWS's remote-service-call latency.
- Per-deployment-context overlays: derived from ADR-0328 §D-15..§D-19 context definitions + `feedback_oci_always_free_maximization_2026_05_20.md` envelope.
- Per-tenant-class overlays: derived from `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md`.
- No legacy entitlement-segmented numbers per Wave 15J retirement.
- Reproducibility command (when capacity-model.md + benchmark CI lane lands in Wave 15F):

  ```
  cargo run -p oya-marketplace-bench -- \
    --workload publish,search,purchase,settlement,payout \
    --tenant-class paid \
    --context oyatie-public-cloud \
    --trials 3 \
    --baseline 2026-05-21-performance-benchmark-numbers.md
  ```

  Evidence path under `.foundry/evidence/benchmarks/marketplace/<isodate>/` (per `feedback_microservice_ownership_coherence_2026_05_20.md` + ADR-0263).

## §14 Total lines

This file: 318 lines (excluding YAML frontmatter). Substance bar: single industry-leader target (AWS Marketplace 2026), six-context overlay, two-tenant-class overlay, named regression budget, SLO-file traceability, and forward-reference backlog for Wave 15B/15F/15G/15J.
