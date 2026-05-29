---
doc_class: User-Journey-README
journey_id: j166-cso-mira-goldberg-strategic-acquisition-go-no-go
slice: cso-strategic-acquisition-go-no-go-cross-tenant-due-diligence-financial-model-ml-forecast-board-resolution
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: CSO Mira Goldberg (white/executive; Chief Strategy Officer)
audience_type: B2B_EXECUTIVE + STRATEGY_OFFICER + M_AND_A_DECISION_MAKER
microservice_count: 5
pack_overlay_anchor: HSR-Act + EU-Merger-Control + UK-CMA-Merger + GDPR + SEC-Pre-IPO + SOC-2 + NDA-Cross-Tenant
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0263-observability-emission-contract
  - ADR-0251-compliance-pack-primitive
  - ADR-0255-intelligence-two-layer-substrate
  - ADR-0245-substrate-vs-product-layering
  - ADR-0252-hlc-default-truetime-tier
  - ADR-0253-http3-quic-default-protocol
---

# j166 — CSO Mira Goldberg drives a $186M acquisition go/no-go decision in 9 days

## At a glance

Mira Goldberg (מירה גולדברג) is a **39-year-old Chief Strategy Officer** at **Skylark Logistics Solutions Inc.**, a Boston-headquartered last-mile supply-chain SaaS company (~480 employees, $148M ARR, Series D, profitable since 2025-Q2). Mira is American-Israeli (dual citizen; mother from Tel Aviv), born in Newton MA in 1987, MBA Wharton 2014, joined Skylark from a Big 4 strategy consulting practice in 2023-09. She reports to the CEO Adrian Cheng-Whitford and to the Strategy + M&A committee chair Director Hannah Beauregard.

It is **Friday May 15, 2027, 07:42 EDT**. The Skylark board approved exploration of a strategic acquisition target two months ago. After 8 weeks of preliminary diligence, Mira is in the final week before the **board go/no-go decision on Monday May 25 at 09:00 EDT** — 7 business days away.

The target: **Mendelsohn Routing Technologies Inc.** ("MRT"), a Berlin-headquartered route-optimization software company. $42M ARR, 138 employees, profitable since 2024-Q4. The deal terms under consideration:

- **Price range**: $172M–$202M (Skylark's working number: $186M = 4.4× ARR)
- **Structure**: 60% cash + 40% Skylark stock (post-money valuation $920M; Skylark's last round priced $1.1B)
- **Regulatory filings**: HSR (US, Hart-Scott-Rodino), EU Merger Control (Regulation 139/2004), UK CMA (Tier 2 voluntary notification), Israeli IMC (parties review)
- **Earnout**: $30M conditional on MRT's lead PM Bjorn Mendelsohn (the founder) remaining post-close for 24 months
- **Closing target**: Q3 2027 (estimated 100 days post board approval)

The journey covers Mira's **9 days** (May 15–25) of:

1. **financial-planning** µservice — Mira's M&A model with ARR/CAC/LTV/payback/CAC-cohort inputs from MRT (received via the cross-tenant due-diligence channel); scenario analysis at $172M / $186M / $202M
2. **intelligence** µservice — ML-driven scenario modeling (Monte-Carlo simulation of MRT's revenue trajectory under 3 macro scenarios; ML-driven customer-churn forecasting using MRT's anonymized customer cohort data; ML-driven integration-cost forecast)
3. **compliance** µservice — pack-manifest cross-check between Skylark + MRT (does MRT's compliance posture create blockers for Skylark's existing customer base?); HSR/EU-MR/UK-CMA filing requirements computation
4. **governance** µservice — board-resolution Cedar gate + audit committee approval flow + the go/no-go decision recording with Merkle anchor
5. **connect** µservice — the cross-tenant NDA-bound due-diligence channel between Skylark + MRT; the secure room for sharing material non-public information

Microservices: `governance`, `financial-planning`, `intelligence`, `compliance`, `connector`. Secondary: `identity`, `tenancy`, `audit-chain`, `notes`, `drive`, `observability`, `cell`, `messenger` (executive-only channel).

## Why this journey matters

Mira Goldberg is **MASTER-ROSTER §3.4 row 234** — the canonical CSO persona at a mid-market growth-stage B2B SaaS company driving M&A. This persona covers ~6,800 CSO-class roles globally in tech-enabled growth companies (BLS 2024 code 11-1011 narrowed to "Chief Strategy"). M&A decision support is one of the most consequential workflows a CSO drives; getting it wrong destroys hundreds of millions in shareholder value.

The journey closes:

- **Critical-path row 102** (Strategic M&A go/no-go with multi-jurisdiction regulatory filing pre-screening — HSR + EU + UK + Israeli)
- **Critical-path row 103** (ML-driven scenario modeling as a first-class capability — Monte-Carlo + cohort churn + integration cost)
- **Critical-path row 104** (Cross-tenant NDA-bound due-diligence channel — the connect µservice's M&A-class secure room)
- **Critical-path row 105** (Pack-manifest cross-check as M&A blocker detection — target's compliance posture vs acquirer's customer base)
- **Critical-path row 106** (Board-resolution Cedar gate for go/no-go decision recording — Merkle-anchored decision provenance)

Hyperscaler benchmark: traditional M&A tools (DealCloud + Intralinks + Datasite) handle the data-room workflow; specialized scenario tools (Pitchbook + S&P Capital IQ) handle valuation; pack-manifest cross-check at this scale is unique to oyatie. ML-driven scenario modeling is partially served by Palantir Foundry but not as an inline computation during decision-prep. The cross-tenant NDA-bound channel with Cedar permits is unique to oyatie's [[substrate-vs-product]] architecture.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat May 15 07:42 EDT → May 25 09:00 EDT across 9 days of analysis | Boston Seaport spring; specific deal terms; named board members + counsel + audit committee; pack manifests; specific financial numbers |
| `ux-flow.md` | Mira's M&A console + financial-planning canvas + ML scenario explorer + NDA-bound channel + pack-cross-check matrix + board-resolution gate | Per-screen Cedar permit + specific metrics + ML provenance metadata + cross-tenant boundary indicator |
| `handshake.md` | Per-µservice API + cross-tenant NDA channel + ML inference flow + pack cross-check | Each row names cross-tenant boundary + Cedar permit + audit class |
| `integration-test-plan.md` | Financial model determinism + ML scenario reproducibility + cross-tenant boundary fuzz + Cedar deny coverage + Merkle anchor | Per-test seed + per-scenario reproducibility + boundary invariant |
| `schemas/openapi-acquisition-decision.json` | OpenAPI for acquisition-decision endpoints | All 5 lifecycle stages + cross-tenant + ML inference |
| `schemas/cedar-policy.cedar` | Acquisition Cedar policy | CSO + counsel + CFO + audit-committee + board permits; NDA-bound cross-tenant rules |
| `schemas/journey-messages.proto` | proto3 for all RPCs | Hebrew + German + Hangul preservation; cross-tenant channel envelopes |
| `schemas/acquisition-state-machine.yaml` | 6-state acquisition lifecycle | due_diligence → financial_model → ml_scenarios → counsel_review → board_resolution → decision_recorded |
| `schemas/m-and-a-deal-form.json` | M&A deal-form schema | Price + structure + earnout + filings + scenarios + cross-tenant evidence |

## The five primary microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `governance` | Board-resolution Cedar gate + audit-chain Merkle anchor + decision recording | row 106 |
| `financial-planning` | M&A model with ARR/CAC/LTV inputs; scenario analysis at 3 prices | row 102 |
| `intelligence` | ML-driven Monte-Carlo + cohort churn + integration cost forecast | row 103 |
| `compliance` | Pack-manifest cross-check + multi-jurisdiction filing requirements | row 102, 105 |
| `connector` | Cross-tenant NDA-bound due-diligence channel | row 104 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `identity` | Mira's passkey + YubiKey 5C NFC; counsel + CFO + audit-committee + board passkeys; MRT-side principals |
| `tenancy` | Skylark tenant `skylark-logistics-solutions-inc` + MRT tenant `mendelsohn-routing-technologies-inc-de`; cross-tenant boundary |
| `audit-chain` | Per-document Merkle seal + decision-record Merkle anchor + external transparency log |
| `notes` | Mira's working diligence notes + meeting prep + scenarios |
| `drive` | Final M&A documentation drive room `skylark/board/2027/q2/mrt-acquisition/`; SEC + audit-committee retention |
| `messenger` | Executive-only channel (Mira + CEO + counsel + CFO) for sensitive private discussion |

## Pack overlays (7 active)

| Pack | Activation reason | Pack ID |
|---|---|---|
| HSR-Act-US | US HSR threshold may trigger (Hart-Scott-Rodino) | `pack-hsr-act-us-2027` |
| EU-Merger-Control | EU Merger Control Regulation 139/2004 (MRT is EU-domiciled) | `pack-eu-merger-control` |
| UK-CMA | UK Competition + Markets Authority (Tier 2 voluntary) | `pack-uk-cma-merger-tier-2` |
| Israeli-IMC | Israeli Merger Control (parties review; Mira is dual-citizen) | `pack-israeli-imc-parties-review` |
| GDPR | EU customers; cross-tenant data flow during diligence | `pack-gdpr-cross-tenant-diligence` |
| SOC-2 | Pack-manifest cross-check baseline | `pack-soc2-cross-check` |
| NDA-Cross-Tenant | NDA-bound cross-tenant due-diligence channel | `pack-nda-cross-tenant-m-a-class` |

## Regulatory anchors

1. **HSR Act** — 15 U.S.C. §18a — Premerger notification + waiting period (currently $111.4M size-of-transaction threshold for 2027)
2. **EU Merger Control Regulation 139/2004** — Article 4 notification + Article 6/7 substantive review + Phase I 25 working days
3. **UK CMA** — Enterprise Act 2002 + UK Internal Market Act 2020; Tier 2 voluntary notification
4. **Israeli Antitrust Law 5748-1988** — Merger control parties review
5. **GDPR Article 28** — Processor obligations (during cross-tenant diligence)
6. **GDPR Article 33** — Breach notification
7. **SEC Schedule 14A** (if Skylark stock is consideration + shareholder approval needed)
8. **SOC-2 Trust Services Criteria** (pack-cross-check baseline)
9. **NDA** — Skylark + MRT mutual NDA dated 2027-03-08
10. **ADR-0244 + ADR-0263 + ADR-0251 + ADR-0245 + ADR-0252 + ADR-0253 + ADR-0255**

## Cell + region matrix

| Cell | Role | Journey use |
|---|---|---|
| `us-east-boston-tier-1-executive` | Skylark CSO primary cell | Mira's M&A console |
| `eu-frankfurt-tier-2-tenant-mrt` | MRT primary cell (EU domiciled) | MRT-side diligence interface |
| `us-east-recordings-worm-board` | SEC-aligned WORM cell | Final decision archive |
| `cross-tenant-channel-skylark-mrt-2027-q2` | NDA-bound cross-tenant channel | Diligence document exchange |
| `external-transparency-log-batch-2027-05-25` | External transparency log | Decision anchor |

## Cedar permits (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
permit (
    principal == User::"mira.goldberg@skylark-logistics-solutions-inc",
    action in [
        Action::"financial_planning.m_a_model_compute",
        Action::"intelligence.scenario_modeling",
        Action::"compliance.pack_manifest_cross_check",
        Action::"compliance.merger_filing_requirements_compute",
        Action::"connect.cross_tenant_nda_channel_send",
        Action::"governance.acquisition_recommendation_propose"
    ],
    resource is AcquisitionDecision
) when {
    principal.role_in_tenant("skylark-logistics-solutions-inc") == "chief_strategy_officer" &&
    resource.tenant_id == "skylark-logistics-solutions-inc" &&
    resource.target_tenant_id == "mendelsohn-routing-technologies-inc-de" &&
    context.passkey_assertion_present == true &&
    context.nda_active_at_time == true
};

permit (
    principal,
    action == Action::"connect.cross_tenant_nda_channel_send",
    resource is CrossTenantChannel
) when {
    resource.channel_id == "cross-tenant-channel-skylark-mrt-2027-q2" &&
    context.nda_record_id == "nda-skylark-mrt-2027-03-08" &&
    context.nda_active == true &&
    context.payload_class in [
        "diligence_request",
        "diligence_response_anonymized",
        "diligence_response_named",
        "valuation_clarification",
        "regulatory_filing_input"
    ]
};

permit (
    principal in Group::"skylark_board_voting_members",
    action == Action::"governance.acquisition_go_no_go_vote",
    resource is AcquisitionDecision
) when {
    context.audit_committee_endorsement_present == true &&
    context.counsel_review_present == true &&
    context.financial_model_signoff_cfo_present == true &&
    context.passkey_assertion_present == true
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J166-001 | NDA-bound cross-tenant channel established between Skylark + MRT; audit `EVT-J166-NDA-CHANNEL-OPEN-001` |
| AC-J166-002 | Cross-tenant diligence document exchange: 28 documents from MRT → Skylark over 9 days; all Cedar-validated + audit-sealed; audit `EVT-J166-DILIGENCE-DOCS-EXCHANGED-002` |
| AC-J166-003 | M&A model computed at 3 price points ($172M / $186M / $202M); ARR/CAC/LTV/payback/CAC-cohort cross-applied; audit `EVT-J166-M-A-MODEL-COMPUTED-003` |
| AC-J166-004 | ML scenario modeling: Monte-Carlo 10K iterations × 3 macro scenarios; cohort churn forecast (5-year horizon); integration cost forecast ($14.2M ± $4.8M); audit `EVT-J166-ML-SCENARIOS-004` |
| AC-J166-005 | Pack-manifest cross-check: Skylark 6 packs × MRT 5 packs = 11 unique with 2 overlaps; no blockers identified; audit `EVT-J166-PACK-CROSS-CHECK-005` |
| AC-J166-006 | Merger filing requirements computed: HSR (required), EU-MR (Phase I expected 25 days), UK CMA (voluntary - file), Israeli IMC (parties review - file); audit `EVT-J166-MERGER-FILINGS-006` |
| AC-J166-007 | Counsel review by GC Daphne Harrowgate; 4 redlines + 1 deal-term clarification; audit `EVT-J166-COUNSEL-REVIEW-007` |
| AC-J166-008 | CFO sign-off on financial model by CFO Reginald Otis; audit `EVT-J166-CFO-SIGNOFF-008` |
| AC-J166-009 | Audit + Strategy committee endorsement (4 of 5 strategy + 3 of 5 audit); audit `EVT-J166-COMMITTEE-ENDORSEMENT-009` |
| AC-J166-010 | Board go/no-go vote May 25 09:00 EDT: 7 of 9 board votes yes; audit `EVT-J166-BOARD-VOTE-010` |
| AC-J166-011 | Decision recorded + Merkle anchor + drive WORM archive + external transparency anchor; audit `EVT-J166-DECISION-RECORDED-011` |
| AC-J166-012 | Hebrew + German + Hangul + diacritic preservation across all artifacts byte-exact |

## Cross-references

- Persona dossier: `docs/personas/cso-mira-goldberg.md`
- MASTER-ROSTER §3.4 row 234
- Matrix §10 j166 recommendation
- Related: j118 (cross-tenant ontology projection), j125 (marketplace acquires supplier merger), j163 (cross-time-zone board), j165 (CCO board compliance report)
- Pack roster: `packs/hsr-act-us-2027/`, `packs/eu-merger-control/`, `packs/uk-cma-merger/`, `packs/israeli-imc/`, `packs/gdpr-cross-tenant-diligence/`, `packs/soc2-cross-check/`, `packs/nda-cross-tenant-m-a-class/`
- ADRs as listed above

## Stop condition

Journey complete when all 12 AC pass on the seeded fixture, the board vote is recorded with 7/9 in favor, the decision record + super-Merkle anchored externally, NDA-bound cross-tenant channel preserved end-of-line, and the audit-chain spine contains the full 9-day evidence trail with no cross-tenant data leakage outside the Cedar-permitted payload classes.
