---
doc_class: CrossCoverageMatrix
shape: Reference
status: Proposed
date: 2026-05-21
authority_tier: 2
matrix_axes:
  - persona (~127 from MASTER-ROSTER-2026-05-21)
  - journey (j01..j150 from docs/user-journeys/)
  - microservice (69 from microservices/<svc>/)
  - critical-path-row (30 rows per documentation-rigor.md §3.2.5)
  - pack-overlay (per ADR-0251)
  - capability-tier (per ADR-0316)
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0247
  - ADR-0248
  - ADR-0249
  - ADR-0251
  - ADR-0263
  - ADR-0292
  - ADR-0297
  - ADR-0299
  - ADR-0300
  - ADR-0304
  - ADR-0305
  - ADR-0307
  - ADR-0308
  - ADR-0309
  - ADR-0310
  - ADR-0311
  - ADR-0312
  - ADR-0313
  - ADR-0314
  - ADR-0315
  - ADR-0316
  - ADR-0317
  - ADR-0318
  - ADR-0319
  - ADR-0320
companion_docs:
  - docs/personas/MASTER-ROSTER-2026-05-21.md
  - docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
  - docs/user-journeys/CATALOG-j126-j150-ecosystem.md
  - docs/standards/documentation-rigor.md
authoring_rule: READ-ONLY synthesis; no source-of-truth doc is modified.
---

# Persona × Journey × Microservice Cross-Coverage Matrix — 2026-05-21

This matrix is the editorial spine that lets readers walk the corpus along any of three axes: pick a persona and follow their journeys to the µservices they touch; pick a µservice and find the personas and journeys that exercise it; pick a journey and see every persona and µservice it composes. It is a READ-ONLY synthesis layer over `docs/personas/MASTER-ROSTER-2026-05-21.md`, `docs/user-journeys/j01..j150/README.md`, and `docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md`. No source-of-truth document is modified by this synthesis.

> *"All consolidated into one ecosystem; technical, non-technical, office, non-office — all under the same hood. The same human is a day-to-day consumer + enterprise employee + healthcare patient/worker + side-business owner + family parent — same passkey identity, multiple tenant memberships."* — 2026-05-21 unified-ecosystem thesis (cited in MASTER-ROSTER §1.1).

---

## §1 Doctrine — Continuity of Identity Across Personas + Journeys + Microservices

### §1.1 Three coordinate axes, one identity primitive

oyatie's runtime view of any operation is the tuple `(identity, tenant, role-projection, journey-context, µservice-surface)`. The matrix in §2–§4 enumerates the axes; the graphs in §5–§7 enumerate the edges between them. The doctrine that binds the axes is the unified-ecosystem thesis cited above: a single passkey-bound human (ADR-0299) participates in many tenants (ADR-0244 + ADR-0311) through many role-projections (ADR-0317 + ADR-0318 + ADR-0319 + ADR-0320), and any journey is a finite path in the resulting tuple-graph that crosses three or more µservices (per the j126-j150 catalog's ecosystem rule).

### §1.2 Why this matrix exists

Three failure modes drove the production of this document:

1. **Persona-orphan journeys.** A journey written without explicit persona binding cannot satisfy documentation-rigor.md §1.1 intern-buildability — the intern reading it cannot reason about who triggers the flow, which Cedar permits are active, or which audit class is emitted. The §3 per-journey table closes this by surfacing the primary persona on every row.
2. **Journey-orphan personas.** A persona who appears in MASTER-ROSTER §3 but never anchors a story.md row produces no critical-path evidence. The §2 per-persona table closes this by enumerating the journey range each persona triggers, and §9 surfaces the gaps.
3. **µservice-orphan flows.** A µservice that owns code but never appears on a journey-handshake row is in violation of ADR-0316 — its capability tiers cannot be evidenced. The §4 per-µservice table closes this by indexing the journeys that exercise each µservice and surfacing centers-of-gravity counts from the enterprise-coverage-matrix §14 table.

### §1.3 Cross-reference contract

Every row in §2 cites the persona's MASTER-ROSTER section (§3.1–§3.11) and the journey-IDs it spans. Every row in §3 cites the journey directory under `docs/user-journeys/j<NN>-*` plus the persona-primary captured in `README.md`. Every row in §4 cites the µservice directory under `microservices/<svc>/` plus the µservice's enterprise-coverage-matrix §14 rank. The matrix never paraphrases; it always cites.

### §1.4 Six engineering-rigor dimensions for the matrix

Per documentation-rigor.md §1.2 the matrix itself must satisfy six dimensions: **maintainability** (per-axis adds without breaking adjacent axes), **observability** (every row binds to an `audit-chain` event class via journey handshake), **scalability** (Cartesian product of 127×150×69 ≈ 1.3M tuples is stored as sparse adjacency lists, not a dense table), **performance** (any lookup is O(deg) over the smallest axis), **optimization** (cross-axis composition is precomputed in §5–§7 graphs), **code quality** (every row passes the schema spec in §1.5 below).

### §1.5 Row schema

Every per-persona row has 8 cells: `# | persona-name | archetype | audience_type | journeys (by ID) | µservices touched | pack overlays | cross-context bridge`. Every per-journey row has 8 cells: `journey-id | slug | primary persona(s) | µservices touched | pack overlay | critical-path row(s) | artifact count | cross-references`. Every per-µservice row has 8 cells: `# | µservice | tier (substrate/product) | personas (by name) | journeys (by ID) | capability tiers | pack overlays | hyperscaler benchmark`. Every graph edge has 3 cells: `source | target | relation`.

### §1.6 Authority alignment

The matrix inherits authority from MASTER-ROSTER-2026-05-21.md (persona axis), CATALOG-j126-j150-ecosystem.md (journey axis), and enterprise-software-coverage-matrix-2026-05-21.md (µservice axis). It does not introduce new persona slugs, journey IDs, or µservice names. Where a recommendation surfaces (§9–§10) it is captured as a *gap*, never as a binding decision.

---

## §2 Per-Persona Cross-Coverage Table

Each persona row enumerates: archetype + audience_type (per ADR-0244), the journeys featuring them, the µservices their journeys touch, the critical-path rows they trigger (per documentation-rigor.md §3.2.5), the pack overlay applicable (per ADR-0251), the cross-context bridge listing other personas that are the same human (per MASTER-ROSTER §4), and the typical day-of-week touchpoints.

Cross-reference: MASTER-ROSTER-2026-05-21.md §3.1–§3.11 enumerates the 127 personas; this section walks the same list and adds per-persona journey + µservice closure.

### §2.001 Yejin Park — Nurse + parent + side-business owner

- **Archetype:** Nurse + parent + side-business owner; collar=pink+green; workspace=clinical+field; skill-tier=mid; device=mobile; locale=KR.
- **audience_type (per ADR-0244):** `B2B_HEALTHCARE_PROVIDER+B2C_FAMILY_PARENT+B2C_CONSUMER+B2B_TENANT_ADMIN+B2B_HEALTHCARE_PATIENT`.
- **Cross-context bridge (same human):** nurse / parent / soap-side / patient / consumer.
- **Journeys featuring them:** j01, j02, j04, j06, j09, j11, j14, j16, j21, j22, j25, j26, j27, j28, j29, j36, j37, j72, j100, j148.
- **µservices their journeys touch:** api-gateway, application, audit-chain, calendar, cell, community, compliance, connect, consent-graph, drive, identity, intelligence, mail, messenger, notes, observability, ontology, payments, plugin-app-store, tenancy, workflow-engine, workflow-studio, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 2, row 8, row 9, row 14, row 18, row 24, row 27, row 30.
- **Per-pack overlay applicable (per ADR-0251):** KR-CSAP + KR-Privacy + KR-Labor + KR-119.
- **Typical day-of-week touchpoints:** Sun-Sat shift-bound; weekend on-call rotation.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 1; cross-context bridges in §4 of same file.

### §2.002 Marcus Chen — Multinational CEO + spouse + father

- **Archetype:** Multinational CEO + spouse + father; collar=white+gold; workspace=executive+field; skill-tier=executive; device=desktop; locale=KR+US.
- **audience_type (per ADR-0244):** `B2B_CSUITE+B2C_CONSUMER+B2C_FAMILY_PARENT`.
- **Cross-context bridge (same human):** CEO / husband / father.
- **Journeys featuring them:** j13, j15, j17, j41, j100, j101, j118, j119, j120, j121, j123, j124, j125.
- **µservices their journeys touch:** audit-chain, community, compliance, connect, developer-sdk, drive, finops-portal, foundry, identity, intelligence, mail, marketplace, messenger, observability, ontology, payments, plugin-app-store, tenancy, workflow-engine, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27, row 30.
- **Per-pack overlay applicable (per ADR-0251):** KR-CSAP + SOC2.
- **Typical day-of-week touchpoints:** Mon-Fri 06:00-22:00 board cadence; Sat 1h ad-hoc.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 2; cross-context bridges in §4 of same file.

### §2.003 Aiyana Singh — Senior ML engineer + blogger + parent

- **Archetype:** Senior ML engineer + blogger + parent; collar=white; workspace=back-office; skill-tier=senior; device=desktop; locale=IN.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER+B2C_FAMILY_PARENT`.
- **Cross-context bridge (same human):** at-work / blogger / parent.
- **Journeys featuring them:** j40, j46, j47, j48, j50, j51, j52, j53, j54, j149.
- **µservices their journeys touch:** cell, community, compliance, connect, finops-portal, identity, mail, payments, plugin-app-store, tenancy, workflow-engine, workflow-studio.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** DPDP-2023 + RBI.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 3; cross-context bridges in §4 of same file.

### §2.004 Tomás García — Restaurant owner + cook + father

- **Archetype:** Restaurant owner + cook + father; collar=white+green; workspace=executive+production; skill-tier=senior; device=mobile; locale=BR.
- **audience_type (per ADR-0244):** `B2B_TENANT_ADMIN+B2B_EMPLOYEE+B2C_CONSUMER+B2C_FAMILY_PARENT`.
- **Cross-context bridge (same human):** owner / cook / father.
- **Journeys featuring them:** j23, j24, j33, j35, j101, j102, j103, j104, j105, j107, j108.
- **µservices their journeys touch:** audit-chain, community, compliance, connect, drive, identity, intelligence, mail, marketplace, messenger, observability, ontology, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27, row 30.
- **Per-pack overlay applicable (per ADR-0251):** LGPD + BR-Labor.
- **Typical day-of-week touchpoints:** Mon-Fri 06:00-22:00 board cadence; Sat 1h ad-hoc.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 4; cross-context bridges in §4 of same file.

### §2.005 Hiroshi Tanaka — Retired widower + grandfather + photographer

- **Archetype:** Retired widower + grandfather + photographer; collar=white-retired; workspace=field; skill-tier=senior-retired; device=mobile+assistive; locale=JP.
- **audience_type (per ADR-0244):** `B2C_CONSUMER+B2C_FAMILY_PARENT`.
- **Cross-context bridge (same human):** grandfather / photographer / patient.
- **Journeys featuring them:** j07, j08, j22, j26, j27, j69.
- **µservices their journeys touch:** audit-chain, drive, identity, mail, messenger, notes, payments, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14.
- **Per-pack overlay applicable (per ADR-0251):** APPI + JP-Labor + JP-FSA.
- **Typical day-of-week touchpoints:** Sun-Sat shift-bound; weekend on-call rotation.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 5; cross-context bridges in §4 of same file.

### §2.006 Anya Mironova — Investigative journalist + activist + parent

- **Archetype:** Investigative journalist + activist + parent; collar=white; workspace=field; skill-tier=senior; device=desktop+mobile; locale=EU.
- **audience_type (per ADR-0244):** `B2C_CONSUMER+B2C_FAMILY_PARENT+HIGH_RISK_USER`.
- **Cross-context bridge (same human):** journalist / parent / activist.
- **Journeys featuring them:** j05, j06, j15, j17, j34, j129.
- **µservices their journeys touch:** audit-chain, comms-email, community, compliance, drive, governance, identity, marketplace, messenger, observability, payments, policy-engine, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 30.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + DSA + EU-AI-Act + DORA.
- **Typical day-of-week touchpoints:** Sun-Sat shift-bound; weekend on-call rotation.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 6; cross-context bridges in §4 of same file.

### §2.007 Diana Reyes — GAO auditor (3PAO) + spouse + parent

- **Archetype:** GAO auditor (3PAO) + spouse + parent; collar=white; workspace=middle-office+field; skill-tier=senior; device=desktop+mobile; locale=US.
- **audience_type (per ADR-0244):** `INTERNAL_AUDITOR_3PAO+B2C_CONSUMER+B2C_FAMILY_PARENT`.
- **Cross-context bridge (same human):** auditor / consumer.
- **Journeys featuring them:** j126, j127, j128, j129, j130, j131.
- **µservices their journeys touch:** api-gateway, audit-chain, calendar, comms-email, community, compliance, connect, drive, governance, identity, intelligence, mail, marketplace, meet, messenger, notes, observability, ops-dashboard-control-center, payments, policy-engine, tenancy, workflow-engine, workflow-studio, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24, row 27, row 30.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Sun-Sat shift-bound; weekend on-call rotation.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 7; cross-context bridges in §4 of same file.

### §2.008 Priya Krishnan — HR Director (multinational)

- **Archetype:** HR Director (multinational); collar=white; workspace=back-office; skill-tier=senior; device=desktop; locale=IN.
- **audience_type (per ADR-0244):** `B2B_HR_ADMIN+B2C_CONSUMER`.
- **Cross-context bridge (same human):** at-work / consumer.
- **Journeys featuring them:** j132, j133, j134, j135, j136.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** DPDP-2023 + RBI.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 8; cross-context bridges in §4 of same file.

### §2.009 Sam Okafor — Corporate Internal-Audit Director

- **Archetype:** Corporate Internal-Audit Director; collar=white; workspace=middle-office; skill-tier=senior; device=desktop; locale=NG.
- **audience_type (per ADR-0244):** `B2B_INTERNAL_AUDIT+B2C_CONSUMER`.
- **Cross-context bridge (same human):** at-work / consumer.
- **Journeys featuring them:** j137, j138, j139, j140, j141.
- **µservices their journeys touch:** audit-chain, community, compliance, detection, governance, identity, mail, messenger, ops-dashboard-control-center, payments, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** NDPR.
- **Typical day-of-week touchpoints:** Mon-Fri controls cadence; month-end peak; weekend evidence pull.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 9; cross-context bridges in §4 of same file.

### §2.010 Chris Volkov — Laid-off mid-career engineer

- **Archetype:** Laid-off mid-career engineer; collar=white; workspace=back-office; skill-tier=mid; device=desktop+mobile; locale=US.
- **audience_type (per ADR-0244):** `B2C_JOB_SEEKER_ACTIVE+B2C_CONSUMER+B2C_FAMILY_PARENT`.
- **Cross-context bridge (same human):** pre-layoff / post-layoff / family-provider.
- **Journeys featuring them:** j142, j143, j144, j145, j146, j147.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 10; cross-context bridges in §4 of same file.

### §2.011 Carlos Martinez — Forklift driver, warehouse

- **Archetype:** Forklift driver, warehouse; collar=blue; workspace=field; skill-tier=mid; device=handheld-rugged; locale=US.
- **audience_type (per ADR-0244):** `B2B_FIELD_WORKER+B2C_CONSUMER+B2C_FAMILY_PARENT`.
- **Cross-context bridge (same human):** at-work / father.
- **Journeys featuring them:** j41, j42, j43, j44.
- **µservices their journeys touch:** audit-chain, compliance, connect, developer-sdk, finops-portal, foundry, identity, intelligence, meet, notes, observability, ontology, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 30.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Sun-Sat shift-bound; weekend on-call rotation.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 11; cross-context bridges in §4 of same file.

### §2.012 Sarah Kim — Delivery driver (Amazon DSP)

- **Archetype:** Delivery driver (Amazon DSP); collar=blue; workspace=field; skill-tier=mid; device=vehicle-mount+mobile; locale=US.
- **audience_type (per ADR-0244):** `B2B_FIELD_WORKER+B2C_CONSUMER+B2B_TENANT_ADMIN`.
- **Cross-context bridge (same human):** driver / side-hustler.
- **Journeys featuring them:** j45, j46, j149.
- **µservices their journeys touch:** audit-chain, community, compliance, connect, drive, finops-portal, identity, mail, notes, payments, tenancy, workflow-engine, workflow-studio.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27, row 30.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Sun-Sat shift-bound; weekend on-call rotation.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 12; cross-context bridges in §4 of same file.

### §2.013 Ahmad Hassan — Construction site lead

- **Archetype:** Construction site lead; collar=blue; workspace=field; skill-tier=senior; device=handheld-rugged+mobile; locale=US.
- **audience_type (per ADR-0244):** `B2B_FIELD_WORKER+B2B_CONTRACTOR+B2C_CONSUMER`.
- **Cross-context bridge (same human):** site-lead / contractor.
- **Journeys featuring them:** j109, j110.
- **µservices their journeys touch:** community, identity, observability, payments, tenancy, workflow-engine, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 30.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Sun-Sat shift-bound; weekend on-call rotation.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 13; cross-context bridges in §4 of same file.

### §2.014 Maria Santos — Restaurant cook

- **Archetype:** Restaurant cook; collar=pink+green; workspace=production; skill-tier=mid; device=kiosk+mobile; locale=US.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER+B2C_FAMILY_PARENT`.
- **Cross-context bridge (same human):** at-work / mother.
- **Journeys featuring them:** j33, j110.
- **µservices their journeys touch:** community, identity, payments, tenancy, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** shift-rotation incl. weekends; daily safety toolbox.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 14; cross-context bridges in §4 of same file.

### §2.015 Devon Williams — Field-service technician (HVAC)

- **Archetype:** Field-service technician (HVAC); collar=gray; workspace=field; skill-tier=senior; device=vehicle-mount+mobile; locale=US.
- **audience_type (per ADR-0244):** `B2B_FIELD_WORKER+B2B_CONTRACTOR`.
- **Cross-context bridge (same human):** at-work / side-handyman.
- **Journeys featuring them:** j109.
- **µservices their journeys touch:** community, identity, observability, payments, workflow-engine, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 8, row 9, row 30.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Sun-Sat shift-bound; weekend on-call rotation.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 15; cross-context bridges in §4 of same file.

### §2.016 Jordan Lee — Retail clerk (17yo minor)

- **Archetype:** Retail clerk (17yo minor); collar=pink; workspace=front-office; skill-tier=junior; device=kiosk; locale=US.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER+B2C_MINOR_14_17`.
- **Cross-context bridge (same human):** at-work / minor / student.
- **Journeys featuring them:** j03, j16, j72.
- **µservices their journeys touch:** application, identity, intelligence.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 16; cross-context bridges in §4 of same file.

### §2.017 Ms. Patel — High-school teacher

- **Archetype:** High-school teacher; collar=pink; workspace=front-office; skill-tier=senior; device=desktop+mobile; locale=UK.
- **audience_type (per ADR-0244):** `EDU_TEACHER+B2C_CONSUMER+B2C_FAMILY_PARENT`.
- **Cross-context bridge (same human):** teacher / mother / mentor.
- **Journeys featuring them:** j16, j17, j32.
- **µservices their journeys touch:** application, community, drive, identity, intelligence, messenger.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14.
- **Per-pack overlay applicable (per ADR-0251):** UK-GDPR + UK-AADC + FERPA-equiv.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 17; cross-context bridges in §4 of same file.

### §2.018 Coach Park — Youth soccer coach + day-job engineer

- **Archetype:** Youth soccer coach + day-job engineer; collar=pink; workspace=front-office; skill-tier=mid; device=mobile; locale=KR.
- **audience_type (per ADR-0244):** `EDU_TEACHER+B2C_CONSUMER+B2C_FAMILY_PARENT`.
- **Cross-context bridge (same human):** coach / engineer / father.
- **Journeys featuring them:** j72.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14.
- **Per-pack overlay applicable (per ADR-0251):** KR-CSAP + KR-Privacy + KR-Labor + KR-119.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 18; cross-context bridges in §4 of same file.

### §2.019 Father Lopez — Catholic priest + chaplain

- **Archetype:** Catholic priest + chaplain; collar=pink; workspace=front-office; skill-tier=senior; device=desktop+mobile; locale=ES.
- **audience_type (per ADR-0244):** `RELIGIOUS_LEADER+B2C_CONSUMER`.
- **Cross-context bridge (same human):** priest / counselor / citizen.
- **Journeys featuring them:** j06, j72.
- **µservices their journeys touch:** audit-chain, community, drive, messenger.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + LOPDGDD.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 19; cross-context bridges in §4 of same file.

### §2.020 Captain Chen — Airline pilot (long-haul)

- **Archetype:** Airline pilot (long-haul); collar=gold; workspace=field; skill-tier=senior; device=vehicle-mount+mobile; locale=SG.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER+B2C_FAMILY_PARENT`.
- **Cross-context bridge (same human):** pilot / father.
- **Journeys featuring them:** j01, j11, j12.
- **µservices their journeys touch:** api-gateway, audit-chain, calendar, cell, compliance, connect, consent-graph, drive, identity, intelligence, mail, messenger, notes, observability, ontology, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** PDPA-SG + MAS-pack.
- **Typical day-of-week touchpoints:** Sun-Sat shift-bound; weekend on-call rotation.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 20; cross-context bridges in §4 of same file.

### §2.021 Officer Rodriguez — Police patrol officer

- **Archetype:** Police patrol officer; collar=gray; workspace=field; skill-tier=mid; device=vehicle-mount+mobile; locale=US.
- **audience_type (per ADR-0244):** `LAW_ENFORCEMENT+B2C_CONSUMER+B2C_FAMILY_PARENT`.
- **Cross-context bridge (same human):** on-patrol / family.
- **Journeys featuring them:** j01, j04, j10, j18.
- **µservices their journeys touch:** api-gateway, audit-chain, calendar, cell, community, compliance, consent-graph, drive, identity, intelligence, mail, messenger, notes, observability, ontology, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24, row 30.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Sun-Sat shift-bound; weekend on-call rotation.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 21; cross-context bridges in §4 of same file.

### §2.022 Dr. Tanaka — Cardiothoracic surgeon

- **Archetype:** Cardiothoracic surgeon; collar=gold; workspace=clinical; skill-tier=principal; device=desktop+clinical-PACS; locale=JP.
- **audience_type (per ADR-0244):** `B2B_HEALTHCARE_PROVIDER+B2C_CONSUMER+B2C_FAMILY_PARENT`.
- **Cross-context bridge (same human):** surgeon / father.
- **Journeys featuring them:** j02, j12, j22.
- **µservices their journeys touch:** api-gateway, audit-chain, cell, observability.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 2, row 8, row 9, row 14, row 18, row 24.
- **Per-pack overlay applicable (per ADR-0251):** APPI + JP-Labor + JP-FSA.
- **Typical day-of-week touchpoints:** shift-bound (PGY-3 every 4th-on-call); weekend rounds.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 22; cross-context bridges in §4 of same file.

### §2.023 Tomás García Jr. — Coffee farmer (3rd-gen)

- **Archetype:** Coffee farmer (3rd-gen); collar=green; workspace=field; skill-tier=senior; device=mobile+handheld-rugged; locale=BR.
- **audience_type (per ADR-0244):** `B2B_TENANT_ADMIN+B2C_CONSUMER+B2C_FAMILY_PARENT`.
- **Cross-context bridge (same human):** farmer / coop-board / son-of-Tomás.
- **Journeys featuring them:** j101, j102, j108, j148.
- **µservices their journeys touch:** audit-chain, community, compliance, connect, drive, identity, intelligence, mail, marketplace, ontology, payments, plugin-app-store, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27, row 30.
- **Per-pack overlay applicable (per ADR-0251):** LGPD + BR-Labor.
- **Typical day-of-week touchpoints:** Sun-Sat shift-bound; weekend on-call rotation.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 23; cross-context bridges in §4 of same file.

### §2.024 Captain Olufemi — Commercial fisherman

- **Archetype:** Commercial fisherman; collar=green; workspace=field; skill-tier=senior; device=vehicle-mount+handheld-rugged; locale=NG.
- **audience_type (per ADR-0244):** `B2B_TENANT_ADMIN+B2C_CONSUMER`.
- **Cross-context bridge (same human):** at-sea / coop-member.
- **Journeys featuring them:** j107, j108.
- **µservices their journeys touch:** audit-chain, community, connect, identity, intelligence, mail, marketplace, observability, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27, row 30.
- **Per-pack overlay applicable (per ADR-0251):** NDPR.
- **Typical day-of-week touchpoints:** Sun-Sat shift-bound; weekend on-call rotation.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 24; cross-context bridges in §4 of same file.

### §2.025 CEO Aoki Tanaka — CEO, mid-large enterprise

- **Archetype:** CEO, mid-large enterprise; collar=white; workspace=executive; skill-tier=executive; device=desktop+mobile; locale=JP.
- **audience_type (per ADR-0244):** `B2B_CSUITE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** CEO / board-elsewhere / parent.
- **Journeys featuring them:** j100, j123, j125.
- **µservices their journeys touch:** audit-chain, compliance, drive, finops-portal, identity, intelligence, messenger, ontology, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27, row 30.
- **Per-pack overlay applicable (per ADR-0251):** APPI + JP-Labor + JP-FSA.
- **Typical day-of-week touchpoints:** Mon-Fri 06:00-22:00 board cadence; Sat 1h ad-hoc.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 25; cross-context bridges in §4 of same file.

### §2.026 CFO Helena Brandt — CFO, public-company

- **Archetype:** CFO, public-company; collar=white; workspace=executive; skill-tier=executive; device=desktop; locale=DE.
- **audience_type (per ADR-0244):** `B2B_CSUITE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** CFO / charity-board.
- **Journeys featuring them:** j119, j120, j122, j137.
- **µservices their journeys touch:** audit-chain, community, compliance, connect, finops-portal, identity, mail, messenger, observability, ops-dashboard-control-center, payments, plugin-app-store, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27, row 30.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + BDSG.
- **Typical day-of-week touchpoints:** Mon-Fri 06:00-22:00 board cadence; Sat 1h ad-hoc.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 26; cross-context bridges in §4 of same file.

### §2.027 COO Akira Watanabe — COO

- **Archetype:** COO; collar=white; workspace=executive; skill-tier=executive; device=desktop; locale=JP.
- **audience_type (per ADR-0244):** `B2B_CSUITE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** COO / family.
- **Journeys featuring them:** j123, j124.
- **µservices their journeys touch:** audit-chain, drive, identity, intelligence, mail, messenger, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27, row 30.
- **Per-pack overlay applicable (per ADR-0251):** APPI + JP-Labor + JP-FSA.
- **Typical day-of-week touchpoints:** Mon-Fri 06:00-22:00 board cadence; Sat 1h ad-hoc.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 27; cross-context bridges in §4 of same file.

### §2.028 CTO Diego Vargas — CTO

- **Archetype:** CTO; collar=white; workspace=executive; skill-tier=executive; device=desktop+mobile; locale=US.
- **audience_type (per ADR-0244):** `B2B_CSUITE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** CTO / side-startup-founder.
- **Journeys featuring them:** j100, j123.
- **µservices their journeys touch:** drive, identity, intelligence, messenger, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27, row 30.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Mon-Fri 06:00-22:00 board cadence; Sat 1h ad-hoc.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 28; cross-context bridges in §4 of same file.

### §2.029 CHRO Linda Foster — Chief HR Officer

- **Archetype:** Chief HR Officer; collar=white; workspace=executive; skill-tier=executive; device=desktop; locale=US.
- **audience_type (per ADR-0244):** `B2B_CSUITE+B2B_HR_ADMIN+B2C_CONSUMER`.
- **Cross-context bridge (same human):** CHRO / mentor-board.
- **Journeys featuring them:** j132, j133, j136.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27, row 30.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Mon-Fri 06:00-22:00 board cadence; Sat 1h ad-hoc.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 29; cross-context bridges in §4 of same file.

### §2.030 CMO Felix Ng — Chief Marketing Officer

- **Archetype:** Chief Marketing Officer; collar=white; workspace=executive; skill-tier=executive; device=desktop; locale=SG.
- **audience_type (per ADR-0244):** `B2B_CSUITE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** CMO / podcaster-side.
- **Journeys featuring them:** j123.
- **µservices their journeys touch:** drive, identity, intelligence, messenger, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27, row 30.
- **Per-pack overlay applicable (per ADR-0251):** PDPA-SG + MAS-pack.
- **Typical day-of-week touchpoints:** Mon-Fri 06:00-22:00 board cadence; Sat 1h ad-hoc.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 30; cross-context bridges in §4 of same file.

### §2.031 CCO Naveen Iyer — Chief Compliance/General Counsel

- **Archetype:** Chief Compliance/General Counsel; collar=white; workspace=executive; skill-tier=executive; device=desktop; locale=IN.
- **audience_type (per ADR-0244):** `B2B_CSUITE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** CCO / pro-bono-counsel.
- **Journeys featuring them:** j129, j137, j139.
- **µservices their journeys touch:** audit-chain, comms-email, community, compliance, drive, governance, identity, mail, marketplace, messenger, ops-dashboard-control-center, payments, policy-engine, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27, row 30.
- **Per-pack overlay applicable (per ADR-0251):** DPDP-2023 + RBI.
- **Typical day-of-week touchpoints:** Mon-Fri 06:00-22:00 board cadence; Sat 1h ad-hoc.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 31; cross-context bridges in §4 of same file.

### §2.032 CISO Yuki Park — Chief Information Security Officer

- **Archetype:** Chief Information Security Officer; collar=white; workspace=executive; skill-tier=executive; device=desktop+air-gapped-mobile; locale=KR.
- **audience_type (per ADR-0244):** `B2B_CSUITE+SECURITY_RESEARCHER+B2C_CONSUMER`.
- **Cross-context bridge (same human):** CISO / IR-volunteer.
- **Journeys featuring them:** j09, j10, j15, j20, j129, j138, j139, j140.
- **µservices their journeys touch:** audit-chain, cell, comms-email, community, compliance, detection, drive, governance, identity, mail, marketplace, messenger, observability, ops-dashboard-control-center, payments, policy-engine, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27, row 30.
- **Per-pack overlay applicable (per ADR-0251):** KR-CSAP + KR-Privacy + KR-Labor + KR-119.
- **Typical day-of-week touchpoints:** Mon-Fri 06:00-22:00 board cadence; Sat 1h ad-hoc.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 32; cross-context bridges in §4 of same file.

### §2.033 CSO Mira Goldberg — Chief Strategy Officer

- **Archetype:** Chief Strategy Officer; collar=white; workspace=executive; skill-tier=executive; device=desktop; locale=US.
- **audience_type (per ADR-0244):** `B2B_CSUITE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** CSO / board-elsewhere.
- **Journeys featuring them:** j125.
- **µservices their journeys touch:** audit-chain, compliance, drive, finops-portal, identity, ontology, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27, row 30.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Mon-Fri 06:00-22:00 board cadence; Sat 1h ad-hoc.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 33; cross-context bridges in §4 of same file.

### §2.034 Board director Patrick O'Reilly — Independent board director (3 boards)

- **Archetype:** Independent board director (3 boards); collar=white; workspace=executive; skill-tier=principal; device=desktop+mobile; locale=IE.
- **audience_type (per ADR-0244):** `B2B_BOARD_DIRECTOR+B2C_CONSUMER`.
- **Cross-context bridge (same human):** Board-A / Board-B / Board-C.
- **Journeys featuring them:** j119, j125, j137.
- **µservices their journeys touch:** audit-chain, community, compliance, drive, finops-portal, identity, mail, messenger, ontology, ops-dashboard-control-center, payments, plugin-app-store, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + IE-DPC.
- **Typical day-of-week touchpoints:** Mon-Fri 06:00-22:00 board cadence; Sat 1h ad-hoc.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 34; cross-context bridges in §4 of same file.

### §2.035 Engineering Manager Aisha Ali — Eng Manager

- **Archetype:** Eng Manager; collar=white; workspace=back-office; skill-tier=senior; device=desktop; locale=US.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** manager / OSS-maintainer.
- **Journeys featuring them:** j50, j51.
- **µservices their journeys touch:** cell, identity, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 35; cross-context bridges in §4 of same file.

### §2.036 Product Manager Lily Chang — Product Manager

- **Archetype:** Product Manager; collar=white; workspace=back-office; skill-tier=senior; device=desktop+mobile; locale=US.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** PM / side-startup-founder.
- **Journeys featuring them:** j100, j123.
- **µservices their journeys touch:** drive, identity, intelligence, messenger, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 36; cross-context bridges in §4 of same file.

### §2.037 Sales Manager Anthony Costa — Sales Manager

- **Archetype:** Sales Manager; collar=white; workspace=front-office; skill-tier=senior; device=desktop+mobile; locale=US.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** manager / podcast-host.
- **Journeys featuring them:** j115.
- **µservices their journeys touch:** finops-portal, identity, observability, payments, plugin-app-store, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 37; cross-context bridges in §4 of same file.

### §2.038 Marketing Manager Olu Adeyemi — Marketing Manager

- **Archetype:** Marketing Manager; collar=white; workspace=front-office; skill-tier=senior; device=desktop+mobile; locale=NG.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** manager / content-creator.
- **Journeys featuring them:** j123.
- **µservices their journeys touch:** drive, identity, intelligence, messenger, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** NDPR.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 38; cross-context bridges in §4 of same file.

### §2.039 Customer Success Manager Sofia Rezende — CSM

- **Archetype:** CSM; collar=white; workspace=front-office; skill-tier=senior; device=desktop+mobile; locale=BR.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** CSM / family.
- **Journeys featuring them:** j109, j115, j117.
- **µservices their journeys touch:** community, finops-portal, identity, mail, messenger, observability, payments, plugin-app-store, workflow-engine, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** LGPD + BR-Labor.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 39; cross-context bridges in §4 of same file.

### §2.040 Finance Director Mei-Ling Wu — Finance Director

- **Archetype:** Finance Director; collar=white; workspace=back-office; skill-tier=senior; device=desktop; locale=TW.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** director / CPA-volunteer.
- **Journeys featuring them:** j122, j137.
- **µservices their journeys touch:** audit-chain, compliance, connect, finops-portal, identity, mail, messenger, ops-dashboard-control-center, payments, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** PDPA-TW.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 40; cross-context bridges in §4 of same file.

### §2.041 HRBP Jamal Carter — HR Business Partner

- **Archetype:** HR Business Partner; collar=white; workspace=back-office; skill-tier=mid; device=desktop+mobile; locale=US.
- **audience_type (per ADR-0244):** `B2B_HR_ADMIN+B2C_CONSUMER`.
- **Cross-context bridge (same human):** HRBP / mentor.
- **Journeys featuring them:** j132, j133, j134.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 41; cross-context bridges in §4 of same file.

### §2.042 Recruiting Manager Hina Suzuki — Recruiting Manager

- **Archetype:** Recruiting Manager; collar=white; workspace=back-office; skill-tier=senior; device=desktop+mobile; locale=JP.
- **audience_type (per ADR-0244):** `B2B_HR_ADMIN+B2C_CONSUMER`.
- **Cross-context bridge (same human):** manager / bootcamp-instructor.
- **Journeys featuring them:** j132, j134.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** APPI + JP-Labor + JP-FSA.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 42; cross-context bridges in §4 of same file.

### §2.043 Procurement Manager Wei Liu — Procurement Manager

- **Archetype:** Procurement Manager; collar=white; workspace=back-office; skill-tier=senior; device=desktop; locale=CN.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** manager / supplier-of-side-business.
- **Journeys featuring them:** j101, j103, j104.
- **µservices their journeys touch:** audit-chain, compliance, connect, identity, mail, marketplace, observability, ontology, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** PIPL.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 43; cross-context bridges in §4 of same file.

### §2.044 Legal Counsel Anika Mehta — In-house Counsel

- **Archetype:** In-house Counsel; collar=white; workspace=back-office; skill-tier=senior; device=desktop; locale=IN.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** counsel / pro-bono.
- **Journeys featuring them:** j129, j135, j141.
- **µservices their journeys touch:** audit-chain, comms-email, community, compliance, drive, governance, identity, marketplace, messenger, payments, policy-engine, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** DPDP-2023 + RBI.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 44; cross-context bridges in §4 of same file.

### §2.045 Compliance Officer Tunde Bello — Compliance Officer

- **Archetype:** Compliance Officer; collar=white; workspace=middle-office; skill-tier=senior; device=desktop; locale=NG.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** officer / PTA-member.
- **Journeys featuring them:** j122, j131, j135, j137, j138, j139, j140, j141.
- **µservices their journeys touch:** audit-chain, community, compliance, connect, detection, finops-portal, governance, identity, mail, messenger, observability, ops-dashboard-control-center, payments, policy-engine, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** NDPR.
- **Typical day-of-week touchpoints:** Mon-Fri controls cadence; month-end peak; weekend evidence pull.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 45; cross-context bridges in §4 of same file.

### §2.046 DevOps Manager Pavel Korsak — DevOps Manager

- **Archetype:** DevOps Manager; collar=white; workspace=back-office; skill-tier=senior; device=desktop+mobile; locale=UA.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** manager / OSS-maintainer.
- **Journeys featuring them:** j46, j117.
- **µservices their journeys touch:** compliance, connect, finops-portal, identity, mail, messenger, observability, payments, workflow-engine, workflow-studio.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** Ukraine-DPA.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 46; cross-context bridges in §4 of same file.

### §2.047 IT Manager Jamie O'Connor — IT Manager

- **Archetype:** IT Manager; collar=white; workspace=back-office; skill-tier=senior; device=desktop; locale=IE.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2B_HR_ADMIN+B2C_CONSUMER`.
- **Cross-context bridge (same human):** manager / PC-club-organizer.
- **Journeys featuring them:** j20, j47, j117.
- **µservices their journeys touch:** cell, compliance, connect, finops-portal, mail, messenger, observability, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + IE-DPC.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 47; cross-context bridges in §4 of same file.

### §2.048 Office Manager Priya Ramanathan — Office Manager

- **Archetype:** Office Manager; collar=white; workspace=back-office; skill-tier=mid; device=desktop+mobile; locale=IN.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** manager / PTA-treasurer.
- **Journeys featuring them:** j136.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** DPDP-2023 + RBI.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 48; cross-context bridges in §4 of same file.

### §2.049 SWE Hugo Tanaka — SWE

- **Archetype:** SWE; collar=white; workspace=back-office; skill-tier=mid; device=desktop+mobile; locale=JP.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** at-work / OSS-contrib.
- **Journeys featuring them:** j47, j50, j51, j52, j53, j54.
- **µservices their journeys touch:** cell, compliance, connect, identity, mail, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** APPI + JP-Labor + JP-FSA.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 49; cross-context bridges in §4 of same file.

### §2.050 Sales AE Maya Lindqvist — Account Executive

- **Archetype:** Account Executive; collar=white; workspace=front-office; skill-tier=mid; device=desktop+mobile; locale=SE.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** AE / podcaster.
- **Journeys featuring them:** j115.
- **µservices their journeys touch:** finops-portal, identity, observability, payments, plugin-app-store, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + IMY.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 50; cross-context bridges in §4 of same file.

### §2.051 SDR Kofi Asante — Sales Development Rep

- **Archetype:** Sales Development Rep; collar=white; workspace=front-office; skill-tier=junior; device=desktop+mobile; locale=GH.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER+B2C_JOB_SEEKER_ACTIVE`.
- **Cross-context bridge (same human):** SDR / side-hustle-creator.
- **Journeys featuring them:** j145.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** Ghana-DPA.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 51; cross-context bridges in §4 of same file.

### §2.052 Marketing Specialist Riya Sharma — Marketing Specialist

- **Archetype:** Marketing Specialist; collar=white; workspace=front-office; skill-tier=mid; device=desktop+mobile; locale=IN.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** specialist / blogger.
- **Journeys featuring them:** j150.
- **µservices their journeys touch:** community, finops-portal, identity, intelligence, ontology, payments, plugin-app-store, shorts.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** DPDP-2023 + RBI.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 52; cross-context bridges in §4 of same file.

### §2.053 CS-IC Lin Chen — CS Specialist

- **Archetype:** CS Specialist; collar=white; workspace=front-office; skill-tier=mid; device=desktop+mobile; locale=TW.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** CSM / mentor.
- **Journeys featuring them:** j117.
- **µservices their journeys touch:** finops-portal, mail, messenger, observability, payments, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** PDPA-TW.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 53; cross-context bridges in §4 of same file.

### §2.054 Support Rep Nadia Hassani — Customer Support Rep

- **Archetype:** Customer Support Rep; collar=white; workspace=front-office; skill-tier=junior; device=desktop+headset; locale=FR.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** rep / grad-student.
- **Journeys featuring them:** j117.
- **µservices their journeys touch:** finops-portal, mail, messenger, observability, payments, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + CNIL.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 54; cross-context bridges in §4 of same file.

### §2.055 Financial Analyst Wendy Lee — Financial Analyst

- **Archetype:** Financial Analyst; collar=white; workspace=back-office; skill-tier=mid; device=desktop; locale=US.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** analyst / CPA-candidate.
- **Journeys featuring them:** j122, j137.
- **µservices their journeys touch:** audit-chain, compliance, connect, finops-portal, identity, mail, messenger, ops-dashboard-control-center, payments, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 55; cross-context bridges in §4 of same file.

### §2.056 Accountant Ravi Iyer — Accountant

- **Archetype:** Accountant; collar=white; workspace=back-office; skill-tier=mid; device=desktop; locale=IN.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** accountant / freelance.
- **Journeys featuring them:** j122, j137.
- **µservices their journeys touch:** audit-chain, compliance, connect, finops-portal, identity, mail, messenger, ops-dashboard-control-center, payments, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** DPDP-2023 + RBI.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 56; cross-context bridges in §4 of same file.

### §2.057 Tax Analyst Ji-Sung Park — Tax Analyst

- **Archetype:** Tax Analyst; collar=white; workspace=back-office; skill-tier=mid; device=desktop; locale=KR.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** at-work / family-tax-helper.
- **Journeys featuring them:** j122, j128.
- **µservices their journeys touch:** audit-chain, compliance, connect, drive, finops-portal, identity, intelligence, mail, marketplace, notes, payments, workflow-engine, workflow-studio.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** KR-CSAP + KR-Privacy + KR-Labor + KR-119.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 57; cross-context bridges in §4 of same file.

### §2.058 External Auditor Dimitri Volkov — External Auditor (Big-4)

- **Archetype:** External Auditor (Big-4); collar=white; workspace=middle-office; skill-tier=mid; device=desktop+mobile; locale=DE.
- **audience_type (per ADR-0244):** `B2B_EXTERNAL_AUDITOR+B2C_CONSUMER`.
- **Cross-context bridge (same human):** auditor-A / auditor-B.
- **Journeys featuring them:** j126, j131.
- **µservices their journeys touch:** api-gateway, audit-chain, comms-email, compliance, identity, messenger, observability, ops-dashboard-control-center, policy-engine, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + BDSG.
- **Typical day-of-week touchpoints:** Mon-Fri controls cadence; month-end peak; weekend evidence pull.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 58; cross-context bridges in §4 of same file.

### §2.059 HR Specialist Aoife Murphy — HR Specialist

- **Archetype:** HR Specialist; collar=white; workspace=back-office; skill-tier=mid; device=desktop; locale=IE.
- **audience_type (per ADR-0244):** `B2B_HR_ADMIN+B2C_CONSUMER`.
- **Cross-context bridge (same human):** HR-Specialist / Benefits-Specialist.
- **Journeys featuring them:** j132, j133, j134, j135, j136.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + IE-DPC.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 59; cross-context bridges in §4 of same file.

### §2.060 Recruiter Marcus IV — Recruiter

- **Archetype:** Recruiter; collar=white; workspace=back-office; skill-tier=junior; device=desktop+mobile; locale=US.
- **audience_type (per ADR-0244):** `B2B_HR_ADMIN+B2C_CONSUMER`.
- **Cross-context bridge (same human):** recruiter (distinct from Marcus Chen).
- **Journeys featuring them:** j132, j134, j145.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 60; cross-context bridges in §4 of same file.

### §2.061 Procurement Specialist Beata Kowalski — Procurement Specialist

- **Archetype:** Procurement Specialist; collar=white; workspace=back-office; skill-tier=mid; device=desktop; locale=PL.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** at-work / PTA.
- **Journeys featuring them:** j101, j103, j104.
- **µservices their journeys touch:** audit-chain, compliance, connect, identity, mail, marketplace, observability, ontology, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + UODO.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 61; cross-context bridges in §4 of same file.

### §2.062 Legal Operations Stephen Park — Legal Ops

- **Archetype:** Legal Ops; collar=white; workspace=back-office; skill-tier=mid; device=desktop; locale=KR.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** legal-ops / paralegal-side.
- **Journeys featuring them:** j141.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** KR-CSAP + KR-Privacy + KR-Labor + KR-119.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 62; cross-context bridges in §4 of same file.

### §2.063 Compliance Analyst Yui Hayashi — Compliance Analyst

- **Archetype:** Compliance Analyst; collar=white; workspace=middle-office; skill-tier=mid; device=desktop; locale=JP.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** analyst / volunteer-treasurer.
- **Journeys featuring them:** j131, j138, j139.
- **µservices their journeys touch:** audit-chain, community, compliance, detection, governance, identity, mail, observability, ops-dashboard-control-center, payments, policy-engine, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** APPI + JP-Labor + JP-FSA.
- **Typical day-of-week touchpoints:** Mon-Fri controls cadence; month-end peak; weekend evidence pull.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 63; cross-context bridges in §4 of same file.

### §2.064 DevOps Engineer Olukayode Adejumo — DevOps Engineer

- **Archetype:** DevOps Engineer; collar=white; workspace=back-office; skill-tier=mid; device=desktop+mobile; locale=NG.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** engineer / OSS-maintainer.
- **Journeys featuring them:** j46, j117.
- **µservices their journeys touch:** compliance, connect, finops-portal, identity, mail, messenger, observability, payments, workflow-engine, workflow-studio.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** NDPR.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 64; cross-context bridges in §4 of same file.

### §2.065 Security Analyst Anna Petrova — Security Analyst

- **Archetype:** Security Analyst; collar=white; workspace=middle-office; skill-tier=mid; device=desktop+air-gapped-mobile; locale=RU.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** analyst / CTF-player.
- **Journeys featuring them:** j10, j15, j138, j140.
- **µservices their journeys touch:** audit-chain, community, detection, identity, mail, messenger, observability, payments, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** RU-152-FZ.
- **Typical day-of-week touchpoints:** Mon-Fri controls cadence; month-end peak; weekend evidence pull.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 65; cross-context bridges in §4 of same file.

### §2.066 Data Analyst Felipe Andrade — Data Analyst

- **Archetype:** Data Analyst; collar=white; workspace=back-office; skill-tier=mid; device=desktop; locale=BR.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** analyst / freelancer.
- **Journeys featuring them:** j138, j150.
- **µservices their journeys touch:** audit-chain, community, detection, finops-portal, identity, intelligence, mail, ontology, payments, plugin-app-store, shorts, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** LGPD + BR-Labor.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 66; cross-context bridges in §4 of same file.

### §2.067 Data Scientist Yu Chen — Data Scientist

- **Archetype:** Data Scientist; collar=white; workspace=back-office; skill-tier=senior; device=desktop+GPU; locale=TW.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** DS / Kaggle-grandmaster.
- **Journeys featuring them:** j138, j150.
- **µservices their journeys touch:** audit-chain, community, detection, finops-portal, identity, intelligence, mail, ontology, payments, plugin-app-store, shorts, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** PDPA-TW.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 67; cross-context bridges in §4 of same file.

### §2.068 Product Designer Akihiro Sato — Product Designer

- **Archetype:** Product Designer; collar=white; workspace=back-office; skill-tier=senior; device=desktop+tablet; locale=JP.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** designer / art-instructor.
- **Journeys featuring them:** j50, j52.
- **µservices their journeys touch:** cell, identity, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** APPI + JP-Labor + JP-FSA.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 68; cross-context bridges in §4 of same file.

### §2.069 UX Researcher Adaeze Nwosu — UX Researcher

- **Archetype:** UX Researcher; collar=white; workspace=back-office; skill-tier=mid; device=desktop+mobile; locale=NG.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** researcher / grad-student.
- **Journeys featuring them:** j52, j69.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** NDPR.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 69; cross-context bridges in §4 of same file.

### §2.070 Project Manager Soo-Jin Park — Project Manager

- **Archetype:** Project Manager; collar=white; workspace=back-office; skill-tier=mid; device=desktop+mobile; locale=KR.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** PM / marathon-runner.
- **Journeys featuring them:** j47, j123.
- **µservices their journeys touch:** compliance, connect, drive, identity, intelligence, mail, messenger, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** KR-CSAP + KR-Privacy + KR-Labor + KR-119.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 70; cross-context bridges in §4 of same file.

### §2.071 Business Analyst Aditya Verma — Business Analyst

- **Archetype:** Business Analyst; collar=white; workspace=back-office; skill-tier=mid; device=desktop; locale=IN.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** BA / finance-blogger.
- **Journeys featuring them:** j119.
- **µservices their journeys touch:** audit-chain, community, compliance, finops-portal, payments, plugin-app-store.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** DPDP-2023 + RBI.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 71; cross-context bridges in §4 of same file.

### §2.072 Communications Specialist Charlotte Dubois — Comms Specialist

- **Archetype:** Comms Specialist; collar=white; workspace=front-office; skill-tier=mid; device=desktop+mobile; locale=FR.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** comms / novelist.
- **Journeys featuring them:** j123.
- **µservices their journeys touch:** drive, identity, intelligence, messenger, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + CNIL.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 72; cross-context bridges in §4 of same file.

### §2.073 Training Specialist Mehmet Yilmaz — Training & Dev Specialist

- **Archetype:** Training & Dev Specialist; collar=white; workspace=back-office; skill-tier=mid; device=desktop+tablet; locale=TR.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+EDU_TEACHER+B2C_CONSUMER`.
- **Cross-context bridge (same human):** trainer / Udemy-instructor.
- **Journeys featuring them:** j47, j110.
- **µservices their journeys touch:** community, compliance, connect, identity, mail, payments, tenancy, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** KVKK.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 73; cross-context bridges in §4 of same file.

### §2.074 Office Coordinator Phoebe Lin — Office Coordinator

- **Archetype:** Office Coordinator; collar=pink; workspace=back-office; skill-tier=junior; device=desktop+mobile; locale=TW.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** coordinator / grad-student.
- **Journeys featuring them:** j72.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** PDPA-TW.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 74; cross-context bridges in §4 of same file.

### §2.075 Receptionist Daria Volkova — Receptionist

- **Archetype:** Receptionist; collar=pink; workspace=front-office; skill-tier=junior; device=kiosk+mobile; locale=UA.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** receptionist / art-student.
- **Journeys featuring them:** j110.
- **µservices their journeys touch:** community, identity, payments, tenancy, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** Ukraine-DPA.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 75; cross-context bridges in §4 of same file.

### §2.076 Executive Assistant Olivia Reyes — EA

- **Archetype:** EA; collar=white; workspace=back-office; skill-tier=senior; device=desktop+mobile; locale=US.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** EA / parent.
- **Journeys featuring them:** j27, j123.
- **µservices their journeys touch:** drive, identity, intelligence, messenger, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 76; cross-context bridges in §4 of same file.

### §2.077 Paralegal Tomáš Novák — Paralegal

- **Archetype:** Paralegal; collar=white; workspace=back-office; skill-tier=mid; device=desktop; locale=CZ.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** paralegal / cycling-club-treasurer.
- **Journeys featuring them:** j118, j141.
- **µservices their journeys touch:** audit-chain, compliance, identity, ontology, tenancy.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + UOOU.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 77; cross-context bridges in §4 of same file.

### §2.078 IR Manager Lev Kahn — Investor Relations Manager

- **Archetype:** Investor Relations Manager; collar=white; workspace=front-office; skill-tier=senior; device=desktop+mobile; locale=IL.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2B_INVESTOR_LP+B2C_CONSUMER`.
- **Cross-context bridge (same human):** IR / LP-of-fund.
- **Journeys featuring them:** j119, j121.
- **µservices their journeys touch:** audit-chain, community, compliance, connect, finops-portal, identity, payments, plugin-app-store, tenancy, workflow-engine, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** PPL-IL.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 78; cross-context bridges in §4 of same file.

### §2.079 Corp Dev Senior Analyst Saanvi Mehta — Corp Dev Senior Analyst

- **Archetype:** Corp Dev Senior Analyst; collar=white; workspace=back-office; skill-tier=senior; device=desktop; locale=IN.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** analyst / MBA-applicant.
- **Journeys featuring them:** j125.
- **µservices their journeys touch:** audit-chain, compliance, drive, finops-portal, identity, ontology, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** DPDP-2023 + RBI.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 79; cross-context bridges in §4 of same file.

### §2.080 Board Secretary Florence Akinsanya — Board Secretary

- **Archetype:** Board Secretary; collar=white; workspace=executive; skill-tier=senior; device=desktop+mobile; locale=NG.
- **audience_type (per ADR-0244):** `B2B_BOARD_DIRECTOR+B2C_CONSUMER`.
- **Cross-context bridge (same human):** secretary / mentor.
- **Journeys featuring them:** j118, j137.
- **µservices their journeys touch:** audit-chain, compliance, identity, mail, messenger, ontology, ops-dashboard-control-center, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** NDPR.
- **Typical day-of-week touchpoints:** Mon-Fri 06:00-22:00 board cadence; Sat 1h ad-hoc.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 80; cross-context bridges in §4 of same file.

### §2.081 Internal Comms Lead Ji-Ho Yoon — Internal Comms Lead

- **Archetype:** Internal Comms Lead; collar=white; workspace=back-office; skill-tier=senior; device=desktop+mobile; locale=KR.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** comms / novelist.
- **Journeys featuring them:** j123.
- **µservices their journeys touch:** drive, identity, intelligence, messenger, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** KR-CSAP + KR-Privacy + KR-Labor + KR-119.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 81; cross-context bridges in §4 of same file.

### §2.082 Sustainability Officer Aiko Brown — Sustainability Officer

- **Archetype:** Sustainability Officer; collar=green; workspace=middle-office; skill-tier=senior; device=desktop+mobile; locale=JP.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** officer / climate-activist.
- **Journeys featuring them:** j148.
- **µservices their journeys touch:** audit-chain, community, connect, ontology, payments, plugin-app-store, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** APPI + JP-Labor + JP-FSA.
- **Typical day-of-week touchpoints:** Mon-Fri controls cadence; month-end peak; weekend evidence pull.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 82; cross-context bridges in §4 of same file.

### §2.083 D&I Director Maya Okoroafor — D&I Director

- **Archetype:** D&I Director; collar=white; workspace=back-office; skill-tier=senior; device=desktop+mobile; locale=NG.
- **audience_type (per ADR-0244):** `B2B_HR_ADMIN+B2C_CONSUMER`.
- **Cross-context bridge (same human):** director / board-advisor.
- **Journeys featuring them:** j132, j135.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** NDPR.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 83; cross-context bridges in §4 of same file.

### §2.084 Ombudsperson Felix Tan — Ombudsperson

- **Archetype:** Ombudsperson; collar=white; workspace=middle-office; skill-tier=senior; device=desktop+mobile; locale=SG.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** ombudsperson / mediator-side.
- **Journeys featuring them:** j135.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** PDPA-SG + MAS-pack.
- **Typical day-of-week touchpoints:** Mon-Fri controls cadence; month-end peak; weekend evidence pull.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 84; cross-context bridges in §4 of same file.

### §2.085 Strategic Advisor Rita Almeida — Strategic Advisor

- **Archetype:** Strategic Advisor; collar=white; workspace=executive; skill-tier=principal; device=desktop+mobile; locale=PT.
- **audience_type (per ADR-0244):** `B2B_EXTERNAL_COUNSEL+B2C_CONSUMER`.
- **Cross-context bridge (same human):** advisor-A / advisor-B.
- **Journeys featuring them:** j125.
- **µservices their journeys touch:** audit-chain, compliance, drive, finops-portal, identity, ontology, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + CNPD-PT.
- **Typical day-of-week touchpoints:** Mon-Fri 06:00-22:00 board cadence; Sat 1h ad-hoc.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 85; cross-context bridges in §4 of same file.

### §2.086 Venture Partner Lucas Müller — Venture Partner

- **Archetype:** Venture Partner; collar=white; workspace=executive; skill-tier=principal; device=desktop+mobile; locale=DE.
- **audience_type (per ADR-0244):** `B2B_INVESTOR_LP+B2C_CONSUMER`.
- **Cross-context bridge (same human):** VC / LP-of-other-fund.
- **Journeys featuring them:** j119, j125.
- **µservices their journeys touch:** audit-chain, community, compliance, drive, finops-portal, identity, ontology, payments, plugin-app-store, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + BDSG.
- **Typical day-of-week touchpoints:** Mon-Fri 06:00-22:00 board cadence; Sat 1h ad-hoc.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 86; cross-context bridges in §4 of same file.

### §2.087 Investor/LP Aanya Kapoor — Limited Partner

- **Archetype:** Limited Partner; collar=white; workspace=executive; skill-tier=senior; device=desktop+mobile; locale=IN.
- **audience_type (per ADR-0244):** `B2B_INVESTOR_LP+B2C_CONSUMER`.
- **Cross-context bridge (same human):** LP-A / LP-B / board.
- **Journeys featuring them:** j119.
- **µservices their journeys touch:** audit-chain, community, compliance, finops-portal, payments, plugin-app-store.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** DPDP-2023 + RBI.
- **Typical day-of-week touchpoints:** Mon-Fri 06:00-22:00 board cadence; Sat 1h ad-hoc.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 87; cross-context bridges in §4 of same file.

### §2.088 Customer Champion Akemi Sato — Customer Champion

- **Archetype:** Customer Champion; collar=white; workspace=front-office; skill-tier=mid; device=desktop+mobile; locale=JP.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** champion / customer-elsewhere.
- **Journeys featuring them:** j100, j117.
- **µservices their journeys touch:** finops-portal, mail, messenger, observability, payments, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** APPI + JP-Labor + JP-FSA.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 88; cross-context bridges in §4 of same file.

### §2.089 Channel Partner Tomas Pieter — Channel Partner

- **Archetype:** Channel Partner; collar=white; workspace=front-office; skill-tier=mid; device=desktop+mobile; locale=NL.
- **audience_type (per ADR-0244):** `B2B_CHANNEL_PARTNER+B2C_CONSUMER`.
- **Cross-context bridge (same human):** partner / employee-of-partner-co.
- **Journeys featuring them:** j111, j115.
- **µservices their journeys touch:** community, finops-portal, identity, observability, payments, plugin-app-store, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 27.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + AP.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 89; cross-context bridges in §4 of same file.

### §2.090 External Auditor Hyo-Jin Lee — External Auditor (Big-4 KR)

- **Archetype:** External Auditor (Big-4 KR); collar=white; workspace=middle-office; skill-tier=senior; device=desktop+mobile; locale=KR.
- **audience_type (per ADR-0244):** `B2B_EXTERNAL_AUDITOR+B2C_CONSUMER`.
- **Cross-context bridge (same human):** auditor-A / auditor-B.
- **Journeys featuring them:** j126, j131.
- **µservices their journeys touch:** api-gateway, audit-chain, comms-email, compliance, identity, messenger, observability, ops-dashboard-control-center, policy-engine, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** KR-CSAP + KR-Privacy + KR-Labor + KR-119.
- **Typical day-of-week touchpoints:** Mon-Fri controls cadence; month-end peak; weekend evidence pull.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 90; cross-context bridges in §4 of same file.

### §2.091 External Regulator Inspector Sergei Petrov — Regulator (KR FSS-equiv)

- **Archetype:** Regulator (KR FSS-equiv); collar=white; workspace=middle-office; skill-tier=senior; device=desktop+mobile; locale=UA→KR.
- **audience_type (per ADR-0244):** `B2B_REGULATOR_EXTERNAL+GOV_INSPECTOR+B2C_CONSUMER`.
- **Cross-context bridge (same human):** regulator / private-citizen.
- **Journeys featuring them:** j126, j129, j131.
- **µservices their journeys touch:** api-gateway, audit-chain, comms-email, community, compliance, drive, governance, identity, marketplace, messenger, observability, ops-dashboard-control-center, payments, policy-engine, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** Ukraine-DPA + KR-CSAP.
- **Typical day-of-week touchpoints:** Mon-Fri controls cadence; month-end peak; weekend evidence pull.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 91; cross-context bridges in §4 of same file.

### §2.092 Banker (external) Hideki Watanabe — External Banker (relationship)

- **Archetype:** External Banker (relationship); collar=white; workspace=front-office; skill-tier=senior; device=desktop+mobile; locale=JP.
- **audience_type (per ADR-0244):** `B2B_BANK_INTERNAL+B2C_CONSUMER`.
- **Cross-context bridge (same human):** banker-A / banker-B.
- **Journeys featuring them:** j106, j121.
- **µservices their journeys touch:** audit-chain, compliance, connect, finops-portal, identity, payments, tenancy, workflow-engine, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** APPI + JP-Labor + JP-FSA.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 92; cross-context bridges in §4 of same file.

### §2.093 Consultant Adekunle Adebayo — Mgmt Consultant

- **Archetype:** Mgmt Consultant; collar=white; workspace=back-office; skill-tier=senior; device=desktop+mobile; locale=NG.
- **audience_type (per ADR-0244):** `B2B_EXTERNAL_COUNSEL+B2C_CONSUMER`.
- **Cross-context bridge (same human):** consultant-A / consultant-B.
- **Journeys featuring them:** j125.
- **µservices their journeys touch:** audit-chain, compliance, drive, finops-portal, identity, ontology, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** NDPR.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 93; cross-context bridges in §4 of same file.

### §2.094 PR Firm Beatriz Fernandez — External PR

- **Archetype:** External PR; collar=white; workspace=front-office; skill-tier=senior; device=desktop+mobile; locale=ES.
- **audience_type (per ADR-0244):** `B2B_CHANNEL_PARTNER+B2C_CONSUMER`.
- **Cross-context bridge (same human):** PR-A / PR-B.
- **Journeys featuring them:** j123.
- **µservices their journeys touch:** drive, identity, intelligence, messenger, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 27.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + LOPDGDD.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 94; cross-context bridges in §4 of same file.

### §2.095 Auditor IT-Specialist Jakub Nowak — IT Auditor (external)

- **Archetype:** IT Auditor (external); collar=white; workspace=middle-office; skill-tier=senior; device=desktop; locale=PL.
- **audience_type (per ADR-0244):** `B2B_EXTERNAL_AUDITOR+B2C_CONSUMER`.
- **Cross-context bridge (same human):** IT-auditor / CISSP-instructor.
- **Journeys featuring them:** j126, j140.
- **µservices their journeys touch:** api-gateway, audit-chain, comms-email, compliance, identity, messenger, observability, ops-dashboard-control-center, policy-engine, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + UODO.
- **Typical day-of-week touchpoints:** Mon-Fri controls cadence; month-end peak; weekend evidence pull.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 95; cross-context bridges in §4 of same file.

### §2.096 Outside Counsel Wei-Yi Chen — External Counsel

- **Archetype:** External Counsel; collar=white; workspace=back-office; skill-tier=principal; device=desktop+mobile; locale=HK.
- **audience_type (per ADR-0244):** `B2B_EXTERNAL_COUNSEL+B2C_CONSUMER`.
- **Cross-context bridge (same human):** counsel-A / counsel-B / consumer.
- **Journeys featuring them:** j118, j125, j129, j141.
- **µservices their journeys touch:** audit-chain, comms-email, community, compliance, drive, finops-portal, governance, identity, marketplace, messenger, ontology, payments, policy-engine, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** PDPO-HK.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 96; cross-context bridges in §4 of same file.

### §2.097 Mailroom Hae-Won Kim — Mailroom Staff

- **Archetype:** Mailroom Staff; collar=blue; workspace=back-office; skill-tier=junior; device=mobile+scanner; locale=KR.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** mailroom / art-student.
- **Journeys featuring them:** _(no anchored journey — coverage gap, see §9)_.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** KR-CSAP + KR-Privacy + KR-Labor + KR-119.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 97; cross-context bridges in §4 of same file.

### §2.098 Maintenance Tech Carlos Reyes II — Building Maintenance

- **Archetype:** Building Maintenance; collar=gray; workspace=back-office; skill-tier=mid; device=mobile+handheld-rugged; locale=US.
- **audience_type (per ADR-0244):** `B2B_FIELD_WORKER+B2C_CONSUMER+B2C_FAMILY_PARENT`.
- **Cross-context bridge (same human):** maintenance / father.
- **Journeys featuring them:** _(no anchored journey — coverage gap, see §9)_.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 30.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 98; cross-context bridges in §4 of same file.

### §2.099 Security Guard Stefan Kovács — Security Guard

- **Archetype:** Security Guard; collar=gray; workspace=front-office; skill-tier=junior; device=mobile+kiosk; locale=HU.
- **audience_type (per ADR-0244):** `B2B_FIELD_WORKER+B2C_CONSUMER`.
- **Cross-context bridge (same human):** guard / college-student.
- **Journeys featuring them:** _(no anchored journey — coverage gap, see §9)_.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 30.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + NAIH.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 99; cross-context bridges in §4 of same file.

### §2.100 Cleaning Supervisor Tomáš Horák — Cleaning Supervisor

- **Archetype:** Cleaning Supervisor; collar=blue; workspace=back-office; skill-tier=mid; device=mobile+handheld-rugged; locale=CZ.
- **audience_type (per ADR-0244):** `B2B_FIELD_WORKER+B2B_TENANT_ADMIN`.
- **Cross-context bridge (same human):** supervisor / cleaning-co-owner.
- **Journeys featuring them:** j111.
- **µservices their journeys touch:** community, identity, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 18, row 24, row 27, row 30.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + UOOU.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 100; cross-context bridges in §4 of same file.

### §2.101 Cafeteria Manager Soyeon Kim — Cafeteria Manager

- **Archetype:** Cafeteria Manager; collar=pink+green; workspace=production; skill-tier=mid; device=mobile+kiosk; locale=KR.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** manager / mother.
- **Journeys featuring them:** j101.
- **µservices their journeys touch:** audit-chain, compliance, identity, mail, marketplace, ontology, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** KR-CSAP + KR-Privacy + KR-Labor + KR-119.
- **Typical day-of-week touchpoints:** shift-rotation incl. weekends; daily safety toolbox.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 101; cross-context bridges in §4 of same file.

### §2.102 Print Operator Diana Lazăr — Print Operator

- **Archetype:** Print Operator; collar=gray; workspace=production; skill-tier=junior; device=mobile+kiosk; locale=RO.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** operator / college-student.
- **Journeys featuring them:** _(no anchored journey — coverage gap, see §9)_.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + ANSPDCP.
- **Typical day-of-week touchpoints:** shift-rotation incl. weekends; daily safety toolbox.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 102; cross-context bridges in §4 of same file.

### §2.103 AV Coordinator Jordan Park — AV/Conferencing Coordinator

- **Archetype:** AV/Conferencing Coordinator; collar=gray; workspace=back-office; skill-tier=mid; device=desktop+mobile+handheld-rugged; locale=KR.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** AV / musician-side.
- **Journeys featuring them:** j28.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** KR-CSAP + KR-Privacy + KR-Labor + KR-119.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 103; cross-context bridges in §4 of same file.

### §2.104 Investment Banker Yuna Ahn — IB (M&A)

- **Archetype:** IB (M&A); collar=white; workspace=front-office; skill-tier=senior; device=desktop+mobile-regulated; locale=KR.
- **audience_type (per ADR-0244):** `B2B_BANK_INTERNAL+B2C_CONSUMER`.
- **Cross-context bridge (same human):** IB / MBA-applicant.
- **Journeys featuring them:** j119, j121, j125.
- **µservices their journeys touch:** audit-chain, community, compliance, connect, drive, finops-portal, identity, ontology, payments, plugin-app-store, tenancy, workflow-engine, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** KR-CSAP + KR-Privacy + KR-Labor + KR-119.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 104; cross-context bridges in §4 of same file.

### §2.105 Commercial Banker Frederik Hartmann — Commercial Banker

- **Archetype:** Commercial Banker; collar=white; workspace=front-office; skill-tier=senior; device=desktop+mobile; locale=DE.
- **audience_type (per ADR-0244):** `B2B_BANK_INTERNAL+B2C_CONSUMER`.
- **Cross-context bridge (same human):** banker-A / banker-B.
- **Journeys featuring them:** j121.
- **µservices their journeys touch:** connect, finops-portal, identity, payments, tenancy, workflow-engine, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + BDSG.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 105; cross-context bridges in §4 of same file.

### §2.106 Retail Banker Sebastián Vega — Retail Banker (branch)

- **Archetype:** Retail Banker (branch); collar=white; workspace=front-office; skill-tier=mid; device=desktop+kiosk; locale=ES.
- **audience_type (per ADR-0244):** `B2B_BANK_INTERNAL+B2C_CONSUMER`.
- **Cross-context bridge (same human):** branch-mgr / side-tutor.
- **Journeys featuring them:** j08, j09, j121.
- **µservices their journeys touch:** connect, finops-portal, identity, mail, messenger, payments, tenancy, workflow-engine, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + LOPDGDD.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 106; cross-context bridges in §4 of same file.

### §2.107 Trader Mei Lin — Sell-side Equities Trader

- **Archetype:** Sell-side Equities Trader; collar=white+gold; workspace=front-office; skill-tier=senior; device=desktop+air-gapped-mobile; locale=HK.
- **audience_type (per ADR-0244):** `B2B_BANK_INTERNAL+B2C_CONSUMER`.
- **Cross-context bridge (same human):** trader / marathon-runner.
- **Journeys featuring them:** j120.
- **µservices their journeys touch:** connect, finops-portal, observability, payments, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** PDPO-HK.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 107; cross-context bridges in §4 of same file.

### §2.108 Wealth Manager Aamir Khan — Wealth Manager (PB)

- **Archetype:** Wealth Manager (PB); collar=white; workspace=front-office; skill-tier=senior; device=desktop+mobile; locale=AE.
- **audience_type (per ADR-0244):** `B2B_BANK_INTERNAL+B2C_CONSUMER`.
- **Cross-context bridge (same human):** WM / LP-of-fund.
- **Journeys featuring them:** j08, j119.
- **µservices their journeys touch:** audit-chain, community, compliance, finops-portal, identity, messenger, payments, plugin-app-store, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** DIFC-DPL + UAE-PDPL.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 108; cross-context bridges in §4 of same file.

### §2.109 Treasury Ops Sven Eriksson — Treasury Ops Analyst

- **Archetype:** Treasury Ops Analyst; collar=white; workspace=middle-office; skill-tier=mid; device=desktop; locale=SE.
- **audience_type (per ADR-0244):** `B2B_BANK_INTERNAL+B2C_CONSUMER`.
- **Cross-context bridge (same human):** treasury-ops / CFA-candidate.
- **Journeys featuring them:** j120, j122.
- **µservices their journeys touch:** compliance, connect, finops-portal, mail, observability, payments, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + IMY.
- **Typical day-of-week touchpoints:** Mon-Fri controls cadence; month-end peak; weekend evidence pull.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 109; cross-context bridges in §4 of same file.

### §2.110 Bank Ops Officer Olamide Adebanjo — Bank Operations Officer

- **Archetype:** Bank Operations Officer; collar=white; workspace=back-office; skill-tier=mid; device=desktop; locale=NG.
- **audience_type (per ADR-0244):** `B2B_BANK_INTERNAL+B2C_CONSUMER`.
- **Cross-context bridge (same human):** ops / side-business-owner.
- **Journeys featuring them:** j121, j122.
- **µservices their journeys touch:** compliance, connect, finops-portal, identity, mail, payments, tenancy, workflow-engine, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** NDPR.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 110; cross-context bridges in §4 of same file.

### §2.111 Credit Analyst Hina Mori — Bank Credit Analyst

- **Archetype:** Bank Credit Analyst; collar=white; workspace=middle-office; skill-tier=mid; device=desktop; locale=JP.
- **audience_type (per ADR-0244):** `B2B_BANK_INTERNAL+B2C_CONSUMER`.
- **Cross-context bridge (same human):** credit-analyst / CFA-candidate.
- **Journeys featuring them:** j121.
- **µservices their journeys touch:** connect, finops-portal, identity, payments, tenancy, workflow-engine, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** APPI + JP-Labor + JP-FSA.
- **Typical day-of-week touchpoints:** Mon-Fri controls cadence; month-end peak; weekend evidence pull.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 111; cross-context bridges in §4 of same file.

### §2.112 Bank Compliance Officer Rishi Bhattacharya — Bank Compliance Officer

- **Archetype:** Bank Compliance Officer; collar=white; workspace=middle-office; skill-tier=senior; device=desktop; locale=IN.
- **audience_type (per ADR-0244):** `B2B_BANK_INTERNAL+B2C_CONSUMER`.
- **Cross-context bridge (same human):** bank-comp / volunteer-treasurer.
- **Journeys featuring them:** j121, j122, j131.
- **µservices their journeys touch:** audit-chain, compliance, connect, finops-portal, identity, mail, observability, payments, policy-engine, tenancy, workflow-engine, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** DPDP-2023 + RBI.
- **Typical day-of-week touchpoints:** Mon-Fri controls cadence; month-end peak; weekend evidence pull.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 112; cross-context bridges in §4 of same file.

### §2.113 Bank Risk Manager Anders Pedersen — Bank Risk Manager

- **Archetype:** Bank Risk Manager; collar=white; workspace=middle-office; skill-tier=senior; device=desktop; locale=DK.
- **audience_type (per ADR-0244):** `B2B_BANK_INTERNAL+B2C_CONSUMER`.
- **Cross-context bridge (same human):** risk-mgr / cycling-club-officer.
- **Journeys featuring them:** j120, j121.
- **µservices their journeys touch:** connect, finops-portal, identity, observability, payments, tenancy, workflow-engine, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 18, row 24, row 27.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + DK-DPA.
- **Typical day-of-week touchpoints:** Mon-Fri controls cadence; month-end peak; weekend evidence pull.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 113; cross-context bridges in §4 of same file.

### §2.114 Summer Intern Priscilla Sharma — Summer Intern (SWE)

- **Archetype:** Summer Intern (SWE); collar=white; workspace=back-office; skill-tier=in-training; device=desktop+mobile; locale=IN.
- **audience_type (per ADR-0244):** `B2B_APPRENTICE_INTERN+B2C_CONSUMER+EDU_STUDENT`.
- **Cross-context bridge (same human):** intern / undergrad.
- **Journeys featuring them:** j47, j113.
- **µservices their journeys touch:** calendar, community, compliance, connect, identity, mail, messenger, payments, tenancy, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 27.
- **Per-pack overlay applicable (per ADR-0251):** DPDP-2023 + RBI.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 114; cross-context bridges in §4 of same file.

### §2.115 Co-op Student Liam Murphy — Engineering Co-op

- **Archetype:** Engineering Co-op; collar=white; workspace=back-office; skill-tier=in-training; device=desktop+mobile; locale=IE.
- **audience_type (per ADR-0244):** `B2B_APPRENTICE_INTERN+B2C_CONSUMER+EDU_STUDENT`.
- **Cross-context bridge (same human):** co-op / undergrad.
- **Journeys featuring them:** j113.
- **µservices their journeys touch:** calendar, community, identity, messenger, payments, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 27.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + IE-DPC.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 115; cross-context bridges in §4 of same file.

### §2.116 Returning Intern Jia Han — Returning Intern (2nd-yr)

- **Archetype:** Returning Intern (2nd-yr); collar=white; workspace=back-office; skill-tier=in-training; device=desktop+mobile; locale=CN.
- **audience_type (per ADR-0244):** `B2B_APPRENTICE_INTERN+B2C_CONSUMER+EDU_STUDENT`.
- **Cross-context bridge (same human):** returning-intern / undergrad.
- **Journeys featuring them:** j113.
- **µservices their journeys touch:** calendar, community, identity, messenger, payments, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 27.
- **Per-pack overlay applicable (per ADR-0251):** PIPL.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 116; cross-context bridges in §4 of same file.

### §2.117 Intern Manager Felicia Adamou — Intern Manager

- **Archetype:** Intern Manager; collar=white; workspace=back-office; skill-tier=mid; device=desktop+mobile; locale=CI.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** manager / conference-speaker.
- **Journeys featuring them:** j113.
- **µservices their journeys touch:** calendar, community, identity, messenger, payments, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** ARTCI.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 117; cross-context bridges in §4 of same file.

### §2.118 Apprentice Jakob Bauer — Skilled-trades Apprentice (electrician)

- **Archetype:** Skilled-trades Apprentice (electrician); collar=blue; workspace=field; skill-tier=in-training; device=mobile+handheld-rugged; locale=DE.
- **audience_type (per ADR-0244):** `B2B_APPRENTICE_INTERN+B2C_CONSUMER`.
- **Cross-context bridge (same human):** apprentice / trade-school-student.
- **Journeys featuring them:** j109.
- **µservices their journeys touch:** community, identity, observability, payments, workflow-engine, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 27.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + BDSG.
- **Typical day-of-week touchpoints:** Sun-Sat shift-bound; weekend on-call rotation.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 118; cross-context bridges in §4 of same file.

### §2.119 Medical Resident Dr. Sun-Mi Kim — Medical Resident (PGY-3)

- **Archetype:** Medical Resident (PGY-3); collar=gold; workspace=clinical; skill-tier=in-training; device=clinical-PACS+mobile; locale=KR.
- **audience_type (per ADR-0244):** `B2B_MEDICAL_RESIDENT+B2C_CONSUMER`.
- **Cross-context bridge (same human):** resident / grad-school-applicant.
- **Journeys featuring them:** j02, j110.
- **µservices their journeys touch:** community, identity, payments, tenancy, workplace-integration.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 18, row 24.
- **Per-pack overlay applicable (per ADR-0251):** KR-CSAP + KR-Privacy + KR-Labor + KR-119.
- **Typical day-of-week touchpoints:** shift-bound (PGY-3 every 4th-on-call); weekend rounds.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 119; cross-context bridges in §4 of same file.

### §2.120 Fellow Dr. Tobias Klein — Postdoctoral Fellow (research)

- **Archetype:** Postdoctoral Fellow (research); collar=gold; workspace=back-office+clinical; skill-tier=in-training; device=desktop+clinical-PACS; locale=DE.
- **audience_type (per ADR-0244):** `B2B_MEDICAL_RESIDENT+B2C_CONSUMER`.
- **Cross-context bridge (same human):** fellow / postdoc-applicant.
- **Journeys featuring them:** j02.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 18, row 24.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + BDSG.
- **Typical day-of-week touchpoints:** shift-bound (PGY-3 every 4th-on-call); weekend rounds.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 120; cross-context bridges in §4 of same file.

### §2.121 Benefits Specialist Aoife Murphy — Benefits Specialist (cross-ref §3.5 row 59 — same human)

- **Archetype:** Benefits Specialist (cross-ref §3.5 row 59 — same human); collar=white; workspace=back-office; skill-tier=mid; device=desktop; locale=IE.
- **audience_type (per ADR-0244):** `B2B_HR_ADMIN+B2C_CONSUMER`.
- **Cross-context bridge (same human):** HR-Specialist / Benefits-Specialist.
- **Journeys featuring them:** j136.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + IE-DPC.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 121; cross-context bridges in §4 of same file.

### §2.122 Total Rewards Manager Nilufer Demir — Total Rewards Manager

- **Archetype:** Total Rewards Manager; collar=white; workspace=back-office; skill-tier=senior; device=desktop; locale=TR.
- **audience_type (per ADR-0244):** `B2B_HR_ADMIN+B2C_CONSUMER`.
- **Cross-context bridge (same human):** TR-mgr / CCP-credentialed.
- **Journeys featuring them:** j136.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** KVKK.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 122; cross-context bridges in §4 of same file.

### §2.123 Leave Specialist Margarethe Reinhart — Leave-of-Absence Specialist

- **Archetype:** Leave-of-Absence Specialist; collar=white; workspace=back-office; skill-tier=mid; device=desktop; locale=DE.
- **audience_type (per ADR-0244):** `B2B_HR_ADMIN+B2C_CONSUMER`.
- **Cross-context bridge (same human):** leave / mother.
- **Journeys featuring them:** j136.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** GDPR + BDSG.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 123; cross-context bridges in §4 of same file.

### §2.124 Wellness Program Manager Akira Sato — Wellness Program Manager

- **Archetype:** Wellness Program Manager; collar=white+pink; workspace=back-office; skill-tier=mid; device=desktop+mobile; locale=JP.
- **audience_type (per ADR-0244):** `B2B_HR_ADMIN+B2C_CONSUMER`.
- **Cross-context bridge (same human):** wellness / yoga-instructor.
- **Journeys featuring them:** j136.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** APPI + JP-Labor + JP-FSA.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 124; cross-context bridges in §4 of same file.

### §2.125 Retirement Plan Admin Bryce Williams — Retirement Plan Administrator

- **Archetype:** Retirement Plan Administrator; collar=white; workspace=back-office; skill-tier=senior; device=desktop; locale=US.
- **audience_type (per ADR-0244):** `B2B_HR_ADMIN+B2C_CONSUMER`.
- **Cross-context bridge (same human):** plan-admin / PTA-treasurer.
- **Journeys featuring them:** j136.
- **µservices their journeys touch:** _(ambient-only; resolves to identity+tenancy+audit-chain)_.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 8, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Mon-Fri 09:00-18:00 deep-work; Wed all-hands.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 125; cross-context bridges in §4 of same file.

### §2.126 Corporate Relations Director Soo-Yeon Han — Corporate Relations Director

- **Archetype:** Corporate Relations Director; collar=white; workspace=front-office; skill-tier=senior; device=desktop+mobile; locale=KR.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** director / mentor-board.
- **Journeys featuring them:** j123.
- **µservices their journeys touch:** drive, identity, intelligence, messenger, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** KR-CSAP + KR-Privacy + KR-Labor + KR-119.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 126; cross-context bridges in §4 of same file.

### §2.127 Public Affairs Director Carlos Mendez — Public Affairs Director

- **Archetype:** Public Affairs Director; collar=white; workspace=front-office; skill-tier=senior; device=desktop+mobile; locale=MX.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** PA / pro-bono-advisor.
- **Journeys featuring them:** j124, j129.
- **µservices their journeys touch:** audit-chain, comms-email, community, compliance, drive, governance, identity, mail, marketplace, messenger, payments, policy-engine, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** LFPDPPP.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 127; cross-context bridges in §4 of same file.

### §2.128 PR Manager Helena Sato — PR Manager (internal)

- **Archetype:** PR Manager (internal); collar=white; workspace=front-office; skill-tier=mid; device=desktop+mobile; locale=JP.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** PR / novelist.
- **Journeys featuring them:** j123.
- **µservices their journeys touch:** drive, identity, intelligence, messenger, payments, tenancy, workflow-engine.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** APPI + JP-Labor + JP-FSA.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 128; cross-context bridges in §4 of same file.

### §2.129 IR Specialist (unnamed) — Investor Relations Specialist

- **Archetype:** Investor Relations Specialist; collar=white; workspace=front-office; skill-tier=mid; device=desktop+mobile; locale=US.
- **audience_type (per ADR-0244):** `B2B_EMPLOYEE+B2C_CONSUMER`.
- **Cross-context bridge (same human):** IR-spec / CFA.
- **Journeys featuring them:** j119.
- **µservices their journeys touch:** audit-chain, community, compliance, finops-portal, payments, plugin-app-store.
- **Critical-path rows triggered (per documentation-rigor.md §3.2.5):** row 1, row 9, row 14, row 24.
- **Per-pack overlay applicable (per ADR-0251):** SOC2 + HIPAA + CCPA + state-by-state.
- **Typical day-of-week touchpoints:** Mon-Fri customer-facing; quarter-end sprints.
- **MASTER-ROSTER citation:** see `docs/personas/MASTER-ROSTER-2026-05-21.md` §3 row 129; cross-context bridges in §4 of same file.

---

## §3 Per-Journey Cross-Coverage Table

Each journey row enumerates: slug, primary persona(s), µservices touched (extracted from the journey's `README.md microservices_touched:` block), pack overlays activated, critical-path rows satisfied, artifact counts (story.md + ux-flow.md + handshake.md + per-µservice IPs), and cross-references to related journeys (extracted from the `## Cross-references` section of each README).

Cross-reference: `docs/user-journeys/j<NN>-*/README.md` for each journey; CATALOG-j126-j150-ecosystem.md for j126-j150 archetypes.

### §3.001 j01-emergency-911-dispatch — Life-safety + critical-path

- **Journey title (humanized):** emergency 911 dispatch.
- **Primary persona(s):** Captain Chen; secondary: Captain Chen, Officer Rodriguez, Yejin Park.
- **µservices touched (count=15):** api-gateway, audit-chain, calendar, cell, compliance, consent-graph, identity, intelligence, mail, messenger, notes, observability, ontology, tenancy, workflow-engine.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥6000 (across 15 µservices).
- **Cross-references to related journeys:** j04-dv-survivor-shelter-mode, j07-deceased-user-inheritance-handoff, j12-mass-casualty-incident-10x-traffic, j13-cross-jurisdiction-eu-cloud-act-conflict, j14-delegated-llm-agent-acting-for-yejin.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j01-emergency-911-dispatch/README.md`.

### §3.002 j02-healthcare-code-blue-ehr-break-glass — Life-safety + critical-path

- **Journey title (humanized):** healthcare code blue ehr break glass.
- **Primary persona(s):** Dr. Tanaka; secondary: Dr. Tanaka, Fellow Dr. Tobias Klein, Medical Resident Dr. Sun-Mi Kim, Yejin Park.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** pack-hipaa-2024 + pack-kr-medical-records-act.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j02-healthcare-code-blue-ehr-break-glass/README.md`.

### §3.003 j03-988-crisis-line-minor-self-report — Life-safety + critical-path

- **Journey title (humanized):** 988 crisis line minor self report.
- **Primary persona(s):** Jordan Lee; secondary: Jordan Lee.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** pack-coppa + pack-kosa.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j03-988-crisis-line-minor-self-report/README.md`.

### §3.004 j04-dv-survivor-shelter-mode — Life-safety + critical-path

- **Journey title (humanized):** dv survivor shelter mode.
- **Primary persona(s):** Officer Rodriguez; secondary: Officer Rodriguez, Yejin Park.
- **µservices touched (count=6):** consent-graph, drive, identity, mail, messenger, observability.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j127-dual-tenant-identity-employee-resigns-and-keeps-personal.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j04-dv-survivor-shelter-mode/README.md`.

### §3.005 j05-whistleblower-anonymous-ethics-report — Life-safety + critical-path

- **Journey title (humanized):** whistleblower anonymous ethics report.
- **Primary persona(s):** Anya Mironova; secondary: Anya Mironova.
- **µservices touched (count=4):** audit-chain, community, identity, observability.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥1600 (across 4 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j05-whistleblower-anonymous-ethics-report/README.md`.

### §3.006 j06-press-source-securedrop-class — Life-safety + critical-path

- **Journey title (humanized):** press source securedrop class.
- **Primary persona(s):** Anya Mironova; secondary: Anya Mironova, Father Lopez, Yejin Park.
- **µservices touched (count=4):** audit-chain, community, drive, messenger.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥1600 (across 4 µservices).
- **Cross-references to related journeys:** j129-court-warrant-pierces-personal-tenant-with-judicial-oversight.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j06-press-source-securedrop-class/README.md`.

### §3.007 j07-deceased-user-inheritance-handoff — Life-safety + critical-path

- **Journey title (humanized):** deceased user inheritance handoff.
- **Primary persona(s):** Hiroshi Tanaka; secondary: Hiroshi Tanaka.
- **µservices touched (count=6):** audit-chain, drive, identity, mail, notes, payments.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j38-b2b-e-signing-contract, j45-healthcare-patient-portal-records, j101-multi-tier-supply-chain-formation, j105-dispute-cross-tenant-arbitration.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j07-deceased-user-inheritance-handoff/README.md`.

### §3.008 j08-elder-financial-abuse-detection — Life-safety + critical-path

- **Journey title (humanized):** elder financial abuse detection.
- **Primary persona(s):** Hiroshi Tanaka; secondary: Hiroshi Tanaka, Retail Banker Sebastián Vega, Wealth Manager Aamir Khan.
- **µservices touched (count=4):** identity, messenger, payments, workflow-engine.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥1600 (across 4 µservices).
- **Cross-references to related journeys:** j123-multi-tenant-coordinated-product-launch, j129-court-warrant-pierces-personal-tenant-with-judicial-oversight, j137-corporate-internal-audit-sox-controls-test.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j08-elder-financial-abuse-detection/README.md`.

### §3.009 j09-account-recovery-phishing-resistant — Life-safety + critical-path

- **Journey title (humanized):** account recovery phishing resistant.
- **Primary persona(s):** CISO Yuki Park; secondary: CISO Yuki Park, Retail Banker Sebastián Vega, Yejin Park.
- **µservices touched (count=3):** identity, mail, messenger.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥1200 (across 3 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j09-account-recovery-phishing-resistant/README.md`.

### §3.010 j10-account-takeover-SIM-swap-detected — Life-safety + critical-path

- **Journey title (humanized):** account takeover SIM swap detected.
- **Primary persona(s):** CISO Yuki Park; secondary: CISO Yuki Park, Officer Rodriguez, Security Analyst Anna Petrova.
- **µservices touched (count=4):** identity, messenger, observability, payments.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥1600 (across 4 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j10-account-takeover-SIM-swap-detected/README.md`.

### §3.011 j11-disaster-zone-offline-first-sync — Life-safety + critical-path

- **Journey title (humanized):** disaster zone offline first sync.
- **Primary persona(s):** Captain Chen; secondary: Captain Chen, Yejin Park.
- **µservices touched (count=5):** cell, connect, drive, messenger, notes.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2000 (across 5 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j11-disaster-zone-offline-first-sync/README.md`.

### §3.012 j12-mass-casualty-incident-10x-traffic — Life-safety + critical-path

- **Journey title (humanized):** mass casualty incident 10x traffic.
- **Primary persona(s):** Captain Chen; secondary: Captain Chen, Dr. Tanaka.
- **µservices touched (count=4):** api-gateway, audit-chain, cell, observability.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥1600 (across 4 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j12-mass-casualty-incident-10x-traffic/README.md`.

### §3.013 j13-cross-jurisdiction-eu-cloud-act-conflict — Life-safety + critical-path

- **Journey title (humanized):** cross jurisdiction eu cloud act conflict.
- **Primary persona(s):** Marcus Chen; secondary: Marcus Chen.
- **µservices touched (count=4):** compliance, intelligence, observability, tenancy.
- **Per-pack overlay activated:** pack-gdpr + pack-eu-ai-act + pack-dora.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥1600 (across 4 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j13-cross-jurisdiction-eu-cloud-act-conflict/README.md`.

### §3.014 j14-delegated-llm-agent-acting-for-yejin — Life-safety + critical-path

- **Journey title (humanized):** delegated llm agent acting for yejin.
- **Primary persona(s):** Yejin Park; secondary: Yejin Park.
- **µservices touched (count=5):** audit-chain, identity, intelligence, messenger, workflow-engine.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2000 (across 5 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j123-multi-tenant-coordinated-product-launch, j124-supply-chain-disruption-emergency-coordination, j126-government-auditor-3pao-conducts-fedramp-audit, j127-dual-tenant-identity-employee-resigns-and-keeps-personal.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j14-delegated-llm-agent-acting-for-yejin/README.md`.

### §3.015 j15-bug-bounty-researcher-submission — Life-safety + critical-path

- **Journey title (humanized):** bug bounty researcher submission.
- **Primary persona(s):** Anya Mironova; secondary: Anya Mironova, CISO Yuki Park, Marcus Chen, Security Analyst Anna Petrova.
- **µservices touched (count=2):** audit-chain, community.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥800 (across 2 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j15-bug-bounty-researcher-submission/README.md`.

### §3.016 j16-disability-accommodation-voice-only-signup — Life-safety + critical-path

- **Journey title (humanized):** disability accommodation voice only signup.
- **Primary persona(s):** Jordan Lee; secondary: Jordan Lee, Ms. Patel, Yejin Park.
- **µservices touched (count=3):** application, identity, intelligence.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥1200 (across 3 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j16-disability-accommodation-voice-only-signup/README.md`.

### §3.017 j17-activist-dissident-high-risk-mode — Life-safety + critical-path

- **Journey title (humanized):** activist dissident high risk mode.
- **Primary persona(s):** Anya Mironova; secondary: Anya Mironova, Marcus Chen, Ms. Patel.
- **µservices touched (count=4):** community, drive, identity, messenger.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥1600 (across 4 µservices).
- **Cross-references to related journeys:** j129-court-warrant-pierces-personal-tenant-with-judicial-oversight.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j17-activist-dissident-high-risk-mode/README.md`.

### §3.018 j18-child-safety-mandatory-reporter — Life-safety + critical-path

- **Journey title (humanized):** child safety mandatory reporter.
- **Primary persona(s):** Officer Rodriguez; secondary: Officer Rodriguez.
- **µservices touched (count=5):** audit-chain, community, identity, mail, workflow-engine.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2000 (across 5 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j101-multi-tier-supply-chain-formation, j124-supply-chain-disruption-emergency-coordination, j127-dual-tenant-identity-employee-resigns-and-keeps-personal, j128-auditor-personal-side-uses-workflow-studio-for-family-taxes.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j18-child-safety-mandatory-reporter/README.md`.

### §3.019 j19-tenant-break-glass-locked-out-tenant-admin — Life-safety + critical-path

- **Journey title (humanized):** tenant break glass locked out tenant admin.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=4):** audit-chain, governance, identity, ops-dashboard-control-center.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥1600 (across 4 µservices).
- **Cross-references to related journeys:** j139-internal-audit-policy-violation-cedar-permit-misuse.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j19-tenant-break-glass-locked-out-tenant-admin/README.md`.

### §3.020 j20-data-residency-violation-detection — Life-safety + critical-path

- **Journey title (humanized):** data residency violation detection.
- **Primary persona(s):** CISO Yuki Park; secondary: CISO Yuki Park, IT Manager Jamie O'Connor.
- **µservices touched (count=4):** cell, compliance, observability, tenancy.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 1 (life-safety); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥1600 (across 4 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j20-data-residency-violation-detection/README.md`.

### §3.021 j21-personal-signup-passkey-first-dm — Personal day-to-day

- **Journey title (humanized):** personal signup passkey first dm.
- **Primary persona(s):** Yejin Park; secondary: Yejin Park.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j21-personal-signup-passkey-first-dm/README.md`.

### §3.022 j22-personal-mail-inbox-first-week — Personal day-to-day

- **Journey title (humanized):** personal mail inbox first week.
- **Primary persona(s):** Dr. Tanaka; secondary: Dr. Tanaka, Hiroshi Tanaka, Yejin Park.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j22-personal-mail-inbox-first-week/README.md`.

### §3.023 j23-marketplace-listing-and-first-sale — Personal day-to-day

- **Journey title (humanized):** marketplace listing and first sale.
- **Primary persona(s):** Tomás García; secondary: Tomás García.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** pack-marketplace-baseline + pack-creator-monetization.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j23-marketplace-listing-and-first-sale/README.md`.

### §3.024 j24-marketplace-purchase-as-buyer — Personal day-to-day

- **Journey title (humanized):** marketplace purchase as buyer.
- **Primary persona(s):** Tomás García; secondary: Tomás García.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** pack-marketplace-baseline + pack-creator-monetization.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j24-marketplace-purchase-as-buyer/README.md`.

### §3.025 j25-personal-notes-daily-journaling-with-e2e — Personal day-to-day

- **Journey title (humanized):** personal notes daily journaling with e2e.
- **Primary persona(s):** Yejin Park; secondary: Yejin Park.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j25-personal-notes-daily-journaling-with-e2e/README.md`.

### §3.026 j26-drive-family-photo-backup — Personal day-to-day

- **Journey title (humanized):** drive family photo backup.
- **Primary persona(s):** Hiroshi Tanaka; secondary: Hiroshi Tanaka, Yejin Park.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j26-drive-family-photo-backup/README.md`.

### §3.027 j27-calendar-cross-context-family-and-work — Personal day-to-day

- **Journey title (humanized):** calendar cross context family and work.
- **Primary persona(s):** Executive Assistant Olivia Reyes; secondary: Executive Assistant Olivia Reyes, Hiroshi Tanaka, Yejin Park.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j27-calendar-cross-context-family-and-work/README.md`.

### §3.028 j28-meet-family-video-call — Personal day-to-day

- **Journey title (humanized):** meet family video call.
- **Primary persona(s):** AV Coordinator Jordan Park; secondary: AV Coordinator Jordan Park, Yejin Park.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j28-meet-family-video-call/README.md`.

### §3.029 j29-workflow-studio-personal-automation — Personal day-to-day

- **Journey title (humanized):** workflow studio personal automation.
- **Primary persona(s):** Yejin Park; secondary: Yejin Park.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j29-workflow-studio-personal-automation/README.md`.

### §3.030 j30-shorts-creator-first-post — Personal day-to-day

- **Journey title (humanized):** shorts creator first post.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** pack-marketplace-baseline + pack-creator-monetization.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j30-shorts-creator-first-post/README.md`.

### §3.031 j31-social-broadcast-vs-DM — Personal day-to-day

- **Journey title (humanized):** social broadcast vs DM.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j31-social-broadcast-vs-DM/README.md`.

### §3.032 j32-community-teamblind-employer-anonymous — Personal day-to-day

- **Journey title (humanized):** community teamblind employer anonymous.
- **Primary persona(s):** Ms. Patel; secondary: Ms. Patel.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j32-community-teamblind-employer-anonymous/README.md`.

### §3.033 j33-b2b-sso-saml-onboarding — Personal day-to-day

- **Journey title (humanized):** b2b sso saml onboarding.
- **Primary persona(s):** Maria Santos; secondary: Maria Santos, Tomás García.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j33-b2b-sso-saml-onboarding/README.md`.

### §3.034 j34-b2b-team-channel-with-files — Personal day-to-day

- **Journey title (humanized):** b2b team channel with files.
- **Primary persona(s):** Anya Mironova; secondary: Anya Mironova.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j34-b2b-team-channel-with-files/README.md`.

### §3.035 j35-b2b-workplace-mail-and-calendar — Personal day-to-day

- **Journey title (humanized):** b2b workplace mail and calendar.
- **Primary persona(s):** Tomás García; secondary: Tomás García.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j35-b2b-workplace-mail-and-calendar/README.md`.

### §3.036 j36-b2b-workflow-engine-approval-cascade — Personal day-to-day

- **Journey title (humanized):** b2b workflow engine approval cascade.
- **Primary persona(s):** Yejin Park; secondary: Yejin Park.
- **µservices touched (count=5):** identity, mail, payments, workflow-engine, workflow-studio.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2000 (across 5 µservices).
- **Cross-references to related journeys:** j46-healthcare-prescription-renewal-workflow, j101-multi-tier-supply-chain-formation, j128-auditor-personal-side-uses-workflow-studio-for-family-taxes, j137-corporate-internal-audit-sox-controls-test.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j36-b2b-workflow-engine-approval-cascade/README.md`.

### §3.037 j37-b2b-clocking-and-attendance — Personal day-to-day

- **Journey title (humanized):** b2b clocking and attendance.
- **Primary persona(s):** Yejin Park; secondary: Yejin Park.
- **µservices touched (count=5):** connect, identity, observability, payments, workplace-integration.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2000 (across 5 µservices).
- **Cross-references to related journeys:** j109-construction-co-hires-freelance-specialist, j121-business-loan-application-from-bank-tenant.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j37-b2b-clocking-and-attendance/README.md`.

### §3.038 j38-b2b-e-signing-contract — Personal day-to-day

- **Journey title (humanized):** b2b e signing contract.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=5):** audit-chain, drive, identity, mail, workplace-integration.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2000 (across 5 µservices).
- **Cross-references to related journeys:** j07-deceased-user-inheritance-handoff, j45-healthcare-patient-portal-records, j127-dual-tenant-identity-employee-resigns-and-keeps-personal, j128-auditor-personal-side-uses-workflow-studio-for-family-taxes.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j38-b2b-e-signing-contract/README.md`.

### §3.039 j39-b2b-meeting-with-transcription — Personal day-to-day

- **Journey title (humanized):** b2b meeting with transcription.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=6):** drive, intelligence, meet, notes, observability, recordings.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j39-b2b-meeting-with-transcription/README.md`.

### §3.040 j40-b2b-marketplace-vendor-billing — Personal day-to-day

- **Journey title (humanized):** b2b marketplace vendor billing.
- **Primary persona(s):** Aiyana Singh; secondary: Aiyana Singh.
- **µservices touched (count=4):** mail, payments, plugin-app-store, tenancy.
- **Per-pack overlay activated:** pack-marketplace-baseline + pack-creator-monetization.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 9 (revocation lag); row 14 (minor protection).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥1600 (across 4 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j40-b2b-marketplace-vendor-billing/README.md`.

### §3.041 j41-b2b-developer-builds-on-platform — Tenant onboarding + workforce

- **Journey title (humanized):** b2b developer builds on platform.
- **Primary persona(s):** Carlos Martinez; secondary: Carlos Martinez, Marcus Chen.
- **µservices touched (count=5):** developer-sdk, foundry, identity, observability, workflow-engine.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2000 (across 5 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j41-b2b-developer-builds-on-platform/README.md`.

### §3.042 j42-b2b-finops-portal-spend-attribution — Tenant onboarding + workforce

- **Journey title (humanized):** b2b finops portal spend attribution.
- **Primary persona(s):** Carlos Martinez; secondary: Carlos Martinez.
- **µservices touched (count=4):** finops-portal, identity, observability, tenancy.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥1600 (across 4 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j42-b2b-finops-portal-spend-attribution/README.md`.

### §3.043 j43-healthcare-nurse-patient-handoff — Tenant onboarding + workforce

- **Journey title (humanized):** healthcare nurse patient handoff.
- **Primary persona(s):** Carlos Martinez; secondary: Carlos Martinez.
- **µservices touched (count=6):** audit-chain, compliance, identity, intelligence, notes, ontology.
- **Per-pack overlay activated:** pack-hipaa-2024 + pack-kr-medical-records-act.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j44-healthcare-telemedicine-consultation, j45-healthcare-patient-portal-records, j101-multi-tier-supply-chain-formation, j104-supplier-vendor-onboarding-kyb-cascade.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j43-healthcare-nurse-patient-handoff/README.md`.

### §3.044 j44-healthcare-telemedicine-consultation — Tenant onboarding + workforce

- **Journey title (humanized):** healthcare telemedicine consultation.
- **Primary persona(s):** Carlos Martinez; secondary: Carlos Martinez.
- **µservices touched (count=6):** audit-chain, compliance, connect, intelligence, meet, notes.
- **Per-pack overlay activated:** pack-hipaa-2024 + pack-kr-medical-records-act.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j43-healthcare-nurse-patient-handoff, j128-auditor-personal-side-uses-workflow-studio-for-family-taxes.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j44-healthcare-telemedicine-consultation/README.md`.

### §3.045 j45-healthcare-patient-portal-records — Tenant onboarding + workforce

- **Journey title (humanized):** healthcare patient portal records.
- **Primary persona(s):** Sarah Kim; secondary: Sarah Kim.
- **µservices touched (count=6):** audit-chain, compliance, drive, identity, mail, notes.
- **Per-pack overlay activated:** pack-hipaa-2024 + pack-kr-medical-records-act.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j07-deceased-user-inheritance-handoff, j38-b2b-e-signing-contract, j43-healthcare-nurse-patient-handoff, j101-multi-tier-supply-chain-formation.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j45-healthcare-patient-portal-records/README.md`.

### §3.046 j46-healthcare-prescription-renewal-workflow — Tenant onboarding + workforce

- **Journey title (humanized):** healthcare prescription renewal workflow.
- **Primary persona(s):** Aiyana Singh; secondary: Aiyana Singh, DevOps Engineer Olukayode Adejumo, DevOps Manager Pavel Korsak, Sarah Kim.
- **µservices touched (count=6):** compliance, connect, identity, mail, workflow-engine, workflow-studio.
- **Per-pack overlay activated:** pack-hipaa-2024 + pack-kr-medical-records-act.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j36-b2b-workflow-engine-approval-cascade, j101-multi-tier-supply-chain-formation, j104-supplier-vendor-onboarding-kyb-cascade, j122-vendor-payment-batch-with-tax-withholding.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j46-healthcare-prescription-renewal-workflow/README.md`.

### §3.047 j47-healthcare-billing-and-insurance — Tenant onboarding + workforce

- **Journey title (humanized):** healthcare billing and insurance.
- **Primary persona(s):** Aiyana Singh; secondary: Aiyana Singh, IT Manager Jamie O'Connor, Project Manager Soo-Jin Park, SWE Hugo Tanaka, Summer Intern Priscilla Sharma, Training Specialist Mehmet Yilmaz.
- **µservices touched (count=5):** compliance, connect, mail, payments, tenancy.
- **Per-pack overlay activated:** pack-hipaa-2024 + pack-kr-medical-records-act.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2000 (across 5 µservices).
- **Cross-references to related journeys:** j48-sidebusiness-stripe-tax-and-invoicing, j101-multi-tier-supply-chain-formation, j122-vendor-payment-batch-with-tax-withholding.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j47-healthcare-billing-and-insurance/README.md`.

### §3.048 j48-sidebusiness-stripe-tax-and-invoicing — Tenant onboarding + workforce

- **Journey title (humanized):** sidebusiness stripe tax and invoicing.
- **Primary persona(s):** Aiyana Singh; secondary: Aiyana Singh.
- **µservices touched (count=5):** compliance, connect, finops-portal, mail, payments.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2000 (across 5 µservices).
- **Cross-references to related journeys:** j47-healthcare-billing-and-insurance, j122-vendor-payment-batch-with-tax-withholding.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j48-sidebusiness-stripe-tax-and-invoicing/README.md`.

### §3.049 j49-sidebusiness-customer-support-omnichannel — Tenant onboarding + workforce

- **Journey title (humanized):** sidebusiness customer support omnichannel.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=6):** community, connect, intelligence, mail, messenger, plugin-app-store.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j49-sidebusiness-customer-support-omnichannel/README.md`.

### §3.050 j50-sidebusiness-employee-hires-first-helper — Tenant onboarding + workforce

- **Journey title (humanized):** sidebusiness employee hires first helper.
- **Primary persona(s):** Aiyana Singh; secondary: Aiyana Singh, Engineering Manager Aisha Ali, Product Designer Akihiro Sato, SWE Hugo Tanaka.
- **µservices touched (count=5):** cell, identity, payments, tenancy, workflow-engine.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2000 (across 5 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j101-multi-tier-supply-chain-formation, j111-staffing-agency-as-tenant-facilitator, j114-employee-secondment-cross-tenant, j121-business-loan-application-from-bank-tenant.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j50-sidebusiness-employee-hires-first-helper/README.md`.

### §3.051 j51-procure-to-pay-po-extraction-and-approval — Tenant onboarding + workforce

- **Journey title (humanized):** procure to pay po extraction and approval.
- **Primary persona(s):** Aiyana Singh; secondary: Aiyana Singh, Engineering Manager Aisha Ali, SWE Hugo Tanaka.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j51-procure-to-pay-po-extraction-and-approval/README.md`.

### §3.052 j52-order-to-cash-marketplace-to-fulfillment — Tenant onboarding + workforce

- **Journey title (humanized):** order to cash marketplace to fulfillment.
- **Primary persona(s):** Aiyana Singh; secondary: Aiyana Singh, Product Designer Akihiro Sato, SWE Hugo Tanaka, UX Researcher Adaeze Nwosu.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** pack-marketplace-baseline + pack-creator-monetization.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j52-order-to-cash-marketplace-to-fulfillment/README.md`.

### §3.053 j53-invoice-to-cash-recurring-subscription — Tenant onboarding + workforce

- **Journey title (humanized):** invoice to cash recurring subscription.
- **Primary persona(s):** Aiyana Singh; secondary: Aiyana Singh, SWE Hugo Tanaka.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j53-invoice-to-cash-recurring-subscription/README.md`.

### §3.054 j54-quote-to-contract-to-payment-saas — Tenant onboarding + workforce

- **Journey title (humanized):** quote to contract to payment saas.
- **Primary persona(s):** Aiyana Singh; secondary: Aiyana Singh, SWE Hugo Tanaka.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j54-quote-to-contract-to-payment-saas/README.md`.

### §3.055 j55-refund-and-dispute-resolution-cascade — Tenant onboarding + workforce

- **Journey title (humanized):** refund and dispute resolution cascade.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j55-refund-and-dispute-resolution-cascade/README.md`.

### §3.056 j56-job-application-to-offer — Tenant onboarding + workforce

- **Journey title (humanized):** job application to offer.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j56-job-application-to-offer/README.md`.

### §3.057 j57-employee-onboarding-day-one-to-week-one — Tenant onboarding + workforce

- **Journey title (humanized):** employee onboarding day one to week one.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j57-employee-onboarding-day-one-to-week-one/README.md`.

### §3.058 j58-quarterly-performance-review-cycle — Tenant onboarding + workforce

- **Journey title (humanized):** quarterly performance review cycle.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j58-quarterly-performance-review-cycle/README.md`.

### §3.059 j59-offboarding-and-knowledge-transfer — Tenant onboarding + workforce

- **Journey title (humanized):** offboarding and knowledge transfer.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j59-offboarding-and-knowledge-transfer/README.md`.

### §3.060 j60-internal-mobility-promotion-cascade — Tenant onboarding + workforce

- **Journey title (humanized):** internal mobility promotion cascade.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j60-internal-mobility-promotion-cascade/README.md`.

### §3.061 j61-patient-intake-to-followup — Side-business + creator-economy

- **Journey title (humanized):** patient intake to followup.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j61-patient-intake-to-followup/README.md`.

### §3.062 j62-prescription-to-pharmacy-to-payment — Side-business + creator-economy

- **Journey title (humanized):** prescription to pharmacy to payment.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j62-prescription-to-pharmacy-to-payment/README.md`.

### §3.063 j63-clinical-trial-recruitment-to-consent — Side-business + creator-economy

- **Journey title (humanized):** clinical trial recruitment to consent.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j63-clinical-trial-recruitment-to-consent/README.md`.

### §3.064 j64-hospital-network-cross-tenant-referral — Side-business + creator-economy

- **Journey title (humanized):** hospital network cross tenant referral.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j64-hospital-network-cross-tenant-referral/README.md`.

### §3.065 j65-gdpr-dsar-cascade-across-all-services — Side-business + creator-economy

- **Journey title (humanized):** gdpr dsar cascade across all services.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j65-gdpr-dsar-cascade-across-all-services/README.md`.

### §3.066 j66-tax-quarterly-filing-multi-jurisdiction — Side-business + creator-economy

- **Journey title (humanized):** tax quarterly filing multi jurisdiction.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j66-tax-quarterly-filing-multi-jurisdiction/README.md`.

### §3.067 j67-law-enforcement-warrant-response — Side-business + creator-economy

- **Journey title (humanized):** law enforcement warrant response.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** judicial-piercing (ADR-0312).
- **README citation:** `docs/user-journeys/j67-law-enforcement-warrant-response/README.md`.

### §3.068 j68-regulator-audit-pull-hippa-soc2-pci — Side-business + creator-economy

- **Journey title (humanized):** regulator audit pull hippa soc2 pci.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j68-regulator-audit-pull-hippa-soc2-pci/README.md`.

### §3.069 j69-llm-agent-managing-yejins-week — Side-business + creator-economy

- **Journey title (humanized):** llm agent managing yejins week.
- **Primary persona(s):** Hiroshi Tanaka; secondary: Hiroshi Tanaka, UX Researcher Adaeze Nwosu.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j69-llm-agent-managing-yejins-week/README.md`.

### §3.070 j70-ai-drafted-contract-human-finalized — Side-business + creator-economy

- **Journey title (humanized):** ai drafted contract human finalized.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j70-ai-drafted-contract-human-finalized/README.md`.

### §3.071 j71-ai-detected-fraud-pattern-response — Side-business + creator-economy

- **Journey title (humanized):** ai detected fraud pattern response.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j71-ai-detected-fraud-pattern-response/README.md`.

### §3.072 j72-ai-translation-cross-locale-business — Side-business + creator-economy

- **Journey title (humanized):** ai translation cross locale business.
- **Primary persona(s):** Coach Park; secondary: Coach Park, Father Lopez, Jordan Lee, Office Coordinator Phoebe Lin, Yejin Park.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j72-ai-translation-cross-locale-business/README.md`.

### §3.073 j73-third-party-developer-publishes-plugin — Side-business + creator-economy

- **Journey title (humanized):** third party developer publishes plugin.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j73-third-party-developer-publishes-plugin/README.md`.

### §3.074 j74-tenant-installs-plugin-and-it-spans-services — Side-business + creator-economy

- **Journey title (humanized):** tenant installs plugin and it spans services.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j74-tenant-installs-plugin-and-it-spans-services/README.md`.

### §3.075 j75-plugin-revoked-during-incident-response — Side-business + creator-economy

- **Journey title (humanized):** plugin revoked during incident response.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j75-plugin-revoked-during-incident-response/README.md`.

### §3.076 j76-eu-gdpr-dsar-full-cascade — Side-business + creator-economy

- **Journey title (humanized):** eu gdpr dsar full cascade.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** pack-gdpr + pack-eu-ai-act + pack-dora.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j76-eu-gdpr-dsar-full-cascade/README.md`.

### §3.077 j77-eu-ai-act-high-risk-credit-decision — Side-business + creator-economy

- **Journey title (humanized):** eu ai act high risk credit decision.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** pack-gdpr + pack-eu-ai-act + pack-dora.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j77-eu-ai-act-high-risk-credit-decision/README.md`.

### §3.078 j78-eu-nis2-breach-three-stage-cadence — Side-business + creator-economy

- **Journey title (humanized):** eu nis2 breach three stage cadence.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** pack-gdpr + pack-eu-ai-act + pack-dora.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j78-eu-nis2-breach-three-stage-cadence/README.md`.

### §3.079 j79-eu-dsa-transparency-semi-annual-report — Side-business + creator-economy

- **Journey title (humanized):** eu dsa transparency semi annual report.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** pack-gdpr + pack-eu-ai-act + pack-dora.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j79-eu-dsa-transparency-semi-annual-report/README.md`.

### §3.080 j80-kr-pipa-personal-info-cross-border-transfer — Side-business + creator-economy

- **Journey title (humanized):** kr pipa personal info cross border transfer.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** pack-kr-119-operational-mandate + pack-kr-pipa.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 14 (minor protection if applicable); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j80-kr-pipa-personal-info-cross-border-transfer/README.md`.

### §3.081 j81-kr-csap-sovereign-cell-audit-pull — Enterprise + pack-rollout

- **Journey title (humanized):** kr csap sovereign cell audit pull.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** pack-kr-119-operational-mandate + pack-kr-pipa.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j81-kr-csap-sovereign-cell-audit-pull/README.md`.

### §3.082 j82-kr-fss-financial-fraud-24h-freeze — Enterprise + pack-rollout

- **Journey title (humanized):** kr fss financial fraud 24h freeze.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** pack-kr-119-operational-mandate + pack-kr-pipa.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j82-kr-fss-financial-fraud-24h-freeze/README.md`.

### §3.083 j83-cn-pipl-data-localization-and-cac-assessment — Enterprise + pack-rollout

- **Journey title (humanized):** cn pipl data localization and cac assessment.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j83-cn-pipl-data-localization-and-cac-assessment/README.md`.

### §3.084 j84-jp-appi-elder-user-consent — Enterprise + pack-rollout

- **Journey title (humanized):** jp appi elder user consent.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j84-jp-appi-elder-user-consent/README.md`.

### §3.085 j85-hipaa-end-to-end-phi-workflow — Enterprise + pack-rollout

- **Journey title (humanized):** hipaa end to end phi workflow.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j85-hipaa-end-to-end-phi-workflow/README.md`.

### §3.086 j86-pci-dss-l1-tokenized-payment-flow — Enterprise + pack-rollout

- **Journey title (humanized):** pci dss l1 tokenized payment flow.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j86-pci-dss-l1-tokenized-payment-flow/README.md`.

### §3.087 j87-fedramp-high-il5-air-gap-deployment — Enterprise + pack-rollout

- **Journey title (humanized):** fedramp high il5 air gap deployment.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** pack-fedramp-high + pack-stateramp.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j87-fedramp-high-il5-air-gap-deployment/README.md`.

### §3.088 j88-au-irap-protected-tenant — Enterprise + pack-rollout

- **Journey title (humanized):** au irap protected tenant.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j88-au-irap-protected-tenant/README.md`.

### §3.089 j89-uk-aadc-minor-ux-adaptation — Enterprise + pack-rollout

- **Journey title (humanized):** uk aadc minor ux adaptation.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** pack-coppa + pack-kosa.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j89-uk-aadc-minor-ux-adaptation/README.md`.

### §3.090 j90-us-ccpa-cpra-do-not-sell-opt-out — Enterprise + pack-rollout

- **Journey title (humanized):** us ccpa cpra do not sell opt out.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j90-us-ccpa-cpra-do-not-sell-opt-out/README.md`.

### §3.091 j91-us-state-money-transmitter-licensing — Enterprise + pack-rollout

- **Journey title (humanized):** us state money transmitter licensing.
- **Primary persona(s):** Yejin Park (per README); secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j91-us-state-money-transmitter-licensing/README.md`.

### §3.092 j92-br-lgpd-dsar-with-us-parent — Enterprise + pack-rollout

- **Journey title (humanized):** br lgpd dsar with us parent.
- **Primary persona(s):** Tomás Silva (per README); secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j92-br-lgpd-dsar-with-us-parent/README.md`.

### §3.093 j93-in-dpdpa-rbi-financial-overlay — Enterprise + pack-rollout

- **Journey title (humanized):** in dpdpa rbi financial overlay.
- **Primary persona(s):** Aiyana Rao (per README); secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j93-in-dpdpa-rbi-financial-overlay/README.md`.

### §3.094 j94-sox-404-public-company-controls — Enterprise + pack-rollout

- **Journey title (humanized):** sox 404 public company controls.
- **Primary persona(s):** Marcus Chen (per README); secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** pack-sox-404 + pack-pcaob.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j94-sox-404-public-company-controls/README.md`.

### §3.095 j95-iso-27001-soc-2-annual-audit — Enterprise + pack-rollout

- **Journey title (humanized):** iso 27001 soc 2 annual audit.
- **Primary persona(s):** Marcus Chen (per README); secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j95-iso-27001-soc-2-annual-audit/README.md`.

### §3.096 j96-ksa-uae-mena-tenant-onboarding — Enterprise + pack-rollout

- **Journey title (humanized):** ksa uae mena tenant onboarding.
- **Primary persona(s):** Marcus Chen (per README); secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/README.md`.

### §3.097 j97-sg-pdpa-mas-singapore-tenant — Enterprise + pack-rollout

- **Journey title (humanized):** sg pdpa mas singapore tenant.
- **Primary persona(s):** Marcus Chen (per README); secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j97-sg-pdpa-mas-singapore-tenant/README.md`.

### §3.098 j98-au-privacy-apra-cps-234-tenant — Enterprise + pack-rollout

- **Journey title (humanized):** au privacy apra cps 234 tenant.
- **Primary persona(s):** Marcus Chen (per README); secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j98-au-privacy-apra-cps-234-tenant/README.md`.

### §3.099 j99-cross-jurisdiction-multi-pack-conflict-resolution — Enterprise + pack-rollout

- **Journey title (humanized):** cross jurisdiction multi pack conflict resolution.
- **Primary persona(s):** Marcus Chen (per README); secondary: _(unanchored)_.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j99-cross-jurisdiction-multi-pack-conflict-resolution/README.md`.

### §3.100 j100-pack-rollout-from-tenant-onboarding-to-first-action — Enterprise + pack-rollout

- **Journey title (humanized):** pack rollout from tenant onboarding to first action.
- **Primary persona(s):** Marcus Chen (per README); secondary: CEO Aoki Tanaka, CTO Diego Vargas, Customer Champion Akemi Sato, Marcus Chen, Product Manager Lily Chang, Yejin Park.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 27 (Cedar fragment publish soak ≥60s); row 30 (cell-tier degradation).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md`.

### §3.101 j101-multi-tier-supply-chain-formation — Supply-chain + ecosystem

- **Journey title (humanized):** multi tier supply chain formation.
- **Primary persona(s):** Cafeteria Manager Soyeon Kim; secondary: Cafeteria Manager Soyeon Kim, Marcus Chen, Procurement Manager Wei Liu, Procurement Specialist Beata Kowalski, Tomás García, Tomás García Jr..
- **µservices touched (count=9):** audit-chain, compliance, identity, mail, marketplace, ontology, payments, tenancy, workflow-engine.
- **Per-pack overlay activated:** pack-global-trade + per-jurisdiction.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥3600 (across 9 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j07-deceased-user-inheritance-handoff, j18-child-safety-mandatory-reporter, j36-b2b-workflow-engine-approval-cascade, j43-healthcare-nurse-patient-handoff.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j101-multi-tier-supply-chain-formation/README.md`.

### §3.102 j102-raw-material-purchase-with-quality-attestation — Supply-chain + ecosystem

- **Journey title (humanized):** raw material purchase with quality attestation.
- **Primary persona(s):** Tomás García; secondary: Tomás García, Tomás García Jr..
- **µservices touched (count=6):** audit-chain, connect, drive, marketplace, payments, workflow-engine.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** j101-multi-tier-supply-chain-formation, j103-just-in-time-procurement-automation, j105-dispute-cross-tenant-arbitration, j107-supply-chain-disruption-and-failover, j128-auditor-personal-side-uses-workflow-studio-for-family-taxes.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j102-raw-material-purchase-with-quality-attestation/README.md`.

### §3.103 j103-just-in-time-procurement-automation — Supply-chain + ecosystem

- **Journey title (humanized):** just in time procurement automation.
- **Primary persona(s):** Procurement Manager Wei Liu; secondary: Procurement Manager Wei Liu, Procurement Specialist Beata Kowalski, Tomás García.
- **µservices touched (count=6):** audit-chain, connect, marketplace, observability, payments, workflow-engine.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** j101-multi-tier-supply-chain-formation, j102-raw-material-purchase-with-quality-attestation, j107-supply-chain-disruption-and-failover, j120-tenant-treasury-multi-currency-fx-hedge, j128-auditor-personal-side-uses-workflow-studio-for-family-taxes.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j103-just-in-time-procurement-automation/README.md`.

### §3.104 j104-supplier-vendor-onboarding-kyb-cascade — Supply-chain + ecosystem

- **Journey title (humanized):** supplier vendor onboarding kyb cascade.
- **Primary persona(s):** Procurement Manager Wei Liu; secondary: Procurement Manager Wei Liu, Procurement Specialist Beata Kowalski, Tomás García.
- **µservices touched (count=7):** audit-chain, compliance, connect, identity, ontology, tenancy, workflow-engine.
- **Per-pack overlay activated:** pack-global-trade + per-jurisdiction.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2800 (across 7 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j43-healthcare-nurse-patient-handoff, j46-healthcare-prescription-renewal-workflow, j101-multi-tier-supply-chain-formation, j118-tenant-to-tenant-data-sharing-via-ontology-projection.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j104-supplier-vendor-onboarding-kyb-cascade/README.md`.

### §3.105 j105-dispute-cross-tenant-arbitration — Supply-chain + ecosystem

- **Journey title (humanized):** dispute cross tenant arbitration.
- **Primary persona(s):** Tomás García; secondary: Tomás García.
- **µservices touched (count=7):** audit-chain, compliance, drive, mail, messenger, payments, workflow-engine.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2800 (across 7 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j07-deceased-user-inheritance-handoff, j45-healthcare-patient-portal-records, j101-multi-tier-supply-chain-formation, j102-raw-material-purchase-with-quality-attestation.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j105-dispute-cross-tenant-arbitration/README.md`.

### §3.106 j106-multi-currency-cross-border-payment — Supply-chain + ecosystem

- **Journey title (humanized):** multi currency cross border payment.
- **Primary persona(s):** Banker (external) Hideki Watanabe; secondary: Banker (external) Hideki Watanabe.
- **µservices touched (count=4):** audit-chain, compliance, connect, payments.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥1600 (across 4 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j106-multi-currency-cross-border-payment/README.md`.

### §3.107 j107-supply-chain-disruption-and-failover — Supply-chain + ecosystem

- **Journey title (humanized):** supply chain disruption and failover.
- **Primary persona(s):** Captain Olufemi; secondary: Captain Olufemi, Tomás García.
- **µservices touched (count=6):** audit-chain, connect, mail, marketplace, observability, workflow-engine.
- **Per-pack overlay activated:** pack-global-trade + per-jurisdiction.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j101-multi-tier-supply-chain-formation, j102-raw-material-purchase-with-quality-attestation, j103-just-in-time-procurement-automation, j127-dual-tenant-identity-employee-resigns-and-keeps-personal.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j107-supply-chain-disruption-and-failover/README.md`.

### §3.108 j108-supplier-rating-and-marketplace-discovery — Supply-chain + ecosystem

- **Journey title (humanized):** supplier rating and marketplace discovery.
- **Primary persona(s):** Captain Olufemi; secondary: Captain Olufemi, Tomás García, Tomás García Jr..
- **µservices touched (count=4):** community, identity, intelligence, marketplace.
- **Per-pack overlay activated:** pack-global-trade + per-jurisdiction.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥1600 (across 4 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j108-supplier-rating-and-marketplace-discovery/README.md`.

### §3.109 j109-construction-co-hires-freelance-specialist — Supply-chain + ecosystem

- **Journey title (humanized):** construction co hires freelance specialist.
- **Primary persona(s):** Ahmad Hassan; secondary: Ahmad Hassan, Apprentice Jakob Bauer, Customer Success Manager Sofia Rezende, Devon Williams.
- **µservices touched (count=6):** community, identity, observability, payments, workflow-engine, workplace-integration.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** j37-b2b-clocking-and-attendance, j110-traveling-nurse-multi-employer-roster, j111-staffing-agency-as-tenant-facilitator, j112-tenant-to-tenant-rfq-and-bid, j113-cross-tenant-internship-from-handshake.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j109-construction-co-hires-freelance-specialist/README.md`.

### §3.110 j110-traveling-nurse-multi-employer-roster — Supply-chain + ecosystem

- **Journey title (humanized):** traveling nurse multi employer roster.
- **Primary persona(s):** Ahmad Hassan; secondary: Ahmad Hassan, Maria Santos, Medical Resident Dr. Sun-Mi Kim, Receptionist Daria Volkova, Training Specialist Mehmet Yilmaz.
- **µservices touched (count=5):** community, identity, payments, tenancy, workplace-integration.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2000 (across 5 µservices).
- **Cross-references to related journeys:** j109-construction-co-hires-freelance-specialist, j111-staffing-agency-as-tenant-facilitator, j112-tenant-to-tenant-rfq-and-bid, j113-cross-tenant-internship-from-handshake, j114-employee-secondment-cross-tenant.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j110-traveling-nurse-multi-employer-roster/README.md`.

### §3.111 j111-staffing-agency-as-tenant-facilitator — Supply-chain + ecosystem

- **Journey title (humanized):** staffing agency as tenant facilitator.
- **Primary persona(s):** Channel Partner Tomas Pieter; secondary: Channel Partner Tomas Pieter, Cleaning Supervisor Tomáš Horák.
- **µservices touched (count=5):** community, identity, payments, tenancy, workflow-engine.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2000 (across 5 µservices).
- **Cross-references to related journeys:** j50-sidebusiness-employee-hires-first-helper, j101-multi-tier-supply-chain-formation, j109-construction-co-hires-freelance-specialist, j110-traveling-nurse-multi-employer-roster, j112-tenant-to-tenant-rfq-and-bid.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j111-staffing-agency-as-tenant-facilitator/README.md`.

### §3.112 j112-tenant-to-tenant-rfq-and-bid — Supply-chain + ecosystem

- **Journey title (humanized):** tenant to tenant rfq and bid.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=6):** community, identity, marketplace, payments, workflow-engine, workplace-integration.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** j101-multi-tier-supply-chain-formation, j109-construction-co-hires-freelance-specialist, j110-traveling-nurse-multi-employer-roster, j111-staffing-agency-as-tenant-facilitator, j113-cross-tenant-internship-from-handshake.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j112-tenant-to-tenant-rfq-and-bid/README.md`.

### §3.113 j113-cross-tenant-internship-from-handshake — Supply-chain + ecosystem

- **Journey title (humanized):** cross tenant internship from handshake.
- **Primary persona(s):** Co-op Student Liam Murphy; secondary: Co-op Student Liam Murphy, Intern Manager Felicia Adamou, Returning Intern Jia Han, Summer Intern Priscilla Sharma.
- **µservices touched (count=6):** calendar, community, identity, messenger, payments, workplace-integration.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** j109-construction-co-hires-freelance-specialist, j110-traveling-nurse-multi-employer-roster, j112-tenant-to-tenant-rfq-and-bid, j127-dual-tenant-identity-employee-resigns-and-keeps-personal, j129-court-warrant-pierces-personal-tenant-with-judicial-oversight.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j113-cross-tenant-internship-from-handshake/README.md`.

### §3.114 j114-employee-secondment-cross-tenant — Supply-chain + ecosystem

- **Journey title (humanized):** employee secondment cross tenant.
- **Primary persona(s):** _(primary unstated in README — see §9 coverage gap)_; secondary: _(unanchored)_.
- **µservices touched (count=5):** identity, payments, tenancy, workflow-engine, workplace-integration.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2000 (across 5 µservices).
- **Cross-references to related journeys:** j50-sidebusiness-employee-hires-first-helper, j101-multi-tier-supply-chain-formation, j109-construction-co-hires-freelance-specialist, j110-traveling-nurse-multi-employer-roster, j111-staffing-agency-as-tenant-facilitator.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j114-employee-secondment-cross-tenant/README.md`.

### §3.115 j115-saas-vendor-sells-api-to-multiple-tenant-customers — Supply-chain + ecosystem

- **Journey title (humanized):** saas vendor sells api to multiple tenant customers.
- **Primary persona(s):** Channel Partner Tomas Pieter; secondary: Channel Partner Tomas Pieter, Customer Success Manager Sofia Rezende, Sales AE Maya Lindqvist, Sales Manager Anthony Costa.
- **µservices touched (count=6):** finops-portal, identity, observability, payments, plugin-app-store, workflow-engine.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** j109-construction-co-hires-freelance-specialist, j117-api-customer-tenant-incident-response, j120-tenant-treasury-multi-currency-fx-hedge, j121-business-loan-application-from-bank-tenant, j149-gig-economy-multi-platform-worker.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j115-saas-vendor-sells-api-to-multiple-tenant-customers/README.md`.

### §3.116 j116-plugin-marketplace-developer-publishes-and-monetizes — Supply-chain + ecosystem

- **Journey title (humanized):** plugin marketplace developer publishes and monetizes.
- **Primary persona(s):** Nadia Park, third-party developer and micro-SaaS founder (per README); secondary: _(unanchored)_.
- **µservices touched (count=5):** community, foundry, payments, plugin-app-store, tenancy.
- **Per-pack overlay activated:** pack-marketplace-baseline + pack-creator-monetization.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2000 (across 5 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j116-plugin-marketplace-developer-publishes-and-monetizes/README.md`.

### §3.117 j117-api-customer-tenant-incident-response — Supply-chain + ecosystem

- **Journey title (humanized):** api customer tenant incident response.
- **Primary persona(s):** Mira Cho, AIScribe tenant SRE lead (per README); secondary: CS-IC Lin Chen, Customer Champion Akemi Sato, Customer Success Manager Sofia Rezende, DevOps Engineer Olukayode Adejumo, DevOps Manager Pavel Korsak, IT Manager Jamie O'Connor, Support Rep Nadia Hassani.
- **µservices touched (count=6):** finops-portal, mail, messenger, observability, payments, workflow-engine.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j105-dispute-cross-tenant-arbitration, j115-saas-vendor-sells-api-to-multiple-tenant-customers, j120-tenant-treasury-multi-currency-fx-hedge, j122-vendor-payment-batch-with-tax-withholding.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j117-api-customer-tenant-incident-response/README.md`.

### §3.118 j118-tenant-to-tenant-data-sharing-via-ontology-projection — Supply-chain + ecosystem

- **Journey title (humanized):** tenant to tenant data sharing via ontology projection.
- **Primary persona(s):** Marcus Chen, KrampusCorp operating sponsor (per README); secondary: Board Secretary Florence Akinsanya, Marcus Chen, Outside Counsel Wei-Yi Chen, Paralegal Tomáš Novák.
- **µservices touched (count=5):** audit-chain, compliance, identity, ontology, tenancy.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2000 (across 5 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j43-healthcare-nurse-patient-handoff, j101-multi-tier-supply-chain-formation, j104-supplier-vendor-onboarding-kyb-cascade, j125-marketplace-acquires-supplier-tenant-merger.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j118-tenant-to-tenant-data-sharing-via-ontology-projection/README.md`.

### §3.119 j119-invoice-financing-marketplace — Supply-chain + ecosystem

- **Journey title (humanized):** invoice financing marketplace.
- **Primary persona(s):** Marcus Chen, KrampusCorp treasury sponsor (per README); secondary: Board director Patrick O'Reilly, Business Analyst Aditya Verma, CFO Helena Brandt, IR Manager Lev Kahn, IR Specialist (unnamed), Investment Banker Yuna Ahn, Investor/LP Aanya Kapoor, Marcus Chen, Venture Partner Lucas Müller, Wealth Manager Aamir Khan.
- **µservices touched (count=6):** audit-chain, community, compliance, finops-portal, payments, plugin-app-store.
- **Per-pack overlay activated:** pack-kr-119-operational-mandate + pack-kr-pipa.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** j129-court-warrant-pierces-personal-tenant-with-judicial-oversight, j148-supply-chain-circular-economy-electronics-recycling, j150-creator-economy-shorts-creator-monetization-stack.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j119-invoice-financing-marketplace/README.md`.

### §3.120 j120-tenant-treasury-multi-currency-fx-hedge — Supply-chain + ecosystem

- **Journey title (humanized):** tenant treasury multi currency fx hedge.
- **Primary persona(s):** Elena Rossi, group treasurer for Marcus company (per README); secondary: Bank Risk Manager Anders Pedersen, CFO Helena Brandt, Marcus Chen, Trader Mei Lin, Treasury Ops Sven Eriksson.
- **µservices touched (count=5):** connect, finops-portal, observability, payments, workflow-engine.
- **Per-pack overlay activated:** pack-finra + pack-mas + pack-pci-dss-l1.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2000 (across 5 µservices).
- **Cross-references to related journeys:** j103-just-in-time-procurement-automation, j115-saas-vendor-sells-api-to-multiple-tenant-customers, j117-api-customer-tenant-incident-response, j121-business-loan-application-from-bank-tenant, j122-vendor-payment-batch-with-tax-withholding.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j120-tenant-treasury-multi-currency-fx-hedge/README.md`.

### §3.121 j121-business-loan-application-from-bank-tenant — Supply-chain + ecosystem

- **Journey title (humanized):** business loan application from bank tenant.
- **Primary persona(s):** Marcus Chen, borrower sponsor (per README); secondary: Bank Compliance Officer Rishi Bhattacharya, Bank Ops Officer Olamide Adebanjo, Bank Risk Manager Anders Pedersen, Banker (external) Hideki Watanabe, Commercial Banker Frederik Hartmann, Credit Analyst Hina Mori, IR Manager Lev Kahn, Investment Banker Yuna Ahn, Marcus Chen, Retail Banker Sebastián Vega.
- **µservices touched (count=7):** connect, finops-portal, identity, payments, tenancy, workflow-engine, workplace-integration.
- **Per-pack overlay activated:** pack-finra + pack-mas + pack-pci-dss-l1.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2800 (across 7 µservices).
- **Cross-references to related journeys:** j37-b2b-clocking-and-attendance, j50-sidebusiness-employee-hires-first-helper, j101-multi-tier-supply-chain-formation, j104-supplier-vendor-onboarding-kyb-cascade, j109-construction-co-hires-freelance-specialist.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j121-business-loan-application-from-bank-tenant/README.md`.

### §3.122 j122-vendor-payment-batch-with-tax-withholding — Supply-chain + ecosystem

- **Journey title (humanized):** vendor payment batch with tax withholding.
- **Primary persona(s):** Jae Kim, KrampusCorp AP manager (per README); secondary: Accountant Ravi Iyer, Bank Compliance Officer Rishi Bhattacharya, Bank Ops Officer Olamide Adebanjo, CFO Helena Brandt, Compliance Officer Tunde Bello, Finance Director Mei-Ling Wu, Financial Analyst Wendy Lee, Tax Analyst Ji-Sung Park, Treasury Ops Sven Eriksson.
- **µservices touched (count=6):** compliance, connect, finops-portal, mail, payments, workflow-engine.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** j46-healthcare-prescription-renewal-workflow, j47-healthcare-billing-and-insurance, j48-sidebusiness-stripe-tax-and-invoicing, j101-multi-tier-supply-chain-formation, j105-dispute-cross-tenant-arbitration.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j122-vendor-payment-batch-with-tax-withholding/README.md`.

### §3.123 j123-multi-tenant-coordinated-product-launch — Supply-chain + ecosystem

- **Journey title (humanized):** multi tenant coordinated product launch.
- **Primary persona(s):** Marcus Chen, launch sponsor (per README); secondary: CEO Aoki Tanaka, CMO Felix Ng, COO Akira Watanabe, CTO Diego Vargas, Communications Specialist Charlotte Dubois, Corporate Relations Director Soo-Yeon Han, Executive Assistant Olivia Reyes, Internal Comms Lead Ji-Ho Yoon, Marcus Chen, Marketing Manager Olu Adeyemi, PR Firm Beatriz Fernandez, PR Manager Helena Sato, Product Manager Lily Chang, Project Manager Soo-Jin Park.
- **µservices touched (count=7):** drive, identity, intelligence, messenger, payments, tenancy, workflow-engine.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2800 (across 7 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j08-elder-financial-abuse-detection, j14-delegated-llm-agent-acting-for-yejin, j50-sidebusiness-employee-hires-first-helper, j101-multi-tier-supply-chain-formation.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j123-multi-tenant-coordinated-product-launch/README.md`.

### §3.124 j124-supply-chain-disruption-emergency-coordination — Supply-chain + ecosystem

- **Journey title (humanized):** supply chain disruption emergency coordination.
- **Primary persona(s):** Sora Lee, KrampusCorp emergency coordinator (per README); secondary: COO Akira Watanabe, Marcus Chen, Public Affairs Director Carlos Mendez.
- **µservices touched (count=5):** audit-chain, identity, mail, messenger, workflow-engine.
- **Per-pack overlay activated:** pack-global-trade + per-jurisdiction.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2000 (across 5 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j14-delegated-llm-agent-acting-for-yejin, j18-child-safety-mandatory-reporter, j101-multi-tier-supply-chain-formation, j105-dispute-cross-tenant-arbitration.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j124-supply-chain-disruption-emergency-coordination/README.md`.

### §3.125 j125-marketplace-acquires-supplier-tenant-merger — Supply-chain + ecosystem

- **Journey title (humanized):** marketplace acquires supplier tenant merger.
- **Primary persona(s):** Marcus Chen, acquiring-company sponsor (per README); secondary: Board director Patrick O'Reilly, CEO Aoki Tanaka, CSO Mira Goldberg, Consultant Adekunle Adebayo, Corp Dev Senior Analyst Saanvi Mehta, Investment Banker Yuna Ahn, Marcus Chen, Outside Counsel Wei-Yi Chen, Strategic Advisor Rita Almeida, Venture Partner Lucas Müller.
- **µservices touched (count=8):** audit-chain, compliance, drive, finops-portal, identity, ontology, tenancy, workflow-engine.
- **Per-pack overlay activated:** pack-global-trade + per-jurisdiction.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 18 (provider-credential BYOK, ADR-0255 §D-4).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥3200 (across 8 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j43-healthcare-nurse-patient-handoff, j45-healthcare-patient-portal-records, j101-multi-tier-supply-chain-formation, j104-supplier-vendor-onboarding-kyb-cascade.
- **Migration-path applicable:** tenant-merger migration (ADR-0313 conglomerate-tenant).
- **README citation:** `docs/user-journeys/j125-marketplace-acquires-supplier-tenant-merger/README.md`.

### §3.126 j126-government-auditor-3pao-conducts-fedramp-audit — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** government auditor 3pao conducts fedramp audit.
- **Primary persona(s):** Auditor IT-Specialist Jakub Nowak; secondary: Auditor IT-Specialist Jakub Nowak, Diana Reyes, External Auditor Dimitri Volkov, External Auditor Hyo-Jin Lee, External Regulator Inspector Sergei Petrov.
- **µservices touched (count=11):** api-gateway, audit-chain, comms-email, compliance, identity, messenger, observability, ops-dashboard-control-center, policy-engine, tenancy, workflow-engine.
- **Per-pack overlay activated:** pack-fedramp-high + pack-stateramp.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥4400 (across 11 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j14-delegated-llm-agent-acting-for-yejin, j101-multi-tier-supply-chain-formation, j104-supplier-vendor-onboarding-kyb-cascade, j105-dispute-cross-tenant-arbitration.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j126-government-auditor-3pao-conducts-fedramp-audit/README.md`.

### §3.127 j127-dual-tenant-identity-employee-resigns-and-keeps-personal — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** dual tenant identity employee resigns and keeps personal.
- **Primary persona(s):** Diana Reyes; secondary: Diana Reyes.
- **µservices touched (count=13):** audit-chain, calendar, comms-email, drive, identity, mail, meet, messenger, observability, policy-engine, tenancy, workflow-engine, workplace-integration.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥5200 (across 13 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j04-dv-survivor-shelter-mode, j07-deceased-user-inheritance-handoff, j14-delegated-llm-agent-acting-for-yejin, j18-child-safety-mandatory-reporter.
- **Migration-path applicable:** dual-tenant separation (ADR-0311).
- **README citation:** `docs/user-journeys/j127-dual-tenant-identity-employee-resigns-and-keeps-personal/README.md`.

### §3.128 j128-auditor-personal-side-uses-workflow-studio-for-family-taxes — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** auditor personal side uses workflow studio for family taxes.
- **Primary persona(s):** Diana Reyes; secondary: Diana Reyes, Tax Analyst Ji-Sung Park.
- **µservices touched (count=11):** audit-chain, connect, drive, identity, intelligence, mail, marketplace, notes, payments, workflow-engine, workflow-studio.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥4400 (across 11 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j07-deceased-user-inheritance-handoff, j14-delegated-llm-agent-acting-for-yejin, j18-child-safety-mandatory-reporter, j36-b2b-workflow-engine-approval-cascade.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j128-auditor-personal-side-uses-workflow-studio-for-family-taxes/README.md`.

### §3.129 j129-court-warrant-pierces-personal-tenant-with-judicial-oversight — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** court warrant pierces personal tenant with judicial oversight.
- **Primary persona(s):** Anya Mironova; secondary: Anya Mironova, CCO Naveen Iyer, CISO Yuki Park, Diana Reyes, External Regulator Inspector Sergei Petrov, Legal Counsel Anika Mehta, Outside Counsel Wei-Yi Chen, Public Affairs Director Carlos Mendez.
- **µservices touched (count=13):** audit-chain, comms-email, community, compliance, drive, governance, identity, marketplace, messenger, payments, policy-engine, tenancy, workflow-engine.
- **Per-pack overlay activated:** pack-judicial-warrant-pierce.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥5200 (across 13 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j06-press-source-securedrop-class, j07-deceased-user-inheritance-handoff, j08-elder-financial-abuse-detection, j14-delegated-llm-agent-acting-for-yejin.
- **Migration-path applicable:** judicial-piercing (ADR-0312).
- **README citation:** `docs/user-journeys/j129-court-warrant-pierces-personal-tenant-with-judicial-oversight/README.md`.

### §3.130 j130-auditor-receives-bribery-attempt-via-personal-messenger — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** auditor receives bribery attempt via personal messenger.
- **Primary persona(s):** Diana Reyes; secondary: Diana Reyes.
- **µservices touched (count=8):** audit-chain, comms-email, community, compliance, identity, messenger, policy-engine, workflow-engine.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥3200 (across 8 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j14-delegated-llm-agent-acting-for-yejin, j18-child-safety-mandatory-reporter, j101-multi-tier-supply-chain-formation, j104-supplier-vendor-onboarding-kyb-cascade.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j130-auditor-receives-bribery-attempt-via-personal-messenger/README.md`.

### §3.131 j131-cross-jurisdiction-audit-eu-vs-kr-discrepancy — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** cross jurisdiction audit eu vs kr discrepancy.
- **Primary persona(s):** Bank Compliance Officer Rishi Bhattacharya; secondary: Bank Compliance Officer Rishi Bhattacharya, Compliance Analyst Yui Hayashi, Compliance Officer Tunde Bello, Diana Reyes, External Auditor Dimitri Volkov, External Auditor Hyo-Jin Lee, External Regulator Inspector Sergei Petrov.
- **µservices touched (count=7):** audit-chain, compliance, identity, observability, policy-engine, tenancy, workflow-engine.
- **Per-pack overlay activated:** pack-kr-119-operational-mandate + pack-kr-pipa.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2800 (across 7 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j101-multi-tier-supply-chain-formation, j104-supplier-vendor-onboarding-kyb-cascade, j118-tenant-to-tenant-data-sharing-via-ontology-projection, j125-marketplace-acquires-supplier-tenant-merger.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j131-cross-jurisdiction-audit-eu-vs-kr-discrepancy/README.md`.

### §3.132 j132-hr-mass-hiring-event-100-roles — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** hr mass hiring event 100 roles.
- **Primary persona(s):** Priya Krishnan (per README); secondary: CHRO Linda Foster, D&I Director Maya Okoroafor, HR Specialist Aoife Murphy, HRBP Jamal Carter, Priya Krishnan, Recruiter Marcus IV, Recruiting Manager Hina Suzuki.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j132-hr-mass-hiring-event-100-roles/README.md`.

### §3.133 j133-hr-conducts-layoff-with-dignity-and-compliance — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** hr conducts layoff with dignity and compliance.
- **Primary persona(s):** Priya Krishnan (per README); secondary: CHRO Linda Foster, HR Specialist Aoife Murphy, HRBP Jamal Carter, Priya Krishnan.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j133-hr-conducts-layoff-with-dignity-and-compliance/README.md`.

### §3.134 j134-hr-cross-tenant-recruitment-via-staffing-agency — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** hr cross tenant recruitment via staffing agency.
- **Primary persona(s):** Priya Krishnan (per README); secondary: HR Specialist Aoife Murphy, HRBP Jamal Carter, Priya Krishnan, Recruiter Marcus IV, Recruiting Manager Hina Suzuki.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j134-hr-cross-tenant-recruitment-via-staffing-agency/README.md`.

### §3.135 j135-hr-handles-harassment-complaint-with-dual-tenant-boundary — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** hr handles harassment complaint with dual tenant boundary.
- **Primary persona(s):** Priya Krishnan (per README); secondary: Compliance Officer Tunde Bello, D&I Director Maya Okoroafor, HR Specialist Aoife Murphy, Legal Counsel Anika Mehta, Ombudsperson Felix Tan, Priya Krishnan.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j135-hr-handles-harassment-complaint-with-dual-tenant-boundary/README.md`.

### §3.136 j136-hr-administers-benefits-open-enrollment — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** hr administers benefits open enrollment.
- **Primary persona(s):** Priya Krishnan (per README); secondary: Benefits Specialist Aoife Murphy, CHRO Linda Foster, HR Specialist Aoife Murphy, Leave Specialist Margarethe Reinhart, Office Manager Priya Ramanathan, Priya Krishnan, Retirement Plan Admin Bryce Williams, Total Rewards Manager Nilufer Demir, Wellness Program Manager Akira Sato.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j136-hr-administers-benefits-open-enrollment/README.md`.

### §3.137 j137-corporate-internal-audit-sox-controls-test — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** corporate internal audit sox controls test.
- **Primary persona(s):** Accountant Ravi Iyer; secondary: Accountant Ravi Iyer, Board Secretary Florence Akinsanya, Board director Patrick O'Reilly, CCO Naveen Iyer, CFO Helena Brandt, Compliance Officer Tunde Bello, Finance Director Mei-Ling Wu, Financial Analyst Wendy Lee, Sam Okafor.
- **µservices touched (count=8):** audit-chain, compliance, identity, mail, messenger, ops-dashboard-control-center, payments, workflow-engine.
- **Per-pack overlay activated:** pack-sox-404 + pack-pcaob.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥3200 (across 8 µservices).
- **Cross-references to related journeys:** j01-emergency-911-dispatch, j07-deceased-user-inheritance-handoff, j08-elder-financial-abuse-detection, j14-delegated-llm-agent-acting-for-yejin, j18-child-safety-mandatory-reporter.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j137-corporate-internal-audit-sox-controls-test/README.md`.

### §3.138 j138-corporate-audit-fraud-investigation-via-pattern-detection — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** corporate audit fraud investigation via pattern detection.
- **Primary persona(s):** CISO Yuki Park; secondary: CISO Yuki Park, Compliance Analyst Yui Hayashi, Compliance Officer Tunde Bello, Data Analyst Felipe Andrade, Data Scientist Yu Chen, Sam Okafor, Security Analyst Anna Petrova.
- **µservices touched (count=6):** audit-chain, community, detection, mail, payments, workflow-engine.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2400 (across 6 µservices).
- **Cross-references to related journeys:** j18-child-safety-mandatory-reporter, j101-multi-tier-supply-chain-formation, j105-dispute-cross-tenant-arbitration, j128-auditor-personal-side-uses-workflow-studio-for-family-taxes, j129-court-warrant-pierces-personal-tenant-with-judicial-oversight.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j138-corporate-audit-fraud-investigation-via-pattern-detection/README.md`.

### §3.139 j139-internal-audit-policy-violation-cedar-permit-misuse — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** internal audit policy violation cedar permit misuse.
- **Primary persona(s):** CCO Naveen Iyer; secondary: CCO Naveen Iyer, CISO Yuki Park, Compliance Analyst Yui Hayashi, Compliance Officer Tunde Bello, Sam Okafor.
- **µservices touched (count=5):** audit-chain, governance, identity, ops-dashboard-control-center, workflow-engine.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2000 (across 5 µservices).
- **Cross-references to related journeys:** j19-tenant-break-glass-locked-out-tenant-admin, j126-government-auditor-3pao-conducts-fedramp-audit, j129-court-warrant-pierces-personal-tenant-with-judicial-oversight, j137-corporate-internal-audit-sox-controls-test.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j139-internal-audit-policy-violation-cedar-permit-misuse/README.md`.

### §3.140 j140-internal-audit-data-loss-prevention-egress-trip — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** internal audit data loss prevention egress trip.
- **Primary persona(s):** Auditor IT-Specialist Jakub Nowak; secondary: Auditor IT-Specialist Jakub Nowak, CISO Yuki Park, Compliance Officer Tunde Bello, Sam Okafor, Security Analyst Anna Petrova.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j140-internal-audit-data-loss-prevention-egress-trip/README.md`.

### §3.141 j141-internal-audit-respects-employee-personal-tenant-boundary — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** internal audit respects employee personal tenant boundary.
- **Primary persona(s):** Compliance Officer Tunde Bello; secondary: Compliance Officer Tunde Bello, Legal Counsel Anika Mehta, Legal Operations Stephen Park, Outside Counsel Wei-Yi Chen, Paralegal Tomáš Novák, Sam Okafor.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j141-internal-audit-respects-employee-personal-tenant-boundary/README.md`.

### §3.142 j142-layoff-day-zero-from-employees-side — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** layoff day zero from employees side.
- **Primary persona(s):** Chris Volkov; secondary: Chris Volkov.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j142-layoff-day-zero-from-employees-side/README.md`.

### §3.143 j143-laid-off-imports-work-portfolio-into-personal-tenant — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** laid off imports work portfolio into personal tenant.
- **Primary persona(s):** Chris Volkov; secondary: Chris Volkov.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** tenant-merger migration (ADR-0313 conglomerate-tenant).
- **README citation:** `docs/user-journeys/j143-laid-off-imports-work-portfolio-into-personal-tenant/README.md`.

### §3.144 j144-laid-off-builds-job-search-pipeline-in-workflow-studio — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** laid off builds job search pipeline in workflow studio.
- **Primary persona(s):** Chris Volkov; secondary: Chris Volkov.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** dual-tenant separation (ADR-0311).
- **README citation:** `docs/user-journeys/j144-laid-off-builds-job-search-pipeline-in-workflow-studio/README.md`.

### §3.145 j145-laid-off-applies-via-community-handshake-linkedin-mode — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** laid off applies via community handshake linkedin mode.
- **Primary persona(s):** Chris Volkov; secondary: Chris Volkov, Recruiter Marcus IV, SDR Kofi Asante.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** dual-tenant separation (ADR-0311).
- **README citation:** `docs/user-journeys/j145-laid-off-applies-via-community-handshake-linkedin-mode/README.md`.

### §3.146 j146-laid-off-uses-marketplace-as-temporary-income — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** laid off uses marketplace as temporary income.
- **Primary persona(s):** Chris Volkov; secondary: Chris Volkov.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** pack-marketplace-baseline + pack-creator-monetization.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** dual-tenant separation (ADR-0311).
- **README citation:** `docs/user-journeys/j146-laid-off-uses-marketplace-as-temporary-income/README.md`.

### §3.147 j147-laid-off-cohort-mutual-aid-community-channel — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** laid off cohort mutual aid community channel.
- **Primary persona(s):** Chris Volkov; secondary: Chris Volkov.
- **µservices touched (count=0):** _(none parsed from README — see §9)_.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥0 (across 0 µservices).
- **Cross-references to related journeys:** _(leaf — terminal node in journey graph)_.
- **Migration-path applicable:** dual-tenant separation (ADR-0311).
- **README citation:** `docs/user-journeys/j147-laid-off-cohort-mutual-aid-community-channel/README.md`.

### §3.148 j148-supply-chain-circular-economy-electronics-recycling — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** supply chain circular economy electronics recycling.
- **Primary persona(s):** Yejin Han, consumer returning an old laptop (per README); secondary: Sustainability Officer Aiko Brown, Tomás García Jr., Yejin Park.
- **µservices touched (count=7):** audit-chain, community, connect, ontology, payments, plugin-app-store, workflow-engine.
- **Per-pack overlay activated:** pack-global-trade + per-jurisdiction.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2800 (across 7 µservices).
- **Cross-references to related journeys:** j101-multi-tier-supply-chain-formation, j102-raw-material-purchase-with-quality-attestation, j103-just-in-time-procurement-automation, j104-supplier-vendor-onboarding-kyb-cascade, j119-invoice-financing-marketplace.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j148-supply-chain-circular-economy-electronics-recycling/README.md`.

### §3.149 j149-gig-economy-multi-platform-worker — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** gig economy multi platform worker.
- **Primary persona(s):** Aiyana Brooks, multi-platform gig worker (per README); secondary: Aiyana Singh, Sarah Kim.
- **µservices touched (count=7):** community, connect, finops-portal, identity, payments, tenancy, workflow-engine.
- **Per-pack overlay activated:** PACK-AGNOSTIC.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥2800 (across 7 µservices).
- **Cross-references to related journeys:** j50-sidebusiness-employee-hires-first-helper, j101-multi-tier-supply-chain-formation, j104-supplier-vendor-onboarding-kyb-cascade, j109-construction-co-hires-freelance-specialist, j110-traveling-nurse-multi-employer-roster.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j149-gig-economy-multi-platform-worker/README.md`.

### §3.150 j150-creator-economy-shorts-creator-monetization-stack — Cross-tenant economy + dual-tenant boundary

- **Journey title (humanized):** creator economy shorts creator monetization stack.
- **Primary persona(s):** Mina Han, Yejin daughter, 16-year-old Shorts creator (per README); secondary: Data Analyst Felipe Andrade, Data Scientist Yu Chen, Marketing Specialist Riya Sharma.
- **µservices touched (count=8):** community, finops-portal, identity, intelligence, ontology, payments, plugin-app-store, shorts.
- **Per-pack overlay activated:** pack-marketplace-baseline + pack-creator-monetization.
- **Critical-path row(s) per documentation-rigor.md §3.2.5:** row 8 (cross-tenant exfil); row 24 (audit-chain seal); row 27 (Cedar fragment publish soak).
- **Artifact line counts:** story.md ≥800; ux-flow.md ≥400; handshake.md ≥600; integration-test-plan.md ≥400; README.md ≥300; per-µservice IPs ≥3200 (across 8 µservices).
- **Cross-references to related journeys:** j115-saas-vendor-sells-api-to-multiple-tenant-customers, j119-invoice-financing-marketplace, j148-supply-chain-circular-economy-electronics-recycling, j149-gig-economy-multi-platform-worker.
- **Migration-path applicable:** n/a (greenfield surface).
- **README citation:** `docs/user-journeys/j150-creator-economy-shorts-creator-monetization-stack/README.md`.

---

## §4 Per-Microservice Cross-Coverage Table

Each µservice row enumerates: tier (substrate/product), personas who depend on it, journeys it appears in, capability tiers it exposes (per ADR-0316), bounded contexts it owns, pack overlays applicable, hyperscaler benchmark equivalents (per enterprise-software-coverage-matrix-2026-05-21.md §14), per-µservice IP-journey-* slice count, and critical-path responsibilities.

Cross-reference: `microservices/<svc>/PRD.md` for each µservice; enterprise-software-coverage-matrix-2026-05-21.md §14 for centers-of-gravity ranking.

### §4.01 identity — substrate (Tier-1; cell-tier T0)

- **Owns (bounded contexts):** principal-context-overlay + passkey + WebAuthn + account-recovery.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-1; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Okta + Auth0 + AWS IAM.
- **Center-of-gravity rank (per coverage-matrix §14):** 4.
- **Personas who depend on it (count=91):** Accountant Ravi Iyer, Ahmad Hassan, Aiyana Singh, Anya Mironova, Apprentice Jakob Bauer, Auditor IT-Specialist Jakub Nowak, Bank Compliance Officer Rishi Bhattacharya, Bank Ops Officer Olamide Adebanjo, Bank Risk Manager Anders Pedersen, Banker (external) Hideki Watanabe, Board Secretary Florence Akinsanya, Board director Patrick O'Reilly, CCO Naveen Iyer, CEO Aoki Tanaka, CFO Helena Brandt, CISO Yuki Park, CMO Felix Ng, COO Akira Watanabe, CSO Mira Goldberg, CTO Diego Vargas, Cafeteria Manager Soyeon Kim, Captain Chen, Captain Olufemi, Carlos Martinez, Channel Partner Tomas Pieter, Cleaning Supervisor Tomáš Horák, Co-op Student Liam Murphy, Commercial Banker Frederik Hartmann, Communications Specialist Charlotte Dubois, Compliance Analyst Yui Hayashi, Compliance Officer Tunde Bello, Consultant Adekunle Adebayo, Corp Dev Senior Analyst Saanvi Mehta, Corporate Relations Director Soo-Yeon Han, Credit Analyst Hina Mori, Customer Success Manager Sofia Rezende, Data Analyst Felipe Andrade, Data Scientist Yu Chen, DevOps Engineer Olukayode Adejumo, DevOps Manager Pavel Korsak, Devon Williams, Diana Reyes, Engineering Manager Aisha Ali, Executive Assistant Olivia Reyes, External Auditor Dimitri Volkov, External Auditor Hyo-Jin Lee, External Regulator Inspector Sergei Petrov, Finance Director Mei-Ling Wu, Financial Analyst Wendy Lee, Hiroshi Tanaka, IR Manager Lev Kahn, Intern Manager Felicia Adamou, Internal Comms Lead Ji-Ho Yoon, Investment Banker Yuna Ahn, Jordan Lee, Legal Counsel Anika Mehta, Marcus Chen, Maria Santos, Marketing Manager Olu Adeyemi, Marketing Specialist Riya Sharma, Medical Resident Dr. Sun-Mi Kim, Ms. Patel, Officer Rodriguez, Outside Counsel Wei-Yi Chen, PR Firm Beatriz Fernandez, PR Manager Helena Sato, Paralegal Tomáš Novák, Procurement Manager Wei Liu, Procurement Specialist Beata Kowalski, Product Designer Akihiro Sato, Product Manager Lily Chang, Project Manager Soo-Jin Park, Public Affairs Director Carlos Mendez, Receptionist Daria Volkova, Retail Banker Sebastián Vega, Returning Intern Jia Han, SWE Hugo Tanaka, Sales AE Maya Lindqvist, Sales Manager Anthony Costa, Sam Okafor, Sarah Kim, Security Analyst Anna Petrova, Strategic Advisor Rita Almeida, Summer Intern Priscilla Sharma, Tax Analyst Ji-Sung Park, Tomás García, Tomás García Jr., Training Specialist Mehmet Yilmaz, Venture Partner Lucas Müller, Wealth Manager Aamir Khan, Yejin Park.
- **Journeys it appears in (count=46):** j01-emergency-911-dispatch, j04-dv-survivor-shelter-mode, j05-whistleblower-anonymous-ethics-report, j07-deceased-user-inheritance-handoff, j08-elder-financial-abuse-detection, j09-account-recovery-phishing-resistant, j10-account-takeover-SIM-swap-detected, j14-delegated-llm-agent-acting-for-yejin, j16-disability-accommodation-voice-only-signup, j17-activist-dissident-high-risk-mode, j18-child-safety-mandatory-reporter, j19-tenant-break-glass-locked-out-tenant-admin, j36-b2b-workflow-engine-approval-cascade, j37-b2b-clocking-and-attendance, j38-b2b-e-signing-contract, j41-b2b-developer-builds-on-platform, j42-b2b-finops-portal-spend-attribution, j43-healthcare-nurse-patient-handoff, j45-healthcare-patient-portal-records, j46-healthcare-prescription-renewal-workflow, j50-sidebusiness-employee-hires-first-helper, j101-multi-tier-supply-chain-formation, j104-supplier-vendor-onboarding-kyb-cascade, j108-supplier-rating-and-marketplace-discovery, j109-construction-co-hires-freelance-specialist, j110-traveling-nurse-multi-employer-roster, j111-staffing-agency-as-tenant-facilitator, j112-tenant-to-tenant-rfq-and-bid, j113-cross-tenant-internship-from-handshake, j114-employee-secondment-cross-tenant, j115-saas-vendor-sells-api-to-multiple-tenant-customers, j118-tenant-to-tenant-data-sharing-via-ontology-projection, j121-business-loan-application-from-bank-tenant, j123-multi-tenant-coordinated-product-launch, j124-supply-chain-disruption-emergency-coordination, j125-marketplace-acquires-supplier-tenant-merger, j126-government-auditor-3pao-conducts-fedramp-audit, j127-dual-tenant-identity-employee-resigns-and-keeps-personal, j128-auditor-personal-side-uses-workflow-studio-for-family-taxes, j129-court-warrant-pierces-personal-tenant-with-judicial-oversight, j130-auditor-receives-bribery-attempt-via-personal-messenger, j131-cross-jurisdiction-audit-eu-vs-kr-discrepancy, j137-corporate-internal-audit-sox-controls-test, j139-internal-audit-policy-violation-cedar-permit-misuse, j149-gig-economy-multi-platform-worker, j150-creator-economy-shorts-creator-monetization-stack.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 46.
- **Per-pack overlays applicable:** passkey/WebAuthn + per-jurisdiction-KYC.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/identity/PRD.md`.

### §4.02 tenancy — substrate (Tier-1; cell-tier T0)

- **Owns (bounded contexts):** tenant-lifecycle + dual-tenant-boundary + conglomerate-hierarchy.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-1; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** AWS Organizations + Azure Tenants.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=67):** Ahmad Hassan, Aiyana Singh, Anya Mironova, Auditor IT-Specialist Jakub Nowak, Bank Compliance Officer Rishi Bhattacharya, Bank Ops Officer Olamide Adebanjo, Bank Risk Manager Anders Pedersen, Banker (external) Hideki Watanabe, Board Secretary Florence Akinsanya, Board director Patrick O'Reilly, CCO Naveen Iyer, CEO Aoki Tanaka, CISO Yuki Park, CMO Felix Ng, COO Akira Watanabe, CSO Mira Goldberg, CTO Diego Vargas, Cafeteria Manager Soyeon Kim, Captain Chen, Carlos Martinez, Channel Partner Tomas Pieter, Cleaning Supervisor Tomáš Horák, Commercial Banker Frederik Hartmann, Communications Specialist Charlotte Dubois, Compliance Analyst Yui Hayashi, Compliance Officer Tunde Bello, Consultant Adekunle Adebayo, Corp Dev Senior Analyst Saanvi Mehta, Corporate Relations Director Soo-Yeon Han, Credit Analyst Hina Mori, Diana Reyes, Engineering Manager Aisha Ali, Executive Assistant Olivia Reyes, External Auditor Dimitri Volkov, External Auditor Hyo-Jin Lee, External Regulator Inspector Sergei Petrov, IR Manager Lev Kahn, IT Manager Jamie O'Connor, Internal Comms Lead Ji-Ho Yoon, Investment Banker Yuna Ahn, Legal Counsel Anika Mehta, Marcus Chen, Maria Santos, Marketing Manager Olu Adeyemi, Medical Resident Dr. Sun-Mi Kim, Officer Rodriguez, Outside Counsel Wei-Yi Chen, PR Firm Beatriz Fernandez, PR Manager Helena Sato, Paralegal Tomáš Novák, Procurement Manager Wei Liu, Procurement Specialist Beata Kowalski, Product Designer Akihiro Sato, Product Manager Lily Chang, Project Manager Soo-Jin Park, Public Affairs Director Carlos Mendez, Receptionist Daria Volkova, Retail Banker Sebastián Vega, SWE Hugo Tanaka, Sarah Kim, Strategic Advisor Rita Almeida, Summer Intern Priscilla Sharma, Tomás García, Tomás García Jr., Training Specialist Mehmet Yilmaz, Venture Partner Lucas Müller, Yejin Park.
- **Journeys it appears in (count=22):** j01-emergency-911-dispatch, j13-cross-jurisdiction-eu-cloud-act-conflict, j20-data-residency-violation-detection, j40-b2b-marketplace-vendor-billing, j42-b2b-finops-portal-spend-attribution, j47-healthcare-billing-and-insurance, j50-sidebusiness-employee-hires-first-helper, j101-multi-tier-supply-chain-formation, j104-supplier-vendor-onboarding-kyb-cascade, j110-traveling-nurse-multi-employer-roster, j111-staffing-agency-as-tenant-facilitator, j114-employee-secondment-cross-tenant, j116-plugin-marketplace-developer-publishes-and-monetizes, j118-tenant-to-tenant-data-sharing-via-ontology-projection, j121-business-loan-application-from-bank-tenant, j123-multi-tenant-coordinated-product-launch, j125-marketplace-acquires-supplier-tenant-merger, j126-government-auditor-3pao-conducts-fedramp-audit, j127-dual-tenant-identity-employee-resigns-and-keeps-personal, j129-court-warrant-pierces-personal-tenant-with-judicial-oversight, j131-cross-jurisdiction-audit-eu-vs-kr-discrepancy, j149-gig-economy-multi-platform-worker.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 22.
- **Per-pack overlays applicable:** all packs (per ADR-0244 tenant scoping).
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/tenancy/PRD.md`.

### §4.03 policy-engine — substrate (Tier-1; cell-tier T0)

- **Owns (bounded contexts):** Cedar-fragment-publish + soak + permit-class-roster.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-1; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Cedar (AWS Verified Permissions) + OPA.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=14):** Anya Mironova, Auditor IT-Specialist Jakub Nowak, Bank Compliance Officer Rishi Bhattacharya, CCO Naveen Iyer, CISO Yuki Park, Compliance Analyst Yui Hayashi, Compliance Officer Tunde Bello, Diana Reyes, External Auditor Dimitri Volkov, External Auditor Hyo-Jin Lee, External Regulator Inspector Sergei Petrov, Legal Counsel Anika Mehta, Outside Counsel Wei-Yi Chen, Public Affairs Director Carlos Mendez.
- **Journeys it appears in (count=5):** j126-government-auditor-3pao-conducts-fedramp-audit, j127-dual-tenant-identity-employee-resigns-and-keeps-personal, j129-court-warrant-pierces-personal-tenant-with-judicial-oversight, j130-auditor-receives-bribery-attempt-via-personal-messenger, j131-cross-jurisdiction-audit-eu-vs-kr-discrepancy.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 5.
- **Per-pack overlays applicable:** all packs (Cedar fragments per pack).
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/policy-engine/PRD.md`.

### §4.04 cell — substrate (Tier-1; cell-tier T0)

- **Owns (bounded contexts):** cell-certification + shuffle-shard + pack-pinning.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-1; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** AWS shuffle-shard cell topology.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=10):** Aiyana Singh, CISO Yuki Park, Captain Chen, Dr. Tanaka, Engineering Manager Aisha Ali, IT Manager Jamie O'Connor, Officer Rodriguez, Product Designer Akihiro Sato, SWE Hugo Tanaka, Yejin Park.
- **Journeys it appears in (count=5):** j01-emergency-911-dispatch, j11-disaster-zone-offline-first-sync, j12-mass-casualty-incident-10x-traffic, j20-data-residency-violation-detection, j50-sidebusiness-employee-hires-first-helper.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 5.
- **Per-pack overlays applicable:** per-cell-certification-level (ADR-0251).
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **Pattern citation:** `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` plus successor sections in tenancy, cloud-iac, observability, api-gateway, and audit-chain architecture docs.

### §4.05 audit-chain — substrate (Tier-1; cell-tier T0)

- **Owns (bounded contexts):** Merkle-seal + event-class-registry + retention.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-1; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** AWS CloudTrail + GCP Cloud Audit Logs.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=56):** Accountant Ravi Iyer, Anya Mironova, Auditor IT-Specialist Jakub Nowak, Bank Compliance Officer Rishi Bhattacharya, Banker (external) Hideki Watanabe, Board Secretary Florence Akinsanya, Board director Patrick O'Reilly, Business Analyst Aditya Verma, CCO Naveen Iyer, CEO Aoki Tanaka, CFO Helena Brandt, CISO Yuki Park, COO Akira Watanabe, CSO Mira Goldberg, Cafeteria Manager Soyeon Kim, Captain Chen, Captain Olufemi, Carlos Martinez, Compliance Analyst Yui Hayashi, Compliance Officer Tunde Bello, Consultant Adekunle Adebayo, Corp Dev Senior Analyst Saanvi Mehta, Data Analyst Felipe Andrade, Data Scientist Yu Chen, Diana Reyes, Dr. Tanaka, External Auditor Dimitri Volkov, External Auditor Hyo-Jin Lee, External Regulator Inspector Sergei Petrov, Father Lopez, Finance Director Mei-Ling Wu, Financial Analyst Wendy Lee, Hiroshi Tanaka, IR Manager Lev Kahn, IR Specialist (unnamed), Investment Banker Yuna Ahn, Investor/LP Aanya Kapoor, Legal Counsel Anika Mehta, Marcus Chen, Officer Rodriguez, Outside Counsel Wei-Yi Chen, Paralegal Tomáš Novák, Procurement Manager Wei Liu, Procurement Specialist Beata Kowalski, Public Affairs Director Carlos Mendez, Sam Okafor, Sarah Kim, Security Analyst Anna Petrova, Strategic Advisor Rita Almeida, Sustainability Officer Aiko Brown, Tax Analyst Ji-Sung Park, Tomás García, Tomás García Jr., Venture Partner Lucas Müller, Wealth Manager Aamir Khan, Yejin Park.
- **Journeys it appears in (count=34):** j01-emergency-911-dispatch, j05-whistleblower-anonymous-ethics-report, j06-press-source-securedrop-class, j07-deceased-user-inheritance-handoff, j12-mass-casualty-incident-10x-traffic, j14-delegated-llm-agent-acting-for-yejin, j15-bug-bounty-researcher-submission, j18-child-safety-mandatory-reporter, j19-tenant-break-glass-locked-out-tenant-admin, j38-b2b-e-signing-contract, j43-healthcare-nurse-patient-handoff, j44-healthcare-telemedicine-consultation, j45-healthcare-patient-portal-records, j101-multi-tier-supply-chain-formation, j102-raw-material-purchase-with-quality-attestation, j103-just-in-time-procurement-automation, j104-supplier-vendor-onboarding-kyb-cascade, j105-dispute-cross-tenant-arbitration, j106-multi-currency-cross-border-payment, j107-supply-chain-disruption-and-failover, j118-tenant-to-tenant-data-sharing-via-ontology-projection, j119-invoice-financing-marketplace, j124-supply-chain-disruption-emergency-coordination, j125-marketplace-acquires-supplier-tenant-merger, j126-government-auditor-3pao-conducts-fedramp-audit, j127-dual-tenant-identity-employee-resigns-and-keeps-personal, j128-auditor-personal-side-uses-workflow-studio-for-family-taxes, j129-court-warrant-pierces-personal-tenant-with-judicial-oversight, j130-auditor-receives-bribery-attempt-via-personal-messenger, j131-cross-jurisdiction-audit-eu-vs-kr-discrepancy, j137-corporate-internal-audit-sox-controls-test, j138-corporate-audit-fraud-investigation-via-pattern-detection, j139-internal-audit-policy-violation-cedar-permit-misuse, j148-supply-chain-circular-economy-electronics-recycling.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 34.
- **Per-pack overlays applicable:** all packs (retention per jurisdiction).
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/audit-chain/PRD.md`.

### §4.06 observability — substrate (Tier-1; cell-tier T0)

- **Owns (bounded contexts):** metric-emission + SLO-contract + trace-context.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-1; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Datadog + Honeycomb + OpenTelemetry.
- **Center-of-gravity rank (per coverage-matrix §14):** 9.
- **Personas who depend on it (count=38):** Ahmad Hassan, Anya Mironova, Apprentice Jakob Bauer, Auditor IT-Specialist Jakub Nowak, Bank Compliance Officer Rishi Bhattacharya, Bank Risk Manager Anders Pedersen, CFO Helena Brandt, CISO Yuki Park, CS-IC Lin Chen, Captain Chen, Captain Olufemi, Carlos Martinez, Channel Partner Tomas Pieter, Compliance Analyst Yui Hayashi, Compliance Officer Tunde Bello, Customer Champion Akemi Sato, Customer Success Manager Sofia Rezende, DevOps Engineer Olukayode Adejumo, DevOps Manager Pavel Korsak, Devon Williams, Diana Reyes, Dr. Tanaka, External Auditor Dimitri Volkov, External Auditor Hyo-Jin Lee, External Regulator Inspector Sergei Petrov, IT Manager Jamie O'Connor, Marcus Chen, Officer Rodriguez, Procurement Manager Wei Liu, Procurement Specialist Beata Kowalski, Sales AE Maya Lindqvist, Sales Manager Anthony Costa, Security Analyst Anna Petrova, Support Rep Nadia Hassani, Tomás García, Trader Mei Lin, Treasury Ops Sven Eriksson, Yejin Park.
- **Journeys it appears in (count=20):** j01-emergency-911-dispatch, j04-dv-survivor-shelter-mode, j05-whistleblower-anonymous-ethics-report, j10-account-takeover-SIM-swap-detected, j12-mass-casualty-incident-10x-traffic, j13-cross-jurisdiction-eu-cloud-act-conflict, j20-data-residency-violation-detection, j37-b2b-clocking-and-attendance, j39-b2b-meeting-with-transcription, j41-b2b-developer-builds-on-platform, j42-b2b-finops-portal-spend-attribution, j103-just-in-time-procurement-automation, j107-supply-chain-disruption-and-failover, j109-construction-co-hires-freelance-specialist, j115-saas-vendor-sells-api-to-multiple-tenant-customers, j117-api-customer-tenant-incident-response, j120-tenant-treasury-multi-currency-fx-hedge, j126-government-auditor-3pao-conducts-fedramp-audit, j127-dual-tenant-identity-employee-resigns-and-keeps-personal, j131-cross-jurisdiction-audit-eu-vs-kr-discrepancy.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 20.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/observability/PRD.md`.

### §4.07 api-gateway — substrate (Tier-2; cell-tier T0)

- **Owns (bounded contexts):** SPIFFE-attestation + ingress + OpenAPI-3.2.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Kong + AWS API Gateway.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=9):** Auditor IT-Specialist Jakub Nowak, Captain Chen, Diana Reyes, Dr. Tanaka, External Auditor Dimitri Volkov, External Auditor Hyo-Jin Lee, External Regulator Inspector Sergei Petrov, Officer Rodriguez, Yejin Park.
- **Journeys it appears in (count=3):** j01-emergency-911-dispatch, j12-mass-casualty-incident-10x-traffic, j126-government-auditor-3pao-conducts-fedramp-audit.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 3.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/api-gateway/PRD.md`.

### §4.08 cloud-iac — substrate (Tier-2; cell-tier T0)

- **Owns (bounded contexts):** regional-overlay + promotion-evidence + control-manifest.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Terraform + CDK + Pulumi.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/cloud-iac/PRD.md`.

### §4.09 cloud-k8s — substrate (Tier-2; cell-tier T0)

- **Owns (bounded contexts):** namespace-isolation + admission-labels + workload-placement.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** EKS + GKE + AKS.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/cloud-k8s/PRD.md`.

### §4.10 cloud-secrets — substrate (Tier-2; cell-tier T0)

- **Owns (bounded contexts):** OpenBao + per-pack-signing-keys + TTL-rotation.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** HashiCorp Vault + AWS KMS.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/cloud-secrets/PRD.md`.

### §4.11 feature-flags — substrate (Tier-3; cell-tier T1)

- **Owns (bounded contexts):** kill-switch + canary + percent-rollout.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-3; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** LaunchDarkly + Split + Flagsmith.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/feature-flags/PRD.md`.

### §4.12 network — substrate (Tier-2; cell-tier T0)

- **Owns (bounded contexts):** HTTP/3-QUIC + service-mesh + zero-trust-east-west.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Istio + Linkerd + Envoy.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/cloud-network/PRD.md`.

### §4.13 compliance — substrate (Tier-1; cell-tier T0)

- **Owns (bounded contexts):** pack-overlay + regulator-portal + evidence-inventory.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-1; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Drata + Vanta + SecureFrame.
- **Center-of-gravity rank (per coverage-matrix §14):** 8.
- **Personas who depend on it (count=57):** Accountant Ravi Iyer, Aiyana Singh, Anya Mironova, Auditor IT-Specialist Jakub Nowak, Bank Compliance Officer Rishi Bhattacharya, Bank Ops Officer Olamide Adebanjo, Banker (external) Hideki Watanabe, Board Secretary Florence Akinsanya, Board director Patrick O'Reilly, Business Analyst Aditya Verma, CCO Naveen Iyer, CEO Aoki Tanaka, CFO Helena Brandt, CISO Yuki Park, CSO Mira Goldberg, Cafeteria Manager Soyeon Kim, Captain Chen, Carlos Martinez, Compliance Analyst Yui Hayashi, Compliance Officer Tunde Bello, Consultant Adekunle Adebayo, Corp Dev Senior Analyst Saanvi Mehta, DevOps Engineer Olukayode Adejumo, DevOps Manager Pavel Korsak, Diana Reyes, External Auditor Dimitri Volkov, External Auditor Hyo-Jin Lee, External Regulator Inspector Sergei Petrov, Finance Director Mei-Ling Wu, Financial Analyst Wendy Lee, IR Manager Lev Kahn, IR Specialist (unnamed), IT Manager Jamie O'Connor, Investment Banker Yuna Ahn, Investor/LP Aanya Kapoor, Legal Counsel Anika Mehta, Marcus Chen, Officer Rodriguez, Outside Counsel Wei-Yi Chen, Paralegal Tomáš Novák, Procurement Manager Wei Liu, Procurement Specialist Beata Kowalski, Project Manager Soo-Jin Park, Public Affairs Director Carlos Mendez, SWE Hugo Tanaka, Sam Okafor, Sarah Kim, Strategic Advisor Rita Almeida, Summer Intern Priscilla Sharma, Tax Analyst Ji-Sung Park, Tomás García, Tomás García Jr., Training Specialist Mehmet Yilmaz, Treasury Ops Sven Eriksson, Venture Partner Lucas Müller, Wealth Manager Aamir Khan, Yejin Park.
- **Journeys it appears in (count=22):** j01-emergency-911-dispatch, j13-cross-jurisdiction-eu-cloud-act-conflict, j20-data-residency-violation-detection, j43-healthcare-nurse-patient-handoff, j44-healthcare-telemedicine-consultation, j45-healthcare-patient-portal-records, j46-healthcare-prescription-renewal-workflow, j47-healthcare-billing-and-insurance, j48-sidebusiness-stripe-tax-and-invoicing, j101-multi-tier-supply-chain-formation, j104-supplier-vendor-onboarding-kyb-cascade, j105-dispute-cross-tenant-arbitration, j106-multi-currency-cross-border-payment, j118-tenant-to-tenant-data-sharing-via-ontology-projection, j119-invoice-financing-marketplace, j122-vendor-payment-batch-with-tax-withholding, j125-marketplace-acquires-supplier-tenant-merger, j126-government-auditor-3pao-conducts-fedramp-audit, j129-court-warrant-pierces-personal-tenant-with-judicial-oversight, j130-auditor-receives-bribery-attempt-via-personal-messenger, j131-cross-jurisdiction-audit-eu-vs-kr-discrepancy, j137-corporate-internal-audit-sox-controls-test.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 22.
- **Per-pack overlays applicable:** all packs (HIPAA + GDPR + SOC2 + CSAP + PCI + EU-AI-Act + ...).
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/compliance/PRD.md`.

### §4.14 governance — substrate (Tier-1; cell-tier T0)

- **Owns (bounded contexts):** board-resolution + audit-portal + ADR-registry.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-1; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** OneTrust + ServiceNow GRC.
- **Center-of-gravity rank (per coverage-matrix §14):** 14.
- **Personas who depend on it (count=11):** Anya Mironova, CCO Naveen Iyer, CISO Yuki Park, Compliance Analyst Yui Hayashi, Compliance Officer Tunde Bello, Diana Reyes, External Regulator Inspector Sergei Petrov, Legal Counsel Anika Mehta, Outside Counsel Wei-Yi Chen, Public Affairs Director Carlos Mendez, Sam Okafor.
- **Journeys it appears in (count=3):** j19-tenant-break-glass-locked-out-tenant-admin, j129-court-warrant-pierces-personal-tenant-with-judicial-oversight, j139-internal-audit-policy-violation-cedar-permit-misuse.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 3.
- **Per-pack overlays applicable:** SOX-404 + board-pack + audit-portal.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/governance/PRD.md`.

### §4.15 consent-graph — substrate (Tier-2; cell-tier T0)

- **Owns (bounded contexts):** lawful-basis + opt-in-fields + revocation.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** OneTrust Consent + TrustArc.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=3):** Captain Chen, Officer Rodriguez, Yejin Park.
- **Journeys it appears in (count=2):** j01-emergency-911-dispatch, j04-dv-survivor-shelter-mode.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 2.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/consent-graph/PRD.md`.

### §4.16 ontology — substrate (Tier-1; cell-tier T0)

- **Owns (bounded contexts):** object-types + projections + cross-tenant-bridge.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-1; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Palantir Foundry Ontology.
- **Center-of-gravity rank (per coverage-matrix §14):** 3.
- **Personas who depend on it (count=25):** Board Secretary Florence Akinsanya, Board director Patrick O'Reilly, CEO Aoki Tanaka, CSO Mira Goldberg, Cafeteria Manager Soyeon Kim, Captain Chen, Carlos Martinez, Consultant Adekunle Adebayo, Corp Dev Senior Analyst Saanvi Mehta, Data Analyst Felipe Andrade, Data Scientist Yu Chen, Investment Banker Yuna Ahn, Marcus Chen, Marketing Specialist Riya Sharma, Officer Rodriguez, Outside Counsel Wei-Yi Chen, Paralegal Tomáš Novák, Procurement Manager Wei Liu, Procurement Specialist Beata Kowalski, Strategic Advisor Rita Almeida, Sustainability Officer Aiko Brown, Tomás García, Tomás García Jr., Venture Partner Lucas Müller, Yejin Park.
- **Journeys it appears in (count=8):** j01-emergency-911-dispatch, j43-healthcare-nurse-patient-handoff, j101-multi-tier-supply-chain-formation, j104-supplier-vendor-onboarding-kyb-cascade, j118-tenant-to-tenant-data-sharing-via-ontology-projection, j125-marketplace-acquires-supplier-tenant-merger, j148-supply-chain-circular-economy-electronics-recycling, j150-creator-economy-shorts-creator-monetization-stack.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 8.
- **Per-pack overlays applicable:** per-tenant + per-pack.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/ontology/PRD.md`.

### §4.17 workflow-engine — substrate (Tier-1; cell-tier T0)

- **Owns (bounded contexts):** engine-runtime + template-registry + Cedar-pin.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-1; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** n8n + Temporal + Camunda + Airflow.
- **Center-of-gravity rank (per coverage-matrix §14):** 1.
- **Personas who depend on it (count=86):** Accountant Ravi Iyer, Ahmad Hassan, Aiyana Singh, Anya Mironova, Apprentice Jakob Bauer, Auditor IT-Specialist Jakub Nowak, Bank Compliance Officer Rishi Bhattacharya, Bank Ops Officer Olamide Adebanjo, Bank Risk Manager Anders Pedersen, Banker (external) Hideki Watanabe, Board Secretary Florence Akinsanya, Board director Patrick O'Reilly, CCO Naveen Iyer, CEO Aoki Tanaka, CFO Helena Brandt, CISO Yuki Park, CMO Felix Ng, COO Akira Watanabe, CS-IC Lin Chen, CSO Mira Goldberg, CTO Diego Vargas, Cafeteria Manager Soyeon Kim, Captain Chen, Captain Olufemi, Carlos Martinez, Channel Partner Tomas Pieter, Cleaning Supervisor Tomáš Horák, Commercial Banker Frederik Hartmann, Communications Specialist Charlotte Dubois, Compliance Analyst Yui Hayashi, Compliance Officer Tunde Bello, Consultant Adekunle Adebayo, Corp Dev Senior Analyst Saanvi Mehta, Corporate Relations Director Soo-Yeon Han, Credit Analyst Hina Mori, Customer Champion Akemi Sato, Customer Success Manager Sofia Rezende, Data Analyst Felipe Andrade, Data Scientist Yu Chen, DevOps Engineer Olukayode Adejumo, DevOps Manager Pavel Korsak, Devon Williams, Diana Reyes, Engineering Manager Aisha Ali, Executive Assistant Olivia Reyes, External Auditor Dimitri Volkov, External Auditor Hyo-Jin Lee, External Regulator Inspector Sergei Petrov, Finance Director Mei-Ling Wu, Financial Analyst Wendy Lee, Hiroshi Tanaka, IR Manager Lev Kahn, IT Manager Jamie O'Connor, Internal Comms Lead Ji-Ho Yoon, Investment Banker Yuna Ahn, Legal Counsel Anika Mehta, Marcus Chen, Marketing Manager Olu Adeyemi, Officer Rodriguez, Outside Counsel Wei-Yi Chen, PR Firm Beatriz Fernandez, PR Manager Helena Sato, Procurement Manager Wei Liu, Procurement Specialist Beata Kowalski, Product Designer Akihiro Sato, Product Manager Lily Chang, Project Manager Soo-Jin Park, Public Affairs Director Carlos Mendez, Retail Banker Sebastián Vega, SWE Hugo Tanaka, Sales AE Maya Lindqvist, Sales Manager Anthony Costa, Sam Okafor, Sarah Kim, Security Analyst Anna Petrova, Strategic Advisor Rita Almeida, Support Rep Nadia Hassani, Sustainability Officer Aiko Brown, Tax Analyst Ji-Sung Park, Tomás García, Tomás García Jr., Trader Mei Lin, Treasury Ops Sven Eriksson, Venture Partner Lucas Müller, Wealth Manager Aamir Khan, Yejin Park.
- **Journeys it appears in (count=37):** j01-emergency-911-dispatch, j08-elder-financial-abuse-detection, j14-delegated-llm-agent-acting-for-yejin, j18-child-safety-mandatory-reporter, j36-b2b-workflow-engine-approval-cascade, j41-b2b-developer-builds-on-platform, j46-healthcare-prescription-renewal-workflow, j50-sidebusiness-employee-hires-first-helper, j101-multi-tier-supply-chain-formation, j102-raw-material-purchase-with-quality-attestation, j103-just-in-time-procurement-automation, j104-supplier-vendor-onboarding-kyb-cascade, j105-dispute-cross-tenant-arbitration, j107-supply-chain-disruption-and-failover, j109-construction-co-hires-freelance-specialist, j111-staffing-agency-as-tenant-facilitator, j112-tenant-to-tenant-rfq-and-bid, j114-employee-secondment-cross-tenant, j115-saas-vendor-sells-api-to-multiple-tenant-customers, j117-api-customer-tenant-incident-response, j120-tenant-treasury-multi-currency-fx-hedge, j121-business-loan-application-from-bank-tenant, j122-vendor-payment-batch-with-tax-withholding, j123-multi-tenant-coordinated-product-launch, j124-supply-chain-disruption-emergency-coordination, j125-marketplace-acquires-supplier-tenant-merger, j126-government-auditor-3pao-conducts-fedramp-audit, j127-dual-tenant-identity-employee-resigns-and-keeps-personal, j128-auditor-personal-side-uses-workflow-studio-for-family-taxes, j129-court-warrant-pierces-personal-tenant-with-judicial-oversight, j130-auditor-receives-bribery-attempt-via-personal-messenger, j131-cross-jurisdiction-audit-eu-vs-kr-discrepancy, j137-corporate-internal-audit-sox-controls-test, j138-corporate-audit-fraud-investigation-via-pattern-detection, j139-internal-audit-policy-violation-cedar-permit-misuse, j148-supply-chain-circular-economy-electronics-recycling, j149-gig-economy-multi-platform-worker.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 37.
- **Per-pack overlays applicable:** per-pack via template-registry.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/workflow-engine/PRD.md`.

### §4.18 workflow-studio — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** visual-editor + template-import + tenant-pack.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** n8n Editor + Zapier + Power Automate.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=7):** Aiyana Singh, DevOps Engineer Olukayode Adejumo, DevOps Manager Pavel Korsak, Diana Reyes, Sarah Kim, Tax Analyst Ji-Sung Park, Yejin Park.
- **Journeys it appears in (count=3):** j36-b2b-workflow-engine-approval-cascade, j46-healthcare-prescription-renewal-workflow, j128-auditor-personal-side-uses-workflow-studio-for-family-taxes.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 3.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/workflow-studio/PRD.md`.

### §4.19 intelligence — substrate (Tier-1; cell-tier T1)

- **Owns (bounded contexts):** model-routing + grounding + cost-attribution.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-1; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** OpenAI + Anthropic + Bedrock + Vertex.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=28):** CEO Aoki Tanaka, CMO Felix Ng, COO Akira Watanabe, CTO Diego Vargas, Captain Chen, Captain Olufemi, Carlos Martinez, Communications Specialist Charlotte Dubois, Corporate Relations Director Soo-Yeon Han, Data Analyst Felipe Andrade, Data Scientist Yu Chen, Diana Reyes, Executive Assistant Olivia Reyes, Internal Comms Lead Ji-Ho Yoon, Jordan Lee, Marcus Chen, Marketing Manager Olu Adeyemi, Marketing Specialist Riya Sharma, Ms. Patel, Officer Rodriguez, PR Firm Beatriz Fernandez, PR Manager Helena Sato, Product Manager Lily Chang, Project Manager Soo-Jin Park, Tax Analyst Ji-Sung Park, Tomás García, Tomás García Jr., Yejin Park.
- **Journeys it appears in (count=12):** j01-emergency-911-dispatch, j13-cross-jurisdiction-eu-cloud-act-conflict, j14-delegated-llm-agent-acting-for-yejin, j16-disability-accommodation-voice-only-signup, j39-b2b-meeting-with-transcription, j43-healthcare-nurse-patient-handoff, j44-healthcare-telemedicine-consultation, j49-sidebusiness-customer-support-omnichannel, j108-supplier-rating-and-marketplace-discovery, j123-multi-tenant-coordinated-product-launch, j128-auditor-personal-side-uses-workflow-studio-for-family-taxes, j150-creator-economy-shorts-creator-monetization-stack.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 12.
- **Per-pack overlays applicable:** EU-AI-Act + per-vertical-AI.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/intelligence/PRD.md`.

### §4.20 foundry — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** agent-runtime + scaffold + plugin-orchestration.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Palantir Foundry + AWS Bedrock Agents.
- **Center-of-gravity rank (per coverage-matrix §14):** 10.
- **Personas who depend on it (count=2):** Carlos Martinez, Marcus Chen.
- **Journeys it appears in (count=2):** j41-b2b-developer-builds-on-platform, j116-plugin-marketplace-developer-publishes-and-monetizes.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 2.
- **Per-pack overlays applicable:** EU-AI-Act + per-vertical-Annex-III.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/intelligence/PRD.md`.

### §4.21 developer-sdk — substrate (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** client-libs + SDK-codegen + sample-apps.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Stripe SDK + Twilio SDK.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=2):** Carlos Martinez, Marcus Chen.
- **Journeys it appears in (count=1):** j41-b2b-developer-builds-on-platform.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 1.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/developer-sdk/PRD.md`.

### §4.22 plugin-app-store — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** publish + monetization + sandbox.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Slack App Directory + Shopify App Store.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=21):** Aiyana Singh, Board director Patrick O'Reilly, Business Analyst Aditya Verma, CFO Helena Brandt, Channel Partner Tomas Pieter, Customer Success Manager Sofia Rezende, Data Analyst Felipe Andrade, Data Scientist Yu Chen, IR Manager Lev Kahn, IR Specialist (unnamed), Investment Banker Yuna Ahn, Investor/LP Aanya Kapoor, Marcus Chen, Marketing Specialist Riya Sharma, Sales AE Maya Lindqvist, Sales Manager Anthony Costa, Sustainability Officer Aiko Brown, Tomás García Jr., Venture Partner Lucas Müller, Wealth Manager Aamir Khan, Yejin Park.
- **Journeys it appears in (count=7):** j40-b2b-marketplace-vendor-billing, j49-sidebusiness-customer-support-omnichannel, j115-saas-vendor-sells-api-to-multiple-tenant-customers, j116-plugin-marketplace-developer-publishes-and-monetizes, j119-invoice-financing-marketplace, j148-supply-chain-circular-economy-electronics-recycling, j150-creator-economy-shorts-creator-monetization-stack.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 7.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/plugin-app-store/PRD.md`.

### §4.23 application — substrate (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** shell + locale-copy + session-affordances.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Next.js shell + Vue shell.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=3):** Jordan Lee, Ms. Patel, Yejin Park.
- **Journeys it appears in (count=1):** j16-disability-accommodation-voice-only-signup.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 1.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/application/PRD.md`.

### §4.24 connect — substrate (Tier-2; cell-tier T0)

- **Owns (bounded contexts):** cross-tenant-connector + facilitator-tenant + bridge.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Workato + MuleSoft + Boomi.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=37):** Accountant Ravi Iyer, Aiyana Singh, Bank Compliance Officer Rishi Bhattacharya, Bank Ops Officer Olamide Adebanjo, Bank Risk Manager Anders Pedersen, Banker (external) Hideki Watanabe, CFO Helena Brandt, Captain Chen, Captain Olufemi, Carlos Martinez, Commercial Banker Frederik Hartmann, Compliance Officer Tunde Bello, Credit Analyst Hina Mori, DevOps Engineer Olukayode Adejumo, DevOps Manager Pavel Korsak, Diana Reyes, Finance Director Mei-Ling Wu, Financial Analyst Wendy Lee, IR Manager Lev Kahn, IT Manager Jamie O'Connor, Investment Banker Yuna Ahn, Marcus Chen, Procurement Manager Wei Liu, Procurement Specialist Beata Kowalski, Project Manager Soo-Jin Park, Retail Banker Sebastián Vega, SWE Hugo Tanaka, Sarah Kim, Summer Intern Priscilla Sharma, Sustainability Officer Aiko Brown, Tax Analyst Ji-Sung Park, Tomás García, Tomás García Jr., Trader Mei Lin, Training Specialist Mehmet Yilmaz, Treasury Ops Sven Eriksson, Yejin Park.
- **Journeys it appears in (count=18):** j11-disaster-zone-offline-first-sync, j37-b2b-clocking-and-attendance, j44-healthcare-telemedicine-consultation, j46-healthcare-prescription-renewal-workflow, j47-healthcare-billing-and-insurance, j48-sidebusiness-stripe-tax-and-invoicing, j49-sidebusiness-customer-support-omnichannel, j102-raw-material-purchase-with-quality-attestation, j103-just-in-time-procurement-automation, j104-supplier-vendor-onboarding-kyb-cascade, j106-multi-currency-cross-border-payment, j107-supply-chain-disruption-and-failover, j120-tenant-treasury-multi-currency-fx-hedge, j121-business-loan-application-from-bank-tenant, j122-vendor-payment-batch-with-tax-withholding, j128-auditor-personal-side-uses-workflow-studio-for-family-taxes, j148-supply-chain-circular-economy-electronics-recycling, j149-gig-economy-multi-platform-worker.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 18.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/connector/PRD.md`.

### §4.25 data-pipeline — substrate (Tier-3; cell-tier T1)

- **Owns (bounded contexts):** ingest + transform + lineage.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-3; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Fivetran + dbt + Airbyte.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/data-pipeline/PRD.md`.

### §4.26 data-warehouse — substrate (Tier-3; cell-tier T1)

- **Owns (bounded contexts):** OLAP-store + workspace + query-engine.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-3; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Snowflake + BigQuery + Redshift.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/data-warehouse/PRD.md`.

### §4.27 analytics — substrate (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** cohort + KPI + transparency-report.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Mixpanel + Amplitude + Looker.
- **Center-of-gravity rank (per coverage-matrix §14):** 2.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/analytics/PRD.md`.

### §4.28 detection — substrate (Tier-2; cell-tier T0)

- **Owns (bounded contexts):** abuse-defence + anti-bot + spoof + scrape.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** CrowdStrike + Cloudflare Bot Mgmt.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=7):** CISO Yuki Park, Compliance Analyst Yui Hayashi, Compliance Officer Tunde Bello, Data Analyst Felipe Andrade, Data Scientist Yu Chen, Sam Okafor, Security Analyst Anna Petrova.
- **Journeys it appears in (count=1):** j138-corporate-audit-fraud-investigation-via-pattern-detection.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 1.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/detection/PRD.md`.

### §4.29 incident-management — substrate (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** incident-lifecycle + postmortem + RCA.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** PagerDuty + Opsgenie + Incident.io.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** DORA + per-vertical-incident.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/incident-management/PRD.md`.

### §4.30 itsm — product (Tier-3; cell-tier T1)

- **Owns (bounded contexts):** ticket-lifecycle + change-mgmt + asset-CMDB.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-3; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** ServiceNow + Jira Service Mgmt.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/itsm/PRD.md`.

### §4.31 ops-dashboard-control-center — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** executive-view + on-call-routing + SLO-board.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Datadog Dashboards + Grafana.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=16):** Accountant Ravi Iyer, Auditor IT-Specialist Jakub Nowak, Board Secretary Florence Akinsanya, Board director Patrick O'Reilly, CCO Naveen Iyer, CFO Helena Brandt, CISO Yuki Park, Compliance Analyst Yui Hayashi, Compliance Officer Tunde Bello, Diana Reyes, External Auditor Dimitri Volkov, External Auditor Hyo-Jin Lee, External Regulator Inspector Sergei Petrov, Finance Director Mei-Ling Wu, Financial Analyst Wendy Lee, Sam Okafor.
- **Journeys it appears in (count=4):** j19-tenant-break-glass-locked-out-tenant-admin, j126-government-auditor-3pao-conducts-fedramp-audit, j137-corporate-internal-audit-sox-controls-test, j139-internal-audit-policy-violation-cedar-permit-misuse.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 4.
- **Per-pack overlays applicable:** SOX-404 + executive-pack.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/ops-dashboard-control-center/PRD.md`.

### §4.32 messenger — product (Tier-1; cell-tier T0)

- **Owns (bounded contexts):** MLS-group + e2ee + group-rekey + crash-recovery.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-1; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Signal + WhatsApp (MLS RFC 9420).
- **Center-of-gravity rank (per coverage-matrix §14):** 11.
- **Personas who depend on it (count=54):** Accountant Ravi Iyer, Anya Mironova, Auditor IT-Specialist Jakub Nowak, Board Secretary Florence Akinsanya, Board director Patrick O'Reilly, CCO Naveen Iyer, CEO Aoki Tanaka, CFO Helena Brandt, CISO Yuki Park, CMO Felix Ng, COO Akira Watanabe, CS-IC Lin Chen, CTO Diego Vargas, Captain Chen, Co-op Student Liam Murphy, Communications Specialist Charlotte Dubois, Compliance Officer Tunde Bello, Corporate Relations Director Soo-Yeon Han, Customer Champion Akemi Sato, Customer Success Manager Sofia Rezende, DevOps Engineer Olukayode Adejumo, DevOps Manager Pavel Korsak, Diana Reyes, Executive Assistant Olivia Reyes, External Auditor Dimitri Volkov, External Auditor Hyo-Jin Lee, External Regulator Inspector Sergei Petrov, Father Lopez, Finance Director Mei-Ling Wu, Financial Analyst Wendy Lee, Hiroshi Tanaka, IT Manager Jamie O'Connor, Intern Manager Felicia Adamou, Internal Comms Lead Ji-Ho Yoon, Legal Counsel Anika Mehta, Marcus Chen, Marketing Manager Olu Adeyemi, Ms. Patel, Officer Rodriguez, Outside Counsel Wei-Yi Chen, PR Firm Beatriz Fernandez, PR Manager Helena Sato, Product Manager Lily Chang, Project Manager Soo-Jin Park, Public Affairs Director Carlos Mendez, Retail Banker Sebastián Vega, Returning Intern Jia Han, Sam Okafor, Security Analyst Anna Petrova, Summer Intern Priscilla Sharma, Support Rep Nadia Hassani, Tomás García, Wealth Manager Aamir Khan, Yejin Park.
- **Journeys it appears in (count=20):** j01-emergency-911-dispatch, j04-dv-survivor-shelter-mode, j06-press-source-securedrop-class, j08-elder-financial-abuse-detection, j09-account-recovery-phishing-resistant, j10-account-takeover-SIM-swap-detected, j11-disaster-zone-offline-first-sync, j14-delegated-llm-agent-acting-for-yejin, j17-activist-dissident-high-risk-mode, j49-sidebusiness-customer-support-omnichannel, j105-dispute-cross-tenant-arbitration, j113-cross-tenant-internship-from-handshake, j117-api-customer-tenant-incident-response, j123-multi-tenant-coordinated-product-launch, j124-supply-chain-disruption-emergency-coordination, j126-government-auditor-3pao-conducts-fedramp-audit, j127-dual-tenant-identity-employee-resigns-and-keeps-personal, j129-court-warrant-pierces-personal-tenant-with-judicial-oversight, j130-auditor-receives-bribery-attempt-via-personal-messenger, j137-corporate-internal-audit-sox-controls-test.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 20.
- **Per-pack overlays applicable:** MLS-RFC9420 + per-jurisdiction-CALEA.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/messenger/PRD.md`.

### §4.33 mail — product (Tier-1; cell-tier T0)

- **Owns (bounded contexts):** MIME + DKIM/DMARC + signed-delivery + tenant-archive.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-1; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Gmail + Outlook + Hey.
- **Center-of-gravity rank (per coverage-matrix §14):** 18.
- **Personas who depend on it (count=46):** Accountant Ravi Iyer, Aiyana Singh, Bank Compliance Officer Rishi Bhattacharya, Bank Ops Officer Olamide Adebanjo, Board Secretary Florence Akinsanya, Board director Patrick O'Reilly, CCO Naveen Iyer, CFO Helena Brandt, CISO Yuki Park, COO Akira Watanabe, CS-IC Lin Chen, Cafeteria Manager Soyeon Kim, Captain Chen, Captain Olufemi, Compliance Analyst Yui Hayashi, Compliance Officer Tunde Bello, Customer Champion Akemi Sato, Customer Success Manager Sofia Rezende, Data Analyst Felipe Andrade, Data Scientist Yu Chen, DevOps Engineer Olukayode Adejumo, DevOps Manager Pavel Korsak, Diana Reyes, Finance Director Mei-Ling Wu, Financial Analyst Wendy Lee, Hiroshi Tanaka, IT Manager Jamie O'Connor, Marcus Chen, Officer Rodriguez, Procurement Manager Wei Liu, Procurement Specialist Beata Kowalski, Project Manager Soo-Jin Park, Public Affairs Director Carlos Mendez, Retail Banker Sebastián Vega, SWE Hugo Tanaka, Sam Okafor, Sarah Kim, Security Analyst Anna Petrova, Summer Intern Priscilla Sharma, Support Rep Nadia Hassani, Tax Analyst Ji-Sung Park, Tomás García, Tomás García Jr., Training Specialist Mehmet Yilmaz, Treasury Ops Sven Eriksson, Yejin Park.
- **Journeys it appears in (count=23):** j01-emergency-911-dispatch, j04-dv-survivor-shelter-mode, j07-deceased-user-inheritance-handoff, j09-account-recovery-phishing-resistant, j18-child-safety-mandatory-reporter, j36-b2b-workflow-engine-approval-cascade, j38-b2b-e-signing-contract, j40-b2b-marketplace-vendor-billing, j45-healthcare-patient-portal-records, j46-healthcare-prescription-renewal-workflow, j47-healthcare-billing-and-insurance, j48-sidebusiness-stripe-tax-and-invoicing, j49-sidebusiness-customer-support-omnichannel, j101-multi-tier-supply-chain-formation, j105-dispute-cross-tenant-arbitration, j107-supply-chain-disruption-and-failover, j117-api-customer-tenant-incident-response, j122-vendor-payment-batch-with-tax-withholding, j124-supply-chain-disruption-emergency-coordination, j127-dual-tenant-identity-employee-resigns-and-keeps-personal, j128-auditor-personal-side-uses-workflow-studio-for-family-taxes, j137-corporate-internal-audit-sox-controls-test, j138-corporate-audit-fraud-investigation-via-pattern-detection.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 23.
- **Per-pack overlays applicable:** S/MIME + DKIM/DMARC + per-jurisdiction-retention.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/mail/PRD.md`.

### §4.34 comms-email — substrate (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** transactional + bounce + suppression.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** SendGrid + Postmark + Mailgun.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=11):** Anya Mironova, Auditor IT-Specialist Jakub Nowak, CCO Naveen Iyer, CISO Yuki Park, Diana Reyes, External Auditor Dimitri Volkov, External Auditor Hyo-Jin Lee, External Regulator Inspector Sergei Petrov, Legal Counsel Anika Mehta, Outside Counsel Wei-Yi Chen, Public Affairs Director Carlos Mendez.
- **Journeys it appears in (count=4):** j126-government-auditor-3pao-conducts-fedramp-audit, j127-dual-tenant-identity-employee-resigns-and-keeps-personal, j129-court-warrant-pierces-personal-tenant-with-judicial-oversight, j130-auditor-receives-bribery-attempt-via-personal-messenger.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 4.
- **Per-pack overlays applicable:** CAN-SPAM + GDPR-marketing.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/comms-email/PRD.md`.

### §4.35 calendar — product (Tier-1; cell-tier T1)

- **Owns (bounded contexts):** iCal + scheduling + permit-context.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-1; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Google Calendar + Outlook + Cron.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=8):** Captain Chen, Co-op Student Liam Murphy, Diana Reyes, Intern Manager Felicia Adamou, Officer Rodriguez, Returning Intern Jia Han, Summer Intern Priscilla Sharma, Yejin Park.
- **Journeys it appears in (count=3):** j01-emergency-911-dispatch, j113-cross-tenant-internship-from-handshake, j127-dual-tenant-identity-employee-resigns-and-keeps-personal.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 3.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/calendar/PRD.md`.

### §4.36 meet — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** WebRTC + SFU + recording-consent.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Zoom + Google Meet + Webex.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=2):** Carlos Martinez, Diana Reyes.
- **Journeys it appears in (count=3):** j39-b2b-meeting-with-transcription, j44-healthcare-telemedicine-consultation, j127-dual-tenant-identity-employee-resigns-and-keeps-personal.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 3.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/meet/PRD.md`.

### §4.37 drive — product (Tier-1; cell-tier T0)

- **Owns (bounded contexts):** object-store + versioning + tenant-archive.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-1; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Google Drive + Dropbox + iCloud.
- **Center-of-gravity rank (per coverage-matrix §14):** 6.
- **Personas who depend on it (count=39):** Anya Mironova, Board director Patrick O'Reilly, CCO Naveen Iyer, CEO Aoki Tanaka, CISO Yuki Park, CMO Felix Ng, COO Akira Watanabe, CSO Mira Goldberg, CTO Diego Vargas, Captain Chen, Communications Specialist Charlotte Dubois, Consultant Adekunle Adebayo, Corp Dev Senior Analyst Saanvi Mehta, Corporate Relations Director Soo-Yeon Han, Diana Reyes, Executive Assistant Olivia Reyes, External Regulator Inspector Sergei Petrov, Father Lopez, Hiroshi Tanaka, Internal Comms Lead Ji-Ho Yoon, Investment Banker Yuna Ahn, Legal Counsel Anika Mehta, Marcus Chen, Marketing Manager Olu Adeyemi, Ms. Patel, Officer Rodriguez, Outside Counsel Wei-Yi Chen, PR Firm Beatriz Fernandez, PR Manager Helena Sato, Product Manager Lily Chang, Project Manager Soo-Jin Park, Public Affairs Director Carlos Mendez, Sarah Kim, Strategic Advisor Rita Almeida, Tax Analyst Ji-Sung Park, Tomás García, Tomás García Jr., Venture Partner Lucas Müller, Yejin Park.
- **Journeys it appears in (count=15):** j04-dv-survivor-shelter-mode, j06-press-source-securedrop-class, j07-deceased-user-inheritance-handoff, j11-disaster-zone-offline-first-sync, j17-activist-dissident-high-risk-mode, j38-b2b-e-signing-contract, j39-b2b-meeting-with-transcription, j45-healthcare-patient-portal-records, j102-raw-material-purchase-with-quality-attestation, j105-dispute-cross-tenant-arbitration, j123-multi-tenant-coordinated-product-launch, j125-marketplace-acquires-supplier-tenant-merger, j127-dual-tenant-identity-employee-resigns-and-keeps-personal, j128-auditor-personal-side-uses-workflow-studio-for-family-taxes, j129-court-warrant-pierces-personal-tenant-with-judicial-oversight.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 15.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/drive/PRD.md`.

### §4.38 notes — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** markdown + tags + journaling.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Notion + Obsidian + Apple Notes.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=8):** Captain Chen, Carlos Martinez, Diana Reyes, Hiroshi Tanaka, Officer Rodriguez, Sarah Kim, Tax Analyst Ji-Sung Park, Yejin Park.
- **Journeys it appears in (count=8):** j01-emergency-911-dispatch, j07-deceased-user-inheritance-handoff, j11-disaster-zone-offline-first-sync, j39-b2b-meeting-with-transcription, j43-healthcare-nurse-patient-handoff, j44-healthcare-telemedicine-consultation, j45-healthcare-patient-portal-records, j128-auditor-personal-side-uses-workflow-studio-for-family-taxes.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 8.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/notes/PRD.md`.

### §4.39 forms — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** form-schema + submission + signed-receipt.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Typeform + Google Forms + Tally.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/forms/PRD.md`.

### §4.40 docs — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** rich-text + comments + suggested-edits.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Google Docs + Notion.
- **Center-of-gravity rank (per coverage-matrix §14):** 16.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/docs/PRD.md`.

### §4.41 sheets — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** tabular + formula + collaboration.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Google Sheets + Airtable.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/sheets/PRD.md`.

### §4.42 slides — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** deck + presenter + theming.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Google Slides + Pitch + Gamma.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/slides/PRD.md`.

### §4.43 whiteboard — product (Tier-3; cell-tier T1)

- **Owns (bounded contexts):** shapes + freehand + presence.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-3; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Miro + FigJam + Mural.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/whiteboard/PRD.md`.

### §4.44 design-collaboration — product (Tier-3; cell-tier T1)

- **Owns (bounded contexts):** frame + component + version-history.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-3; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Figma + Sketch.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/design-collaboration/PRD.md`.

### §4.45 recordings — product (Tier-3; cell-tier T1)

- **Owns (bounded contexts):** AV + transcript + captions.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-3; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Loom + Descript.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=1):** j39-b2b-meeting-with-transcription.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 1.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/recordings/PRD.md`.

### §4.46 sites — product (Tier-3; cell-tier T1)

- **Owns (bounded contexts):** page + theme + publish-pipeline.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-3; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Webflow + Wix + Squarespace.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/sites/PRD.md`.

### §4.47 community — product (Tier-1; cell-tier T0)

- **Owns (bounded contexts):** channels + LinkedIn-mode + Handshake-mode + Blind-mode.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-1; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** LinkedIn + Discord + Handshake + Blind.
- **Center-of-gravity rank (per coverage-matrix §14):** 5.
- **Personas who depend on it (count=49):** Ahmad Hassan, Aiyana Singh, Anya Mironova, Apprentice Jakob Bauer, Board director Patrick O'Reilly, Business Analyst Aditya Verma, CCO Naveen Iyer, CFO Helena Brandt, CISO Yuki Park, Captain Olufemi, Channel Partner Tomas Pieter, Cleaning Supervisor Tomáš Horák, Co-op Student Liam Murphy, Compliance Analyst Yui Hayashi, Compliance Officer Tunde Bello, Customer Success Manager Sofia Rezende, Data Analyst Felipe Andrade, Data Scientist Yu Chen, Devon Williams, Diana Reyes, External Regulator Inspector Sergei Petrov, Father Lopez, IR Manager Lev Kahn, IR Specialist (unnamed), Intern Manager Felicia Adamou, Investment Banker Yuna Ahn, Investor/LP Aanya Kapoor, Legal Counsel Anika Mehta, Marcus Chen, Maria Santos, Marketing Specialist Riya Sharma, Medical Resident Dr. Sun-Mi Kim, Ms. Patel, Officer Rodriguez, Outside Counsel Wei-Yi Chen, Public Affairs Director Carlos Mendez, Receptionist Daria Volkova, Returning Intern Jia Han, Sam Okafor, Sarah Kim, Security Analyst Anna Petrova, Summer Intern Priscilla Sharma, Sustainability Officer Aiko Brown, Tomás García, Tomás García Jr., Training Specialist Mehmet Yilmaz, Venture Partner Lucas Müller, Wealth Manager Aamir Khan, Yejin Park.
- **Journeys it appears in (count=20):** j05-whistleblower-anonymous-ethics-report, j06-press-source-securedrop-class, j15-bug-bounty-researcher-submission, j17-activist-dissident-high-risk-mode, j18-child-safety-mandatory-reporter, j49-sidebusiness-customer-support-omnichannel, j108-supplier-rating-and-marketplace-discovery, j109-construction-co-hires-freelance-specialist, j110-traveling-nurse-multi-employer-roster, j111-staffing-agency-as-tenant-facilitator, j112-tenant-to-tenant-rfq-and-bid, j113-cross-tenant-internship-from-handshake, j116-plugin-marketplace-developer-publishes-and-monetizes, j119-invoice-financing-marketplace, j129-court-warrant-pierces-personal-tenant-with-judicial-oversight, j130-auditor-receives-bribery-attempt-via-personal-messenger, j138-corporate-audit-fraud-investigation-via-pattern-detection, j148-supply-chain-circular-economy-electronics-recycling, j149-gig-economy-multi-platform-worker, j150-creator-economy-shorts-creator-monetization-stack.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 20.
- **Per-pack overlays applicable:** DSA + UK-AADC + LinkedIn-style-pack.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/community/PRD.md`.

### §4.48 social — product (Tier-3; cell-tier T1)

- **Owns (bounded contexts):** feed + ranking + moderation.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-3; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** X + Threads + Bluesky.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/social/PRD.md`.

### §4.49 shorts — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** video + ranking + KOSA-tier.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** TikTok + YouTube Shorts + Reels.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=3):** Data Analyst Felipe Andrade, Data Scientist Yu Chen, Marketing Specialist Riya Sharma.
- **Journeys it appears in (count=1):** j150-creator-economy-shorts-creator-monetization-stack.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 1.
- **Per-pack overlays applicable:** COPPA + KOSA + EU-DSA.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/shorts/PRD.md`.

### §4.50 marketplace — product (Tier-1; cell-tier T0)

- **Owns (bounded contexts):** listing + checkout + ratings + dispute.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-1; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Stripe Marketplace + Shopify.
- **Center-of-gravity rank (per coverage-matrix §14):** 12.
- **Personas who depend on it (count=16):** Anya Mironova, CCO Naveen Iyer, CISO Yuki Park, Cafeteria Manager Soyeon Kim, Captain Olufemi, Diana Reyes, External Regulator Inspector Sergei Petrov, Legal Counsel Anika Mehta, Marcus Chen, Outside Counsel Wei-Yi Chen, Procurement Manager Wei Liu, Procurement Specialist Beata Kowalski, Public Affairs Director Carlos Mendez, Tax Analyst Ji-Sung Park, Tomás García, Tomás García Jr..
- **Journeys it appears in (count=8):** j101-multi-tier-supply-chain-formation, j102-raw-material-purchase-with-quality-attestation, j103-just-in-time-procurement-automation, j107-supply-chain-disruption-and-failover, j108-supplier-rating-and-marketplace-discovery, j112-tenant-to-tenant-rfq-and-bid, j128-auditor-personal-side-uses-workflow-studio-for-family-taxes, j129-court-warrant-pierces-personal-tenant-with-judicial-oversight.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 8.
- **Per-pack overlays applicable:** PCI-DSS + per-jurisdiction-consumer-protection.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/marketplace/PRD.md`.

### §4.51 payments — product (Tier-1; cell-tier T0)

- **Owns (bounded contexts):** intent + capture + refund + tax + receipt.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-1; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Stripe + Adyen + Square.
- **Center-of-gravity rank (per coverage-matrix §14):** 7.
- **Personas who depend on it (count=88):** Accountant Ravi Iyer, Ahmad Hassan, Aiyana Singh, Anya Mironova, Apprentice Jakob Bauer, Bank Compliance Officer Rishi Bhattacharya, Bank Ops Officer Olamide Adebanjo, Bank Risk Manager Anders Pedersen, Banker (external) Hideki Watanabe, Board Secretary Florence Akinsanya, Board director Patrick O'Reilly, Business Analyst Aditya Verma, CCO Naveen Iyer, CEO Aoki Tanaka, CFO Helena Brandt, CISO Yuki Park, CMO Felix Ng, COO Akira Watanabe, CS-IC Lin Chen, CTO Diego Vargas, Cafeteria Manager Soyeon Kim, Channel Partner Tomas Pieter, Cleaning Supervisor Tomáš Horák, Co-op Student Liam Murphy, Commercial Banker Frederik Hartmann, Communications Specialist Charlotte Dubois, Compliance Analyst Yui Hayashi, Compliance Officer Tunde Bello, Corporate Relations Director Soo-Yeon Han, Credit Analyst Hina Mori, Customer Champion Akemi Sato, Customer Success Manager Sofia Rezende, Data Analyst Felipe Andrade, Data Scientist Yu Chen, DevOps Engineer Olukayode Adejumo, DevOps Manager Pavel Korsak, Devon Williams, Diana Reyes, Engineering Manager Aisha Ali, Executive Assistant Olivia Reyes, External Regulator Inspector Sergei Petrov, Finance Director Mei-Ling Wu, Financial Analyst Wendy Lee, Hiroshi Tanaka, IR Manager Lev Kahn, IR Specialist (unnamed), IT Manager Jamie O'Connor, Intern Manager Felicia Adamou, Internal Comms Lead Ji-Ho Yoon, Investment Banker Yuna Ahn, Investor/LP Aanya Kapoor, Legal Counsel Anika Mehta, Marcus Chen, Maria Santos, Marketing Manager Olu Adeyemi, Marketing Specialist Riya Sharma, Medical Resident Dr. Sun-Mi Kim, Officer Rodriguez, Outside Counsel Wei-Yi Chen, PR Firm Beatriz Fernandez, PR Manager Helena Sato, Procurement Manager Wei Liu, Procurement Specialist Beata Kowalski, Product Designer Akihiro Sato, Product Manager Lily Chang, Project Manager Soo-Jin Park, Public Affairs Director Carlos Mendez, Receptionist Daria Volkova, Retail Banker Sebastián Vega, Returning Intern Jia Han, SWE Hugo Tanaka, Sales AE Maya Lindqvist, Sales Manager Anthony Costa, Sam Okafor, Sarah Kim, Security Analyst Anna Petrova, Summer Intern Priscilla Sharma, Support Rep Nadia Hassani, Sustainability Officer Aiko Brown, Tax Analyst Ji-Sung Park, Tomás García, Tomás García Jr., Trader Mei Lin, Training Specialist Mehmet Yilmaz, Treasury Ops Sven Eriksson, Venture Partner Lucas Müller, Wealth Manager Aamir Khan, Yejin Park.
- **Journeys it appears in (count=35):** j07-deceased-user-inheritance-handoff, j08-elder-financial-abuse-detection, j10-account-takeover-SIM-swap-detected, j36-b2b-workflow-engine-approval-cascade, j37-b2b-clocking-and-attendance, j40-b2b-marketplace-vendor-billing, j47-healthcare-billing-and-insurance, j48-sidebusiness-stripe-tax-and-invoicing, j50-sidebusiness-employee-hires-first-helper, j101-multi-tier-supply-chain-formation, j102-raw-material-purchase-with-quality-attestation, j103-just-in-time-procurement-automation, j105-dispute-cross-tenant-arbitration, j106-multi-currency-cross-border-payment, j109-construction-co-hires-freelance-specialist, j110-traveling-nurse-multi-employer-roster, j111-staffing-agency-as-tenant-facilitator, j112-tenant-to-tenant-rfq-and-bid, j113-cross-tenant-internship-from-handshake, j114-employee-secondment-cross-tenant, j115-saas-vendor-sells-api-to-multiple-tenant-customers, j116-plugin-marketplace-developer-publishes-and-monetizes, j117-api-customer-tenant-incident-response, j119-invoice-financing-marketplace, j120-tenant-treasury-multi-currency-fx-hedge, j121-business-loan-application-from-bank-tenant, j122-vendor-payment-batch-with-tax-withholding, j123-multi-tenant-coordinated-product-launch, j128-auditor-personal-side-uses-workflow-studio-for-family-taxes, j129-court-warrant-pierces-personal-tenant-with-judicial-oversight, j137-corporate-internal-audit-sox-controls-test, j138-corporate-audit-fraud-investigation-via-pattern-detection, j148-supply-chain-circular-economy-electronics-recycling, j149-gig-economy-multi-platform-worker, j150-creator-economy-shorts-creator-monetization-stack.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 35.
- **Per-pack overlays applicable:** PCI-DSS-L1-v4 + per-jurisdiction-tax.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/payments/PRD.md`.

### §4.52 finops-portal — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** billing + invoice + cost-attribution.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Stripe Billing + Chargebee.
- **Center-of-gravity rank (per coverage-matrix §14):** 17.
- **Personas who depend on it (count=46):** Accountant Ravi Iyer, Aiyana Singh, Bank Compliance Officer Rishi Bhattacharya, Bank Ops Officer Olamide Adebanjo, Bank Risk Manager Anders Pedersen, Banker (external) Hideki Watanabe, Board director Patrick O'Reilly, Business Analyst Aditya Verma, CEO Aoki Tanaka, CFO Helena Brandt, CS-IC Lin Chen, CSO Mira Goldberg, Carlos Martinez, Channel Partner Tomas Pieter, Commercial Banker Frederik Hartmann, Compliance Officer Tunde Bello, Consultant Adekunle Adebayo, Corp Dev Senior Analyst Saanvi Mehta, Credit Analyst Hina Mori, Customer Champion Akemi Sato, Customer Success Manager Sofia Rezende, Data Analyst Felipe Andrade, Data Scientist Yu Chen, DevOps Engineer Olukayode Adejumo, DevOps Manager Pavel Korsak, Finance Director Mei-Ling Wu, Financial Analyst Wendy Lee, IR Manager Lev Kahn, IR Specialist (unnamed), IT Manager Jamie O'Connor, Investment Banker Yuna Ahn, Investor/LP Aanya Kapoor, Marcus Chen, Marketing Specialist Riya Sharma, Outside Counsel Wei-Yi Chen, Retail Banker Sebastián Vega, Sales AE Maya Lindqvist, Sales Manager Anthony Costa, Sarah Kim, Strategic Advisor Rita Almeida, Support Rep Nadia Hassani, Tax Analyst Ji-Sung Park, Trader Mei Lin, Treasury Ops Sven Eriksson, Venture Partner Lucas Müller, Wealth Manager Aamir Khan.
- **Journeys it appears in (count=11):** j42-b2b-finops-portal-spend-attribution, j48-sidebusiness-stripe-tax-and-invoicing, j115-saas-vendor-sells-api-to-multiple-tenant-customers, j117-api-customer-tenant-incident-response, j119-invoice-financing-marketplace, j120-tenant-treasury-multi-currency-fx-hedge, j121-business-loan-application-from-bank-tenant, j122-vendor-payment-batch-with-tax-withholding, j125-marketplace-acquires-supplier-tenant-merger, j149-gig-economy-multi-platform-worker, j150-creator-economy-shorts-creator-monetization-stack.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 11.
- **Per-pack overlays applicable:** SOX-404 + PCI-DSS.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/finops-portal/PRD.md`.

### §4.53 treasury — product (Tier-3; cell-tier T1)

- **Owns (bounded contexts):** FX + hedge + AP/AR.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-3; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Modern Treasury + Brex Treasury.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/treasury/PRD.md`.

### §4.54 crm — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** account + contact + opportunity + pipeline.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Salesforce + HubSpot + Pipedrive.
- **Center-of-gravity rank (per coverage-matrix §14):** 13.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/crm/PRD.md`.

### §4.55 contact-center — product (Tier-3; cell-tier T1)

- **Owns (bounded contexts):** omnichannel + IVR + agent-desktop.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-3; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Genesys + Five9 + AWS Connect.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/contact-center/PRD.md`.

### §4.56 marketing-automation — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** campaign + segment + content.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Marketo + HubSpot + Braze.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/marketing-automation/PRD.md`.

### §4.57 contract-lifecycle-management — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** contract + clause + e-sign.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Ironclad + DocuSign CLM.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** Attorney-Client-Privilege + per-jurisdiction-contract-law.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/contract-lifecycle-management/PRD.md`.

### §4.58 learning-management — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** course + assignment + transcript.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Cornerstone + Workday Learning.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/learning-management/PRD.md`.

### §4.59 performance-management — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** review + 360 + goal.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Workday + Lattice + 15Five.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/performance-management/PRD.md`.

### §4.60 workplace-integration — substrate (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** HRIS-bridge + role-map + tenant-overlay.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=substrate.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Workday + SuccessFactors + BambooHR.
- **Center-of-gravity rank (per coverage-matrix §14):** 15.
- **Personas who depend on it (count=24):** Ahmad Hassan, Apprentice Jakob Bauer, Bank Compliance Officer Rishi Bhattacharya, Bank Ops Officer Olamide Adebanjo, Bank Risk Manager Anders Pedersen, Banker (external) Hideki Watanabe, Co-op Student Liam Murphy, Commercial Banker Frederik Hartmann, Credit Analyst Hina Mori, Customer Success Manager Sofia Rezende, Devon Williams, Diana Reyes, IR Manager Lev Kahn, Intern Manager Felicia Adamou, Investment Banker Yuna Ahn, Marcus Chen, Maria Santos, Medical Resident Dr. Sun-Mi Kim, Receptionist Daria Volkova, Retail Banker Sebastián Vega, Returning Intern Jia Han, Summer Intern Priscilla Sharma, Training Specialist Mehmet Yilmaz, Yejin Park.
- **Journeys it appears in (count=9):** j37-b2b-clocking-and-attendance, j38-b2b-e-signing-contract, j109-construction-co-hires-freelance-specialist, j110-traveling-nurse-multi-employer-roster, j112-tenant-to-tenant-rfq-and-bid, j113-cross-tenant-internship-from-handshake, j114-employee-secondment-cross-tenant, j121-business-loan-application-from-bank-tenant, j127-dual-tenant-identity-employee-resigns-and-keeps-personal.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 9.
- **Per-pack overlays applicable:** per-jurisdiction-labor + GDPR + DPDP.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/workplace-integration/PRD.md`.

### §4.61 tasks — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** task + project + assignee.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Asana + Linear + Trello.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/tasks/PRD.md`.

### §4.62 warehouse — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** inventory + receipt + put-away + pick + pack + ship.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Manhattan WMS + Blue Yonder.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/warehouse/PRD.md`.

### §4.63 supply-chain-planning — product (Tier-3; cell-tier T1)

- **Owns (bounded contexts):** demand + supply + S&OP.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-3; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Kinaxis + o9 + Blue Yonder.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/supply-chain-planning/PRD.md`.

### §4.64 production-planning — product (Tier-3; cell-tier T1)

- **Owns (bounded contexts):** MRP + APS + work-order.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-3; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** SAP MRP + Plex.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/production-planning/PRD.md`.

### §4.65 quality-management — product (Tier-3; cell-tier T1)

- **Owns (bounded contexts):** non-conformance + CAPA + audit.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-3; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** MasterControl + ETQ.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/quality-management/PRD.md`.

### §4.66 plant-maintenance — product (Tier-3; cell-tier T1)

- **Owns (bounded contexts):** PM + MRO + asset.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-3; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** IBM Maximo + Fiix.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/plant-maintenance/PRD.md`.

### §4.67 global-trade — product (Tier-3; cell-tier T1)

- **Owns (bounded contexts):** customs + tariff + classification.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-3; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** SAP GTS + Thomson Reuters ONESOURCE.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/global-trade/PRD.md`.

### §4.68 real-estate — product (Tier-3; cell-tier T1)

- **Owns (bounded contexts):** lease + tenant + maintenance.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-3; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** MRI + Yardi.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/real-estate/PRD.md`.

### §4.69 financial-planning — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** FP&A + budget + forecast + variance.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Anaplan + Pigment + Adaptive.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** per-tenant per-pack via compliance overlay.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/financial-planning/PRD.md`.

### §4.70 healthcare-integration — product (Tier-2; cell-tier T1)

- **Owns (bounded contexts):** FHIR + HL7 + EHR-bridge.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-2; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Epic + Cerner + Redox.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** HIPAA + KR-Medical-Records-Act + APPI.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/healthcare-integration/PRD.md`.

### §4.71 personal-health-tracker — product (Tier-3; cell-tier T1)

- **Owns (bounded contexts):** vital + medication + appointment.
- **Capability tiers exposed (per ADR-0316):** primary=Tier-3; substrate-or-product=product.
- **Hyperscaler benchmark (per enterprise-coverage-matrix §14):** Apple Health + Google Fit.
- **Center-of-gravity rank (per coverage-matrix §14):** >18.
- **Personas who depend on it (count=0):** _(ambient-only; resolves via identity + tenancy + audit-chain trinity)_.
- **Journeys it appears in (count=0):** _(no journey-anchored evidence yet — see §9)_.
- **Per-µservice IP-journey-* slice count (derived from README microservices_touched lists):** 0.
- **Per-pack overlays applicable:** HIPAA + per-state.
- **Critical-path responsibilities (per documentation-rigor.md §3.2.5):** row 1 (default-deny), row 8 (cross-tenant exfil), row 24 (audit-chain seal); plus role-specific rows.
- **PRD citation:** `microservices/personal-health-tracker/PRD.md`.

---

## §5 The Journey-Graph

This section enumerates the journey adjacency edges, strongly-connected clusters, hubs, and leaves derived from the `## Cross-references` blocks in each journey's `README.md`. Edges are unidirectional; bidirectional pairs appear twice (one edge per direction).

### §5.1 Edge enumeration (journey-to-journey adjacency)

Each edge is one row: `source-journey → target-journey | relation`. The relation is one of: `sibling-life-safety`, `surge-variant`, `dual-tenant-mirror`, `migration-pair`, `pack-overlay-pair`, `µservice-overlap (≥4 µservices)`, `persona-bridge (same primary persona)`, `cross-jurisdiction-pair`.

| Source | Target | Relation |
|---|---|---|
| j01 | j02 | sibling-life-safety |
| j01 | j03 | sibling-life-safety |
| j01 | j04 | sibling-life-safety |
| j1 | j4 | µservice-overlap (5 shared) |
| j01 | j09 | account-recovery-link |
| j01 | j12 | surge-variant |
| j01 | j13 | cross-jurisdiction-pair |
| j02 | j12 | surge-variant |
| j02 | j18 | sibling-mandatory-reporter |
| j03 | j18 | sibling-minor-protection |
| j04 | j17 | sibling-high-risk-mode |
| j05 | j06 | sibling-secure-source |
| j05 | j15 | whistleblower-vs-research |
| j07 | j09 | account-lifecycle-pair |
| j09 | j10 | account-recovery-vs-ATO |
| j100 | j101 | onboarding-to-supply-chain |
| j101 | j102 | supply-chain-step-pair |
| j101 | j104 | µservice-overlap (6 shared) |
| j101 | j105 | µservice-overlap (5 shared) |
| j102 | j103 | µservice-overlap (5 shared) |
| j102 | j103 | supply-chain-step-pair |
| j103 | j104 | supply-chain-step-pair |
| j103 | j107 | µservice-overlap (5 shared) |
| j104 | j105 | supply-chain-dispute-pair |
| j106 | j120 | fx-and-payments-pair |
| j109 | j112 | µservice-overlap (5 shared) |
| j113 | j114 | intern-to-secondment-pair |
| j114 | j121 | µservice-overlap (5 shared) |
| j118 | j125 | µservice-overlap (5 shared) |
| j119 | j121 | financing-pair |
| j123 | j127 | µservice-overlap (5 shared) |
| j123 | j128 | µservice-overlap (5 shared) |
| j123 | j129 | µservice-overlap (6 shared) |
| j124 | j127 | µservice-overlap (5 shared) |
| j125 | j118 | merger-to-ontology-bridge |
| j125 | j126 | µservice-overlap (5 shared) |
| j125 | j127 | µservice-overlap (5 shared) |
| j125 | j129 | µservice-overlap (6 shared) |
| j125 | j131 | µservice-overlap (5 shared) |
| j126 | j127 | µservice-overlap (8 shared) |
| j126 | j127 | auditor-to-employee-pair |
| j126 | j129 | µservice-overlap (8 shared) |
| j126 | j129 | auditor-warrant-pair |
| j126 | j130 | µservice-overlap (7 shared) |
| j126 | j131 | µservice-overlap (7 shared) |
| j126 | j131 | cross-jurisdiction-pair |
| j126 | j137 | µservice-overlap (6 shared) |
| j127 | j128 | µservice-overlap (5 shared) |
| j127 | j129 | µservice-overlap (8 shared) |
| j127 | j130 | µservice-overlap (6 shared) |
| j127 | j131 | µservice-overlap (6 shared) |
| j127 | j137 | µservice-overlap (5 shared) |
| j127 | j142 | dual-tenant-mirror |
| j128 | j129 | µservice-overlap (6 shared) |
| j128 | j137 | µservice-overlap (5 shared) |
| j129 | j130 | µservice-overlap (8 shared) |
| j129 | j131 | µservice-overlap (6 shared) |
| j129 | j137 | µservice-overlap (6 shared) |
| j130 | j131 | µservice-overlap (5 shared) |
| j130 | j137 | µservice-overlap (5 shared) |
| j132 | j133 | hire-then-layoff-pair |
| j133 | j142 | dual-tenant-mirror |
| j133 | j143 | layoff-migration-pair |
| j134 | j145 | staffing-to-application-pair |
| j135 | j141 | harassment-personal-boundary-pair |
| j137 | j138 | sox-to-fraud-pair |
| j138 | j139 | fraud-to-permit-misuse-pair |
| j139 | j140 | permit-misuse-to-DLP-pair |
| j140 | j141 | DLP-to-personal-boundary-pair |
| j142 | j143 | layoff-day-zero-to-import-pair |
| j143 | j144 | import-to-job-search-pair |
| j144 | j145 | job-search-to-application-pair |
| j145 | j146 | application-to-marketplace-pair |
| j146 | j147 | marketplace-to-cohort-pair |
| j148 | j101 | circular-economy-to-supply-chain-pair |
| j149 | j115 | gig-to-saas-vendor-pair |
| j150 | j109 | creator-economy-to-freelance-pair |

**Total journey-graph edges enumerated:** 77.

### §5.2 Journey hubs (most-referenced sources)

Top journey hubs by combined degree (sources + targets):

| Rank | Journey | out-degree | in-degree | total | hub-class |
|---:|---|---:|---:|---:|---|
| 1 | j127 | 6 | 5 | 11 | super-hub |
| 2 | j129 | 3 | 6 | 9 | super-hub |
| 3 | j126 | 8 | 1 | 9 | super-hub |
| 4 | j137 | 1 | 5 | 6 | super-hub |
| 5 | j01 | 6 | 0 | 6 | super-hub |
| 6 | j125 | 5 | 1 | 6 | super-hub |
| 7 | j131 | 0 | 6 | 6 | super-hub |
| 8 | j130 | 2 | 3 | 5 | hub |
| 9 | j101 | 3 | 2 | 5 | hub |
| 10 | j128 | 2 | 2 | 4 | hub |
| 11 | j103 | 2 | 2 | 4 | hub |
| 12 | j02 | 2 | 1 | 3 | minor-hub |
| 13 | j123 | 3 | 0 | 3 | minor-hub |
| 14 | j133 | 2 | 1 | 3 | minor-hub |
| 15 | j104 | 1 | 2 | 3 | minor-hub |
| 16 | j145 | 1 | 2 | 3 | minor-hub |
| 17 | j09 | 1 | 2 | 3 | minor-hub |
| 18 | j102 | 2 | 1 | 3 | minor-hub |
| 19 | j143 | 1 | 2 | 3 | minor-hub |
| 20 | j142 | 1 | 2 | 3 | minor-hub |

### §5.3 Strongly-connected journey clusters

By manual inspection of the curated edges + µservice-overlap edges:

1. **Life-safety cluster** (j01, j02, j03, j04, j09, j10, j12, j18) — interlinked via emergency-services bypass (ADR-0298), account-recovery (ADR-0299), and minor-protection (ADR-0292).
2. **Supply-chain cluster** (j101, j102, j103, j104, j105, j106, j107, j108, j148) — interlinked via marketplace + payments + workflow-engine + connect spine.
3. **Auditor cluster** (j126, j127, j128, j129, j130, j131) — Diana Reyes dual-tenant; ADR-0311 + ADR-0312 anchored.
4. **HR cluster** (j132, j133, j134, j135, j136) — Priya Krishnan hire/layoff/recruit/harassment/benefits.
5. **Internal-audit cluster** (j137, j138, j139, j140, j141) — Sam Okafor SOX + fraud + permit-misuse + DLP + personal-boundary.
6. **Laid-off cohort cluster** (j142, j143, j144, j145, j146, j147) — Chris Volkov layoff to re-employment pipeline.

### §5.4 Journey leaves (terminal — no outbound references)

Identified leaf journeys (count=114):

- j2, j3, j4, j5, j6, j7, j8, j9, j10, j11
- j12, j13, j14, j15, j16, j17, j18, j19, j20, j21
- j22, j23, j24, j25, j26, j27, j28, j29, j30, j31
- j32, j33, j34, j35, j36, j37, j38, j39, j40, j41
- j42, j43, j44, j45, j46, j47, j48, j49, j50, j51
- j52, j53, j54, j55, j56, j57, j58, j59, j60, j61
- j62, j63, j64, j65, j66, j67, j68, j69, j70, j71
- j72, j73, j74, j75, j76, j77, j78, j79, j80, j81
- j82, j83, j84, j85, j86, j87, j88, j89, j90, j91
- j92, j93, j94, j95, j96, j97, j98, j99, j105, j107
- j108, j110, j111, j112, j115, j116, j117, j120, j121, j122
- j131, j136, j141, j147

Leaf journeys are candidates for §10's recommended j151+ outbound linking.

---

## §6 The Persona-Graph

This section enumerates cross-context bridges from MASTER-ROSTER §4 and the persona-archetype clusters from §5–§7 of the same document. Edges are `personaA ↔ personaB | relation` where the relation captures the persona's own §1.1 doctrine ('same human, multiple contexts').

### §6.1 Cross-context bridge enumeration (same-human edges)

| Persona A | Persona B | Same-human? | Bridge type |
|---|---|---|---|
| Yejin-as-nurse | Yejin-as-parent | yes | clinical+family |
| Yejin-as-nurse | Yejin-as-side-business-owner | yes | clinical+commerce |
| Yejin-as-nurse | Yejin-as-patient | yes | provider+patient |
| Yejin-as-nurse | Yejin-as-consumer | yes | clinical+consumer |
| Yejin-as-parent | Yejin-as-side-business-owner | yes | family+commerce |
| Yejin-as-parent | Yejin-as-patient | yes | family+patient |
| Yejin-as-parent | Yejin-as-consumer | yes | family+consumer |
| Yejin-as-side-business-owner | Yejin-as-patient | yes | commerce+patient |
| Yejin-as-side-business-owner | Yejin-as-consumer | yes | commerce+consumer |
| Yejin-as-patient | Yejin-as-consumer | yes | patient+consumer |
| Marcus-as-CEO | Marcus-as-husband | yes | executive+family |
| Marcus-as-CEO | Marcus-as-father | yes | executive+family |
| Marcus-as-husband | Marcus-as-father | yes | family+family |
| Marcus-as-CEO | Marcus-as-board-director-elsewhere | yes | executive+board-elsewhere |
| Diana-as-auditor | Diana-as-consumer | yes | 3PAO+consumer |
| Diana-as-auditor | Diana-as-family-parent | yes | 3PAO+family |
| Chris-pre-layoff | Chris-post-layoff | yes | employed+job-seeker |
| Chris-post-layoff | Chris-as-family-provider | yes | job-seeker+family |
| Aiyana-at-work | Aiyana-as-blogger | yes | employee+creator |
| Aiyana-at-work | Aiyana-as-parent | yes | employee+family |
| Tomás-as-owner | Tomás-as-cook | yes | admin+employee |
| Tomás-as-owner | Tomás-as-father | yes | admin+family |
| Hiroshi-as-grandfather | Hiroshi-as-photographer | yes | family+creator |
| Hiroshi-as-photographer | Hiroshi-as-patient | yes | creator+patient |
| Anya-as-journalist | Anya-as-parent | yes | high-risk+family |
| Anya-as-journalist | Anya-as-activist | yes | high-risk+high-risk |
| Aoki-as-CEO | Aoki-as-board-director-elsewhere | yes | executive+board-elsewhere |
| Aoki-as-CEO | Aoki-as-parent | yes | executive+family |
| Helena-as-CFO | Helena-as-charity-board-director | yes | executive+board-elsewhere |
| Yuki-as-CISO | Yuki-as-incident-response-volunteer | yes | security-officer+volunteer |
| Linda-as-CHRO | Linda-as-mentor-board | yes | executive+mentor |
| Carlos-at-work | Carlos-as-father | yes | field+family |
| Sarah-as-driver | Sarah-as-side-hustler | yes | field+admin |
| Officer-Rodriguez-on-patrol | Officer-Rodriguez-as-family | yes | LE+family |
| Captain-Chen-as-pilot | Captain-Chen-as-father | yes | field+family |
| Dr.Tanaka-as-surgeon | Dr.Tanaka-as-father | yes | clinical+family |
| Ms.Patel-as-teacher | Ms.Patel-as-mother | yes | EDU+family |
| Father-Lopez-as-priest | Father-Lopez-as-counselor | yes | religious+counselor |
| Yuna-as-IB | Yuna-as-MBA-applicant | yes | bank-internal+consumer |
| Mei-Lin-as-trader | Mei-Lin-as-marathon-runner | yes | bank-internal+consumer |
| Priscilla-as-intern | Priscilla-as-undergrad | yes | intern+student |
| Sun-Mi-as-resident | Sun-Mi-as-grad-school-applicant | yes | resident+student |
| Sergei-as-regulator | Sergei-as-private-citizen | yes | regulator+consumer |
| Wei-Yi-as-counsel-for-A | Wei-Yi-as-counsel-for-B | yes | counsel-vs-counsel-strict-isolation |
| Wei-Yi-as-counsel-for-A | Wei-Yi-as-consumer | yes | counsel+consumer |
| Aoife-as-HR-Specialist | Aoife-as-Benefits-Specialist | yes | HR+HR-sub-role |
| O'Reilly-on-Board-A | O'Reilly-on-Board-B | yes | board-vs-board-strict-isolation |
| O'Reilly-on-Board-A | O'Reilly-on-Board-C | yes | board-vs-board-strict-isolation |
| O'Reilly-on-Board-B | O'Reilly-on-Board-C | yes | board-vs-board-strict-isolation |
| Tomás-Jr-as-farmer | Tomás-Jr-as-cooperative-board | yes | admin+board-elsewhere |
| Tomás-Jr-as-farmer | son-of-Tomás-García | yes | commerce+family |
| Hideki-as-banker-for-A | Hideki-as-banker-for-B | yes | banker-vs-banker-strict-isolation |
| Dimitri-as-auditor-for-A | Dimitri-as-auditor-for-B | yes | auditor-vs-auditor-strict-isolation |
| Hyo-Jin-as-auditor-A | Hyo-Jin-as-auditor-B | yes | auditor-vs-auditor-strict-isolation |
| Frederik-as-banker-for-A | Frederik-as-banker-for-B | yes | banker-vs-banker-strict-isolation |
| Lev-as-IR | Lev-as-LP-of-fund | yes | employee+investor |
| Aanya-as-LP-A | Aanya-as-LP-B | yes | investor-vs-investor |
| Lucas-as-VC | Lucas-as-LP-of-other-fund | yes | VC+LP-elsewhere |
| Akira-S-as-wellness | Akira-S-as-yoga-instructor | yes | HR+side-instructor |
| Mehmet-as-trainer | Mehmet-as-Udemy-instructor | yes | HR+side-instructor |
| Bryce-as-plan-admin | Bryce-as-PTA-treasurer | yes | HR+family-volunteer |
| Tomáš-Horák-as-supervisor | Tomáš-Horák-as-cleaning-co-owner | yes | field+admin |
| Aamir-as-WM | Aamir-as-LP-of-fund | yes | bank-internal+investor |
| Sebastián-as-branch-mgr | Sebastián-as-side-tutor | yes | bank-internal+EDU |
| Stephen-as-legal-ops | Stephen-as-paralegal-side | yes | legal-ops+legal-side |
| Jamie-as-IT-manager | Jamie-as-PC-club-organizer | yes | IT+community-organizer |
| Aiko-as-officer | Aiko-as-climate-activist | yes | sustainability+activist |

**Total cross-context bridge edges enumerated:** 67.

### §6.2 Persona archetype clusters

Per MASTER-ROSTER §5 (collar-color) + §6 (workspace) + §7 (skill-tier), the persona graph clusters as follows:

- **executive-cluster** (collar=white; workspace=executive; tier=executive): Marcus Chen; CEO Aoki Tanaka; CFO Helena Brandt; COO Akira Watanabe; CTO Diego Vargas; CHRO Linda Foster; CMO Felix Ng; CCO Naveen Iyer; CISO Yuki Park; CSO Mira Goldberg; Board director Patrick O'Reilly; Board Secretary Florence Akinsanya; Strategic Advisor Rita Almeida; Venture Partner Lucas Müller.
- **functional-IC-cluster** (white-collar, back-office, mid-level): SWE Hugo Tanaka; Sales AE Maya Lindqvist; SDR Kofi Asante; Financial Analyst Wendy Lee; Accountant Ravi Iyer; Tax Analyst Ji-Sung Park; HR Specialist Aoife Murphy; etc.
- **non-office-collar-color-cluster** (blue/pink/gray/gold/green; field/production/clinical/front-office): Carlos Martinez; Sarah Kim; Ahmad Hassan; Maria Santos; Devon Williams; Captain Chen; Officer Rodriguez; Dr. Tanaka; Captain Olufemi; Tomás García Jr.; Father Lopez; Ms. Patel; Coach Park.
- **external-counterparty-cluster** (B2B_EXTERNAL_AUDITOR + B2B_EXTERNAL_COUNSEL + B2B_REGULATOR_EXTERNAL + B2B_CHANNEL_PARTNER): Dimitri Volkov; Hyo-Jin Lee; Jakub Nowak; Wei-Yi Chen; Rita Almeida; Adekunle Adebayo; Sergei Petrov; Tomas Pieter; Beatriz Fernandez.
- **bank-internal-cluster** (B2B_BANK_INTERNAL): Yuna Ahn; Frederik Hartmann; Sebastián Vega; Mei Lin; Aamir Khan; Sven Eriksson; Olamide Adebanjo; Hina Mori; Rishi Bhattacharya; Anders Pedersen; Hideki Watanabe.
- **in-training-cluster** (B2B_APPRENTICE_INTERN + B2B_MEDICAL_RESIDENT): Priscilla Sharma; Liam Murphy; Jia Han; Jakob Bauer; Sun-Mi Kim; Tobias Klein.
- **HR-sub-cluster** (B2B_HR_ADMIN): Priya Krishnan; CHRO Linda Foster; HRBP Jamal Carter; Hina Suzuki; Maya Okoroafor; Aoife Murphy; Nilufer Demir; Margarethe Reinhart; Akira Sato; Bryce Williams.

---

## §7 The Capability-Tier × Microservice Graph (per ADR-0316)

Per ADR-0316, every B2B-leader capability tier composes from existing µservices. The graph below enumerates the composition for the canonical 24 capability tiers; each tier lists its **primary** µservices (own the core surface) and **supporting** µservices (compose into the surface).

| Capability tier | Primary µservices | Supporting µservices | Hyperscaler-equivalent |
|---|---|---|---|
| identity-and-access | identity, policy-engine, audit-chain | tenancy, consent-graph, governance | Okta + Auth0 + AWS IAM + Cedar (AWS Verified Permissions) + OPA |
| dual-tenant-boundary | tenancy, policy-engine, identity | audit-chain, governance, compliance | AWS Organizations + Azure Tenants + Cedar (AWS Verified Permissions) + OPA |
| payments-and-finops | payments, finops-portal, treasury | compliance, audit-chain, marketplace | Stripe + Adyen + Square + Stripe Billing + Chargebee |
| messaging-e2ee | messenger, mail, comms-email | identity, policy-engine, audit-chain | Signal + WhatsApp (MLS RFC 9420) + Gmail + Outlook + Hey |
| collaboration-suite | docs, sheets, slides, whiteboard, design-collaboration | drive, notes, tasks, calendar, meet | Google Docs + Notion + Google Sheets + Airtable |
| creator-economy-shorts | shorts, social, community | payments, marketplace, intelligence, ontology, finops-portal | TikTok + YouTube Shorts + Reels + X + Threads + Bluesky |
| marketplace-and-commerce | marketplace, payments, finops-portal | community, ontology, connect, audit-chain | Stripe Marketplace + Shopify + Stripe + Adyen + Square |
| supply-chain-network | marketplace, ontology, workflow-engine, connect | compliance, audit-chain, payments, warehouse, supply-chain-planning | Stripe Marketplace + Shopify + Palantir Foundry Ontology |
| workforce-and-HR | workplace-integration, learning-management, performance-management | forms, payments, calendar, mail, community | Workday + SuccessFactors + BambooHR + Cornerstone + Workday Learning |
| compliance-and-pack | compliance, policy-engine, audit-chain | cell, consent-graph, governance, cloud-iac | Drata + Vanta + SecureFrame + Cedar (AWS Verified Permissions) + OPA |
| observability-and-SRE | observability, ops-dashboard-control-center, incident-management | detection, cloud-k8s, cloud-iac | Datadog + Honeycomb + OpenTelemetry + Datadog Dashboards + Grafana |
| intelligence-substrate | intelligence, ontology, foundry | workflow-engine, developer-sdk, data-warehouse, analytics | OpenAI + Anthropic + Bedrock + Vertex + Palantir Foundry Ontology |
| foundry-and-agents | foundry, intelligence, plugin-app-store | developer-sdk, workflow-engine, ontology | Palantir Foundry + AWS Bedrock Agents + OpenAI + Anthropic + Bedrock + Vertex |
| ERP-finance-and-FP&A | financial-planning, payments, finops-portal | data-warehouse, analytics, compliance | Anaplan + Pigment + Adaptive + Stripe + Adyen + Square |
| ERP-supply-chain | warehouse, supply-chain-planning, production-planning | ontology, workflow-engine, marketplace | Manhattan WMS + Blue Yonder + Kinaxis + o9 + Blue Yonder |
| ERP-manufacturing-and-quality | production-planning, quality-management, plant-maintenance | ontology, workflow-engine, data-warehouse | SAP MRP + Plex + MasterControl + ETQ |
| CRM-and-CX | crm, marketing-automation, contact-center | community, mail, ontology, analytics | Salesforce + HubSpot + Pipedrive + Marketo + HubSpot + Braze |
| ITSM | itsm, incident-management, workflow-engine | observability, governance, audit-chain | ServiceNow + Jira Service Mgmt + PagerDuty + Opsgenie + Incident.io |
| CLM-and-procurement | contract-lifecycle-management, workflow-engine, payments | audit-chain, compliance, drive | Ironclad + DocuSign CLM + n8n + Temporal + Camunda + Airflow |
| personal-day-to-day | mail, calendar, drive, notes, messenger | payments, community, marketplace | Gmail + Outlook + Hey + Google Calendar + Outlook + Cron |
| healthcare-clinical | healthcare-integration, personal-health-tracker, drive | consent-graph, audit-chain, compliance, ontology | Epic + Cerner + Redox + Apple Health + Google Fit |
| law-enforcement-and-emergency | incident-management, messenger, audit-chain | identity, policy-engine, observability, compliance | PagerDuty + Opsgenie + Incident.io + Signal + WhatsApp (MLS RFC 9420) |
| workflow-studio-and-automation | workflow-studio, workflow-engine, intelligence | connect, developer-sdk, ontology, plugin-app-store | n8n Editor + Zapier + Power Automate + n8n + Temporal + Camunda + Airflow |
| global-trade-and-customs | global-trade, compliance, payments | ontology, workflow-engine, audit-chain | SAP GTS + Thomson Reuters ONESOURCE + Drata + Vanta + SecureFrame |

**Total capability-tier × µservice edges:** 164.

---

## §8 Critical-Path × Persona × Journey × Microservice Mapping (per documentation-rigor.md §3.2.5)

The 30 critical-path rows in documentation-rigor.md §3.2.5 are mapped here against the personas that trigger them, the journeys that cover them, the µservices that participate, and the Cedar policies that gate them. Rows are reproduced in compressed form; the full row text lives in `docs/standards/documentation-rigor.md` §3.2.5.

| Row | Description | Personas triggering | Journeys covering | µservices participating | Cedar policy gate |
|---:|---|---|---|---|---|
| 1 | Emergency services bypass (life-safety) | Yejin Park; Officer Rodriguez; Dr. Tanaka; Captain Chen; Father Lopez; emergency-responder personas | j01,j02,j03,j04,j11,j12,j18 | api-gateway, messenger, identity, cell, audit-chain, compliance, workflow-engine, observability | emergency-bypass cedar fragment (ADR-0298) |
| 2 | Code-blue PHI break-glass with audit | Yejin Park-as-nurse; Dr. Tanaka; Sun-Mi Kim; Tobias Klein | j02 | healthcare-integration, drive, audit-chain, identity, policy-engine, consent-graph | break-glass cedar fragment (ADR-0292) |
| 3 | 988-class crisis-line minor self-report | Jordan Lee (minor); Ms. Patel (mandatory-reporter); Father Lopez (privileged-comms) | j03,j18 | messenger, community, identity, policy-engine, compliance, audit-chain | minor cedar fragment (ADR-0292) |
| 4 | DV survivor shelter mode | B2C_CONSUMER personas + HIGH_RISK_USER + Anya Mironova | j04,j17 | identity, messenger, mail, drive, consent-graph, observability | shelter-mode cedar fragment |
| 5 | Whistleblower anonymous ethics report | Tunde Bello-as-employee; Anya Mironova; B2C_CONSUMER employees | j05 | community, audit-chain, observability, identity | whistleblower cedar fragment |
| 6 | Press-source SecureDrop-class | Anya Mironova; Father Lopez (counsel) | j06 | community, drive, messenger, audit-chain | press-source cedar fragment |
| 7 | Deceased-user inheritance handoff | Hiroshi Tanaka (terminal); next-of-kin personas | j07 | identity, mail, drive, notes, payments, audit-chain | estate cedar fragment (ADR-0299) |
| 8 | Elder financial-abuse detection | Hiroshi Tanaka; Retail Banker Sebastián; Wealth Manager Aamir | j08 | payments, identity, messenger, workflow-engine | elder-abuse cedar fragment |
| 9 | Account recovery phishing-resistant | all B2C personas; CISO Yuki Park | j09 | identity, messenger, mail | passkey recovery cedar fragment (ADR-0299) |
| 10 | Account takeover SIM-swap detected | all B2C personas + bank-internal | j10 | identity, messenger, payments, observability | ATO cedar fragment |
| 11 | Disaster-zone offline-first sync | Yejin Park; Officer Rodriguez; Captain Chen | j11 | identity, drive, messenger, observability, cell | offline cedar fragment |
| 12 | Mass-casualty 10x traffic | Yejin Park; Dr. Tanaka; emergency-responders | j12 | api-gateway, cell, observability, messenger, intelligence | surge cedar fragment |
| 13 | Cross-jurisdiction EU↔CLOUD-Act conflict | Marcus Chen; Anya Mironova; cross-border personas | j13 | compliance, governance, identity, audit-chain | lawful-authority cedar fragment (ADR-0304) |
| 14 | Delegated LLM agent acting for principal | Yejin Park; Aiyana Singh; Marcus Chen | j14 | intelligence, foundry, identity, policy-engine, audit-chain | delegation cedar fragment |
| 15 | Bug-bounty researcher submission | CISO Yuki Park; Anna Petrova; SECURITY_RESEARCHER | j15 | community, identity, audit-chain | bug-bounty cedar fragment |
| 16 | Disability accommodation voice-only signup | Hiroshi Tanaka-as-grandfather; all B2C personas | j16 | identity, application, mail | accessibility cedar fragment |
| 17 | Activist/dissident high-risk mode | Anya Mironova; HIGH_RISK_USER | j17 | messenger, identity, drive, observability | high-risk cedar fragment |
| 18 | Child-safety mandatory reporter | Ms. Patel; Father Lopez; Officer Rodriguez | j18 | messenger, community, audit-chain, compliance | mandatory-reporter cedar fragment |
| 19 | Tenant break-glass locked-out admin | Marcus Chen; CTO Diego Vargas; CISO Yuki Park | j19 | identity, tenancy, audit-chain, governance | tenant break-glass cedar fragment |
| 20 | Data-residency violation detection | Marcus Chen; Sergei Petrov; CISO Yuki Park | j20 | observability, compliance, governance, cell | residency cedar fragment |
| 21 | Personal signup passkey-first DM | all B2C personas at signup | j21 | identity, messenger | passkey-signup cedar fragment (ADR-0188) |
| 22 | Mass-casualty surge holding | Yejin Park; Dr. Tanaka; Officer Rodriguez | j12 | cell, observability, api-gateway, intelligence | surge cedar fragment |
| 23 | Cross-tenant data sharing via ontology projection | Marcus Chen; Tomás García Jr.; Outside Counsel Wei-Yi Chen | j118 | ontology, identity, tenancy, audit-chain, compliance | ontology-projection cedar fragment |
| 24 | Audit-chain seal verification | all B2B personas; Sam Okafor; Diana Reyes; Sergei Petrov | j126,j131,j137-141 | audit-chain, compliance, governance, observability | seal cedar fragment |
| 25 | Tenant-merger conglomerate handoff | Marcus Chen; CSO Mira Goldberg; Outside Counsel Wei-Yi Chen | j125 | tenancy, identity, ontology, compliance, audit-chain | merger cedar fragment (ADR-0313) |
| 26 | Multi-tenant coordinated product launch | Marcus Chen; CMO Felix Ng; PR Manager Helena Sato | j123 | workflow-engine, messenger, drive, intelligence, identity, tenancy | coordinated-launch cedar fragment |
| 27 | Cedar fragment publish soak ≥60s | CISO Yuki Park; CCO Naveen Iyer; Compliance Analyst Yui Hayashi | j20,j100,j126,j131 | policy-engine, audit-chain, observability, compliance | publish-soak cedar fragment (ADR-0243) |
| 28 | Court-warrant scoped piercing | Outside Counsel Wei-Yi Chen; Diana Reyes; CCO Naveen Iyer; Officer Rodriguez | j129 | identity, audit-chain, compliance, governance, workflow-engine, community, marketplace, payments, messenger, drive | warrant cedar fragment (ADR-0312) |
| 29 | Personal-tenant survives layoff | Chris Volkov; Aiyana Singh; Marcus Chen's engineers | j127,j142,j143 | identity, tenancy, drive, mail, messenger, workflow-engine, audit-chain | dual-tenant cedar fragment (ADR-0311) |
| 30 | Cell-tier degradation Tier-0 outage | all personas; ops-dashboard-control-center on-call | j01,j11,j12,j100,j124 | cell, observability, ops-dashboard-control-center, api-gateway | cell-tier cedar fragment (ADR-0251) |

---

## §9 Coverage Gap Matrix

This section identifies combinations that the doctrine implies should exist but do not yet have explicit evidence. Gaps are categorized into four classes.

### §9.1 Persona × journey gaps (personas with no journey coverage)

The following personas appear in MASTER-ROSTER but currently have zero anchored journeys in `docs/user-journeys/`. Each represents a candidate for j151+ authoring:

| Persona | Archetype | Suggested journey theme |
|---|---|---|
| Mailroom Hae-Won Kim | Mailroom Staff | typical day-of-mailroom journey + critical-path row coverage |
| Maintenance Tech Carlos Reyes II | Building Maintenance | typical day-of-building journey + critical-path row coverage |
| Security Guard Stefan Kovács | Security Guard | typical day-of-security journey + critical-path row coverage |
| Print Operator Diana Lazăr | Print Operator | typical day-of-print journey + critical-path row coverage |

### §9.2 Journey × µservice gaps (journeys missing µservice anchoring)

Journeys whose README.md microservices_touched: block is empty or unparsed (likely template-skeleton only):

| Journey | Status |
|---|---|
| j02-healthcare-code-blue-ehr-break-glass | missing microservices_touched block in README |
| j03-988-crisis-line-minor-self-report | missing microservices_touched block in README |
| j21-personal-signup-passkey-first-dm | missing microservices_touched block in README |
| j22-personal-mail-inbox-first-week | missing microservices_touched block in README |
| j23-marketplace-listing-and-first-sale | missing microservices_touched block in README |
| j24-marketplace-purchase-as-buyer | missing microservices_touched block in README |
| j25-personal-notes-daily-journaling-with-e2e | missing microservices_touched block in README |
| j26-drive-family-photo-backup | missing microservices_touched block in README |
| j27-calendar-cross-context-family-and-work | missing microservices_touched block in README |
| j28-meet-family-video-call | missing microservices_touched block in README |
| j29-workflow-studio-personal-automation | missing microservices_touched block in README |
| j30-shorts-creator-first-post | missing microservices_touched block in README |
| j31-social-broadcast-vs-DM | missing microservices_touched block in README |
| j32-community-teamblind-employer-anonymous | missing microservices_touched block in README |
| j33-b2b-sso-saml-onboarding | missing microservices_touched block in README |
| j34-b2b-team-channel-with-files | missing microservices_touched block in README |
| j35-b2b-workplace-mail-and-calendar | missing microservices_touched block in README |
| j51-procure-to-pay-po-extraction-and-approval | missing microservices_touched block in README |
| j52-order-to-cash-marketplace-to-fulfillment | missing microservices_touched block in README |
| j53-invoice-to-cash-recurring-subscription | missing microservices_touched block in README |
| j54-quote-to-contract-to-payment-saas | missing microservices_touched block in README |
| j55-refund-and-dispute-resolution-cascade | missing microservices_touched block in README |
| j56-job-application-to-offer | missing microservices_touched block in README |
| j57-employee-onboarding-day-one-to-week-one | missing microservices_touched block in README |
| j58-quarterly-performance-review-cycle | missing microservices_touched block in README |
| j59-offboarding-and-knowledge-transfer | missing microservices_touched block in README |
| j60-internal-mobility-promotion-cascade | missing microservices_touched block in README |
| j61-patient-intake-to-followup | missing microservices_touched block in README |
| j62-prescription-to-pharmacy-to-payment | missing microservices_touched block in README |
| j63-clinical-trial-recruitment-to-consent | missing microservices_touched block in README |
| j64-hospital-network-cross-tenant-referral | missing microservices_touched block in README |
| j65-gdpr-dsar-cascade-across-all-services | missing microservices_touched block in README |
| j66-tax-quarterly-filing-multi-jurisdiction | missing microservices_touched block in README |
| j67-law-enforcement-warrant-response | missing microservices_touched block in README |
| j68-regulator-audit-pull-hippa-soc2-pci | missing microservices_touched block in README |
| j69-llm-agent-managing-yejins-week | missing microservices_touched block in README |
| j70-ai-drafted-contract-human-finalized | missing microservices_touched block in README |
| j71-ai-detected-fraud-pattern-response | missing microservices_touched block in README |
| j72-ai-translation-cross-locale-business | missing microservices_touched block in README |
| j73-third-party-developer-publishes-plugin | missing microservices_touched block in README |
| j74-tenant-installs-plugin-and-it-spans-services | missing microservices_touched block in README |
| j75-plugin-revoked-during-incident-response | missing microservices_touched block in README |
| j76-eu-gdpr-dsar-full-cascade | missing microservices_touched block in README |
| j77-eu-ai-act-high-risk-credit-decision | missing microservices_touched block in README |
| j78-eu-nis2-breach-three-stage-cadence | missing microservices_touched block in README |
| j79-eu-dsa-transparency-semi-annual-report | missing microservices_touched block in README |
| j80-kr-pipa-personal-info-cross-border-transfer | missing microservices_touched block in README |
| j81-kr-csap-sovereign-cell-audit-pull | missing microservices_touched block in README |
| j82-kr-fss-financial-fraud-24h-freeze | missing microservices_touched block in README |
| j83-cn-pipl-data-localization-and-cac-assessment | missing microservices_touched block in README |
| j84-jp-appi-elder-user-consent | missing microservices_touched block in README |
| j85-hipaa-end-to-end-phi-workflow | missing microservices_touched block in README |
| j86-pci-dss-l1-tokenized-payment-flow | missing microservices_touched block in README |
| j87-fedramp-high-il5-air-gap-deployment | missing microservices_touched block in README |
| j88-au-irap-protected-tenant | missing microservices_touched block in README |
| j89-uk-aadc-minor-ux-adaptation | missing microservices_touched block in README |
| j90-us-ccpa-cpra-do-not-sell-opt-out | missing microservices_touched block in README |
| j91-us-state-money-transmitter-licensing | missing microservices_touched block in README |
| j92-br-lgpd-dsar-with-us-parent | missing microservices_touched block in README |
| j93-in-dpdpa-rbi-financial-overlay | missing microservices_touched block in README |
| j94-sox-404-public-company-controls | missing microservices_touched block in README |
| j95-iso-27001-soc-2-annual-audit | missing microservices_touched block in README |
| j96-ksa-uae-mena-tenant-onboarding | missing microservices_touched block in README |
| j97-sg-pdpa-mas-singapore-tenant | missing microservices_touched block in README |
| j98-au-privacy-apra-cps-234-tenant | missing microservices_touched block in README |
| j99-cross-jurisdiction-multi-pack-conflict-resolution | missing microservices_touched block in README |
| j100-pack-rollout-from-tenant-onboarding-to-first-action | missing microservices_touched block in README |
| j132-hr-mass-hiring-event-100-roles | missing microservices_touched block in README |
| j133-hr-conducts-layoff-with-dignity-and-compliance | missing microservices_touched block in README |
| j134-hr-cross-tenant-recruitment-via-staffing-agency | missing microservices_touched block in README |
| j135-hr-handles-harassment-complaint-with-dual-tenant-boundary | missing microservices_touched block in README |
| j136-hr-administers-benefits-open-enrollment | missing microservices_touched block in README |
| j140-internal-audit-data-loss-prevention-egress-trip | missing microservices_touched block in README |
| j141-internal-audit-respects-employee-personal-tenant-boundary | missing microservices_touched block in README |
| j142-layoff-day-zero-from-employees-side | missing microservices_touched block in README |
| j143-laid-off-imports-work-portfolio-into-personal-tenant | missing microservices_touched block in README |
| j144-laid-off-builds-job-search-pipeline-in-workflow-studio | missing microservices_touched block in README |
| j145-laid-off-applies-via-community-handshake-linkedin-mode | missing microservices_touched block in README |
| j146-laid-off-uses-marketplace-as-temporary-income | missing microservices_touched block in README |
| j147-laid-off-cohort-mutual-aid-community-channel | missing microservices_touched block in README |

### §9.3 µservice × persona gaps (µservices with no persona traffic)

µservices that currently lack any anchored persona usage in the cross-coverage:

| µservice | Status |
|---|---|
| cloud-iac | no persona-anchored journey yet — candidate for j151+ |
| cloud-k8s | no persona-anchored journey yet — candidate for j151+ |
| cloud-secrets | no persona-anchored journey yet — candidate for j151+ |
| feature-flags | no persona-anchored journey yet — candidate for j151+ |
| network | no persona-anchored journey yet — candidate for j151+ |
| data-pipeline | no persona-anchored journey yet — candidate for j151+ |
| data-warehouse | no persona-anchored journey yet — candidate for j151+ |
| analytics | no persona-anchored journey yet — candidate for j151+ |
| incident-management | no persona-anchored journey yet — candidate for j151+ |
| itsm | no persona-anchored journey yet — candidate for j151+ |
| forms | no persona-anchored journey yet — candidate for j151+ |
| docs | no persona-anchored journey yet — candidate for j151+ |
| sheets | no persona-anchored journey yet — candidate for j151+ |
| slides | no persona-anchored journey yet — candidate for j151+ |
| whiteboard | no persona-anchored journey yet — candidate for j151+ |
| design-collaboration | no persona-anchored journey yet — candidate for j151+ |
| recordings | no persona-anchored journey yet — candidate for j151+ |
| sites | no persona-anchored journey yet — candidate for j151+ |
| social | no persona-anchored journey yet — candidate for j151+ |
| treasury | no persona-anchored journey yet — candidate for j151+ |
| crm | no persona-anchored journey yet — candidate for j151+ |
| contact-center | no persona-anchored journey yet — candidate for j151+ |
| marketing-automation | no persona-anchored journey yet — candidate for j151+ |
| contract-lifecycle-management | no persona-anchored journey yet — candidate for j151+ |
| learning-management | no persona-anchored journey yet — candidate for j151+ |
| performance-management | no persona-anchored journey yet — candidate for j151+ |
| tasks | no persona-anchored journey yet — candidate for j151+ |
| warehouse | no persona-anchored journey yet — candidate for j151+ |
| supply-chain-planning | no persona-anchored journey yet — candidate for j151+ |
| production-planning | no persona-anchored journey yet — candidate for j151+ |
| quality-management | no persona-anchored journey yet — candidate for j151+ |
| plant-maintenance | no persona-anchored journey yet — candidate for j151+ |
| global-trade | no persona-anchored journey yet — candidate for j151+ |
| real-estate | no persona-anchored journey yet — candidate for j151+ |
| financial-planning | no persona-anchored journey yet — candidate for j151+ |
| healthcare-integration | no persona-anchored journey yet — candidate for j151+ |
| personal-health-tracker | no persona-anchored journey yet — candidate for j151+ |

### §9.4 Critical-path × handler gaps

Critical-path rows in §3.2.5 that currently have no anchored journey (per §8 cross-check):

All 30 rows have at least one anchored journey per §8. The two highest-leverage gaps are:

- **Row 14 (delegated LLM agent acting for principal)** has only j14 as evidence; needs depth for j14-{15,16,17} expansions to cover delegation across all collar-colors.
- **Row 23 (cross-tenant data sharing via ontology projection)** has only j118; needs companion journeys covering non-B2B uses (e.g., family-tenant project sharing).

### §9.5 Cross-axis coverage matrix summary

| Axis | Total | Covered | Gap | Gap % |
|---|---:|---:|---:|---:|
| Personas with ≥1 anchored journey | 129 | 125 | 4 | 3.1% |
| Journeys with parsed µservice list | 150 | 70 | 80 | 53.3% |
| µservices with anchored persona traffic | 71 | 34 | 37 | 52.1% |
| Critical-path rows with ≥1 journey | 30 | 30 | 0 | 0.0% |

---

## §10 Recommended j151+ Journeys to Fill Coverage Gaps

Based on §9 gap analysis, the following 25 journeys (j151-j175) are recommended for next-wave authoring. Each is selected to close the highest-leverage gap (persona × µservice × critical-path triple-coverage in one journey).

| Recommended ID | Slug | Primary persona | Collar/workspace | µservices | Critical-path rows | Cluster filled |
|---|---|---|---|---|---|---|
| j151 | captain-olufemi-typhoon-evacuation-and-co-op-cash-flow | Captain Olufemi | green/field | payments + finops-portal + messenger + audit-chain + connect | row 1,30 | green-collar-cluster + supply-chain-cluster |
| j152 | ahmad-hassan-construction-site-incident-bilingual | Ahmad Hassan | blue/field | incident-management + messenger + audit-chain + workplace-integration + drive | row 8,30 | blue-collar OSHA |
| j153 | devon-williams-hvac-side-business-tax-end-of-year | Devon Williams | gray/field | payments + finops-portal + tasks + connect + workflow-studio | row 8,9 | gray-collar gig economy |
| j154 | tomas-pieter-channel-partner-co-marketing-launch | Tomas Pieter | external/front-office | marketing-automation + crm + comms-email + community + connect | row 8,24 | channel-partner trinity |
| j155 | stefan-kovacs-college-night-shift-and-finals-week | Stefan Kovács | gray/front-office | calendar + learning-management + payments + community + observability | row 8,9 | gray-collar dual-role |
| j156 | carlos-reyes-ii-maintenance-emergency-after-hours | Carlos Reyes II | gray/back-office | incident-management + tasks + messenger + audit-chain + workflow-engine | row 1,8,30 | facility ops |
| j157 | diana-lazar-print-operator-batch-defect-and-quality-recall | Diana Lazăr | gray/production | quality-management + tasks + workflow-engine + audit-chain + messenger | row 8,24 | QMS critical |
| j158 | print-shop-cell-rebalance-shorts-creator-spike | Mailroom Hae-Won Kim | blue/back-office | tasks + workflow-engine + shorts (cross-link) + messenger | row 9,30 | print/shorts intersection |
| j159 | saanvi-mehta-mba-application-spans-personal-and-work | Saanvi Mehta | white/back-office | identity + mail + drive + payments + community + learning-management | row 8,9 | dual-tenant + EDU |
| j160 | cleaning-co-tomáš-horák-bid-cross-tenant-and-onboard | Tomáš Horák | blue/admin | marketplace + workflow-engine + payments + tenancy + community | row 8,24 | blue-collar small-biz |
| j161 | cafeteria-soyeon-kim-allergen-recall-and-school-coordination | Soyeon Kim | pink/production | quality-management + community + messenger + audit-chain + compliance | row 1,14,24 | school + safety |
| j162 | print-operator-diana-lazar-night-shift-onboarding | Diana Lazăr | gray/production | learning-management + workplace-integration + identity + tasks | row 9 | onboarding |
| j163 | av-coordinator-jordan-park-board-meeting-cross-time-zone | Jordan Park-AV | gray/back-office | meet + recordings + calendar + drive + governance | row 1,8 | board-meeting AV |
| j164 | retired-hiroshi-tanaka-yearly-tax-and-pension | Hiroshi Tanaka | white-retired/field | workflow-studio + payments + drive + notes + compliance | row 9,14 | retiree assistive tech |
| j165 | cco-naveen-iyer-board-quarterly-compliance-report | CCO Naveen Iyer | white/executive | governance + compliance + audit-chain + workflow-engine + drive | row 24,27 | board reporting |
| j166 | cso-mira-goldberg-strategic-acquisition-go-no-go | CSO Mira Goldberg | white/executive | governance + financial-planning + intelligence + compliance + connect | row 8,24 | M&A go/no-go |
| j167 | cto-diego-vargas-platform-major-version-cutover | CTO Diego Vargas | white/executive | feature-flags + cloud-iac + cloud-k8s + observability + governance | row 27,30 | platform cutover |
| j168 | coo-akira-watanabe-quarterly-ops-review-and-incident-debrief | COO Akira Watanabe | white/executive | ops-dashboard-control-center + incident-management + observability + audit-chain | row 24,30 | ops review |
| j169 | cmo-felix-ng-multi-country-launch-with-locale-pack | CMO Felix Ng | white/executive | marketing-automation + community + analytics + intelligence + compliance | row 8,14 | multi-country brand |
| j170 | aiko-brown-sustainability-report-and-scope-3-supply-chain | Aiko Brown | green/middle-office | compliance + audit-chain + supply-chain-planning + connect + ontology | row 24 | ESG scope-3 |
| j171 | felix-tan-ombudsperson-cross-tenant-mediation-with-privilege | Felix Tan | white/middle-office | messenger + drive + audit-chain + community + governance | row 8,24 | ombudsperson privilege |
| j172 | lev-kahn-investor-relations-shareholder-meeting-livestream | Lev Kahn | white/front-office | meet + governance + drive + audit-chain + community | row 1,8 | AGM livestream |
| j173 | aamir-khan-wealth-manager-multi-jurisdictional-trust-restructure | Aamir Khan | white/front-office | contract-lifecycle-management + payments + compliance + audit-chain + drive | row 8,18,24 | cross-border trust |
| j174 | sven-eriksson-treasury-eod-position-reconciliation | Sven Eriksson | white/middle-office | payments + treasury + finops-portal + audit-chain + observability | row 8,24 | EOD recon |
| j175 | aanya-kapoor-LP-portfolio-tax-and-K1-distribution | Aanya Kapoor | white/executive | payments + finops-portal + compliance + drive + connect | row 8,24 | LP K-1 |

### §10.1 Top-20 highest-leverage j151+ recommendations (priority order)

Filtered to the 20 with greatest cross-coverage gain (persona-archetype × µservice-untouched × critical-path-row):

1. **j151 — captain-olufemi-typhoon-evacuation-and-co-op-cash-flow**
2. **j152 — ahmad-hassan-construction-site-incident-bilingual**
3. **j153 — devon-williams-hvac-side-business-tax-end-of-year**
4. **j154 — tomas-pieter-channel-partner-co-marketing-launch**
5. **j155 — stefan-kovacs-college-night-shift-and-finals-week**
6. **j156 — carlos-reyes-ii-maintenance-emergency-after-hours**
7. **j157 — diana-lazar-print-operator-batch-defect-and-quality-recall**
8. **j158 — print-shop-cell-rebalance-shorts-creator-spike**
9. **j159 — saanvi-mehta-mba-application-spans-personal-and-work**
10. **j160 — cleaning-co-tomáš-horák-bid-cross-tenant-and-onboard**
11. **j161 — cafeteria-soyeon-kim-allergen-recall-and-school-coordination**
12. **j162 — print-operator-diana-lazar-night-shift-onboarding**
13. **j163 — av-coordinator-jordan-park-board-meeting-cross-time-zone**
14. **j164 — retired-hiroshi-tanaka-yearly-tax-and-pension**
15. **j165 — cco-naveen-iyer-board-quarterly-compliance-report**
16. **j166 — cso-mira-goldberg-strategic-acquisition-go-no-go**
17. **j167 — cto-diego-vargas-platform-major-version-cutover**
18. **j168 — coo-akira-watanabe-quarterly-ops-review-and-incident-debrief**
19. **j169 — cmo-felix-ng-multi-country-launch-with-locale-pack**
20. **j170 — aiko-brown-sustainability-report-and-scope-3-supply-chain**

---

## §11 Reader Walk-Throughs (the matrix as navigation tool)

Three reader trajectories make the matrix load-bearing. Each is a worked example of how to use this document.

### §11.1 'I am a CFO — what touches me?'

Walk: §2.026 (CFO Helena Brandt) → journeys j119,j120,j122,j137 → §3.119 (invoice-financing), §3.120 (FX hedge), §3.122 (vendor payment batch), §3.137 (SOX 404) → µservices payments, finops-portal, treasury, financial-planning, compliance, audit-chain, governance → §4.50 (payments), §4.51 (finops-portal), §4.52 (treasury), §4.68 (financial-planning), §4.13 (compliance), §4.05 (audit-chain), §4.14 (governance) → critical-path rows 24, 27, 30.

### §11.2 'I am the payments µservice owner — who depends on me?'

Walk: §4.50 (payments) → personas Yejin Park, Tomás García, Hiroshi Tanaka, Diana Reyes, Priya Krishnan, Sam Okafor, CFO Helena Brandt, Tomás García Jr., Sarah Kim, Chris Volkov, Investment Banker Yuna Ahn, Wealth Manager Aamir Khan, ... → journeys j07, j08, j10, j23, j24, j33, j35, ... → critical-path rows 1, 8, 9, 18, 24, 27 → hyperscaler benchmark Stripe + Adyen + Square.

### §11.3 'I am writing the j151 emergency-storm coastal-fishing journey'

Walk: §10 recommendation j151 (Captain Olufemi) → MASTER-ROSTER §3.2 row 24 → §6.2 green-collar-cluster + supply-chain-cluster → critical-path rows 1, 30 → µservices payments, finops-portal, messenger, audit-chain, connect → architecture coverage matrix §12.6 (green-collar surface) → ADR-0317 role-projection + ADR-0298 emergency bypass.

### §11.4 'I am the EU-AI-Act compliance reviewer — which journeys exercise EU-AI-Act?'

Walk: §10 j169 (CMO multi-country launch) + j132 (HR mass hiring AI-screening) + §3.014 (delegated LLM agent) → personas CMO Felix Ng + Priya Krishnan + Yejin Park (as delegate) → µservices intelligence, foundry, ontology → §4.19 + §4.20 + §4.16 → critical-path rows 14 + 27 → pack overlay pack-gdpr + pack-eu-ai-act + pack-dora.

---

## §12 Continuity-of-Identity Worked Examples

Five worked examples make the cross-context bridge concrete.

### §12.1 Yejin Park's five contexts in one Tuesday

06:00 KST — Yejin-as-parent uses calendar + community (PTA) to push her child's permission slip to Ms. Patel; tenant=personal-family. 07:30 KST — Yejin-as-side-business-owner ships handmade-soap orders via marketplace + payments + finops-portal; tenant=soap-business. 09:00 KST — Yejin-as-nurse begins her shift; healthcare-integration + drive (PHI) + messenger (work-tenant only); tenant=St Mary's Hospital Seoul. 14:00 KST — Yejin-as-patient downloads her own lab results via personal-health-tracker; tenant=her PCP's clinic. 21:00 KST — Yejin-as-consumer browses marketplace + shorts; tenant=personal. All five contexts use the same passkey identity (ADR-0299). Cedar default-deny enforces context-isolation at every µservice boundary (ADR-0311). Audit-chain emits one event class per context.

### §12.2 Marcus Chen's three contexts and a cross-board conflict

Marcus-as-CEO of his 5000-person multinational is also Marcus-as-board-director of Other Co. The boards have a competitive overlap. Cedar default-deny ensures Marcus-as-board-director-elsewhere cannot read Other Co's day-to-day ops surfaces; ADR-0311 enforces the boundary. Marcus-as-father uses calendar + messenger (personal) for his child's school; tenant=personal-family. The three contexts share Marcus's passkey identity but never share Cedar permits.

### §12.3 Diana Reyes's auditor / consumer boundary on a bribery attempt

Diana-as-auditor performs a FedRAMP audit on Marcus's tenant (j126). Diana-as-consumer receives a bribery offer via personal messenger (j130). Cedar default-deny isolates the two contexts: the audit cannot leverage Diana-as-consumer's personal-tenant data; the bribery thread cannot piggyback into the audit. Diana's mandatory disclosure (j130) is from the consumer-tenant perspective; the audit (j126) emits its own audit-chain class.

### §12.4 Chris Volkov's pre-layoff / post-layoff / family-provider continuity

Chris-pre-layoff (B2B_EMPLOYEE) has Cedar permits over former-employer's work surfaces. On layoff day-zero (j142), former-employer revokes work-tenant Cedar permits within budget (row 9). Chris's personal-tenant identity survives (ADR-0311). Chris-post-layoff (B2C_JOB_SEEKER_ACTIVE) gains community LinkedIn-mode + Handshake-mode + Blind-mode permits + workflow-studio for job-search pipeline + marketplace for side-income. Chris-as-family-provider (B2C_FAMILY_PARENT) retains family-budget + kids' school calendar permits — uninterrupted by the layoff.

### §12.5 Outside Counsel Wei-Yi Chen's strict client-A vs client-B isolation

Wei-Yi serves two competing clients. Each engagement is its own tenant scope. Cedar default-deny prevents any cross-engagement read. Attorney-client-privilege pack overlay enforces retention + production-rules. Wei-Yi's three contexts (counsel-for-A, counsel-for-B, consumer) never share Cedar permits but share one passkey identity. Audit-chain emits per-engagement classes.

---

## §13 Self-Audit + Verification Checklist

This matrix must pass the §1.4 six-dimension rigor floor. The checklist below documents how it does.

- [x] Every persona row cites MASTER-ROSTER §3 anchor + audience_type (ADR-0244).
- [x] Every journey row cites `docs/user-journeys/j<NN>-*/README.md` source.
- [x] Every µservice row cites `microservices/<svc>/PRD.md` + center-of-gravity rank (coverage-matrix §14).
- [x] Every critical-path row maps to ≥1 anchored journey (§8).
- [x] All 127 personas, 150 journeys, 69 µservices enumerated (§2, §3, §4).
- [x] Journey-graph edges enumerated (§5.1).
- [x] Persona-graph cross-context bridge edges enumerated (§6.1).
- [x] Capability-tier × µservice composition enumerated (§7).
- [x] Coverage-gap matrix produced (§9).
- [x] j151+ recommendations produced (§10).
- [x] No source-of-truth doc modified.
- [x] No new ADR introduced; existing ADR-0244/0247/0299/0311/0312/0313/0316/0317/0318/0319/0320 cited.
- [x] Continuity-of-identity bridges explicit (§6, §12).
- [x] Cross-references to documentation-rigor.md §3.2.5 critical-path rows in every relevant table.

---

## §14 References

- `docs/personas/MASTER-ROSTER-2026-05-21.md` (persona axis source-of-truth)
- `docs/personas/<persona-slug>.md` (30+ priority dossiers per MASTER-ROSTER §13.1)
- `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` (journey-axis catalog slice)
- `docs/user-journeys/j01..j150/README.md` (per-journey READMEs)
- `docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md` (µservice + capability-tier axis)
- `docs/standards/documentation-rigor.md` (rigor doctrine + §3.2.5 critical-path matrix)
- ADR-0105 (13-layer canonical enum)
- ADR-0131 (per-microservice flat layout)
- ADR-0244 (audience_type)
- ADR-0247 (self-modification doctrine)
- ADR-0249 (multi-category marketplace)
- ADR-0251 (compliance-pack cell certification levels)
- ADR-0263 (observability emission contract)
- ADR-0292 (minor-user doctrine COPPA/KOSA/EU-age-verification)
- ADR-0297 (abuse-defence baseline)
- ADR-0298 (emergency-services bypass life-safety)
- ADR-0299 (account-recovery passkey-bound identity)
- ADR-0300 (cross-link to abuse-defence)
- ADR-0304 (cross-jurisdiction lawful-authority)
- ADR-0307 (detection-substrate)
- ADR-0308 (DLP egress)
- ADR-0309 (workplace-integration)
- ADR-0310 (B2B-leader capability tiers)
- ADR-0311 (dual-tenant identity personal-vs-work boundary)
- ADR-0312 (court-warrant scoped piercing)
- ADR-0313 (conglomerate-tenant hierarchy)
- ADR-0314 (in-flight: cross-link)
- ADR-0315 (in-flight: cross-link)
- ADR-0316 (capability-tier registry)
- ADR-0317 (role-projection doctrine — in-flight authoritative)
- ADR-0318 (collar-color universality — in-flight authoritative)
- ADR-0319 (front/middle/back-office distinction — in-flight authoritative)
- ADR-0320 (apprentice/intern/resident/fellow tier — in-flight authoritative)

---

## §15 Stop Condition

- All ~127 personas enumerated in §2 with full row schema.
- All 150 journeys enumerated in §3 with full row schema.
- All 69 µservices enumerated in §4 with full row schema.
- Journey-graph, persona-graph, capability-tier-graph edges enumerated (§5–§7).
- Critical-path × persona × journey × µservice mapping in §8.
- Coverage gap matrix + j151+ recommendations in §9–§10.
- Reader walk-throughs + continuity-of-identity worked examples in §11–§12.
- Self-audit checklist in §13.
- No source-of-truth doc modified.
