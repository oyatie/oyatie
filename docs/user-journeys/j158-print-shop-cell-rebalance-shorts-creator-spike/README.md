---
doc_class: User-Journey-README
journey_id: j158-print-shop-cell-rebalance-shorts-creator-spike
slice: dual-tenant-employer-side-cell-rebalance-driven-by-personal-side-shorts-content-spike
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Mailroom Hae-Won Kim (cross-context with shorts creator side-business)
audience_type: B2B_BACK_OFFICE + B2C_CONSUMER_CREATOR + DUAL_TENANT
microservice_count: 4
pack_overlay_anchor: KR-Labour-Standards-Act + KR-PIPA + EU-Cell-Scaling-policy + B2C-Creator-Pack + Workplace-Side-Business-Disclosure
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0263-observability-emission-contract
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0252-hlc-default-truetime-tier
  - ADR-0249-multi-category-marketplace-doctrine
  - ADR-0255-intelligence-two-layer-substrate
---

# j158 — Print shop cell rebalance triggered by Hae-Won's shorts creator spike

## At a glance

Hae-Won Kim (김해원) is a **mailroom operator and inter-departmental logistics coordinator** at **Sungkyul-Sangsa Print Shop Co.** (성결상사 인쇄소; "Sungkyul-Sangsa" — a Seoul-based commercial print shop in the Mapo-gu district, established 1973 by the current owner's grandfather). She is 29 years old, Korean, born in Daegu, graduated Seoul National University of Science and Technology (서울과학기술대학교) graphic communications, lives in Mapo-gu walking-distance from the plant, and has been at Sungkyul-Sangsa for 5 years.

She is also — and this is the critical fact — **a part-time short-form video creator** on her personal time. Her creator handle is `@haewon_paperlife` (1.4M followers on Korean short-video platforms, 380K on global oyatie shorts). Her content niche: 30-second behind-the-scenes from a Korean print shop — paper-folding ASMR, the squelch of fountain solution rollers, the smell of fresh ink translated through visual composition. She has done this since 2024 and her audience has grown from 12K to 1.4M in 18 months. She filmed every short on her own phone outside her work shifts, with her employer's written consent (her employer, Lee Min-Jun, is mildly amused by the attention and signed a "personal-creator side-business disclosure" back in 2024-08-12).

Her shorts side-business is a **personal-tenant** activity (`personal-haewon-kim-kr`) entirely separate from her **employer-tenant** workspace (`sungkyul-sangsa-print-co-kr`). The two tenants do NOT share data under ADR-0311. Cedar policy is surgical at the boundary — her employer cannot see her creator metrics, her creators-pool followers cannot see her work identity.

It is **Wednesday March 17, 2027, 14:18 KST**. Three days ago, on Sunday March 14, Hae-Won posted a 28-second short titled "**8시간 동안 종이 접는 소리만 (eight hours of folding paper sounds)**" — a meditative ASMR piece of the print shop's folding line. The short went viral starting Monday afternoon. By Wednesday 14:18 KST it has 21.7M views and is trending #2 in Korean short-video platforms.

The viral spike is doing something unexpected: **her shorts µservice cell (cell `kr-seoul-shorts-creator-tier-4`) is consuming compute at 8.4× the capacity-plan baseline**. The platform's autoscaling responds at the consumer-side cell. But the spike is also driving a real business effect at the employer side — small print-shop owners across Seoul are messaging Sungkyul-Sangsa because the shop is having a moment of cultural relevance, and the shop's order intake has jumped 3.7× over Mon–Wed.

The **print-shop cell** at Sungkyul-Sangsa side (`kr-seoul-employer-print-shop-mid-volume`) is now near capacity ceiling. The plant manager Lee Min-Jun and Hae-Won need to coordinate a **cell rebalance** — re-allocating compute + workflow-engine throughput to handle the 3.7× order intake — without bleeding any data across the dual-tenant boundary.

This journey covers the next 4 hours of Hae-Won's professional life (14:18–18:42 KST):

1. **Tasks** µservice surfaces the queued workload + the rebalance request at the employer-tenant side; Hae-Won (in her work-tenant role as logistics coordinator) drives capacity reallocation across the four print-shop cells the company runs
2. **Workflow-engine** drives the cell-rebalance workflow with a **5-stage state machine** (`capacity_signal_detected` → `rebalance_proposed` → `cross_cell_grant_negotiated` → `traffic_shift` → `post_rebalance_validation`); each transition Cedar-gated; the rebalance is **internal to the employer tenant** so no cross-tenant data flows
3. **Shorts** µservice (consumer side, in Hae-Won's personal tenant) drives the autoscale on its own cell; the personal-side observability emits a signal to the employer-side IF AND ONLY IF Hae-Won explicitly grants a one-way correlation permit — which she does, because she wants Lee Min-Jun to know what's happening so he can capitalize on the moment professionally
4. **Messenger** carries the explicit cross-tenant signal: a one-way info-only message from Hae-Won's personal tenant ("hey, the short is at 21M; expect 3-4× order intake; I can help if needed") to her employer-tenant principal; the message exists at the **dual-tenant boundary** per ADR-0311 with explicit Cedar permits per ADR-0311 §dual-tenant-boundary-with-creator-employer-disclosure

Microservices: `tasks`, `workflow-engine`, `shorts`, `messenger`. Secondary: `identity` (dual-tenant binding), `tenancy` (two tenants in scope), `observability` (cell capacity telemetry), `cell` (cell-bind operations), `analytics` (creator metrics on personal side; order-intake metrics on employer side), `compliance` (KR-LSA + KR-PIPA + creator-disclosure pack), `audit-chain`.

This is a **dual-tenant cross-context** journey demonstrating that:

(a) the consumer side (shorts creator viral spike) and the employer side (print-shop order intake spike) are **causally linked but data-isolated**;
(b) a person can **proactively cross the boundary** with an explicit Cedar permit and explicit creator-employer disclosure rather than passively letting data bleed;
(c) the cell-rebalance is an internal operational concern of the employer tenant — the consumer-side personal-tenant is NOT involved in the rebalance compute decisions; it is involved only as a **causal signal**.

## Why this journey matters

Hae-Won Kim is **MASTER-ROSTER §3.4 row 102** — the canonical blue/back-office persona with an active personal-tenant consumer-creator side-business. This persona covers an estimated 8% of Gen-Z and millennial workers in OECD economies who hold a primary day job AND an active creator/streamer/content business. The category is acutely under-served because most enterprise software refuses to acknowledge the side-business; most consumer software refuses to acknowledge the day job; the two contexts collide messily.

The journey closes:

- **Critical-path row 29** (Dual-tenant cross-context with explicit boundary crossing — creator side notifies employer side at user's discretion)
- **Critical-path row 30** (Cell rebalance driven by external causal signal; internal tenant-bounded execution)
- **Critical-path row 31** (Consumer-side viral spike → consumer-side autoscale → causal signal → employer-side capacity decision)
- **Critical-path row 32** (Creator-employer side-business disclosure as first-class Cedar permit)

Hyperscaler benchmark: TikTok/Douyin's creator-monetization platform + a separate B2B Workday/SAP environment + no integration between them. The unique part of oyatie is that **the same human can hold both contexts cleanly under one identity root**, can **explicitly bridge data with surgical Cedar permits**, and the **cell architecture rebalances internally without cross-tenant compute coupling**.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat 14:18 KST signal → 18:42 KST rebalance validated | Korean diacritic/Hangul preservation, specific neighborhoods (Mapo-gu, Hongdae), specific shorts metrics (21.7M views, 3.7× order intake), specific cell IDs |
| `ux-flow.md` | Hae-Won's phone (work tenant + personal tenant dual view) + Lee Min-Jun's desktop + the cell-rebalance ops console | Active-tenant pill behavior; the cell-rebalance dashboard; the explicit disclosure-bridge screen |
| `handshake.md` | Per-µservice API + dual-tenant scoping | Each row names source tenant + target tenant + Cedar permit + dual-tenant disclosure context |
| `integration-test-plan.md` | Cell rebalance tests + dual-tenant boundary fuzz + explicit disclosure permit + autoscale signal tests | Each test names seed values + expected event chain + dual-seal pass/fail |
| `schemas/openapi-cell-rebalance.json` | OpenAPI for cell-rebalance lifecycle endpoints | All 5 rebalance stages + capacity-signal endpoint |
| `schemas/cedar-policy.cedar` | Dual-tenant boundary + creator-employer disclosure Cedar policy | Personal-tenant write → employer-tenant read-info-only permit |
| `schemas/journey-messages.proto` | proto3 for all RPCs | Hangul-safe string handling; explicit disclosure context field |
| `schemas/cell-rebalance-state-machine.yaml` | 5-state rebalance lifecycle | Per-state Cedar guards + audit class + autoscale gates |
| `schemas/creator-employer-disclosure-form.json` | Side-business disclosure form schema | Required fields + signed-by-employer + signed-by-employee + active period |

## The four microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `tasks` | Surfaces order-intake queue + the rebalance workitem on Hae-Won's employer-tenant dashboard | row 29, 30 |
| `workflow-engine` | Drives the 5-state cell-rebalance lifecycle; Cedar-gated transitions; cross-cell grant negotiation | row 30 |
| `shorts` | Consumer µservice in Hae-Won's personal tenant; emits autoscale telemetry to its own cell; cross-link signal to employer side gated by disclosure-permit | row 31 |
| `messenger` | Carries the explicit cross-tenant disclosure message Hae-Won (personal) → Hae-Won (employer-role) + Lee Min-Jun | row 29, 32 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `identity` | Hae-Won's passkey root + dual-tenant binding + creator-employer disclosure record |
| `tenancy` | Two tenants in scope: `personal-haewon-kim-kr` + `sungkyul-sangsa-print-co-kr`; lifecycle isolation per ADR-0311 |
| `observability` | Cell capacity telemetry on both sides; cell-bound metric streams |
| `cell` | Cell-bind operations for the employer tenant's 4 cells; cross-cell capacity grants |
| `analytics` | Personal-side: creator metrics (views, watch-time, retention); employer-side: order-intake trend |
| `compliance` | Activates KR-LSA (Labour Standards Act), KR-PIPA (Personal Information Protection Act), Creator-Disclosure pack |
| `audit-chain` | Per-tenant audit streams; cross-tenant correlation gated by disclosure-permit context |
| `production-planning` | Translates the 3.7× order intake into additional press slots + paper inventory + binding capacity |
| `crm` | Logs the incoming inquiries from new SMB customers (employer side) |

## Pack overlays

| Pack | Activation reason |
|---|---|
| KR-Labour-Standards-Act | Korean labour law: §17 maximum weekly hours; §50 overtime premium; §54 mandatory rest |
| KR-PIPA | Korean Personal Information Protection Act: consent-bound personal data flow |
| EU-Cell-Scaling-policy | Cellular architecture ops standards (oyatie internal); applies to all regions |
| B2C-Creator-Pack | Consumer-creator µservice pack: monetization, copyright, audience-data governance |
| Workplace-Side-Business-Disclosure | Employer-aware side-business pack: surfaces disclosure consent + boundary policy |
| Cross-tenant-disclosure-permit-v1 | Explicit one-way info-only permit class (introduced for ADR-0311 §dual-tenant-boundary-with-creator-employer-disclosure) |

## Regulatory anchors

1. KR-Labour Standards Act (근로기준법) §17 + §50 + §54
2. KR-PIPA (개인정보 보호법) — consent for cross-context personal data flow
3. KR-Promotion of Information and Communications Network Utilization and Information Protection Act (정보통신망법)
4. ADR-0311 dual-tenant identity boundary
5. ADR-0244 tenant scoping
6. ADR-0248 cellular architecture: cell capacity + rebalance discipline
7. ADR-0263 observability emission contract: signal-only cross-tenant correlation
8. ADR-0249 multi-category marketplace doctrine (creator-pack is one category)

## Cell + certification matrix

| Cell | Role | Journey use |
|---|---|---|
| `kr-seoul-shorts-creator-tier-4` | Personal-tenant tier-4 (high-throughput consumer creator) | Hosts Hae-Won's @haewon_paperlife shorts; autoscaling during viral spike |
| `kr-seoul-employer-print-shop-mid-volume` | Employer-tenant primary cell | Hosts Sungkyul-Sangsa workflows + tasks + crm |
| `kr-seoul-employer-print-shop-burst-1` | Employer-tenant burst cell #1 | Sister cell brought online during rebalance |
| `kr-seoul-employer-print-shop-burst-2` | Employer-tenant burst cell #2 | Second burst cell |
| `kr-seoul-employer-print-shop-secondary` | Employer-tenant secondary cell | Read replica + DR |
| `kr-busan-employer-readonly-replica` | DR replica | Cross-region replica per KR-PIPA |

## Cedar dual-tenant disclosure permit (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
// Hae-Won (personal-tenant principal) grants info-only signal to her employer-tenant principal
// This is the disclosure-permit that ADR-0311 introduces.
// The permit is one-way (personal → employer signal), info-only (no data structure crosses;
// only a coarse-grained "spike happening" signal), and audited dual-seal.

permit (
    principal == User::"haewon.kim@personal-haewon-kim-kr",
    action == Action::"messenger.creator_employer_disclosure_signal",
    resource is Tenant
) when {
    resource.tenant_id == "sungkyul-sangsa-print-co-kr" &&
    principal.has_active_disclosure_record(
        disclosure_id = "disclosure-haewon-kim-sungkyul-sangsa-2024-08-12"
    ) &&
    context.payload_class == "creator_spike_info_only" &&
    context.payload_no_audience_pii == true &&
    context.payload_no_revenue_figures == true &&
    context.payload_max_size_bytes <= 1024
};

// The reverse direction (employer → personal) is FORBID by default; employer never queries
// personal-tenant data even with disclosure active. The disclosure is one-way.
forbid (
    principal,
    action == Action::"query.personal_tenant_from_employer",
    resource is Tenant
) when {
    resource.tenant_id == "personal-haewon-kim-kr" &&
    principal.acting_tenant == "sungkyul-sangsa-print-co-kr"
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J158-001 | Personal-side autoscale on `kr-seoul-shorts-creator-tier-4` engages at ≥4× baseline; audit `EVT-J158-AUTOSCALE-PERSONAL-001` sealed in `personal-haewon-kim-kr` only |
| AC-J158-002 | Hae-Won (personal-tenant) issues info-only disclosure signal to her employer-tenant principal via messenger; audit `EVT-J158-DISCLOSURE-SIGNAL-002` dual-sealed |
| AC-J158-003 | Employer-side `tasks` µservice surfaces the rebalance workitem to Hae-Won (logistics coordinator role) + Lee Min-Jun |
| AC-J158-004 | Cell-rebalance workflow reaches `traffic_shift` state within 90 min of disclosure signal; audit `EVT-J158-REBALANCE-TRAFFIC-SHIFT-004` |
| AC-J158-005 | Burst cells `kr-seoul-employer-print-shop-burst-1` + `burst-2` provisioned; cross-cell capacity grants Cedar-validated |
| AC-J158-006 | Production-planning ingests 3.7× order-intake forecast; additional press slots + paper inventory + binding capacity allocated |
| AC-J158-007 | KR-LSA §17 weekly-hours evaluator passes for all staff impacted by additional shifts |
| AC-J158-008 | Reverse cross-tenant probe (employer → personal) denied; `EVT-J158-CEDAR-DENY-EMPLOYER-TO-PERSONAL-008` dual-sealed |
| AC-J158-009 | Post-rebalance validation at 18:42 KST: capacity headroom restored; latency p95 within SLA; audit `EVT-J158-POST-REBALANCE-VALIDATION-009` |
| AC-J158-010 | Hangul + Hanja preservation: 김해원 + 성결상사 + 마포구 stored byte-exact across both tenants |

## Cross-references

- Persona dossier: `docs/personas/mailroom-haewon-kim.md`
- MASTER-ROSTER §3.4 row 102
- Matrix §10 j158 recommendation
- Related: j155 (dual-tenant identity student + employee), j156 (cross-tenant emergency vendor), j157 (operator-authoritative line stop), j119 (creator monetization)
- Pack roster: `packs/kr-lsa/`, `packs/kr-pipa/`, `packs/b2c-creator/`, `packs/workplace-side-business-disclosure/`
- ADR-0311 dual-tenant identity boundary
- ADR-0244 tenant scoping
- ADR-0248 cellular architecture
- ADR-0249 multi-category marketplace
- ADR-0255 intelligence two-layer

## Stop condition

This journey is complete when all 10 acceptance criteria pass on the seeded two-tenant fixture, the cell-rebalance reaches `post_rebalance_validation`, the disclosure permit holds dual-seal across the personal→employer signal, the reverse direction denies, the Hangul preservation invariant holds, and the KR-LSA weekly-hours evaluator stays green throughout the burst period.
