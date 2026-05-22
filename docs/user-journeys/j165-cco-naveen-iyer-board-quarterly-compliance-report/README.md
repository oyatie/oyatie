---
doc_class: User-Journey-README
journey_id: j165-cco-naveen-iyer-board-quarterly-compliance-report
slice: cco-quarterly-board-compliance-report-cross-pack-assembly-merkle-anchored-worm-archived
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: CCO Naveen Iyer (white/executive; Chief Compliance Officer)
audience_type: B2B_EXECUTIVE + COMPLIANCE_OFFICER + BOARD_FACING
microservice_count: 5
pack_overlay_anchor: SOC-2 + HIPAA + GDPR + EU-AI-Act + KR-PIPA + CSAP + PCI-DSS + SEC-Form-NT-8K
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0263-observability-emission-contract
  - ADR-0251-compliance-pack-primitive
  - ADR-0250-build-ahead-of-certification
  - ADR-0253-http3-quic-default-protocol
  - ADR-0255-intelligence-two-layer-substrate
---

# j165 — CCO Naveen Iyer assembles the Q1-2027 board compliance report across 8 packs

## At a glance

Naveen Iyer (नवीन अय्यर) is a **44-year-old Chief Compliance Officer** at **Tessellate Health AI Inc.**, a Wilmington-Delaware-incorporated B2B SaaS healthcare-AI company (Series C, $640M ARR, ~720 employees, HQ Boston Seaport, satellite engineering offices in Bangalore + Berlin + Seoul). The company sells a clinical-decision-support platform to mid-market hospitals + Medicare-Advantage payors. Naveen is South-Indian (Tamil heritage), born in Chennai, raised in Boston since age 7, JD/MBA from Northwestern, joined Tessellate from a Big Four firm in 2024-09. He reports directly to the board via the Audit Committee chair (Jasmine Wells-Okafor, retired SVP from a competitor).

Today is **Thursday April 8, 2027, 06:18 EDT**. The Tessellate Q1-2027 board meeting is **next Wednesday April 14 at 14:00 EDT**. Naveen has six business days to assemble the **Q1-2027 Quarterly Board Compliance Report** — a 47-page bound document that the board reads cover-to-cover, the General Counsel cites in his counsel-review, and the audit committee scrutinizes line-by-line. The report is the single most consequential piece of paper Naveen produces each quarter; it determines whether the board approves the audit committee's recommendations on compensation, regulatory standing, and product release gating.

This is the **first Q1 report** that consolidates across **eight active compliance packs**:

- **SOC 2 Type II** (audited annually; quarterly evidence sweep required)
- **HIPAA** (Tessellate is a Business Associate to ~140 hospital + payor customers under BAA contracts)
- **GDPR** (EU customers in Germany, France, Netherlands, Ireland; ~18% of ARR)
- **EU AI Act** (high-risk AI system per Annex III §5(a) — credit / employment exclusion not applicable; medical-device classification: per MDR / Class IIa under Regulation 2017/745)
- **KR PIPA** (Korean hospital customer Severance Hospital + private clinic chain MEDI Plus)
- **CSAP** (Korean Cloud Security Assurance Program — required because Tessellate hosts data on Naver Cloud for KR customers)
- **PCI DSS** (limited scope — only billing/invoicing surface; not PHI)
- **SEC-pending-IPO** (Tessellate filed an S-1 in 2026-12; subject to S-1 forward-looking disclosure rules + 17 CFR §229.105 risk factors)

This journey covers Naveen's **3-day** assembly + review + archival (April 8 06:18 EDT → April 10 17:42 EDT) through:

1. **governance** µservice — the board-resolution Cedar gate + audit-chain Merkle anchor + retention-policy declaration; provides the report-bundle state machine
2. **compliance** µservice — the cross-pack evidence aggregator; queries each pack's evidence ledger; pulls findings + remediation status + open risks
3. **audit-chain** µservice — the per-pack Merkle root attestation; each pack's evidence bundle gets its own Merkle root, then the cross-pack super-bundle gets a Merkle root of Merkle roots
4. **workflow-engine** µservice — the 4-state lifecycle "Draft → Counsel Review → Audit Committee Sign-off → Board Presentation"; Cedar-gated transitions; each transition emits an audit event
5. **drive** µservice — the final WORM-retention drive room `tessellate/board/2027/q1/compliance-report/`; SEC 17 CFR §240.17a-3 + §240.17a-4 retention rules adapted for a pre-IPO issuer

Microservices: `governance`, `compliance`, `audit-chain`, `workflow-engine`, `drive`. Secondary: `identity`, `tenancy`, `intelligence` (LLM-assisted draft generation for the executive summary), `notes`, `observability`, `cell`.

## Why this journey matters

Naveen Iyer is **MASTER-ROSTER §3.4 row 211** — the canonical CCO persona at a pre-IPO healthcare-AI company operating across 4 jurisdictions with 8 active packs. This persona covers ~8,400 CCO-class roles in US healthcare-tech + ~12,000 globally in regulated-tech B2B (BLS 2024 occupational code 11-3031 + EU EUROSTAT 2024).

The journey closes:

- **Critical-path row 89** (Cross-pack quarterly board report consolidation across 8 active packs)
- **Critical-path row 90** (Per-pack Merkle attestation + super-Merkle-of-Merkles for the bundled artifact)
- **Critical-path row 91** (Workflow-engine state-machine Draft→Counsel→Audit-Committee→Board with Cedar at each transition)
- **Critical-path row 92** (Drive WORM with SEC 17a-3/17a-4 + adapted retention for pre-IPO issuer)
- **Critical-path row 93** (SEC Form-NT / 8-K trigger evaluation as inline computation during report drafting)

Hyperscaler benchmark: ServiceNow GRC + AuditBoard + LogicGate Risk Cloud + Drata all produce quarterly-style compliance reports; none of them issue per-pack Merkle roots + a super-Merkle anchor; none of them cite the workflow-engine state-machine transitions as Cedar-gated; none of them inline-compute SEC 8-K materiality triggers. oyatie ships all three day one because [[autonomous-decision-principles]] + [[build-ahead-of-certification]] (the audit-chain spine is a first-class capability, not a bolt-on).

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat 06:18 EDT April 8 → 17:42 EDT April 10 across 3 days of assembly | Boston Seaport spring weather; named board members + counsel + audit committee; pack-by-pack findings; specific remediation statuses |
| `ux-flow.md` | Naveen's GRC console + cross-pack assembly canvas + workflow state-machine view + Merkle-anchor confirmation + board-distribution screen | Per-pack pane; aggregated risk heatmap; state-machine transition modal; SEC 8-K trigger evaluation inline |
| `handshake.md` | Per-µservice API + per-pack evidence query + Merkle anchor flow | Named pack + named evidence ledger + Cedar permit + audit class per call |
| `integration-test-plan.md` | Pack evidence completeness + Merkle determinism + workflow state-machine transitions + SEC trigger + drive WORM | Per-pack seed + Cedar deny coverage + per-state transition |
| `schemas/openapi-board-compliance-report.json` | OpenAPI for board-report assembly endpoints | All 4 state-machine stages + per-pack pull + Merkle bundle |
| `schemas/cedar-policy.cedar` | Board-compliance-report Cedar policy | CCO + counsel + audit-committee + board permits; per-stage gate |
| `schemas/journey-messages.proto` | proto3 for all RPCs | Tamil + Devanagari + Hangul preservation; per-pack evidence bundle |
| `schemas/board-report-state-machine.yaml` | 4-state board-report lifecycle | draft → counsel_review → audit_committee_sign_off → board_presentation |
| `schemas/cross-pack-evidence-bundle.json` | Cross-pack evidence bundle schema | Per-pack Merkle root + super-Merkle of Merkles + SEC trigger evaluation |

## The five primary microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `governance` | Board-resolution Cedar gate + audit-chain Merkle anchor + retention-policy declaration | row 90 |
| `compliance` | Cross-pack evidence aggregator; per-pack findings + remediation + open risk pulls | row 89 |
| `audit-chain` | Per-pack Merkle attestation + super-Merkle-of-Merkles + external transparency log anchor | row 90 |
| `workflow-engine` | 4-state lifecycle; Cedar-gated transitions | row 91 |
| `drive` | Final WORM-retention archive | row 92 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `identity` | Naveen's passkey + YubiKey 5C NFC; counsel + audit-committee + board passkeys |
| `tenancy` | Tenant `tessellate-health-ai-inc`; sub-organizations `compliance`, `audit-committee`, `board-of-directors` |
| `intelligence` | LLM-assisted executive summary draft (Naveen reviews + edits; LLM never finalizes) |
| `notes` | Naveen's working draft notes + meeting prep notes for the audit committee |
| `observability` | Pack-evidence-pull latency + Merkle-compute determinism + SEC trigger evaluation latency |
| `cell` | `us-east-boston-tier-1-compliance` (Naveen's primary) + `eu-frankfurt-evidence-mirror` (EU evidence GDPR-locality) + `kr-seoul-evidence-mirror` (CSAP) |
| `audit-chain` | Per-stage transition audit + super-Merkle anchor + external transparency log |

## Pack overlays (8 active)

| Pack | Activation reason | Pack ID |
|---|---|---|
| SOC-2 | Annual Type II; quarterly evidence sweep | `pack-soc2-type2-fy2026` |
| HIPAA | Business Associate role; ~140 BAAs | `pack-hipaa-business-associate` |
| GDPR | EU customers ~18% ARR | `pack-gdpr-controller-processor-mixed` |
| EU AI Act | High-risk AI system per Annex III §5(a) + MDR Class IIa | `pack-eu-ai-act-high-risk-medical` |
| KR PIPA | Korean hospital customers | `pack-kr-pipa-controller` |
| CSAP | Naver Cloud hosting for KR | `pack-csap-naver-cloud-tier-3` |
| PCI DSS | Billing/invoicing surface only | `pack-pci-dss-saq-c` |
| SEC-Pending-IPO | S-1 filed 2026-12 | `pack-sec-pre-ipo-s1-active` |

## Regulatory anchors

1. **SOC 2 Type II** — Trust Services Criteria (Security + Availability + Confidentiality + Processing Integrity + Privacy)
2. **HIPAA Security Rule** — 45 CFR §164.308 (administrative) + §164.310 (physical) + §164.312 (technical)
3. **HIPAA Privacy Rule** — 45 CFR §164.502 + §164.508 + §164.524 (right of access)
4. **HIPAA Breach Notification** — 45 CFR §164.404 (subject notification within 60 days)
5. **GDPR Article 30** — Records of processing activities
6. **GDPR Article 32** — Security of processing
7. **GDPR Article 33** — Breach notification (72-hour rule)
8. **EU AI Act Article 9** — Risk management system (mandatory for high-risk AI)
9. **EU AI Act Article 11** — Technical documentation
10. **EU AI Act Article 12** — Record-keeping (automatic logging)
11. **EU AI Act Article 13** — Transparency
12. **EU AI Act Article 14** — Human oversight
13. **EU AI Act Article 15** — Accuracy + robustness + cybersecurity
14. **KR PIPA Article 29** — Protective measures
15. **KR PIPA Article 33** — Privacy impact assessment
16. **CSAP** — Naver Cloud Tier 3 controls
17. **PCI DSS v4.0** — Requirements 1–12 (SAQ-C scope)
18. **SEC 17 CFR §229.105** — Risk factors disclosure
19. **SEC 17 CFR §240.17a-3 + §240.17a-4** — Books and records (adapted for pre-IPO issuer)
20. **SEC Form NT** — Notification of inability to timely file
21. **SEC Form 8-K** — Current report (Item 1.01 material agreements + Item 2.02 results + Item 4.02 non-reliance + Item 5.02 officer changes + Item 8.01 other events)

## Cell + region matrix

| Cell | Role | Journey use |
|---|---|---|
| `us-east-boston-tier-1-compliance` | Naveen's primary GRC console cell | Cross-pack assembly + workflow + Merkle compute |
| `eu-frankfurt-evidence-mirror` | EU GDPR-locality evidence cell | GDPR + EU AI Act evidence stays in EU |
| `kr-seoul-evidence-mirror` | KR CSAP-locality evidence cell | KR PIPA + CSAP evidence stays in KR |
| `us-east-recordings-worm-board` | SEC-aligned WORM storage cell for board records | Final report archival |
| `external-transparency-log-batch-2027-04-10` | External CT-log-style transparency anchor | Independent verifiability of Merkle super-root |

## Cedar permits (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
permit (
    principal == User::"naveen.iyer@tessellate-health-ai-inc",
    action in [
        Action::"compliance.cross_pack_evidence_pull",
        Action::"compliance.report_draft",
        Action::"governance.merkle_per_pack_compute",
        Action::"governance.merkle_super_root_compute",
        Action::"workflow_engine.transition_propose",
        Action::"workflow_engine.transition_confirm_cco",
        Action::"intelligence.executive_summary_assist",
        Action::"compliance.sec_8k_trigger_evaluate"
    ],
    resource is BoardComplianceReport
) when {
    principal.role_in_tenant("tessellate-health-ai-inc") == "chief_compliance_officer" &&
    resource.tenant_id == "tessellate-health-ai-inc" &&
    resource.report_class == "quarterly_board_compliance" &&
    context.passkey_assertion_present == true
};

permit (
    principal == User::"hampton.reese@tessellate-health-ai-inc",
    action in [
        Action::"governance.counsel_review",
        Action::"workflow_engine.transition_counsel_to_audit_committee"
    ],
    resource is BoardComplianceReport
) when {
    principal.role_in_tenant("tessellate-health-ai-inc") == "general_counsel" &&
    context.cco_signoff_present == true
};

permit (
    principal == User::"jasmine.wells-okafor@tessellate-health-ai-inc",
    action == Action::"workflow_engine.transition_audit_committee_to_board",
    resource is BoardComplianceReport
) when {
    principal.role_in_tenant("tessellate-health-ai-inc") == "audit_committee_chair" &&
    context.counsel_review_present == true &&
    context.audit_committee_quorum_reached == true
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J165-001 | All 8 active pack evidence ledgers queried; ≥ 480 evidence artifacts retrieved; audit `EVT-J165-PACK-EVIDENCE-PULL-001` |
| AC-J165-002 | Per-pack Merkle root computed for each of 8 packs; audit `EVT-J165-PER-PACK-MERKLE-002` × 8 |
| AC-J165-003 | Cross-pack super-Merkle of Merkles computed deterministically; audit `EVT-J165-SUPER-MERKLE-003` |
| AC-J165-004 | LLM-assisted executive summary drafted; Naveen reviews + edits; LLM provenance metadata preserved; audit `EVT-J165-LLM-DRAFT-ASSIST-004` |
| AC-J165-005 | SEC Form 8-K trigger evaluation: 0 triggers fire in Q1; audit `EVT-J165-SEC-8K-EVAL-005` |
| AC-J165-006 | Cedar transition Draft → Counsel Review with CCO signoff; audit `EVT-J165-TRANSITION-DRAFT-TO-COUNSEL-006` |
| AC-J165-007 | Counsel review by Hampton Reese (General Counsel); 3 redline edits; audit `EVT-J165-COUNSEL-REVIEW-007` |
| AC-J165-008 | Cedar transition Counsel → Audit Committee with counsel review present; audit `EVT-J165-TRANSITION-COUNSEL-TO-AC-008` |
| AC-J165-009 | Audit committee quorum (3 of 5) sign-off by Jasmine Wells-Okafor + 2 independents; audit `EVT-J165-AUDIT-COMMITTEE-SIGNOFF-009` |
| AC-J165-010 | Cedar transition Audit Committee → Board with quorum present; audit `EVT-J165-TRANSITION-AC-TO-BOARD-010` |
| AC-J165-011 | Final report archived to drive WORM `tessellate/board/2027/q1/compliance-report/`; 7-year retention timer engaged; audit `EVT-J165-REPORT-ARCHIVED-011` |
| AC-J165-012 | Cross-region evidence preserved per region: US-East / EU-Frankfurt / KR-Seoul each retain their pack evidence locally; audit `EVT-J165-REGIONAL-EVIDENCE-PRESERVED-012` |
| AC-J165-013 | External transparency log anchor verifiable from independent observer; audit `EVT-J165-EXTERNAL-ANCHOR-013` |
| AC-J165-014 | Hangul + Devanagari + Tamil + German diacritic preservation across LLM draft + counsel redlines + audit + drive byte-exact |

## Cross-references

- Persona dossier: `docs/personas/cco-naveen-iyer.md`
- MASTER-ROSTER §3.4 row 211
- Matrix §10 j165 recommendation
- Related: j163 (board meeting AV — different surface but adjacent), j100 (pack rollout), j92 (multi-jurisdiction sweep), j118 (cross-tenant ontology projection)
- Pack roster: `packs/soc2-type2/`, `packs/hipaa-business-associate/`, `packs/gdpr-mixed/`, `packs/eu-ai-act-high-risk-medical/`, `packs/kr-pipa-controller/`, `packs/csap-naver-cloud/`, `packs/pci-dss-saq-c/`, `packs/sec-pre-ipo-s1-active/`
- ADR-0244 + ADR-0263 + ADR-0251 + ADR-0250 + ADR-0253

## Stop condition

Journey complete when all 14 acceptance criteria pass on the seeded `tessellate-health-ai-inc` fixture, the report reaches `board_presentation` state with quorum-validated sign-offs, super-Merkle anchored to external transparency log, regional evidence preserved per jurisdiction, and the WORM-retention timer engaged for the SEC-adapted 7-year period.
