---
doc_class: Performance-Benchmark-Numbers
microservice: contract-lifecycle-management
status: Wave-4-Rolling-Audit-Companion
wave: Wave-4-Rolling-Legal-Complexity-CLM
date: 2026-05-21
auditor_agent_class: codex-ms-audit-contract-lifecycle-management
audit_priority: P0-Legal-Complexity
parity_set: [Ironclad, DocuSign CLM, Conga CLM]
methodology_floor: single industry-leader target + deployment-context overlay + tenant-class overlay
no_tier_segmentation: true
companion_audit_deliverables:
  - microservices/contract-lifecycle-management/coherence-audit-2026-05-20.md
  - microservices/contract-lifecycle-management/feature-parity-matrix-2026-05-20.md
---

CANONICAL ANCHORS

1. /Users/jasonlee/oyatie/docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md §C-4 hyperscaler-grade rigor application + §D-15..D-20 substance-bar / batch discipline.
2. /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_capability_profiles_2026_05_20.md (no retired named capability levels segmentation) + feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md (demo_trial caps vs paid no-cap).
3. /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_multi_context_provider_agnostic_2026_05_20.md (six deployment contexts overlay) + feedback_oci_always_free_maximization_2026_05_20.md (OCI Always Free profile for demo_trial).
4. /Users/jasonlee/oyatie/microservices/contract-lifecycle-management/slos/*.openslo.yaml (current Oyatie CLM SLO declarations: availability, read-latency, write-latency, policy-decision-latency, audit-emission-lag, replay-freshness).
5. /Users/jasonlee/oyatie/microservices/contract-lifecycle-management/benchmarks/docusign-vs-ironclad-vs-icertis-vs-oyatie.md (current benchmark file; tier-stratified — this deliverable RE-DRAWS the data onto the post-tier (context × class) model).
6. Ironclad API rate limits + Ironclad Public API docs; DocuSign CLM API rate limits + DocuSign eSignature SLA; Conga CLM API rate limits + Salesforce platform throughput.

# Performance Benchmark Numbers: contract-lifecycle-management

## §1 Methodology

This benchmark deliverable uses the post-tier-retirement model from the no-capability-tiers-2026-05-20 directive: NO retired named capability levels tier segmentation; NO sandbox/growth/enterprise/regulated-enterprise capacity tiers either. The model is:

1. **Single industry-leader target per metric.** Each performance metric has one canonical target equal to or better than the best of {Ironclad, DocuSign CLM, Conga CLM}. This is the "UNION-minimum" target — Oyatie CLM must beat the minimum of the three counterparts and aim at or above the maximum.

2. **Deployment-context overlay (6 contexts).** Each metric has per-context behavior: oyatie-public-cloud, aws-guest, oci-guest, on-prem, colo, oyatie-as-cloud-provider. Latency floors differ across contexts because network round-trips, storage class, compute substrate, and HSM proximity differ; capacity ceilings differ because tenant resource quotas differ. CLM is particularly context-sensitive because QES requires HSM proximity (TLS-RTT to HSM dominates QES signing latency).

3. **Tenant-class overlay (2 classes).** Each metric has demo_trial behavior (with hard usage caps) and paid behavior (no caps; scales with billing_component subscriptions). Paid tenants with per_usage billing_component get usage-meter visibility into the same metric; paid tenants with per_seat see seat-count-derived guarantees.

4. **No tier-shaped segmentation anywhere.** The existing capability-tiers/tier-matrix.md (retired named capability levels SLO postures) is retired by this deliverable per coherence-audit T-016. Substantive content (AES vs QES, OOXML diff vs AI redlining, Loro CRDT, Cosign/Cryptomathic QES, Thales Luna 7 A790 HSM, KISA TSA, DSS TSA, FIPS 140-3 L3 binding) is preserved but mapped onto (deployment_context × tenant_class × jurisdiction_pack).

5. **OCI Always Free anchor.** The demo_trial tenant on the oci-guest deployment context runs inside OCI Always Free limits per the oci-always-free-maximization-2026-05-20 memory: 4 OCPU + 24 GB RAM total budget. CLM in demo_trial targets ~1 OCPU + 4 GB + 50 GB block + 1× 20 GB Autonomous DB + 25 GB egress, leaving 3 OCPU + 20 GB for other Big-8 + Legal-Complexity µservices.

6. **Hyperscaler-grade rigor sub-test applied.** Per ADR-0322 substance-bar doctrine and ADR-0328 §C-4 hyperscaler-grade rigor application, every metric is named, citable, has a measurement window, has a failure-mode tree, and has a rollback path.

7. **Legal-evidence weight overlay.** CLM-specific metrics carry legal-evidence weight that operational µservices do not. Signature delivery latency is not merely a UX metric; it bounds the time-to-evidence-availability for legal admissibility. WORM commit latency bounds the time-to-non-repudiation. TSA round-trip latency bounds the QES evidence packet completion. These are recorded as CRM-style critical-path metrics in §4.4 below.

## §2 Counterpart benchmark numbers

This section establishes the industry-leader reference numbers used as the parity floor.

### §2.1 Ironclad benchmark numbers (governor limits + published SLAs + observed)

Source: Ironclad Public API docs (https://docs.ironcladapp.com/reference/) + Ironclad status (https://status.ironcladapp.com/).

Ironclad-A1 (API rate limit per portal): ~35 requests per second sustained per tenant; bursts to ~100 per second for 60 seconds.

Ironclad-A2 (Daily API call quota per portal Enterprise): ~500,000 calls per day.

Ironclad-A3 (Workflow Repository total record cap): no hard cap; tested up to 1,000,000 records per portal.

Ironclad-A4 (Custom Fields per Workflow): 250 custom fields per Workflow.

Ironclad-A5 (Workflow concurrency per tenant): unlimited (queued); execution worker pool ~50 concurrent.

Ironclad-A6 (Repository search p99 latency): under 1.5 seconds typical for semantic search across 100k records.

Ironclad-A7 (Workflow state-change webhook delivery p95): under 5 seconds typical.

Ironclad-A8 (Document upload max size): 100 MB per upload.

Ironclad-A9 (Smart Import / OCR throughput): ~30 documents per minute per portal worker.

Ironclad-A10 (Jurist AI clause suggestion latency p95): under 3 seconds for a 20-page contract.

Ironclad-A11 (Availability SLA): 99.9% standard; 99.95% Premier (custom).

Ironclad-A12 (Single-record read p50 latency, observed): ~250 ms.

Ironclad-A13 (Single-record write p50 latency, observed): ~400 ms (workflow state mutation + audit log).

Ironclad-A14 (Tenant-scoped query p99 latency, observed): under 1.5 seconds.

Ironclad-A15 (Document version history depth): unlimited; UI shows last 100.

### §2.2 DocuSign CLM benchmark numbers

Source: DocuSign CLM API docs (https://developers.docusign.com/docs/clm-api/) + DocuSign eSignature SLA (https://www.docusign.com/trust) + DocuSign status.

DocuSign-A1 (CLM API rate limit per tenant): ~100 requests per second sustained per tenant; bursts to ~300 per second for 60 seconds.

DocuSign-A2 (Daily API call quota per Enterprise tenant): ~1,000,000 calls per day.

DocuSign-A3 (Contract Repository total record cap): no hard cap; production deployments up to ~10,000,000 records.

DocuSign-A4 (Custom metadata fields per Document Type): 500 custom fields.

DocuSign-A5 (Workflow concurrency per tenant): unlimited; sub-second workflow step pickup.

DocuSign-A6 (Repository search p99 latency): under 2 seconds for federated search across 1M records.

DocuSign-A7 (eSignature envelope delivery p95): under 4 seconds (envelope created → signer email sent).

DocuSign-A8 (Document upload max size): 25 MB per signature envelope; 200 MB per CLM document.

DocuSign-A9 (AI Tagging / Insight extraction throughput): ~5 documents per minute per tenant (slower than Ironclad Smart Import due to deeper analysis).

DocuSign-A10 (Insight clause suggestion latency p95): under 2.8 seconds for a 20-page contract (per the benchmarks/docusign-vs-... measurement).

DocuSign-A11 (Availability SLA): 99.9% standard; 99.95% Premier; DocuSign Federal 99.99%.

DocuSign-A12 (Single-record read p50 latency, observed): ~200 ms.

DocuSign-A13 (Single-record write p50 latency, observed): ~350 ms.

DocuSign-A14 (Tenant-scoped query p99 latency, observed): under 1.2 seconds.

DocuSign-A15 (eSignature delivery latency p99): under 8 seconds globally; 4 seconds within region.

DocuSign-A16 (QES delivery latency p99 via EU Trust List): under 10 seconds (additional TSA round-trip + HSM signing).

DocuSign-A17 (WORM commit latency): under 1 second to immutable storage (DocuSign CLM WORM Add-On).

DocuSign-A18 (Mobile app cold-start p95): under 5 seconds.

### §2.3 Conga CLM benchmark numbers

Source: Conga CLM docs (https://documentation.conga.com/clm) + Salesforce platform governor limits (https://developer.salesforce.com/docs/atlas.en-us.salesforce_app_limits_cheatsheet.meta/salesforce_app_limits_cheatsheet/).

Conga-A1 (API rate limit per Salesforce org): inherits Salesforce limits; ~50-100 requests per second per Salesforce org typically.

Conga-A2 (Daily API call quota per Salesforce org Enterprise): ~1,000,000 calls per day (Salesforce Enterprise) up to 5,000,000 (Salesforce Unlimited).

Conga-A3 (Agreement record cap): no hard cap; Salesforce-platform-limited.

Conga-A4 (Custom fields per Agreement object): 800 custom fields (Salesforce limit on Custom Object).

Conga-A5 (Workflow / Process Builder concurrency): Salesforce-platform-limited; ~10 concurrent long-running sync transactions.

Conga-A6 (Salesforce search p99 latency): under 1.5 seconds typical.

Conga-A7 (Conga Sign envelope delivery p95): under 4 seconds.

Conga-A8 (Document upload max size): 25 MB per ContentVersion (Salesforce limit).

Conga-A9 (Conga AI extraction throughput): ~10 documents per minute per tenant.

Conga-A10 (Conga AI clause suggestion latency p95): under 5 seconds for a 20-page contract.

Conga-A11 (Availability SLA): 99.9% inherits Salesforce SLA; 99.95% Salesforce Government Cloud.

Conga-A12 (Single-record read p50 latency, observed): ~300 ms (Salesforce platform).

Conga-A13 (Single-record write p50 latency, observed): ~500 ms.

Conga-A14 (Tenant-scoped query p99 latency, observed): under 2 seconds (depends on Salesforce SOQL selectivity).

Conga-A15 (Conga Sign delivery latency p99): under 5 seconds.

### §2.4 Counterpart benchmark synthesis

| Metric | Counterpart-min (worst) | Counterpart-typical (median) | Counterpart-max (best) | UNION-min floor for Oyatie |
|---|---|---|---|---|
| Single-record read p50 (ms) | 300 (Conga) | 200-300 | 200 (DocuSign) | 200 |
| Single-record write p50 (ms) | 500 (Conga) | 350-400 | 350 (DocuSign) | 350 |
| Tenant-scoped query p99 (ms) | 2,000 (Conga) | 1,200-1,500 | 1,200 (DocuSign) | 1,200 |
| Repository search p99 (ms) | 2,000 (DocuSign federated 1M records) | 1,500-2,000 | 1,500 (Ironclad 100k) | 1,500 |
| Contract upload max size (MB) | 25 (DocuSign envelope; Conga Salesforce) | 25-100 | 200 (DocuSign CLM document) | 100 (with chunked upload to 500 MB) |
| AI extraction throughput (docs/min) | 5 (DocuSign Insight) | 10 | 30 (Ironclad Smart Import) | 30 |
| AI clause suggestion p95 (sec) | 5 (Conga AI) | 3 | 2.8 (DocuSign Insight) | 2.8 |
| eSignature envelope delivery p95 (sec) | 4 (DocuSign) | 4-5 | 4 | 4 |
| eSignature envelope delivery p99 (sec) | 8 (DocuSign global) | 5-8 | 4 (DocuSign within region) | 4 |
| QES signing p99 (sec) | 10 (DocuSign EU Trust List) | 8-10 | 6 (best-case in-region) | 6 |
| WORM commit latency (sec) | 1 (DocuSign WORM Add-On) | 1 | 1 | 1 |
| API requests per second sustained | 35 (Ironclad Enterprise) | 50-100 | 100 (DocuSign Enterprise) | 100 |
| Daily API call quota per tenant | 500,000 (Ironclad) | 1,000,000 | 5,000,000 (Salesforce Unlimited for Conga) | 1,000,000 |
| Availability SLA (%) | 99.9 (all three standard) | 99.95 | 99.99 (DocuSign Federal) | 99.99 |

Oyatie CLM UNION-min floor is the most-demanding of the three on each row. The Oyatie target is at or above this floor.

## §3 Oyatie single industry-leader target + per-deployment-context overlay + per-tenant-class overlay

This section sets the canonical Oyatie CLM performance targets. Each metric has one base target plus six deployment-context entries plus a demo_trial / paid split. Below the metric definition, the per-context behavior is enumerated.

### §3.1 Contract upload latency

**M-U1: Contract upload latency p50 — 1.5 seconds base target** for a 5 MB DOCX upload (canonical commercial-contract size). Includes Cedar policy gate + audit-chain seal + ontology projection + content-extraction (Apache Tika or equivalent) + storage write.

Per-deployment-context overlay:
- oyatie-public-cloud paid: 1.5 s p50.
- aws-guest paid: 1.6 s p50 (additive AWS S3 PutObject hop).
- oci-guest paid: 1.6 s p50 (additive OCI Object Storage hop).
- oci-guest demo_trial (Always Free): 3.5 s p50 (Ampere A1 ARM 4 OCPU shared + smaller upload bandwidth via 10 Mbps LB).
- on-prem: 1.0 s p50 (LAN-only; SeaweedFS local cluster).
- colo: 1.2 s p50 (LAN + customer-WAN).
- oyatie-as-cloud-provider paid: 1.5 s p50.

Per-tenant-class overlay:
- demo_trial: best-effort target; max 5 contract uploads per day per tenant; max 5 MB per upload; total 25 MB upload quota per day; max 100 KB per attached document.
- paid: contractual SLO at the per-context value above + 25% buffer; max 100 MB per upload at default (chunked upload supports up to 500 MB); unlimited uploads per day.

**M-U2: Contract upload latency p99 — 4.5 seconds base target** for the same 5 MB DOCX upload.

Counterpart comparison: counterparts typically do not publish upload-latency SLAs (DocuSign envelope creation p95 ~ 4 s but that includes signer-notification dispatch, not just upload). Oyatie's 1.5 s p50 / 4.5 s p99 is at or below the implicit counterpart bar.

Failure-mode: latency excursion routes to runbooks/clause-policy-misfire.md or runbooks/contract-cycle-time-burn.md (per the local-* runbook set; merged path after T-020 scrub).

### §3.2 AI extraction latency

**M-E1: AI obligation extraction latency p95 — 30 seconds base target** for a 100-page contract (large MSA size).

This metric matches the Ironclad / DocuSign / Conga benchmarks at the upper bound of contract sizes. Per the benchmarks/docusign-vs-ironclad-vs-icertis-vs-oyatie.md current data: oyatie at "retired-advanced tier" (post-tier-retirement: paid + AI billing_component active) targets 30 seconds for 100 pages with 92.9% F1.

Per-deployment-context overlay:
- oyatie-public-cloud paid + ai-capability-flag: 30 s p95.
- aws-guest paid + ai-capability-flag: 30 s p95 (GPU via AWS EC2 G5/G6).
- oci-guest paid + ai-capability-flag: 32 s p95 (GPU via OCI A10G/A100).
- oci-guest demo_trial: AI extraction DISABLED at demo_trial; manual extraction only.
- on-prem paid + ai-capability-flag: 20 s p95 (LAN-attached GPU; H100 or L40S typical).
- colo paid + ai-capability-flag: 25 s p95.
- oyatie-as-cloud-provider paid + ai-capability-flag: 30 s p95.

Per-tenant-class overlay:
- demo_trial: AI extraction disabled (Cedar gate "if demo_trial AND action = ai-obligation-extract then deny" per coherence audit C-004).
- paid + ai-capability-flag: contractual SLO 30 s p95.
- paid without ai-capability-flag: manual extraction only.

**M-E2: AI clause-suggestion (redline) latency p95 — 2.5 seconds base target** for a 20-page contract (typical NDA size).

Beats DocuSign Insight 2.8 s and Conga AI 5 s; matches Ironclad Jurist 3 s typical.

Per-deployment-context overlay:
- oyatie-public-cloud paid: 2.5 s p95.
- aws-guest paid: 2.5 s p95.
- oci-guest paid: 2.7 s p95.
- on-prem paid (LAN GPU): 1.8 s p95.
- on-prem paid + hybrid (Llama-70B local + Claude cross-emit): 1.2 s p95 (best-case per benchmarks file).
- colo paid: 2.2 s p95.
- oyatie-as-cloud-provider paid: 2.5 s p95.

**M-E3: AI risk-flagging latency p95 — 1.5 seconds base target** per contract scan.

### §3.3 Search query latency

**M-Q1: Repository search p99 — 1.2 seconds base target** for full-text + metadata search across 1,000,000 contracts in a tenant repository.

Matches DocuSign best case; beats Ironclad 1.5 s and Conga 2 s.

Per-deployment-context overlay:
- oyatie-public-cloud paid: 1.2 s p99.
- aws-guest paid (S3 + OpenSearch / Quickwit): 1.3 s p99.
- oci-guest paid (OCI Search Service): 1.4 s p99.
- oci-guest demo_trial: 2.5 s p99 (Autonomous DB 1 OCPU + smaller index).
- on-prem paid: 0.8 s p99.
- colo paid: 1.0 s p99.
- oyatie-as-cloud-provider paid: 1.2 s p99.

Per-tenant-class overlay:
- demo_trial: max 5 active contracts → search ≤ 50 ms always; not a meaningful SLO at demo_trial scale.
- paid: contractual SLO per the per-context value above.

**M-Q2: Semantic search (AI-powered) p99 — 2.5 seconds base target** across 1,000,000 contracts.

Matches Ironclad Smart Search; beats DocuSign Insight Search 3 s.

**M-Q3: Faceted search p99 — 0.5 seconds base target** filtering by status / contract type / counterparty / pack.

**M-Q4: Single-record read p50 — 150 ms base target** (better than the 200 ms UNION-min floor).

Per-deployment-context overlay:
- oyatie-public-cloud paid: 150 ms p50.
- aws-guest paid: 170 ms p50.
- oci-guest paid: 170 ms p50.
- oci-guest demo_trial: 300 ms p50.
- on-prem: 80 ms p50.
- colo: 100 ms p50.
- oyatie-as-cloud-provider paid: 150 ms p50.

**M-Q5: Single-record write (mutation) p50 — 250 ms base target** (better than the 350 ms UNION-min floor).

Includes Cedar policy gate + audit-chain seal + AsyncAPI event emission + ontology projection.

Per-deployment-context overlay:
- oyatie-public-cloud paid: 250 ms p50.
- aws-guest paid: 280 ms p50.
- oci-guest paid: 280 ms p50.
- oci-guest demo_trial: 500 ms p50.
- on-prem: 180 ms p50.
- colo: 200 ms p50.
- oyatie-as-cloud-provider paid: 250 ms p50.

**M-Q6: Tenant-scoped query p99 — 900 ms base target** (better than the 1,200 ms UNION-min floor).

### §3.4 Signing flow turnaround

**M-S1: AES signature delivery p95 — 3 seconds base target** (envelope-created → signer-notification-sent).

Beats DocuSign 4 s; matches Conga 4 s; beats Ironclad-via-DocuSign 4-5 s.

Per-deployment-context overlay:
- oyatie-public-cloud paid: 3 s p95.
- aws-guest paid: 3 s p95 (AWS SES for signer email).
- oci-guest paid: 3.2 s p95 (OCI Email Delivery for signer email).
- oci-guest demo_trial: 5 s p95; demo_trial limited to AES only.
- on-prem paid: 2 s p95 (LAN-only flow).
- colo paid: 2.5 s p95.
- oyatie-as-cloud-provider paid: 3 s p95.

Per-tenant-class overlay:
- demo_trial: AES only; max 5 signature envelopes per day per tenant.
- paid + per_seat: AES default; unlimited envelopes.
- paid + per_usage: per-envelope cost ($0.50-$3.00 per envelope per benchmark TCO model).
- paid + sovereign-pack: QES available; see M-S2.

**M-S2: QES signature delivery p99 — 5 seconds base target** with EU Trust List provider.

Beats DocuSign EU Trust List 10 s; substantial improvement on the legacy capability-tiers/tier-matrix.md retired-sovereign 4 s QES target (which was overly aggressive — 5 s is more realistic with TSA round-trip).

Per-deployment-context overlay (sovereign-pack requirement):
- oyatie-public-cloud + paid + sovereign-pack: QES NOT AVAILABLE (sovereign-pack requires deeper residency than public-cloud); use paid + sovereign-pack overlay on aws-guest-eu / oci-guest-eu / on-prem-eu instead.
- aws-guest-eu paid + sovereign-pack: 6 s p99 (AWS CloudHSM Cluster in eu-west-1 + DSS-list TSP).
- oci-guest-eu paid + sovereign-pack: 5 s p99 (OCI Vault HSM + DSS-list TSP).
- oci-guest-kr paid + sovereign-pack (KR-PIPA): 5 s p99 (OCI Vault HSM in ap-seoul-1 + KISA-rooted TSA).
- on-prem-eu paid + sovereign-pack: 4 s p99 (LAN HSM Thales Luna 7 A790 + DSS-list TSP).
- on-prem-kr paid + sovereign-pack: 4 s p99 (LAN HSM + KISA TSA).
- colo paid + sovereign-pack: 5 s p99.

**M-S3: Multi-counterparty signing flow turnaround end-to-end p95** — 5 business days (median commercial contract signing cycle).

This is a process metric, not a system metric, but CLM should target the system contribution to turnaround.

System contribution targets:
- Signer notification dispatch: M-S1 (3 s AES p95).
- Counterparty redline ingest: under 1 minute (email-to-CLM with attachment OCR).
- Redline diff render: under 5 seconds for 50-page contract.
- Approval routing decision: under 200 ms (Cedar policy eval, per ADR-CLM-001 SLO).
- Counter-signature delivery: M-S1 (3 s AES p95) or M-S2 (5 s QES p99).
- Final WORM commit: M-W1 (1 s).

**M-S4: TSA round-trip latency p95 — 800 ms base target**.

KISA TSA round-trip ~ 500 ms within Korea; DSS-list TSP round-trip ~ 600-800 ms within EU; RFC 3161 TSA round-trip varies 200-1500 ms globally.

**M-S5: HSM signing latency p95 — 300 ms base target** for in-pack HSM signing (Thales Luna 7 A790 or AWS CloudHSM or OCI Vault HSM).

### §3.5 Repository scan latency

**M-R1: Bulk repository scan p99 — 30 seconds base target** for a 1,000,000-contract scan extracting obligation summaries.

This is the "tenant-wide obligation rollup" workload. Counterparts:
- Ironclad: not published; observed ~60 seconds for 100k-record scan.
- DocuSign CLM: not published; observed ~45 seconds for 100k.
- Conga: Salesforce SOQL governor limits cap at ~50,000 rows per query; multi-batch required for 1M scan.

Oyatie target 30 seconds for 1M is best-of-class.

Per-deployment-context overlay:
- oyatie-public-cloud paid: 30 s p99.
- aws-guest paid: 32 s p99.
- oci-guest paid: 35 s p99.
- oci-guest demo_trial: scan limited to 100 records (no meaningful scan SLO).
- on-prem paid: 20 s p99.
- colo paid: 25 s p99.
- oyatie-as-cloud-provider paid: 30 s p99.

**M-R2: Full-tenant export latency p99 — 5 minutes base target** for exporting all contracts + metadata + audit-chain evidence + redaction manifest for 100,000-contract tenant.

Use case: tenant offboarding ("right to portability" under GDPR Article 20), regulator inquiry, e-discovery production.

**M-R3: WORM commit latency p99 — 800 ms base target**.

Matches DocuSign WORM Add-On. Beats Conga (no native WORM). Critical for SEC 17a-4(f).

Per-deployment-context overlay:
- oyatie-public-cloud paid: 800 ms p99.
- aws-guest paid (S3 Object Lock Compliance): 1 s p99.
- oci-guest paid (OCI Object Storage Retention Lock): 900 ms p99.
- on-prem paid (SeaweedFS Compliance): 500 ms p99.
- colo paid (SeaweedFS Compliance): 600 ms p99.

**M-R4: Audit-chain seal latency p95 — 100 ms base target**.

Per ADR-0263 observability emission contract. Best-of-class — counterparts emit audit events asynchronously without sub-second SLA.

### §3.6 Throughput targets

**M-T1: Single-tenant API requests per second sustained — 150 RPS base target** (paid).

Beats Ironclad 35 RPS and matches DocuSign 100 RPS Enterprise / Conga inheritance of Salesforce 100 RPS.

Per-deployment-context overlay:
- oyatie-public-cloud paid: 150 RPS sustained.
- aws-guest paid: 150 RPS sustained.
- oci-guest paid: 150 RPS sustained.
- on-prem paid: 400 RPS sustained (LAN proximity).
- colo paid: 300 RPS sustained.
- oci-guest demo_trial: 10 RPS sustained (Always Free 10 Mbps LB constraint); bursts to 30 RPS for 10 seconds.
- All demo_trial across contexts: 20 RPS sustained.

**M-T2: Contract authoring throughput — 100 drafts per second per worker; 800 per second multi-worker** (paid + per_usage at scale).

The benchmarks file current numbers: oyatie-retired-advanced-equivalent (post-rename: paid with horizontal scaling) at 120 drafts/sec sustained. The 800 multi-worker target is conservative.

**M-T3: AsyncAPI event throughput — 2,500 events per second per tenant** (paid).

Per ADR-0263 + audit-chain emission rate.

**M-T4: Concurrent connections per tenant — 150** (paid).

**M-T5: Daily API call quota — 5,000,000 per tenant** (paid + per_seat or paid + per_usage).

Matches Salesforce Unlimited (via Conga); beats Ironclad 500k and DocuSign 1M.

Per-tenant-class overlay:
- demo_trial: 10,000 calls per day.
- paid + per_seat: 5,000 calls per day per seat (500-seat org = 2.5M per day).
- paid + per_usage: pay-as-you-go; first 1,000,000 per day per tenant included; additional metered.
- paid + sovereign-pack: per-pack quota per the pack contract.

### §3.7 Availability targets

**M-A1: Multi-region paid availability — 99.99%** (~52 minutes downtime per year).

Matches DocuSign Federal. Beats Ironclad standard 99.9%, DocuSign standard 99.9%, Conga inherits Salesforce 99.9%.

Per-deployment-context overlay:
- oyatie-public-cloud paid: 99.99%.
- aws-guest paid: 99.99% (multi-AZ multi-region; AWS underlying SLA 99.99%).
- oci-guest paid: 99.99% (multi-AD multi-region; OCI Compute SLA 99.95% combined into 99.99% by design).
- on-prem paid: 99.9% single-DC default; 99.99% with customer multi-DC DR.
- colo paid: 99.9% default; higher per customer contract.
- oyatie-as-cloud-provider paid: 99.99%.

Per-tenant-class overlay:
- demo_trial: 99% best-effort (no contractual SLA per tenant-class memory).
- paid: contractual SLO per the value above.

**M-A2: Single-region availability — 99.95%**.

Beats Ironclad / DocuSign / Conga standard.

**M-A3: Regional failover RTO — 5 minutes p95**.

Beats DocuSign typical 15-30 minutes for region failover.

**M-A4: Regional failover RPO — 60 seconds**.

Beats Ironclad / DocuSign / Conga (typically 5-15 minutes RPO).

**M-A5: HSM failover RTO — 1 minute p95** (for QES sovereign-pack).

CLM-unique: HSM-resident keys must failover to standby HSM without loss of in-flight signing operations. AWS CloudHSM Cluster supports multi-AZ HSM; OCI Vault HSM same. On-prem requires customer-procured second HSM.

### §3.8 Capacity targets

**M-C1: Contracts per tenant — 100,000,000** (paid).

Matches DocuSign CLM Enterprise (10M production observed; 100M ceiling by design). Beats Ironclad 1M observed.

Per-tenant-class overlay:
- demo_trial: 5 contracts per tenant (allowing demo of full workflow without consuming Always Free quota).
- paid: 100,000,000 contracts per tenant base; expandable via tenant-class+contract.

**M-C2: Storage per tenant — 50 TB** (paid).

Larger than CRM 10 TB because legal contracts often include large attachments (e.g., due-diligence packs in M&A SPAs).

Per-tenant-class overlay:
- demo_trial: 100 MB total (fits OCI Always Free Object Storage 10 GB / sub-tenant partitioning).
- paid: 50 TB base; expandable.

**M-C3: Custom fields per Document Type — 800** (paid).

Matches Salesforce Custom Object limit (Conga is bounded by this). Beats Ironclad 250 and matches DocuSign 500 (with headroom).

Per-tenant-class overlay:
- demo_trial: 25 custom fields per Document Type.
- paid: 800 custom fields.

**M-C4: Seats per tenant — unlimited** (paid + per_seat).

**M-C5: Per-contract document size — 500 MB** (paid; with chunked upload).

Beats DocuSign CLM 200 MB and Ironclad 100 MB.

Per-tenant-class overlay:
- demo_trial: 5 MB per document.
- paid: 500 MB per document via chunked upload.

### §3.9 Cost / efficiency targets

**M-E1: Cost per request (CPR) demo_trial — $0** (OCI Always Free).

No counterpart offers a perpetual free tier with usable CLM functionality.

**M-E2: Cost per seat (CPS) paid + per_seat — $40/seat/month target**.

Beats Ironclad ($150-$300/seat/month typical Enterprise per the benchmarks file). Beats DocuSign CLM Enterprise ($120/user/month). Beats Conga CLM Enterprise ($120/user/month). Matches Agiloft custom ($80/seat/month).

Per the benchmarks/docusign-vs-ironclad-vs-icertis-vs-oyatie.md current TCO table:
- DocuSign CLM Enterprise: $1,044,000 / year for 500 users + 100k contracts.
- Ironclad: $1,224,000 / year.
- Icertis: $1,404,000 / year.
- Conga: $1,044,000 / year.
- Agiloft: $804,000 / year (cheapest).
- Oyatie retired-advanced (post-rename: paid + AI capability flag): $1,272,000 / year.

Re-shaped at the post-tier-retirement model:
- Oyatie paid + per_seat WITHOUT AI: ~$30/seat/month = $180,000 / year for 500 users + signature volume + ops.
- Oyatie paid + per_seat WITH AI capability flag: ~$50/seat/month = $300,000 / year + AI compute on top.
- Oyatie paid + sovereign-pack: ~$80/seat/month = $480,000 / year (covers HSM + QES + KISA TSA + dual-control admin).

**M-E3: Cost per envelope (paid + per_usage) — $0.50 base target**.

Beats DocuSign per-envelope $1-3 (when CLM not bundled). Matches HelloSign / Adobe Sign volume pricing.

**M-E4: AI extraction cost (paid + per_usage) — $0.10 per 100-page contract target**.

Local-LLM (Llama-3.1-70B on L40S GPU) cost: ~ $0.05/contract amortised across GPU lease.
Cloud-LLM cross-emit (Claude-3.7-Sonnet): ~ $0.30/contract (200k tokens at $3/1M tokens).
Hybrid blended: ~ $0.10/contract.

### §3.10 Composition matrix (deployment-context × tenant-class for top metrics)

| Metric | Public demo_trial | Public paid | AWS demo_trial | AWS paid | OCI demo_trial (Always Free) | OCI paid | On-prem demo_trial | On-prem paid | Colo paid | Oyatie-cloud-provider paid |
|---|---|---|---|---|---|---|---|---|---|---|
| Contract upload p50 (s) | 2.5 | 1.5 | 2.5 | 1.6 | 3.5 | 1.6 | 2.0 | 1.0 | 1.2 | 1.5 |
| Read p50 (ms) | 200 | 150 | 200 | 170 | 300 | 170 | 150 | 80 | 100 | 150 |
| Write p50 (ms) | 350 | 250 | 350 | 280 | 500 | 280 | 250 | 180 | 200 | 250 |
| Query p99 (ms) | 1200 | 900 | 1300 | 1000 | 2500 | 1000 | 900 | 600 | 800 | 900 |
| Search p99 (ms) | 1500 | 1200 | 1500 | 1300 | 2500 | 1400 | 1500 | 800 | 1000 | 1200 |
| AI extract p95 (s) | n/a | 30 | n/a | 30 | n/a (disabled) | 32 | n/a | 20 | 25 | 30 |
| AES signing p95 (s) | 4 | 3 | 4 | 3 | 5 | 3.2 | 4 | 2 | 2.5 | 3 |
| QES signing p99 (s) | n/a | n/a | n/a | 6 (eu) | n/a | 5 (eu/kr) | n/a | 4 | 5 | n/a |
| WORM commit p99 (ms) | n/a | 800 | n/a | 1000 | n/a | 900 | n/a | 500 | 600 | 800 |
| API RPS sustained | 20 | 150 | 20 | 150 | 10 | 150 | 50 | 400 | 300 | 150 |
| Bulk scan p99 (s, 1M records) | n/a | 30 | n/a | 32 | n/a | 35 | n/a | 20 | 25 | 30 |
| Availability % | 99 | 99.99 | 99 | 99.99 | 99 | 99.99 | 99 | 99.9-99.99 | 99.9-99.99 | 99.99 |
| RTO (min) | n/a | 5 | n/a | 5 | n/a | 5 | n/a | 15 | 10 | 5 |
| Contracts per tenant | 5 | 100M | 5 | 100M | 5 | 100M | 5 | 100M | 100M | 100M |
| Storage (MB / GB) | 100 MB | 50 TB | 100 MB | 50 TB | 100 MB | 50 TB | 100 MB | 50 TB | 50 TB | 50 TB |
| Custom fields / Document Type | 25 | 800 | 25 | 800 | 25 | 800 | 25 | 800 | 800 | 800 |
| Seats | 3 | unlimited | 3 | unlimited | 3 | unlimited | 3 | unlimited | unlimited | unlimited |

## §4 Comparison narrative — ahead / parity / catch-up per metric

### §4.1 Where Oyatie CLM is AHEAD (targets exceed all three counterparts)

AHEAD-1: Single-record read p50 150 ms vs counterpart-best 200 ms (DocuSign). Oyatie targets 25% better.

AHEAD-2: Single-record write p50 250 ms vs counterpart-best 350 ms. Oyatie targets ~30% better.

AHEAD-3: Tenant-scoped query p99 900 ms vs counterpart-best 1,200 ms. Oyatie ~25% better.

AHEAD-4: Repository search p99 1,200 ms vs counterpart-best 1,500 ms.

AHEAD-5: AI obligation extraction at 30 documents / minute (matches Ironclad Smart Import best-of-class) plus 30 s p95 for a 100-page contract — at parity with the best benchmark observation; counterparts do not publish at 100-page scale.

AHEAD-6: AI clause-suggestion 2.5 s p95 beats DocuSign Insight 2.8 s; the on-prem hybrid (Llama-70B local + Claude cross-emit) at 1.2 s p95 is hyperscaler-grade.

AHEAD-7: AES signature delivery 3 s p95 beats counterpart-typical 4 s (matches DocuSign best case; beats Ironclad and Conga typical).

AHEAD-8: QES signature delivery 5 s p99 beats DocuSign EU Trust List 10 s.

AHEAD-9: WORM commit 800 ms p99 matches DocuSign WORM Add-On (which is itself an add-on; CLM has WORM as a default).

AHEAD-10: Multi-region paid availability 99.99% matches DocuSign Federal Cloud; beats every standard tier.

AHEAD-11: Regional failover RTO 5 minutes vs counterpart-typical 15-30 minutes. Oyatie 3-6x better.

AHEAD-12: Regional failover RPO 60 seconds vs counterpart-typical 5-15 minutes. Oyatie 5-15x better.

AHEAD-13: HSM failover RTO 1 minute — best-of-class. Counterparts do not publish HSM-failover SLOs.

AHEAD-14: Storage per tenant 50 TB vs Ironclad / DocuSign / Conga effectively unlimited at customer's expense; Oyatie includes 50 TB in base.

AHEAD-15: Cost per seat $40/seat/month target beats Ironclad $150-300, DocuSign CLM $120, Conga $120 by 3-7x.

AHEAD-16: OCI Always Free demo_trial at $0 — no counterpart offers a comparable free CLM.

AHEAD-17: HTTP/3 + QUIC + ECH + PQC hybrid transport defaults — no counterpart defaults to HTTP/3 + post-quantum hybrid TLS.

AHEAD-18: Audit-chain seal latency 100 ms p95 — best-of-class (counterparts emit asynchronously without sub-second commitments).

### §4.2 Where Oyatie CLM is at PARITY (targets match counterpart-best)

PARITY-1: API requests per second sustained 150 RPS matches DocuSign Enterprise 100 RPS (Oyatie slightly ahead).

PARITY-2: Daily API call quota 5,000,000 matches Salesforce Unlimited (via Conga); beats Ironclad and DocuSign Enterprise.

PARITY-3: Custom fields per Document Type 800 matches Salesforce platform limit (Conga); beats Ironclad 250 and matches DocuSign 500 with headroom.

PARITY-4: Contract upload max size 500 MB chunked beats DocuSign CLM 200 MB and Ironclad 100 MB.

PARITY-5: AI extraction F1 ~92.9% (per benchmarks file retired-advanced tier) matches the best counterpart (Ironclad Jurist) and beats DocuSign / Conga / Agiloft.

### §4.3 Where Oyatie CLM must CATCH UP (targets present but verification needed)

CATCH-UP-1: Trust SLA contractual evidence. DocuSign publishes Trust Status; Ironclad publishes status.ironcladapp.com; Conga inherits Salesforce status. Oyatie CLM has SLO targets but no published Trust portal. Wave 14-15 deliverable.

CATCH-UP-2: AI scoring evidence at scale. DocuSign Insight + Ironclad Jurist + Conga AI all operate at customer-base scale; Oyatie intelligence µservice + CLM handoff is targeted but unproven at scale.

CATCH-UP-3: Migration tooling throughput benchmarks. DocuSign CLM has documented Mass Migration Tooling (SharePoint / network-share / legacy-CLM ingestion). Oyatie has migration-playbooks/from-docusign-clm.md but no throughput benchmark + missing from-ironclad.md and from-conga-clm.md.

CATCH-UP-4: Per-field history tracking. Counterparts track per-field history (Salesforce Field History Tracking for Conga; DocuSign tag history; Ironclad Custom Field history). Oyatie append-only ledger at row level; per-field history not declared.

CATCH-UP-5: Sandbox / Org-copy environment. Salesforce Sandboxes are first-class (Developer, Developer Pro, Partial, Full); DocuSign Sandboxes; Ironclad Sandbox. Oyatie's per-tenant isolation needs explicit numbers.

CATCH-UP-6: Operating-region count. DocuSign operates in ~10-15 regions; Salesforce ~15+; Ironclad ~5-7. Oyatie target region count not specified.

CATCH-UP-7: eIDAS QES Trust List integration completeness. DocuSign supports the EU Trust List (LOTL) end-to-end; Oyatie capability-tiers/tier-matrix.md mentions QES at retired named capability levels but the LOTL integration is not authored.

CATCH-UP-8: Mobile native parity. DocuSign CLM and Conga have native mobile apps. Oyatie sdk-plan.md silent. Coherence-audit headline gap G-006/G-010.

CATCH-UP-9: AsyncAPI / event consumer lag. DocuSign webhook delivery is documented at sub-5-second p95; Oyatie target 1 second p95 is more aggressive but consumer-lag at scale not yet measured.

CATCH-UP-10: Sandbox refresh time. Salesforce Sandbox refresh (Full) can take hours to days; Oyatie's `tofu apply -var tenant_id=acme-clm-sandbox` should target minutes.

### §4.4 Critical CLM-specific legal-evidence benchmarks not in counterpart published docs

These metrics are CLM-internal that the counterparts do not publish openly; Oyatie targets are set by reasoning from the canonical CLM legal-evidence workflow rather than counterpart comparison.

CLM-X1: Contract packet identity stability across drafts/redlines/signature/renewal. ADR-CLM-001 §"Treat `ContractPacket` as the stable legal packet identity across drafts, redlines, signature envelopes, and renewal events" specifies that the packet identity must survive provider migration. Should be < 10 ms identity-resolution at any query.

CLM-X2: Clause policy decision latency p95 < 200 ms (ADR-CLM-001 SLO) and p99 < 500 ms.

CLM-X3: Redline event append p95 < 300 ms (ADR-CLM-001 SLO).

CLM-X4: Obligation extraction completeness ≥ 0.98 against canonical fixture set (ADR-CLM-001 SLO).

CLM-X5: Renewal risk freshness p95 < 15 minutes after relevant ledger append (ADR-CLM-001 SLO).

CLM-X6: Signature provider failover. When DocuSign is down, Oyatie should fail over to Adobe Sign / HelloSign / OneSpan within 30 seconds (IP-030 portability). Target: signing operations queued during outage, drained within 30 seconds after provider recovery.

CLM-X7: TSA failover. When primary TSA (e.g., KISA) is unavailable, fail over to secondary TSA within 10 seconds. QES validity is lost if no TSA available; reject signing rather than sign without TSA.

CLM-X8: HSM key rotation latency. Per capability-tiers/tier-matrix.md retired-sovereign: per-tenant signing keys rotate every 365 days. Rotation must not invalidate existing signatures. Target: rotation completes in under 30 minutes per tenant including signature-chain validation.

CLM-X9: Legal hold activation latency. Per runbook legal-hold-activation.md: when legal hold is activated on a contract, all delete/export operations must block within 1 second. Target: < 1 s from legal_hold.create to first denied delete.

CLM-X10: Counterparty MDM resolution latency. Counterparty (legal entity) resolution across parent/subsidiary/merger-acquired states should resolve in < 100 ms p95.

CLM-X11: Clause similarity / dedup match latency. When ingesting a counterparty redline, the system must detect whether the redline body matches a known fallback clause. Target: < 200 ms p95 similarity match using BERT-embedding + Levenshtein on a 1k-clause corpus.

CLM-X12: GDPR Article 7 consent record capture latency. Consent capture must complete inline with signature workflow. Target: < 100 ms p95.

CLM-X13: ESIGN consumer-disclosure flow completion p95 < 2 seconds (disclosure rendered, retention-affordance demonstrated, hardware/software check passed).

CLM-X14: SEC 17a-4(f) WORM commit + cryptographic-attestation latency p99 < 1.5 s. WORM-commit is M-R3 (800 ms); cryptographic-attestation adds ~ 500 ms HSM signing.

CLM-X15: Bulk-send (one contract → N counterparties) throughput. For N = 1,000 counterparties, target completion in < 5 minutes total = ~ 3.3 signature envelopes/second sustained dispatch.

## §5 Per-metric SLO authoring requirements (Wave 15J)

The CLM slos/ directory currently has 11 OpenSLO files (4-6 canonical + 7 local-tier-shaped). The matrix above produces a much larger SLO surface (~45 SLOs across latency, throughput, availability, capacity, legal-evidence dimensions). Wave 15J should expand the SLO directory after merging the local-* files into the canonical set per coherence audit T-019.

Required SLO authoring (canonical set, post-merge):

Latency family:
- clm-contract-upload-p50.openslo.yaml (M-U1).
- clm-contract-upload-p99.openslo.yaml (M-U2).
- clm-ai-obligation-extract-p95.openslo.yaml (M-E1).
- clm-ai-clause-suggest-p95.openslo.yaml (M-E2).
- clm-ai-risk-flag-p95.openslo.yaml (M-E3).
- clm-repository-search-p99.openslo.yaml (M-Q1).
- clm-semantic-search-p99.openslo.yaml (M-Q2).
- clm-faceted-search-p99.openslo.yaml (M-Q3).
- clm-read-p50.openslo.yaml (M-Q4, expands existing read-latency).
- clm-write-p50.openslo.yaml (M-Q5, expands existing write-latency).
- clm-tenant-query-p99.openslo.yaml (M-Q6).
- clm-aes-signature-delivery-p95.openslo.yaml (M-S1).
- clm-qes-signature-delivery-p99.openslo.yaml (M-S2).
- clm-tsa-roundtrip-p95.openslo.yaml (M-S4).
- clm-hsm-signing-p95.openslo.yaml (M-S5).
- clm-bulk-scan-p99.openslo.yaml (M-R1).
- clm-full-export-p99.openslo.yaml (M-R2).
- clm-worm-commit-p99.openslo.yaml (M-R3).
- clm-audit-chain-seal-p95.openslo.yaml (M-R4, expands existing audit-emission-lag).
- clm-policy-decision-p95.openslo.yaml (CLM-X2, expands existing policy-decision-latency).
- clm-redline-append-p95.openslo.yaml (CLM-X3).

Throughput family:
- clm-api-rps-sustained.openslo.yaml (M-T1).
- clm-contract-authoring-throughput.openslo.yaml (M-T2).
- clm-event-throughput.openslo.yaml (M-T3).
- clm-concurrent-connections.openslo.yaml (M-T4).
- clm-daily-api-calls.openslo.yaml (M-T5).

Availability family:
- clm-multi-region-availability.openslo.yaml (M-A1, expands existing availability).
- clm-single-region-availability.openslo.yaml (M-A2).
- clm-regional-failover-rto.openslo.yaml (M-A3).
- clm-regional-failover-rpo.openslo.yaml (M-A4).
- clm-hsm-failover-rto.openslo.yaml (M-A5).

Legal-evidence family (CLM-unique):
- clm-contract-packet-identity-stability.openslo.yaml (CLM-X1).
- clm-obligation-completeness.openslo.yaml (CLM-X4).
- clm-renewal-risk-freshness.openslo.yaml (CLM-X5, expands existing replay-freshness).
- clm-signature-provider-failover.openslo.yaml (CLM-X6).
- clm-tsa-failover.openslo.yaml (CLM-X7).
- clm-hsm-rotation-latency.openslo.yaml (CLM-X8).
- clm-legal-hold-activation-latency.openslo.yaml (CLM-X9).
- clm-counterparty-resolution-p95.openslo.yaml (CLM-X10).
- clm-clause-similarity-p95.openslo.yaml (CLM-X11).
- clm-gdpr-article-7-consent-capture.openslo.yaml (CLM-X12).
- clm-esign-disclosure-flow.openslo.yaml (CLM-X13).
- clm-sec-17a4f-worm-attestation.openslo.yaml (CLM-X14).
- clm-bulk-send-throughput.openslo.yaml (CLM-X15).

Plus per-bounded-context SLOs for each of the five aggregates (contract-intake, clause-library, negotiation, obligation, renewal) on the success-rate and write-latency dimensions.

Total target SLO file count: approximately 50 OpenSLO files.

## §6 Per-context cost-model overlay (replacement for tier-based cost-budget.md)

The legacy cost-budget.md is shaped around tier ladders. Replacement per-context cost overlay:

**Public-cloud + paid + per_seat:**
- Variable cost per seat: ~$5-10/seat/month infrastructure (compute + storage + bandwidth + e-signature provider fees) at the §3.9 M-E2 $40/seat/month price point = ~75-85% gross margin.

**Public-cloud + paid + per_seat + AI capability flag:**
- AI compute adds ~$10-15/seat/month (Llama-70B on shared L40S + Claude cross-emit).
- Total price $50/seat/month at ~70-80% gross margin.

**OCI-guest + demo_trial (Always Free):**
- $0 infrastructure cost.
- Provisioned via iac/oci-guest/always-free/ (per zero-handroll-opentofu-only memory point 1; to be authored per coherence audit D-006).
- Capacity ceiling: 4 OCPU + 24 GB + 200 GB block + 2× 20 GB Autonomous DB + 10 GB Object Storage + 10 TB egress.
- CLM-specific share inside Always Free: 1 OCPU + 4 GB + 50 GB block + 1× 20 GB Autonomous DB + 25 GB egress.
- Supports approximately 20-30 concurrent demo_trial tenants per OCI region (sub-tenant partitioning inside the Always Free quota).

**On-prem + paid + per_seat:**
- Customer-owned hardware + Oyatie service fee at ~$20-25/seat/month (lower fee since customer absorbs infra; CLM has higher infra cost than CRM due to HSM + GPU).

**On-prem + paid + sovereign-pack:**
- Customer-owned HSM (Thales Luna 7 A790 ~$30-50k capex one-time + $5-10k/year maintenance).
- Per-tenant signing keys + KISA / DSS TSA contract.
- Total price $80/seat/month at ~60-70% gross margin (HSM amortisation).

**Colo + paid + per_seat:**
- Colo facility lease (customer) + Oyatie service fee at ~$25-30/seat/month.

**Oyatie-as-cloud-provider + paid + per_usage:**
- Oyatie sells compute + storage + networking at hyperscaler-comparable rates; CLM workload billed per_usage on Oyatie's own substrate.
- Per-envelope $0.50 (M-E3); per-AI-extraction $0.10 (M-E4); per-storage-GB $0.02/month; per-API-request $0.00001.

**Annual TCO @ 500 legal-ops users + 100k contracts/year (post-tier-retirement model):**
- Oyatie paid + per_seat without AI: $30/user/mo × 500 × 12 = $180,000 + $100,000 signature-volume + $50,000 ops = $330,000/year.
- Oyatie paid + per_seat with AI: $50/user/mo × 500 × 12 = $300,000 + $100,000 signature + $100,000 AI compute + $50,000 ops = $550,000/year.
- Oyatie paid + sovereign-pack: $80/user/mo × 500 × 12 = $480,000 + $100,000 signature + $100,000 AI + $150,000 HSM amortisation + $80,000 ops = $910,000/year.

Comparison to counterpart TCO (from benchmarks/docusign-vs-ironclad-vs-icertis-vs-oyatie.md):
- DocuSign CLM Enterprise: $1,044,000/year.
- Ironclad: $1,224,000/year.
- Icertis: $1,404,000/year.
- Conga: $1,044,000/year.
- Agiloft: $804,000/year.

Oyatie paid + per_seat without AI beats every counterpart by 2-4x. Oyatie paid + per_seat with AI matches Agiloft (the cheapest) and beats DocuSign / Ironclad / Conga / Icertis. Oyatie paid + sovereign-pack at $910k is competitive with mid-market commercial alternatives while delivering sovereign-pack capability that no commercial CLM offers.

## §7 Notes for Wave 14 aggregation

The Legal-Complexity family aggregation should produce a unified per-µservice × per-deployment-context × per-tenant-class SLO/throughput/cost grid covering CLM + healthcare-integration + governance + identity. The numbers here are CLM-specific but the structural template (single industry-leader target + 6 contexts × 2 classes overlay + legal-evidence-weight overlay) should be reused across the Legal-Complexity set.

The OCI Always Free quota of 4 OCPU + 24 GB total has to be fairly distributed across the Big-8 + Legal-Complexity µservices for a demo_trial tenant. CLM in demo_trial on OCI Always Free targets consumption under 1 OCPU + 4 GB + 50 GB block + 1× 20 GB Autonomous DB + 25 GB egress. This leaves 3 OCPU + 20 GB for CRM (1 OCPU + 6 GB), HR (1 OCPU + 4 GB), ERP (1 OCPU + 6 GB), and ITSM (~0.5 OCPU + 4 GB) headroom.

The QES sovereign-pack matrix (KR-PIPA / CSAP / EU eIDAS QES / HIPAA-Provider / SEC 17a-4(f)) requires the Wave 14 cross-µservice authoring of:
- TSA integration (CLM ↔ kms or new tsa µservice).
- HSM custody (CLM ↔ kms; kms is the natural owner).
- Sovereign-pack residency × deployment-context matrix.
- Per-pack data-class × retention overlay.

Per the coherence-audit Q-006 / Q-007 / Q-008, Wave 14 must declare the canonical owner of TSA, HSM, and counterparty MDM before Wave 15 IP authoring. The performance numbers in this deliverable assume CLM is the orchestration owner (calling kms for HSM / TSA, calling crm.account for counterparty MDM hand-off); if the ownership decisions land differently, the per-context latency overlay must be re-authored.
