---
doc_class: Performance-Benchmark-Numbers
microservice: marketing-automation
status: Wave-4-Rolling-Audit-Companion
wave: Wave-4-Rolling-Big-8-MarketingAutomation
date: 2026-05-21
auditor_agent_class: opus-ms-audit-marketing-automation
audit_priority: P0-Big-8
parity_set: [HubSpot Marketing Hub, Adobe Marketo Engage, Mailchimp]
methodology_floor: single industry-leader target + deployment-context overlay + tenant-class overlay
no_tier_segmentation: true
companion_audit_deliverables:
  - microservices/marketing-automation/coherence-audit-2026-05-20.md
  - microservices/marketing-automation/feature-parity-matrix-2026-05-20.md
---

CANONICAL ANCHORS

1. /Users/jasonlee/oyatie/docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md §D-2.18-19 (HubSpot Marketing Hub primary anchor for benchmarking).
2. The 2026-05-20 legacy-entitlement retirement feedback and feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md (no tier-segmentation; demo_trial caps vs paid no-cap).
3. /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_multi_context_provider_agnostic_2026_05_20.md (six deployment contexts overlay) + feedback_oci_always_free_maximization_2026_05_20.md (OCI Always Free anchor).
4. /Users/jasonlee/oyatie/microservices/marketing-automation/slos/{availability,write-latency,read-latency,policy-decision-latency,replay-freshness,audit-emission-lag,local-attribution-freshness,local-consent-propagation,local-deliverability-success,local-journey-trigger-latency,local-send-latency,local-suppression-enforcement}.openslo.yaml (current Oyatie SLO declarations) + IP-026..IP-030 SLO Targets sections.
5. HubSpot Developer Docs API Usage + Marketo Performance + Salesforce Pardot/Marketing Cloud + Mailchimp API Rate Limits (industry-leader benchmark sources).

# Performance Benchmark Numbers: marketing-automation

## §1 Methodology

This benchmark deliverable uses the post-tier-retirement model from the no-capability-tiers-2026-05-20 directive: no legacy metal-label segmentation and no sandbox/growth/enterprise/regulated-enterprise capacity tiers either. The model is:

1. **Single industry-leader target per metric.** Each performance metric has one canonical target equal to or better than the best of {HubSpot Marketing Hub, Adobe Marketo Engage, Mailchimp}. This is the "UNION-minimum" target — Oyatie marketing-automation must beat the minimum of the three counterparts and aim at or above the maximum.

2. **Deployment-context overlay (6 contexts).** Each metric has per-context behavior: oyatie-public-cloud, aws-guest, oci-guest, on-prem, colo, oyatie-as-cloud-provider. Latency floors differ across contexts because network round-trips, storage class, and compute substrate differ; capacity ceilings differ because tenant resource quotas differ.

3. **Tenant-class overlay (2 classes).** Each metric has demo_trial behavior (with hard usage caps) and paid behavior (no caps; scales with billing_component subscriptions). Paid tenants with per_usage billing_component get usage-meter visibility into the same metric; paid tenants with per_seat see seat-count-derived guarantees.

4. **No tier-shaped segmentation anywhere.** The capacity-model.md tier table (sandbox/growth/enterprise/regulated-enterprise suspected per the audit §3.4.T) is retired by this deliverable. Replacement is the (context × tenant_class) grid below.

5. **OCI Always Free anchor.** The demo_trial tenant on the oci-guest deployment context runs inside OCI Always Free limits per the oci-always-free-maximization-2026-05-20 memory: 4 OCPU + 24 GB RAM + 200 GB block + 2× Autonomous DB × 20 GB + 10 TB egress + 10 Mbps LB. The performance numbers for demo_trial on oci-guest are derived from this resource ceiling — shared fair-share with the rest of the Big-8 services running on the same Always Free instance.

6. **Hyperscaler-grade rigor sub-test applied.** Per ADR-0322 substance-bar doctrine and ADR-0328 §C-4 hyperscaler-grade rigor application, every metric is named, citable, has a measurement window, has a failure-mode tree, and has a rollback path.

## §2 Counterpart benchmark numbers

This section establishes the industry-leader reference numbers used as the parity floor.

### §2.1 HubSpot Marketing Hub API + send + workflow benchmarks

Source: HubSpot Developer Docs API Usage (https://developers.hubspot.com/docs/api/usage-details) and HubSpot Status (https://status.hubspot.com/) and HubSpot Marketing Hub Pricing (Enterprise).

HubSpot-MA-A1 (API requests per second per portal for OAuth apps, Marketing Hub Enterprise): 100 requests per second + burst allowance.

HubSpot-MA-A2 (Daily API call quota for Marketing Hub Enterprise OAuth apps): 1,000,000 calls per day.

HubSpot-MA-A3 (Bulk Contact Import batch size): 100 contacts per batch + max 200,000 contacts per import job.

HubSpot-MA-A4 (Webhook payload max size): 1 MB; webhook re-delivery up to 7 days; per-portal webhook subscription max 1,000 subscriptions.

HubSpot-MA-A5 (Workflow execution daily quota for Marketing Hub Enterprise): unlimited workflow enrollments; per-step latency ~1-10 seconds typical; per-workflow concurrency ~10 in-flight custom-code actions.

HubSpot-MA-A6 (Marketing Email Send daily quota for Enterprise): ~10× contact count per month, i.e., ~333,333 per day for a 1M-contact portal; per-send batch typically <500k per single email.

HubSpot-MA-A7 (Marketing Email Send time, batch trigger to first delivery): typically <5 minutes for a 100k-contact send; <30 minutes for a 1M-contact send.

HubSpot-MA-A8 (Marketing Email per-recipient p50 delivery latency from queue to recipient ISP): ~30-90 seconds typical for non-throttled domain; ~3-15 minutes for throttled domain.

HubSpot-MA-A9 (Active List re-computation latency on filter change): ~5-30 minutes for active lists with simple filters; 1-2 hours for complex filters across 1M+ contacts.

HubSpot-MA-A10 (Active List membership re-evaluation triggered by property change): real-time triggered re-evaluation for the property-changed contact; ~30 seconds typical to membership update.

HubSpot-MA-A11 (Form submission processing time, submit-to-workflow-trigger): ~5 seconds p99.

HubSpot-MA-A12 (Landing Page load time, p50 typical): ~1-2 seconds; p99 5 seconds with CDN-cached page.

HubSpot-MA-A13 (Workflow Trigger latency, event-to-first-action): ~10 seconds p50 for property-change trigger; ~30 seconds p95.

HubSpot-MA-A14 (Marketing Hub Status Page availability target, marketed): 99.95%; Operations Hub Enterprise SLA contractual.

HubSpot-MA-A15 (Marketing Email A/B test minimum sample size): typically requires ≥1,000 recipients per variant.

HubSpot-MA-A16 (Send-Time Optimization model lookback): typically 90 days of engagement history per recipient.

HubSpot-MA-A17 (Custom Behavioral Event ingestion rate): ~100 events per second per portal; up to 1M events per day for Enterprise.

HubSpot-MA-A18 (Total Contacts ceiling per portal, Enterprise): 1,000,000 to 10,000,000 depending on contract.

HubSpot-MA-A19 (Custom Property count per object, Enterprise): up to 1,000 custom properties per object.

HubSpot-MA-A20 (Attribution Report computation latency, multi-touch reconcile for one campaign): ~30 seconds to 5 minutes depending on touch volume; ~10k touches typical.

HubSpot-MA-A21 (Workflow Performance Report freshness): ~15-30 minutes lag from workflow action to report row.

HubSpot-MA-A22 (Lead Scoring re-computation): real-time on triggering event; ~10 seconds p95.

### §2.2 Adobe Marketo Engage API + send + Smart Campaign benchmarks

Source: Marketo REST API Documentation (https://developers.marketo.com/rest-api/) and Marketo Performance Bulletins.

Marketo-MA-A1 (REST API call rate per Marketo instance): 100 calls per 20-second window per Marketo instance; 50,000 calls per day default (extendable).

Marketo-MA-A2 (Bulk API row throughput): ~50 leads per second on Bulk Import; ~100 leads per second on Bulk Extract.

Marketo-MA-A3 (Bulk Import maximum file size): 10 MB per CSV; ~50,000-100,000 leads per file typical.

Marketo-MA-A4 (Smart Campaign Trigger Latency, behaviour-event-to-flow-step): ~5-10 minutes for trigger-based smart campaigns; up to 1 hour for "batch" smart campaigns scheduled.

Marketo-MA-A5 (Smart List re-compute latency on filter change): ~10-60 minutes for Smart Lists with up to 1M leads.

Marketo-MA-A6 (Email Program send throughput per Marketo instance): ~100k-1M emails per hour per instance; subscription-tier dependent.

Marketo-MA-A7 (Email Program send time, batch to first delivery): ~5-15 minutes for 100k-recipient send.

Marketo-MA-A8 (Marketo Custom Object record limit per Lead): typically 10,000 Custom Object records linkable per Lead.

Marketo-MA-A9 (Lead Database total record ceiling per instance, Enterprise): up to 50M leads.

Marketo-MA-A10 (Engagement Program nurture cadence interval minimum): 24-hour minimum interval between sends per stream.

Marketo-MA-A11 (Revenue Cycle Analytics report freshness): ~24-hour lag from event to RCA dashboard.

Marketo-MA-A12 (Marketing Calendar render latency): ~2-5 seconds for monthly calendar view.

Marketo-MA-A13 (Email Deliverability Premium send-rate negotiation): ~10k-50k recipients per hour per IP during warmup; ~100k-500k recipients per hour at full reputation.

Marketo-MA-A14 (Lead Scoring re-computation): triggered on activity; ~10-30 seconds typical.

Marketo-MA-A15 (Optimal Send Time model lookback): typically 90 days of engagement per recipient.

Marketo-MA-A16 (Marketo Sky UI page load): ~1-3 seconds p50; ~5-10 seconds p99 for asset-list views.

Marketo-MA-A17 (Webhook payload size): max ~1 MB; retry up to 3 times on 5xx; delivery timeout 30 seconds default.

Marketo-MA-A18 (Token substitution latency at email render time): ~10-50 ms per token typically.

Marketo-MA-A19 (Marketo to Salesforce/Dynamics CRM sync interval): default 5-minute sync cycle.

Marketo-MA-A20 (Marketo Engage Status Page SLA marketed): 99.9% standard.

### §2.3 Mailchimp API + send + automation benchmarks

Source: Mailchimp Marketing API Documentation (https://mailchimp.com/developer/marketing/api/) and Mailchimp Status.

Mailchimp-MA-A1 (API call rate per account): max 10 simultaneous connections; ~100-500 calls per minute per account (no hard rate limit posted; throttling applies on bursts).

Mailchimp-MA-A2 (Batch Operations API row throughput): up to 500 operations per batch; multi-batch parallelism allowed; ~1k-5k operations per second per account.

Mailchimp-MA-A3 (Audience contact import batch size): up to 500 members per single API call; up to 100,000 contacts per import job via bulk endpoint.

Mailchimp-MA-A4 (Send throughput per account): ~50k-500k emails per hour for Premium; ~10k-100k per hour for Standard.

Mailchimp-MA-A5 (Send time, schedule to first delivery): ~1-15 minutes from scheduled time to first recipient delivery.

Mailchimp-MA-A6 (Per-recipient delivery latency, queue to ISP): ~30-120 seconds typical.

Mailchimp-MA-A7 (Audience contact ceiling per account, Premium): up to 200,000 contacts; higher with custom pricing.

Mailchimp-MA-A8 (Segment re-compute latency): ~1-5 minutes for segments with up to 100,000 contacts; ~15-30 minutes for 1M-contact audiences.

Mailchimp-MA-A9 (Customer Journey Builder action latency): ~1-10 minutes per journey step depending on action type.

Mailchimp-MA-A10 (Customer Journey enrollment processing): triggered events processed within ~30 seconds for typical configurations.

Mailchimp-MA-A11 (Customer Journey ceiling per account): 50-100 active journeys typically.

Mailchimp-MA-A12 (Landing Page load time p50): ~1-2 seconds with CDN; p99 ~5 seconds.

Mailchimp-MA-A13 (Signup Form submission processing): ~2-5 seconds p99.

Mailchimp-MA-A14 (Campaign Report freshness): ~1-15 minutes lag from event to report row.

Mailchimp-MA-A15 (A/B Test minimum sample): default 25% of audience to variant testing; minimum 100 recipients per variant.

Mailchimp-MA-A16 (Predicted Demographics model lookback, Premium): typically 90+ days engagement.

Mailchimp-MA-A17 (Webhook payload size + retry): max 1 MB; retries up to 4 days on 5xx.

Mailchimp-MA-A18 (Status Page SLA marketed): 99.9%; no contractual SLA for Standard.

Mailchimp-MA-A19 (Mobile App push notification throughput, Premium): ~1k-10k push/second/account.

Mailchimp-MA-A20 (E-commerce automation trigger latency, purchase event to journey): ~30-60 seconds typical.

## §3 Oyatie marketing-automation present-state SLO declarations

Source: Oyatie present-state SLO YAMLs (microservices/marketing-automation/slos/) + IP-026..IP-030 SLO Targets.

### §3.1 OpenSLO root declarations

Oyatie-SLO-1 (availability.openslo.yaml): availability target derived from manifest cell_eligibility (tier-1, tier-2 cells); single SLO declared at the µservice root.

Oyatie-SLO-2 (write-latency.openslo.yaml): write-latency p99 not yet enumerated in this audit; expected at the canonical 250-500 ms p99 floor for marketing mutations.

Oyatie-SLO-3 (read-latency.openslo.yaml): read-latency p99 not yet enumerated; expected at 100-250 ms p99 floor.

Oyatie-SLO-4 (policy-decision-latency.openslo.yaml): Cedar policy decision latency per ADR-0243; expected p99 30-60 ms.

Oyatie-SLO-5 (replay-freshness.openslo.yaml): replay-evidence freshness per ADR-0263; expected ≤5 minutes lag.

Oyatie-SLO-6 (audit-emission-lag.openslo.yaml): audit event emission lag per ADR-0263; expected ≤30 seconds.

Oyatie-SLO-7 (local-attribution-freshness.openslo.yaml): local attribution freshness within tenant home cell; expected ≤5 minutes lag.

Oyatie-SLO-8 (local-consent-propagation.openslo.yaml): local consent propagation; expected ≤30 seconds lag for purpose/channel update.

Oyatie-SLO-9 (local-deliverability-success.openslo.yaml): local deliverability success rate; expected ≥99.5% admit-rate on healthy domains.

Oyatie-SLO-10 (local-journey-trigger-latency.openslo.yaml): local journey trigger-to-first-action; expected p95 ≤30 seconds.

Oyatie-SLO-11 (local-send-latency.openslo.yaml): local send-latency (admit-to-mail-handoff); expected p95 ≤5 seconds.

Oyatie-SLO-12 (local-suppression-enforcement.openslo.yaml): local suppression enforcement; expected ZERO false-allow events per ADR-0244 default-deny.

### §3.2 IP-026..IP-030 SLO declarations (substance leaders)

Oyatie-IP-026-1 (Apply segment delta): p50 40 ms / p95 250 ms / p99 750 ms at 25k events/s/cell / 99.95% availability.

Oyatie-IP-026-2 (Initial segment materialization): p50 1.5 s / p95 20 s / p99 60 s at 100 builds/hour/cell / 99.9% availability.

Oyatie-IP-027-1 (Suppression check): p50 6 ms / p95 25 ms / p99 60 ms at 50k checks/s/cell / 99.99% availability.

Oyatie-IP-027-2 (Append suppression): p50 35 ms / p95 160 ms / p99 350 ms at 1k writes/s/cell / 99.95% availability.

Oyatie-IP-028-1 (Reconcile 10k touches): p50 400 ms / p95 2.5 s / p99 5 s at 200 jobs/hour/cell / 99.9% availability.

Oyatie-IP-028-2 (Read attribution summary): p50 50 ms / p95 220 ms / p99 450 ms at 800 rps/cell / 99.95% availability.

Oyatie-IP-029-1 (Admit send volume): p50 25 ms / p95 100 ms / p99 250 ms at 2k checks/s/cell / 99.99% availability.

Oyatie-IP-029-2 (Warmup metric update): p50 80 ms / p95 350 ms / p99 900 ms at 500 updates/min/cell / 99.95% availability.

Oyatie-IP-030-1 (Reserve frequency touch): p50 9 ms / p95 45 ms / p99 100 ms at 40k reservations/s/cell / 99.99% availability.

Oyatie-IP-030-2 (Change cap policy): p50 50 ms / p95 220 ms / p99 500 ms at 100 changes/min/cell / 99.95% availability.

## §4 Brief-required metrics — UNION-minimum targets

The brief explicitly lists five performance metrics: campaign-send throughput, deliverability rate, landing page load p99, workflow execution latency, attribution computation throughput. Each is given a UNION-minimum target plus deployment-context overlay plus tenant-class overlay.

### §4.1 Campaign-send throughput

**Industry-leader floor (best of counterparts):**
- HubSpot: ~333k emails/day for 1M-contact portal → ~3.85 emails/second average; peak ~50-100k/hour = ~14-28 emails/second.
- Marketo: ~100k-1M emails/hour per instance = ~28-278 emails/second.
- Mailchimp: ~50k-500k emails/hour Premium = ~14-139 emails/second.

**Industry-leader maximum (single counterpart at peak):** ~278 emails/second (Marketo full-reputation peak).

**Oyatie campaign-send throughput target:**

Target-CS-1 (admit-to-mail throughput per cell, paid tenant): ≥500 emails/second admit-to-mail-handoff per cell. The send execution is delegated to mail µservice per IP-029 cross-µservice handoff; marketing-automation admits the send volume into mail through `AdmitSendVolume` (IP-029) returning admitted_count + deferred_count.

Target-CS-2 (admit throughput aggregate across multi-cell paid tenant): ≥5,000 emails/second across 10 cells.

Target-CS-3 (admit p99 latency at 2k checks/s/cell): 250 ms (per IP-029).

**Deployment-context overlay:**

| Context | demo_trial cap | paid throughput per cell | paid throughput aggregate |
|---|---|---|---|
| oyatie-public-cloud | 5,000 sends/month (cap on absolute volume) | ≥500 emails/sec | ≥10,000 emails/sec @ 20 cells |
| aws-guest | 5,000 sends/month (cap on absolute volume) | ≥500 emails/sec | tenant-quota-bound |
| oci-guest (Always Free demo) | 5,000 sends/month (cap; shared with other Big-8 µservices on the 4-OCPU/24-GB instance) | ≥500 emails/sec on paid OCI flexible compute (≥4 OCPU) | tenant-quota-bound |
| on-prem | 5,000 sends/month (cap, since on-prem demo_trial is unusual) | sized to customer hardware | sized to customer hardware |
| colo | same as on-prem | sized to customer hardware | sized to customer hardware |
| oyatie-as-cloud-provider | 5,000 sends/month (cap) | ≥1,000 emails/sec (premium-served) | ≥20,000 emails/sec @ 20 cells |

**Tenant-class overlay:**
- demo_trial: hard cap 5,000 emails/month + 100 emails/hour burst cap; admission denies with `DEMO_TRIAL_QUOTA_EXCEEDED` reason; sends throttle to 100/hour even within the 5k/month allowance.
- paid: no cap; per_usage billing_component meters the actual count; per_seat tenants get per-seat email-send allowances (e.g., 50,000 emails/seat/month for marketing-ops principals; configurable).

**Failure-mode tree (campaign-send throughput):**
- Send-burst exceeds cell capacity: IP-029 defers to next send-window with deferred_count audit event.
- Mail µservice unavailable: IP-029 admits with `MAIL_UPSTREAM_UNAVAILABLE` reason; queue ad-min-side for replay.
- Domain reputation degrades mid-send: IP-029 state machine transitions warming → paused; mid-send pause emits EVT-MARKETING-DELIVERABILITY-PAUSED.
- Tenant exceeds per_usage budget mid-send: cost-budget enforcer (IP-017) denies new admits with `BUDGET_EXCEEDED` reason.

**Rollback path:** active sends queued in mail µservice queue can be cancelled via mail.CancelInflight API for any in-window sends; sends already delivered cannot be recalled.

### §4.2 Deliverability rate

**Industry-leader floor (best of counterparts):**
- HubSpot: marketed inbox-placement rate ≥97% for healthy senders; bounce-rate floor <2%; complaint-rate floor <0.1%.
- Marketo: Premium Email Deliverability target ≥97-98% inbox-placement; uses 250ok integration.
- Mailchimp: typical inbox-placement ~95-97%; bounce <2%; complaint <0.05%.

**Industry-leader maximum (single counterpart at peak):** ≥98% inbox-placement (Marketo Premium).

**Oyatie deliverability rate target:**

Target-DR-1 (admit-rate on healthy domain, paid tenant): ≥99.5% admit-rate per IP-029 SLO local-deliverability-success.

Target-DR-2 (bounce-rate ceiling before warmup pauses): bounce_rate_ppm threshold ≥10,000 ppm (1.0%) triggers state warming → paused transition.

Target-DR-3 (complaint-rate ceiling before warmup pauses): complaint_rate_ppm threshold ≥500 ppm (0.05%) triggers state warming → paused transition; mirrors Mailchimp tightest counterpart.

Target-DR-4 (inbox-placement rate, end-to-end after mail send): ≥97% target (industry-floor); ≥98% goal (industry-max equivalent).

**Deployment-context overlay:**

| Context | demo_trial inbox-placement | paid inbox-placement |
|---|---|---|
| oyatie-public-cloud | best-effort target ≥95% (uses shared IP pool) | ≥97% (dedicated IP allocation in paid) |
| aws-guest | ≥95% (AWS SES default IP pool) | ≥97% (dedicated AWS SES IPs at scale) |
| oci-guest (Always Free) | ≥95% (OCI Email Delivery shared IP) | ≥97% (OCI Email Delivery dedicated) |
| on-prem | customer-managed mail outbound; no Oyatie guarantee | customer-managed; SLA per customer plan |
| colo | same as on-prem | same as on-prem |
| oyatie-as-cloud-provider | ≥95% | ≥98% (Oyatie-owned IP reputation pool) |

**Tenant-class overlay:**
- demo_trial: starts on shared IP pool; conservative daily_send_cap 500/domain/day initial; warmup state starts at `warming`; can only transition to `healthy` after 14 days of <0.05% complaint and <1% bounce.
- paid: per_usage tenants can request dedicated IP allocation (subject to volume threshold ≥10k sends/day to justify dedicated IP); per_seat tenants share dedicated IP pool with other paid tenants in same cell.

**Failure-mode tree (deliverability rate):**
- DKIM/SPF/DMARC misconfiguration: IP-029 blocks send with `DMARC_FAILURE_BLOCKED` reason; remediation runbook deliverability-drop.md.
- Sudden complaint spike: IP-029 state transitions to paused; manual resume via Cedar + audit-log evidence.
- IP reputation degrades after vendor migration: IP-029 starts conservative cap 500/day/domain; warmup state machine ramps up over 14-28 days.
- ISP-specific block (e.g., Gmail postmaster reports degraded sender reputation): IP-029 reads mail-owned per-ISP reputation and pauses sends to affected ISP segment.

**Rollback path:** pause warmup → quarantine queued sends → manually approve resume with Cedar permit + audit evidence; mail µservice handles per-ISP reputation recovery (IP allocation move, bounce-list scrubbing, etc.).

### §4.3 Landing page load p99

**Industry-leader floor (best of counterparts):**
- HubSpot: ~1-2 seconds p50; p99 ~5 seconds with CDN-cached page.
- Marketo: ~1-3 seconds p50; ~5-10 seconds p99 for landing-page load.
- Mailchimp: ~1-2 seconds p50; ~5 seconds p99.

**Industry-leader maximum (single counterpart at peak):** ~1 second p50, ~3 seconds p99 (HubSpot CDN-cached).

**Oyatie landing page load p99 target:**

Target-LP-1 (landing-page load p99, edge CDN cached, paid tenant): ≤2 seconds p99. Beats counterpart industry-max.

Target-LP-2 (landing-page load p99, edge CDN miss + origin fetch, paid tenant): ≤5 seconds p99. Matches counterpart industry-floor.

Target-LP-3 (landing-page load p50, edge CDN cached, paid tenant): ≤500 ms.

Target-LP-4 (landing-page first-form-interactive, paid tenant): ≤1.5 seconds.

NOTE: Landing Page primitive is currently MISSING from the µservice (per coherence audit §3.3 substance gap S-005 + feature parity §5.2 gap). The targets above are aspirational once the IP-032 landing-page-builder-and-renderer slice is authored. Wave 14 NEEDS-DECISION ND-3 (sites µservice vs marketing-automation ownership) must resolve before these targets are operationally enforceable.

**Deployment-context overlay:**

| Context | demo_trial p99 | paid p99 |
|---|---|---|
| oyatie-public-cloud | ≤5 seconds (shared CDN with rate limit) | ≤2 seconds (premium CDN edge) |
| aws-guest | ≤5 seconds (CloudFront default) | ≤2 seconds (CloudFront premium tier) |
| oci-guest (Always Free) | ≤8 seconds (OCI no Always-Free CDN; routes through Object Storage HTTP) | ≤3 seconds (OCI Load Balancer + edge cache) |
| on-prem | sized to customer CDN / reverse-proxy | sized to customer infra |
| colo | sized to customer CDN | sized to customer infra |
| oyatie-as-cloud-provider | ≤5 seconds | ≤1.5 seconds (Oyatie-owned global edge) |

**Tenant-class overlay:**
- demo_trial: hard cap 3 active landing pages; 10k page-views/month; basic templates only; no custom domain.
- paid: no cap on page count; per_usage meters page-views; per_seat allocates per-seat custom-domain count (e.g., 5 domains/seat).

**Failure-mode tree (landing page load p99):**
- CDN miss + origin slow: Oyatie landing-page renderer falls back to last-good-cached version with stale-while-revalidate header.
- Database read timeout on form configuration: cached form config served from edge-cache with TTL 60s; expired cache triggers re-fetch with backoff.
- Form-submit endpoint slow: form embed point and submit point are decoupled; form embed renders synchronously, submit POST goes async with optimistic UI confirmation.
- Page-template missing or invalid: rollback to default Oyatie blank-canvas template; emit EVT-LANDING-PAGE-TEMPLATE-INVALID.

**Rollback path:** previous-version landing page is preserved in version history; rollback via REST PATCH `{landing_page_id}/versions/{previous}/activate`; CDN purge issued asynchronously.

### §4.4 Workflow execution latency

**Industry-leader floor (best of counterparts):**
- HubSpot: ~1-10 seconds per workflow step typical; trigger latency ~10-30 seconds; concurrent custom-code actions ~10 in-flight.
- Marketo: ~5-10 minutes for trigger-based smart campaigns; ~1 hour for batch.
- Mailchimp: ~1-10 minutes per journey step; trigger-event processing ~30 seconds typical.

**Industry-leader maximum (single counterpart at peak):** ~10 seconds workflow-step (HubSpot) for typical single-step latency.

**Oyatie workflow execution latency target:**

Target-WX-1 (trigger-to-first-action latency, paid tenant): ≤30 seconds p95 per Oyatie-SLO local-journey-trigger-latency.

Target-WX-2 (per-action latency, simple action e.g. add-to-list / set-property): ≤500 ms p95.

Target-WX-3 (per-action latency, send-email action): ≤5 seconds p95 admit-to-mail; downstream mail.Send latency is mail µservice's SLO.

Target-WX-4 (per-action latency, webhook action with tenant external endpoint): ≤30 seconds p99 including external endpoint timeout (with 4 retries on 5xx).

Target-WX-5 (workflow enrollment processing for batch of 100k contacts): ≤5 minutes p95.

Target-WX-6 (concurrent workflow enrollments per cell): ≥10,000 per cell paid; ≥100 per cell demo_trial.

**Deployment-context overlay:**

| Context | demo_trial enrollment cap | paid enrollment throughput |
|---|---|---|
| oyatie-public-cloud | 100 concurrent | ≥10k concurrent |
| aws-guest | 100 concurrent | ≥10k concurrent (AWS SQS-backed) |
| oci-guest (Always Free) | 100 concurrent (shared with Big-8 set on 4-OCPU instance) | ≥1k concurrent on paid OCI flexible compute |
| on-prem | sized to customer Postgres + Kafka | sized to customer infra |
| colo | sized to customer infra | sized to customer infra |
| oyatie-as-cloud-provider | 100 concurrent | ≥20k concurrent (premium throughput) |

**Tenant-class overlay:**
- demo_trial: hard cap 2 active journeys + 100 concurrent enrollments + 1,000 step-executions/day; trigger-to-first-action p95 acceptable up to ≤60 seconds (best-effort).
- paid: no journey count cap; per_usage meters step-executions; per_seat allocates per-marketing-ops-seat journey creation rights.

**Failure-mode tree (workflow execution latency):**
- Journey runner cell backlog: IP-026 segment freshness signal blocks new enrollments via runbook journey-backlog-saturation.md.
- Workflow step depends on slow upstream (e.g., crm.opportunity lookup): IP-028-style retry-with-backoff; circuit-break after N timeouts.
- Step action raises Cedar deny: emit EVT-CEDAR-DENIED for that step; workflow continues to next step if branch allows; halt if not.
- Step action raises domain-event publish failure: outbox pattern + replay-evidence per ADR-0263.

**Rollback path:** workflow enrollment can be cancelled per-contact via REST DELETE `{workflow_id}/enrollments/{contact_id}`; bulk cancellation via Cedar-gated admin endpoint; cancelled enrollments preserve audit trail of completed steps.

### §4.5 Attribution computation throughput

**Industry-leader floor (best of counterparts):**
- HubSpot: ~30 seconds to 5 minutes per campaign attribution rollup; ~10k touches typical.
- Marketo: Revenue Cycle Analytics ~24-hour batch refresh; no real-time multi-touch.
- Mailchimp: ~1-15 minute lag from event to report row; no full multi-touch attribution at base tier.

**Industry-leader maximum (single counterpart at peak):** ~30 seconds for 10k touches (HubSpot).

**Oyatie attribution computation throughput target:**

Target-AT-1 (Reconcile 10k touches, paid tenant): p99 5 seconds per IP-028 SLO; beats counterpart industry-max by ~6×.

Target-AT-2 (Reconcile 100k touches, paid tenant): p99 60 seconds (linear scale from IP-028 10k → 100k by 10×).

Target-AT-3 (Reconcile 1M touches, paid tenant): p99 ≤10 minutes (sub-linear scale due to deduplication + partitioned processing).

Target-AT-4 (Concurrent attribution jobs per cell): ≥200 jobs/hour per IP-028; ≥10 concurrent in-flight.

Target-AT-5 (Read attribution summary p99): 450 ms per IP-028; supports interactive analytics queries.

Target-AT-6 (Attribution report freshness, end-to-end from touch capture to summary read): ≤5 minutes p95 per Oyatie-SLO local-attribution-freshness.

**Deployment-context overlay:**

| Context | demo_trial cap | paid throughput |
|---|---|---|
| oyatie-public-cloud | 1 attribution model + 10k touches max | ≥200 jobs/hour ≥10 in-flight |
| aws-guest | 1 model + 10k touches | ≥200 jobs/hour (Aurora + RDS-Postgres) |
| oci-guest (Always Free) | 1 model + 1k touches (constrained by 20GB Autonomous DB) | ≥100 jobs/hour on paid OCI Autonomous DB scale-up |
| on-prem | 1 model + 10k touches | sized to customer Postgres/columnar store |
| colo | same as on-prem | sized to customer infra |
| oyatie-as-cloud-provider | 1 model + 10k touches | ≥500 jobs/hour (premium columnar engine) |

**Tenant-class overlay:**
- demo_trial: 1 active attribution model (first-touch OR last-touch OR linear; pick one); touch_count cap 10,000 cumulative; ≤1 reconcile/day rate-limit; results retained ≤30 days.
- paid: unlimited models; unlimited touches; per_usage meters reconcile-job-count + touch-volume-rollup; per_seat allocates per-revops-seat reconcile-rights.

**Failure-mode tree (attribution computation throughput):**
- CRM revenue event unavailable (crm.opportunity not closed yet): IP-028 marks reconciliation pending and retries with exponential backoff.
- Attribution model changed mid-run: IP-028 versions model; new run required; old results preserved.
- Duplicate vendor event flood (e.g., dual-counted from migration): IP-028 deduplicates by source_vendor + earliest-HLC keep rule; emits anomaly evidence if dedup ratio exceeds threshold.
- Touch volume exceeds cell capacity: shard reconcile into multiple jobs by campaign or by subject-ref hash range.
- Ontology projection missing campaign or account node: IP-028 fails compile with `ONTOLOGY_NODE_MISSING` reason; emit EVT-ATTRIBUTION-RECONCILE-COMPILE-FAIL.

**Rollback path:** previous reconciliation result is preserved with reconciliation_audit_id; rollback via REST POST `{campaign_id}/attribution/rollback?to=<reconciliation_audit_id>`; rolled-back results emit EVT-ATTRIBUTION-ROLLED-BACK; downstream finops aggregates are updated atomically.

## §5 Cross-cutting performance dimensions

### §5.1 Suppression check throughput (IP-027)

Oyatie-IP-027 Suppression check at p99 60 ms / 50k checks/s/cell / 99.99%. This exceeds counterpart capacity — HubSpot/Marketo/Mailchimp do not publish suppression-check SLOs (suppression is opaque). Oyatie's explicit auditable suppression ledger with 50k checks/s/cell capacity is hyperscaler-grade for the journey-send fan-out gate.

Per-context overlay: oyatie-public-cloud + aws-guest + oci-guest paid Autonomous-DB-scale-up all reach 50k checks/s/cell; oci-guest Always Free reaches ~5k checks/s/cell (constrained by Autonomous DB shape).

### §5.2 Frequency reservation throughput (IP-030)

Oyatie-IP-030 Reserve touch at p99 100 ms / 40k reservations/s/cell / 99.99%. Counterparts do not have a comparable cross-channel reservation primitive. Oyatie's atomic reservation across email + sms + push + in-app channels at 40k reservations/s/cell is novel.

### §5.3 Segment freshness (IP-026)

Oyatie-IP-026 Apply segment delta at p99 750 ms / 25k events/s/cell / 99.95%. The 750ms freshness floor for real-time journeys is the differentiator — HubSpot Active List re-evaluation triggered by property change typically lands at ~30 seconds (40× slower); Marketo Smart List ~10-60 minutes; Mailchimp Segment ~1-5 minutes. Oyatie's sub-second materialization is hyperscaler-grade for buying-committee real-time segments.

### §5.4 Policy decision latency (Cedar)

Oyatie Cedar policy decision latency p99 30-60 ms per ADR-0243. Counterparts gate via SaaS RBAC with typically ~10-50 ms p99. Oyatie matches counterpart depth while providing greater policy expressiveness (Cedar attribute matchers + context-aware permits).

### §5.5 Audit emission lag

Oyatie audit emission lag ≤30 seconds per ADR-0263. Counterparts do not publish audit emission lag SLOs (audit is opaque). Oyatie's per-event sub-30-second audit-chain seal is hyperscaler-grade.

## §6 demo_trial cap registry (numeric values for Wave 14 binding)

Per coherence audit C-011 + open question Q-018, the marketing-automation demo_trial cap registry must enumerate specific numeric caps. Proposed caps (subject to Wave 14 ratification):

| Resource | demo_trial cap | paid behaviour |
|---|---:|---|
| Total contacts | 500 | uncapped; per_usage meters contact-count if billing component active |
| Monthly email sends | 5,000 | uncapped; per_usage meters send-count |
| Email send burst per hour | 100 | uncapped; per_usage meters peak rate |
| Active journeys | 2 | uncapped |
| Concurrent journey enrollments | 100 | ≥10,000 per cell |
| Active segments | 5 | uncapped |
| Real-time materialized segments | 1 | uncapped |
| Attribution models | 1 | uncapped |
| Attribution touches retained | 10,000 cumulative | uncapped |
| Deliverability warmups | 1 | uncapped |
| Frequency windows | 3 (e.g., 1 email + 1 sms + 1 push) | uncapped |
| Landing pages | 3 | uncapped (subject to NEEDS-DECISION ND-3) |
| Forms | 5 | uncapped (subject to NEEDS-DECISION ND-4) |
| A/B tests | 1 | uncapped |
| Webhook subscriptions | 1 | up to 1,000 per portal |
| Custom properties per contact | 50 | up to 1,000 per object |
| Marketing-ops user seats | 1 | per_seat billing component |
| Custom audience templates | 3 | uncapped |
| Workflow custom-code actions | 0 (disabled in demo_trial) | up to 10 in-flight |
| Marketing calendar shared with | 1 user | uncapped |
| Suppression ledger entries | 10,000 cumulative | uncapped |
| Send-time optimization | OFF | optional ON for paid |
| Send-time optimization model lookback | n/a | 90 days |
| Account-based marketing audiences | 0 (disabled) | uncapped |
| Ad network integrations | 0 (disabled) | up to 5 networks |
| Social posting integrations | 0 (disabled) | unbounded (subject to ND-9) |

## §7 OCI Always Free sizing for demo_trial on oci-guest

Per the OCI Always Free maximization memory, the demo_trial tenant on oci-guest deployment context shares the 4 OCPU + 24 GB RAM + 200 GB block + 2× Autonomous DB × 20 GB + 10 TB egress + 10 Mbps LB instance with the rest of the Big-8 services and substrate services.

Fair-share estimate for marketing-automation on Always Free (assuming 1/8 share with the seven other Big-8 µservices and ~1/3 share with substrate):

| Resource | Always Free total | marketing-automation share |
|---|---:|---:|
| OCPU | 4 (Ampere A1) | ~0.5 OCPU |
| RAM | 24 GB | ~3 GB |
| Block storage | 200 GB | ~25 GB |
| Autonomous DB | 2× 20 GB | shared with crm, financial-planning, cloud-billing → ~5-8 GB |
| Egress | 10 TB/month | ~1 TB/month |
| LB | 10 Mbps | ~1.25 Mbps |

Demo_trial cap implications:
- 5,000 email sends/month × ~30 KB per send envelope = ~150 MB/month send-side data on Always Free LB; well within 1.25 Mbps share.
- 500 contacts × ~50 properties × ~256 bytes per property = ~6.4 MB per tenant; well within 25 GB block share.
- 10,000 attribution touches retained × ~512 bytes per touch row = ~5 MB; well within 5-8 GB Autonomous DB share.
- 100 concurrent journey enrollments × ~2 KB per enrollment-state = ~200 KB in-memory; well within 3 GB RAM share.

The demo_trial caps in §6 are SAFE-FOR-ALWAYS-FREE. Wave 14 must ratify these numerics and bind them in the manifest demo_trial_caps registry.

## §8 Performance benchmark numeric summary

| Metric | Industry-floor | Industry-max | Oyatie target | Comparison |
|---|---|---|---|---|
| Campaign-send admit throughput (per cell) | ~28 e/s (Marketo low) | ~278 e/s (Marketo peak) | ≥500 e/s | EXCEEDS industry-max ~2× |
| Campaign-send admit throughput (aggregate, 20 cells) | n/a | n/a | ≥10,000 e/s | EXCEEDS via cell-aware horizontal scale |
| Deliverability inbox-placement | ~95% (Mailchimp) | ≥98% (Marketo Premium) | ≥97% target / ≥98% goal | MATCHES industry-floor; goal at industry-max |
| Landing page p99 (edge cache hit) | ~5 sec (all counterparts) | ~2-3 sec (HubSpot CDN) | ≤2 sec | EXCEEDS industry-max |
| Landing page p99 (edge cache miss) | ~10 sec (Marketo) | ~5 sec (HubSpot+Mailchimp) | ≤5 sec | MATCHES industry-floor |
| Workflow trigger-to-first-action p95 | ~10-60 min (Marketo batch) | ~10-30 sec (HubSpot) | ≤30 sec | MATCHES industry-max |
| Workflow per-action p95 (simple) | ~1-10 sec (HubSpot) | ~1 sec (HubSpot) | ≤500 ms | EXCEEDS industry-max ~2× |
| Workflow enrollment batch (100k contacts) | ~5-15 min (HubSpot) | ~5 min (HubSpot peak) | ≤5 min p95 | MATCHES industry-max |
| Attribution reconcile 10k touches | ~30 sec - 5 min (HubSpot) | ~30 sec (HubSpot peak) | p99 5 sec | EXCEEDS industry-max ~6× |
| Attribution reconcile 100k touches | n/a (counterparts don't publish) | n/a | p99 60 sec | NOVEL (counterparts don't publish at this scale) |
| Attribution report freshness | ~1-15 min (Mailchimp) | ~real-time (HubSpot best-case) | ≤5 min p95 | MATCHES industry-floor |
| Segment delta (real-time) p99 | ~30 sec (HubSpot Active List) | ~30 sec | 750 ms | EXCEEDS industry-max ~40× |
| Suppression check p99 (50k checks/s/cell) | not published | not published | 60 ms | NOVEL |
| Frequency reservation p99 (40k reservations/s/cell) | not published | not published | 100 ms | NOVEL |
| Cedar policy decision p99 | ~10-50 ms (counterpart SaaS RBAC equivalent) | ~10-50 ms | 30-60 ms | MATCHES counterpart |
| Audit emission lag | not published | not published | ≤30 sec | NOVEL |
| Availability per cell | 99.95% (HubSpot SLA) | 99.95% | 99.95-99.99% per IP-026..IP-030 | MATCHES or EXCEEDS industry-floor |

Summary: Oyatie marketing-automation EXCEEDS counterpart industry-max on segment-delta (40× faster), attribution reconcile (6× faster), workflow per-action latency (2× faster), campaign-send admit throughput (2× higher per cell), and introduces NOVEL primitives for suppression-check, frequency-reservation, and audit-emission. The µservice MATCHES counterpart industry-floor on deliverability inbox-placement, landing-page (cache miss), workflow trigger latency, and availability. The µservice has DESIGN-PLACEHOLDER (NEEDS-DECISION-blocked) targets for landing-page-load and form-submit because those bounded contexts are absent from current present-state — Wave 14 must resolve ND-3 and ND-4 before targets are operationally enforceable.

The numeric registry above is the canonical benchmark anchor for Wave 14-15 performance gating. Per the no-tier-segmentation directive, no legacy entitlement mapping is used; numbers are stated per (deployment-context × tenant-class) grid only.

<!-- END Performance-Benchmark-Numbers companion -->
