---
doc_class: User-Journey-README
journey_id: j167-cto-diego-vargas-platform-major-version-cutover
slice: platform-major-version-cutover-v3-to-v4-with-staged-rollout-and-rollback
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Aurelia Robotics CTO Diego Vargas
audience_type: EXECUTIVE_CTO + B2B_PLATFORM_ENGINEERING
microservice_count: 5
pack_overlay_anchor: ISO-27001-A.12.1.2-change-management + ITIL-v4-change-enablement + NIST-800-53-CM-3 + SOC2-CC8.1 + EU-AI-Act-Art-17-quality-management-system + ISO-22301-business-continuity
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0263-observability-emission-contract
  - ADR-0245-substrate-vs-product-layering
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0252-hlc-default-truetime-tier
  - ADR-0254-kubernetes-everywhere-pods-cloud-hypervisor
  - ADR-0250-build-ahead-of-certification
  - ADR-0110-changeset-state-machine
  - ADR-0111-merge-queue-projected-state-fix-at-any-stage
---

# j167 — CTO Diego Vargas: platform v3.0 → v4.0 cutover with staged rollout

## At a glance

Diego Vargas is a **47-year-old Chief Technology Officer** of **Aurelia Robotics International, S.A. de C.V.** ("Aurelia Robotics"), a Mexico-City-headquartered industrial-robotics SaaS that builds fleet-orchestration software for warehouse + manufacturing robot fleets (AGVs, AMRs, cobots) deployed across 412 customer sites in 27 countries. The company was founded in 2017, IPO'd on the BMV (Bolsa Mexicana de Valores) in 2024 under ticker `AURELIA-A`, employs 1,840 staff (of whom 612 are engineers across 4 R&D centers: Mexico City + Guadalajara + Querétaro + Austin TX), and has annual recurring revenue of **MXN 7.84 billion (≈ USD 412M)**. Diego joined as VP Engineering #14 in 2018, promoted to CTO in 2021 after the Series-C raise.

Diego is Mexican (born in Monterrey, Nuevo León; raised between Monterrey and Tampico after his father — a PEMEX refinery process-engineer — was transferred), speaks Spanish (native, Mexican-northern dialect with regiomontano stretch), English (C2; MIT Sloan MBA 2014; daily working language with the Austin office and 60% of US-based customers), and reading-level German (his maternal grandmother emigrated from Stuttgart in 1952; he can read Siemens + Bosch industrial-robotics technical specs but does not negotiate in German). His oyatie tenant chip reads `aurelia-robotics-internacional-sa-de-cv-mx`. He runs his office out of the 14th floor of Torre Manacar in Mexico City's Insurgentes Sur corridor (Colonia del Valle, CDMX); his secondary desk is at the Austin office on East 6th Street.

It is **Tuesday October 20, 2026, 07:42 CDT (Central Daylight Time, UTC-5)**. Diego is at his Mexico-City desk — Torre Manacar floor 14, corner office with a south-facing window overlooking the World Trade Center complex — drinking a black Café de Olla from the floor's espresso machine and reviewing the **last green-light dashboard** for **Aurelia Platform v4.0 cutover**. The v4.0 release is the **biggest contract-breaking change in the company's history**: it migrates the fleet-orchestration API from REST/HTTP-1.1 (v3.x) to **gRPC over HTTP/3 + QUIC** per ADR-0253, replaces the legacy session-token auth with **WebAuthn-passkey-derived workload identities** per ADR-0263, splits the monolithic `fleet-coordinator` µservice into **9 cellular sub-services** following ADR-0248 Amazon-shape Tiers 0–4, and introduces breaking changes to **47 published API contracts** with a six-month deprecation runway that started April 20, 2026 (the deprecation announcement) and reaches the cutover-hard-fence on **Friday October 30, 2026 23:59 UTC**.

The cutover is structured as a **four-cohort staged rollout** with explicit Cedar-permit go/no-go gates at each cohort transition:

1. **Cohort A (1% canary)** — Tuesday Oct 20, 08:00 CDT → 4 cells × 1% traffic (≈ 41 customer sites)
2. **Cohort B (10%)** — Wednesday Oct 21, 08:00 CDT (if A green) → 12 cells × 10% traffic
3. **Cohort C (50%)** — Friday Oct 23, 08:00 CDT (if B green) → 24 cells × 50% traffic
4. **Cohort D (100%)** — Tuesday Oct 27, 08:00 CDT (if C green) → all 47 Tier-1 cells × 100% traffic
5. **v3.x hard sunset** — Friday Oct 30, 23:59 UTC

This journey covers the **10 days from green-light review through 100%-cutover-stable** with the following spine of beats:

1. **Tue Oct 20 07:42–08:00 CDT** — final pre-cutover review; Diego signs the **Cohort-A go/no-go Cedar permit** at 07:58:42 CDT; the `cloud-iac` µservice executes the Terraform-module v-bump cascade across 4 canary cells
2. **Tue Oct 20 08:00–14:00 CDT** — Cohort A live; canary traffic enters via `feature-flags` µservice's split rules; observability SLO regression detector watches p99 latency, error budget, downstream-dependency-error-rate, and Cedar-policy-decision-latency
3. **Tue Oct 20 14:00 CDT** — **first canary-spike alarm**: production-engineer Sofía Ramírez in Querétaro pages Diego; p99 latency on the `dispatch-cell-qro` (Querétaro canary) jumped from 84 ms baseline to 312 ms over a 12-minute window
4. **Tue Oct 20 14:00–16:42 CDT** — war-room ops in Slack `#cutover-v4-warroom` (channel created 2026-09-15, locked to the cutover incident commanders); root cause traced to a Cedar-policy bytecode hot-cache miss for the new principal shape; mitigation = pre-warm the bytecode cache via a one-shot job
5. **Wed Oct 21 08:00 CDT** — Cohort B Cedar-permit go-vote: Diego + COO Akira Watanabe (cross-reference j168) + VP-Eng Yamilet Solís + SVP-Customer-Success Brian Tate; all 4 PERMIT votes captured; `governance` µservice's change-management workflow advances state `cohort_a_stable → cohort_b_initiating`
6. **Wed Oct 21–Fri Oct 23 CDT** — Cohort B steady-state; one minor regression (a customer's custom-policy bundle didn't compile under the new Cedar v4 evaluator); rollback NOT triggered; localized hotfix shipped via `feature-flags` per-tenant override
7. **Fri Oct 23 08:00 CDT** — Cohort C 50% gate; Cedar permit with 4-of-4 votes; rollout proceeds
8. **Sat Oct 24 22:14 CDT** — **SEV-2 alarm**: cell `dispatch-cell-aus-tx` (Austin) reports 18% error-budget burn over 90-minute window; root cause = a feature-flag staleness in one Kubernetes pod due to a CRD-watch lag; war-room reopens
9. **Sun Oct 25 09:18 CDT** — fix shipped; cell stabilizes; Cedar-permit "continue-rollout" vote passes 4-0
10. **Tue Oct 27 08:00 CDT** — Cohort D 100% Cedar permit vote: PERMIT 4-0; cutover proceeds; `observability` µservice's golden-signals dashboard goes solid-green at 10:42 CDT
11. **Wed Oct 28–Thu Oct 29 CDT** — stabilization period; v3.x traffic dwindles from 0.4% to 0.01% (long-tail customers' integrations finally migrate)
12. **Fri Oct 30 23:59 UTC** — v3.x hard sunset; `feature-flags` flips `v3_api_enabled` to `false` globally; legacy systems **Aurelia FleetSync v3.x**, **Aurelia GatewayBridge v3.x**, **Aurelia ContractAdapter v3.x** are dispatched into shutdown lifecycle; `governance` µservice closes the change-record at 23:59:42 UTC

Primary microservices: `feature-flags`, `cloud-iac`, `cloud-k8s`, `observability`, `governance`. Secondary: `compliance` (ISO-27001-A.12.1.2 + ITIL-v4-change-enablement + SOC2-CC8.1 + EU-AI-Act-Art-17), `identity` (workload-identity passkey rotation), `audit-chain` (every cohort gate dual-sealed), `messenger` (Slack-bridge cross-tenant), `tasks` (war-room runbook tasks), `notes` (Diego's decision-record annotations), `policy-engine` (Cedar bytecode cache pre-warm + per-cohort permit), `incident-management` (SEV-2 + 1 SEV-2 + 1 SEV-3), `slo-budgets` (per-service error-budget burn-down).

This is an **executive-platform-engineering, multi-cohort, change-management-heavy** journey. It demonstrates that oyatie's `feature-flags + cloud-iac + cloud-k8s + observability + governance` substrate, gated by ISO-27001-CM + ITIL-v4-CAB + SOC2-CC8 + EU-AI-Act-Art-17 packs, supports a hyperscaler-grade major platform version cutover with explicit Cedar-permit gates at each cohort, deterministic rollback decision trees, and audit-chain dual-sealed evidence. Diego is a competent CTO who has run cutovers before — including the v2.x → v3.x cutover in 2023 that took **17 days and cost MXN 4.2M in customer service-credits** because of an undetected back-pressure regression — and this journey shows that the v3 → v4 cutover lands in **10 days with MXN 142,000 in service-credits** (a 30× improvement) because the substrate enforces the gate discipline mechanically rather than relying on human vigilance.

## Why this journey matters

Diego Vargas is **MASTER-ROSTER §3.2 row 41** — the canonical CTO-of-mid-stage-public-company persona. He is the test bench for oyatie's claim that the same substrate that lets a small business win a bid (j160) also runs an industrial-robotics SaaS's platform cutover with all the change-management rigor that a publicly-listed company must demonstrate to its auditors (PwC México for ISO-27001 + SOC2; KPMG México for IFRS-software-development-cost capitalization treatment) and to its enterprise customers (47 of whom have contractual cutover-notification-windows ranging from 30 days to 180 days).

The persona covers an estimated **8,200 global mid-stage public-or-late-private SaaS companies** (USD 100M–USD 2B ARR) where the CTO is personally on the hook for change-management discipline, where one bad cutover can cost 0.5–2% of ARR in service-credits + cause a multi-week customer-trust deficit + trigger SOC2-CC8.1 audit findings + (for EU-AI-Act-Article-17-covered AI/ML systems) trigger a notified-body re-assessment. The category is acutely under-served by SaaS — there are change-management tools (ServiceNow Change, Jira Change, Atlassian Compass), there are feature-flag platforms (LaunchDarkly, Statsig, Split.io), there are observability platforms (Datadog, New Relic, Honeycomb), there are GitOps tools (Argo, Flux, Terraform Cloud) — but no integrated substrate that enforces **Cedar-permit-gated cohort transitions with audit-chain dual-sealed evidence and built-in rollback decision trees**, with the policy bundle attesting to ISO-27001-A.12.1.2 + SOC2-CC8.1 + EU-AI-Act-Art-17 + ITIL-v4-change-enablement obligations in a single attestation chain.

The journey closes:

- **Critical-path row 41** (Major version cutover with staged cohort rollout — 1% → 10% → 50% → 100%)
- **Critical-path row 42** (Cedar-permit go/no-go gate at each cohort transition with quorum)
- **Critical-path row 43** (Observability SLO regression detection with auto-triggered war-room)
- **Critical-path row 44** (Rollback decision tree — when to roll back vs hotfix vs continue)
- **Critical-path row 45** (Cloud-IaC Terraform module v-bump cascade across N cells with state-machine ordering)
- **Critical-path row 46** (Change-management audit-chain dual-seal — every cohort gate + every alarm + every decision sealed under TrueTime fence for IFRS-related capitalization attestation)

Hyperscaler benchmark: AWS Cell-based deployment + Google SRE cohort-rollout playbook + Meta Configerator + Microsoft Azure Safe Deployment Practices. The unique part of oyatie is that **Cedar policy gates each cohort transition AS A FIRST-CLASS PRIMITIVE** (not an after-the-fact ServiceNow approval ticket), AND that **the audit-chain dual-seals every gate under TrueTime fence with uncertainty ≤ 10 ms** so that a regulator or auditor can replay the exact decision-time temporal ordering without ambiguity, AND that **the rollback decision tree is encoded in the workflow-engine state machine** rather than living in a confluence runbook that goes stale.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat 10-day cutover from green-light review through v3.x hard sunset | Spanish-language CDT-Mexico-City office dialogue, named war-room participants (Sofía Ramírez Querétaro, Yamilet Solís Mexico City, Brian Tate Austin), specific Slack channels (`#cutover-v4-warroom`, `#aurelia-ops-onefloor14`, `#aurelia-customer-canary-cohort-a`), specific dashboard URLs + Grafana panel IDs, Cedar-permit votes with timestamps + principal IDs, named legacy systems being deprecated (Aurelia FleetSync v3.x + Aurelia GatewayBridge v3.x + Aurelia ContractAdapter v3.x), specific customer names anonymized as Customer A/B/C with industry tags, MXN/USD currency throughout |
| `ux-flow.md` | Diego's MacBook Pro M4 in Torre Manacar floor 14 + iPad Pro for cohort-review-meeting + Sofía Ramírez's NixOS workstation in Querétaro NOC + Brian Tate's iPhone 15 Pro for after-hours pages + Yamilet's ThinkPad X1 in CDMX office | Spanish-language primary UI for Diego-side; English-language for Austin-side; Cedar-permit modal with quorum-status pill; cohort-rollout dashboard with traffic-percentage + p99-latency + error-budget-burn three-pane layout; rollback-decision-tree wizard |
| `handshake.md` | Per-µservice API across `aurelia-robotics-internacional-sa-de-cv-mx` + `aws-cdmx-cell-tier-1-primary` + `aws-aus-tx-cell-tier-1-secondary` + `aws-qro-cell-tier-1-tertiary` + 44 additional Tier-1 cells | Each row names source + target tenant + cell, Cedar permit, cross-cell observability scrape, cohort-gate dual-seal class |
| `integration-test-plan.md` | Cohort transition tests + Cedar-permit quorum tests + observability regression tests + rollback decision tests + Terraform module v-bump cascade tests + canary-spike alarm tests + v3.x sunset tests | Each test names seed values + expected event chain + SLO threshold + Cedar policy assertion |
| `schemas/openapi-cutover.json` | OpenAPI for cohort go/no-go endpoints + feature-flag rollout endpoints + cloud-iac apply endpoints + observability SLO check endpoints + governance change-record endpoints | Cohort-state-transition state machine + Cedar permit context shape + rollback-trigger shape |
| `schemas/cedar-policy.cedar` | Cohort transition + canary-traffic + rollback + change-record Cedar policy | Per-cohort quorum gate + observability SLO precondition + business-hours window for non-emergency transitions + audit-chain dual-seal mandatory |
| `schemas/journey-messages.proto` | proto3 for all RPCs | Cohort state messages + Cedar permit messages + observability sample messages + rollback-trigger messages + change-record messages |
| `schemas/cutover-state-machine.yaml` | 11-state cutover lifecycle | `pre_review → cohort_a_initiating → cohort_a_stable → cohort_b_initiating → cohort_b_stable → cohort_c_initiating → cohort_c_stable → cohort_d_initiating → cohort_d_stable → stabilization → v3_sunset_complete`; Cedar guards per transition; rollback edges from each `_stable` back to previous `_stable` |
| `schemas/cohort-rollout-config.json` | Per-cohort config: percentage, cell allocation, SLO thresholds, alert routing | Cohort A: 1% traffic, 4 cells, p99 ≤ 200ms, page Diego+Sofía; Cohort B: 10% traffic, 12 cells, p99 ≤ 180ms; Cohort C: 50% traffic, 24 cells, p99 ≤ 160ms; Cohort D: 100% traffic, 47 cells, p99 ≤ 150ms |

## The five microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `feature-flags` | Per-cohort traffic-split rules; per-tenant override during hotfix; v3_api_enabled flag for sunset | rows 41 + 42 |
| `cloud-iac` | Terraform module v-bump cascade across 47 cells; per-cell apply with ordering constraint; drift detection | row 45 |
| `cloud-k8s` | Per-cell K8s namespace deploy of v4 workloads; pod-rolling-strategy; CRD-watch lag monitoring | row 45 |
| `observability` | Golden-signals dashboard; SLO regression detector; auto-trigger war-room on threshold breach | row 43 |
| `governance` | Change-management workflow; cohort-gate Cedar-permit quorum vote; audit-chain dual-seal; CRA (change risk assessment) document store | rows 41 + 42 + 46 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `compliance` | Activates ISO-27001-A.12.1.2 (change-management), ITIL-v4 (change-enablement), SOC2-CC8.1 (change-management), NIST-800-53-CM-3 (configuration-change-control), EU-AI-Act-Art-17 (quality-management-system for high-risk AI), ISO-22301 (business-continuity) |
| `identity` | Workload-identity passkey rotation as part of v4 cutover; per-cell SPIFFE-ID re-issuance |
| `audit-chain` | Every cohort gate + every Cedar permit vote + every alarm + every rollback decision dual-sealed under TrueTime |
| `messenger` | Slack-bridge for `#cutover-v4-warroom` + `#aurelia-ops-onefloor14` + `#aurelia-customer-canary-cohort-a`; MLS-encrypted internal messages |
| `tasks` | War-room runbook task materialization; per-cohort go/no-go review checklist (87 items per cohort × 4 cohorts) |
| `notes` | Diego's decision-record annotations; Sofía's NOC incident log; Yamilet's VP-Eng readiness notes |
| `policy-engine` | Cedar bytecode cache pre-warm; per-cohort permit evaluation; per-tenant override compile |
| `incident-management` | 1 SEV-2 (the Austin cell error-budget burn Saturday Oct 24); 1 SEV-3 (the customer custom-policy bundle compile failure during Cohort B) |
| `slo-budgets` | Per-service error-budget burn-down; cohort-gate SLO precondition check |
| `analytics` | Cutover-velocity metric; service-credit-burn metric; customer-migration-completion-percent metric |

## Pack overlays

| Pack | Activation reason |
|---|---|
| ISO-27001-A.12.1.2 | Change management; mandatory for any platform change with customer impact |
| ITIL-v4-change-enablement | CAB (Change Advisory Board) record + CRA (Change Risk Assessment); maps to Cedar-permit quorum vote |
| SOC2-CC8.1 | Change management criteria; auditor PwC México requires evidence at year-end |
| NIST-800-53-CM-3 | Configuration change control; for US-federal-customer-eligible workloads (3 of Aurelia's customers are US-federal-adjacent) |
| EU-AI-Act-Art-17 | Quality management system for high-risk AI (Aurelia's robot-fleet-coordination contains an AI-driven path-planning module classified Annex-III high-risk) |
| ISO-22301 | Business continuity; rollback decision tree maps to ISO-22301 BCP |
| IFRS-IAS-38 | Software development cost capitalization; KPMG México requires evidence that the v4 cutover is a discrete capital event |
| MX-NOM-151-SCFI-2016 | Mexican conservation-of-data-messages standard; cutover audit-chain compliant |

## Regulatory anchors

1. ADR-0248 Amazon-shape cellular architecture (Tier-0 to Tier-4; cohort rollout respects cell-tier ordering)
2. ADR-0244 tenant scoping primitive
3. ADR-0263 audit dual-seal on cross-tenant cohort transitions
4. ADR-0252 HLC + TrueTime for cohort-gate signing fence
5. ADR-0110 changeset state machine (the cutover IS a changeset traversing the state machine)
6. ADR-0111 merge-queue projected state fix at any stage (rollback semantics)
7. ISO/IEC 27001:2022 A.12.1.2 (change management)
8. SOC2-2017-CC8.1 (change-management criteria)
9. NIST-SP-800-53-Rev5 CM-3 (configuration change control)
10. EU-AI-Act Article 17 (quality management system for high-risk AI systems)
11. ISO-22301:2019 §8.4 (business continuity strategies)
12. ITIL v4 (change enablement practice)
13. IFRS / IAS 38 §57 (intangible asset capitalization criteria — internally developed software)

## Cell + certification matrix

| Cell | Certification | Journey use |
|---|---|---|
| `aws-cdmx-cell-tier-1-primary` | ISO 27001 + SOC2 + MX-NOM-151 | Primary cell for Aurelia tenant + 8 Mexican customer tenants |
| `aws-aus-tx-cell-tier-1-secondary` | ISO 27001 + SOC2 + FedRAMP-Moderate-equivalent | Texas cell; 11 US customer tenants; the SEV-2 occurred here |
| `aws-qro-cell-tier-1-tertiary` | ISO 27001 + SOC2 + MX-NOM-151 | Querétaro cell; NOC primary; Sofía Ramírez's home cell |
| `aws-gdl-cell-tier-1-quaternary` | ISO 27001 + SOC2 + MX-NOM-151 | Guadalajara cell; canary cohort A member |
| ...47 cells total across the cohort plan | Various per region | Per cohort C + D rollout |

## Cedar cohort-gate policy (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
// Cohort transition Cedar gate — quorum vote + SLO precondition + business-hours window
permit (
    principal,
    action == Action::"governance.cohort_transition_vote",
    resource is CohortGate
) when {
    resource.target_cohort in ["cohort_a", "cohort_b", "cohort_c", "cohort_d"] &&
    resource.quorum_count >= 4 &&
    resource.previous_cohort_slo_green == true &&
    resource.cra_document_signed == true &&
    context.business_hours_cdt == true &&
    principal in Group::"aurelia-cutover-quorum-members"
};

// Rollback Cedar gate — looser quorum (3-of-4) but must be triggered within 4-hour window of alarm
permit (
    principal,
    action == Action::"governance.cohort_rollback",
    resource is CohortGate
) when {
    resource.quorum_count >= 3 &&
    resource.active_sev_level in ["sev-1", "sev-2"] &&
    context.minutes_since_alarm <= 240 &&
    principal in Group::"aurelia-cutover-incident-commanders"
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J167-001 | Pre-cutover review Tuesday Oct 20 07:42 CDT renders 87-item readiness checklist; all 87 items green; audit `EVT-J167-PRE-REVIEW-COMPLETE-001` sealed |
| AC-J167-002 | Cohort A Cedar-permit vote at 07:58:42 CDT: 4-of-4 PERMIT; audit `EVT-J167-COHORT-A-PERMIT-002` dual-sealed under TrueTime ≤ 10 ms |
| AC-J167-003 | Cohort A traffic-split active at 08:00:00 CDT; 1% canary traffic on 4 cells (CDMX + AUS-TX + QRO + GDL); audit `EVT-J167-COHORT-A-LIVE-003` |
| AC-J167-004 | Canary-spike alarm at 14:00 CDT: p99 latency on dispatch-cell-qro from 84ms → 312ms over 12-min window; observability SLO regression detector fires; auto-page to Diego + Sofía + Yamilet; audit `EVT-J167-CANARY-SPIKE-ALARM-004` |
| AC-J167-005 | Root cause traced within 38 minutes (Cedar bytecode hot-cache miss); mitigation = pre-warm cache one-shot job; audit `EVT-J167-MITIGATION-APPLIED-005` |
| AC-J167-006 | Cohort B Cedar-permit vote Wednesday Oct 21 08:00 CDT: 4-of-4 PERMIT; audit `EVT-J167-COHORT-B-PERMIT-006` dual-sealed |
| AC-J167-007 | Cohort C Cedar-permit vote Friday Oct 23 08:00 CDT: 4-of-4 PERMIT; audit `EVT-J167-COHORT-C-PERMIT-007` dual-sealed |
| AC-J167-008 | SEV-2 alarm Saturday Oct 24 22:14 CDT on cell `dispatch-cell-aus-tx`: 18% error-budget burn 90-min window; CRD-watch-lag root cause; war-room reopened; fix shipped Sunday 09:18 CDT; NO rollback triggered (within 4-hour mitigation window); audit `EVT-J167-SEV2-AUS-TX-008` |
| AC-J167-009 | Cohort D Cedar-permit vote Tuesday Oct 27 08:00 CDT: 4-of-4 PERMIT; cutover proceeds to 100%; audit `EVT-J167-COHORT-D-PERMIT-009` dual-sealed |
| AC-J167-010 | v3.x hard sunset Friday Oct 30 23:59 UTC; `feature-flags` flips `v3_api_enabled` to `false` globally; legacy Aurelia FleetSync v3.x + GatewayBridge v3.x + ContractAdapter v3.x enter shutdown lifecycle; audit `EVT-J167-V3-SUNSET-010` |
| AC-J167-011 | Total service-credits issued = MXN 142,000 (vs MXN 4.2M from v2→v3 cutover; 30× improvement); SOC2-CC8.1 audit evidence assembled; ISO-27001-A.12.1.2 attestation packet generated |
| AC-J167-012 | Audit-chain dual-seal invariant: every cohort permit vote + every alarm + every rollback decision dual-sealed in Aurelia tenant AND in `oya-governance-change-management-system-tenant` under TrueTime fence ≤ 10 ms |
| AC-J167-013 | EU-AI-Act-Art-17 attestation: the AI-driven path-planning module's QMS re-attestation post-cutover signed by Yamilet Solís (VP-Eng) and CTO Diego Vargas; notified-body re-assessment NOT required because cutover preserves AI module's safety-relevant interfaces |
| AC-J167-014 | Cutover concludes Thursday Oct 29 with `cutover_complete=true` in governance state machine; CRA document archived; change-record closed |

## Cross-references

- Persona dossier: `docs/personas/executive-cto-diego-vargas.md`
- MASTER-ROSTER §3.2 row 41
- Matrix §7 j167 recommendation
- Related: j168 (COO ops review — Akira Watanabe references Diego's cutover in her quarterly review), j169 (CMO multi-country launch — uses the same feature-flags substrate), j166 (CSO strategic acquisition — pre-cutover diligence), j41 (initial v3.x → v4.x deprecation announcement journey), j112 (tenant-to-tenant RFQ + bid — same governance substrate)
- Pack roster: `packs/iso-27001-a-12-1-2/`, `packs/itil-v4-change-enablement/`, `packs/soc2-cc8-1/`, `packs/nist-800-53-cm-3/`, `packs/eu-ai-act-art-17/`, `packs/iso-22301/`, `packs/ifrs-ias-38/`, `packs/mx-nom-151-scfi-2016/`
- ADR-0248 cellular architecture
- ADR-0244 tenant scoping
- ADR-0263 audit dual-seal
- ADR-0252 TrueTime fence
- ADR-0110 changeset state machine
- ADR-0111 merge queue projected state

## Stop condition

This journey is complete when all 14 acceptance criteria pass on the seeded fixture (Aurelia tenant + 47 cell fixtures + 4 named quorum-member identities + 1 SEV-2 + 1 SEV-3 + 1 canary-spike alarm + the v3.x sunset cascade), the cutover state machine reaches `v3_sunset_complete`, the audit-chain dual-seal invariant holds across all gate decisions, the total service-credit MXN-burn is ≤ MXN 200,000, and the EU-AI-Act-Art-17 QMS re-attestation completes without triggering a notified-body re-assessment.
