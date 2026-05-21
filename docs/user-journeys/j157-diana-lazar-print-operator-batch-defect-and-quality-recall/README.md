---
doc_class: User-Journey-README
journey_id: j157-diana-lazar-print-operator-batch-defect-and-quality-recall
slice: production-line-stop-and-customer-recall-with-quality-management-cedar-gate
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Print-Shop Production Operator Diana Lazăr
audience_type: B2B_PRODUCTION_WORKER + B2B_SUPPLIER_QUALITY
microservice_count: 5
pack_overlay_anchor: ISO-9001-QMS + ISO-12647-2-print-color + FOGRA-PSO + EU-GPSR-2023-988 + RO-OUG-21-2021-consumer + EU-GDPR
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0263-observability-emission-contract
  - ADR-0252-hlc-default-truetime-tier
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0250-build-ahead-of-certification
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0247-self-modification-doctrine
---

# j157 — Diana Lazăr: batch defect mid-shift, full quality recall

## At a glance

Diana Lazăr is a **certified print-production operator** (FOGRA-PSO Operator Level 2 + ISO-12647-2 trained) employed by **Tipografia Lazăr-Petrescu SRL** — a third-generation family-owned commercial printer in Cluj-Napoca, Romania, founded 1987 by her grandfather Vasile Lazăr-Petrescu after the fall of communism. She is 34 years old, Romanian, born in Bistriţa, educated at Universitatea Tehnică din Cluj-Napoca (graphic engineering), trilingual RO/HU/EN, lives in Mănăștur district with her daughter Maria (age 8), and works the day shift (06:00–14:30 EET) on the **Heidelberg Speedmaster CX 102-6+LX** offset press at the firm's Strada Mihail Kogălniceanu plant.

It is **Tuesday February 23, 2027, 11:42 EET**. The press has been running batch `BCH-2027-02-23-0612-pharma-leaflet-NSAID-RO` since 06:18 EET — a 47,500-unit run of patient information leaflets (PIL) for a Romanian generics pharmaceutical company **Antibiotice SA** (Iași, RO) covering a new ibuprofen 400mg formulation. The PIL is a single-fold A5 sheet, 4-color front, 1-color (PMS Black) back, printed on 70gsm bible paper. The customer's regulatory submission to **ANMDMR** (Agenția Națională a Medicamentului și a Dispozitivelor Medicale din România) was approved on 2027-01-18; the leaflet wording is legally required to match the approved text **exactly**.

At 11:42:14 EET Diana's inline **GMI ColorProof + Heidelberg Prinect Inpress Control** flags a ΔE2000 spike in solid C/M registration on sheets 23,847 onward — drift from ΔE 1.4 (in-tolerance) to ΔE 4.7 (out of FOGRA-PSO tolerance, which is ΔE ≤ 3.0 for 100% solid). Worse, the operator-side visual inspection a moment later catches that **the ibuprofen 400mg PIL's bold-red allergy-warning box on the front is registering 1.2 mm low**, which clips the bottom of the legally-mandated text reading "**Nu administrați copiilor sub 6 ani fără sfatul medicului**" — at the truncated rendering, "**ub 6 ani**" is partially missing on the affected sheets.

This is the kind of defect that, if it ships, becomes a regulatory recall under EU-GPSR Article 33 + RO-OUG-21-2021 + the customer's ANMDMR submission. Diana has 4 minutes to decide.

This journey covers the next 9 hours of Diana's professional life:

1. **Immediate line stop** via the `quality-management` µservice — Cedar permit `quality.production_line_stop` against the printer-tenant resource `press-line-heidelberg-cx-102-6-lx-01`; the permit grants Diana **production line stop authority** without requiring manager approval (FOGRA-PSO Operator Level 2 holds this authority by certification, mirrored in Cedar context)
2. **Recall workflow** triggered through `workflow-engine` with a 6-stage state machine (`stop_called` → `quarantine` → `defect_root_cause` → `customer_notify` → `recall_execute` → `closure_post_mortem`); each transition Cedar-gated, audit-chain sealed
3. **Tasks µservice** materializes 14 atomic tasks (segregate suspect sheets, count clean sheets vs defect sheets, sample 47 sheets across the batch for retrospective inspection, photograph each defect type, ship samples to QA lab, draft customer notification, draft regulator notification, etc.)
4. **Customer notification via `messenger`** — MLS-encrypted cross-tenant thread between Tipografia Lazăr-Petrescu (Diana + her father Mihai Lazăr-Petrescu, current managing director) and Antibiotice SA (their Director Calitate, Dr. Cristina Munteanu, plus their Responsabil Persoană Calificată Dr. Andrei Popescu); third party invitations to ANMDMR's inspector channel if recall is escalated to a regulatory event
5. **Audit-chain anchoring** of every recall decision with merkle anchoring — ISO-9001 audit trail must be reconstructible 7 years later; the customer's ANMDMR file must include the provenance chain

Microservices: `quality-management`, `tasks`, `workflow-engine`, `audit-chain`, `messenger`. Secondary: `identity`, `tenancy`, `compliance` (ISO-9001 + FOGRA-PSO + ISO-12647-2 + EU-GPSR + RO-OUG-21-2021 + EU-GDPR), `observability`, `production-planning`, `analytics`, `plant-maintenance`, `notes` (Diana's bilingual root-cause writeup), `learning-management`, `crm`, `contract-lifecycle-management`.

This is a **gray-collar, mid-shift, regulator-touching, customer-facing recall** journey. It demonstrates that oyatie's `quality-management → tasks → workflow-engine → audit-chain` triad supports **operator-initiated authoritative production stop** with proper Cedar permits AND end-to-end customer + regulator notification AND ISO-grade evidence retention — all without a frantic phone-tree call.

## Why this journey matters

Diana Lazăr is **MASTER-ROSTER §3.4 row 101** — the canonical gray-collar production-quality operator persona. She is also the canonical **diacritic-aware identity** demonstration: her name carries the Romanian "ă" character; the system must render and store it without normalization loss, must match in search both with and without diacritics (Lazar = Lazăr in fuzzy match), and must respect that "ă" is NOT equal to "a" in legal contexts (her ID card spells "Lazăr" and so do her contracts).

The persona covers an estimated 11 million EU industrial production workers in regulated industries (pharma packaging, food, automotive, medical devices) where defect detection at the operator level can stop a regulatory recall before it escalates. The category is acutely under-served by enterprise QMS software because most products require a manager to sign off on a line stop — a friction that historically causes operators to keep running marginal batches "just to finish the shift", which is the root cause of many regulator-escalated recalls.

The journey closes:

- **Critical-path row 25** (Operator-authoritative production stop with Cedar permit gating)
- **Critical-path row 26** (Cross-tenant customer + regulator recall notification)
- **Critical-path row 27** (Diacritic-aware identity + bilingual workflow)
- **Critical-path row 28** (ISO-9001-grade evidence retention with merkle anchoring)

Hyperscaler benchmark: SAP QM + Oracle Quality + Siemens Opcenter Quality + Plex QMS + ETQ Reliance + MasterControl. The unique part of oyatie is that **Cedar policy makes "operator can stop a line" a first-class permit gated on certification + role + active shift + product context** — not a flag in a config table — and the resulting audit chain is regulator-grade by default, not as a paid add-on.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat 11:42 EET defect detection → 20:17 EET shift handoff after recall closure | Diacritic-correct dialogue (RO + HU + EN); specific equipment (Heidelberg Speedmaster CX 102-6+LX; GMI ColorProof; Prinect Inpress Control); specific ΔE2000 readings; specific regulator anchors (ANMDMR, EU-GPSR Art 33); specific RO names (Cristina Munteanu, Andrei Popescu, Mihai Lazăr-Petrescu) |
| `ux-flow.md` | Print operator's industrial-tablet HMI (Heidelberg's Prinect Cockpit + oyatie overlay); manager-mobile + customer-side screens | Big-button glove-mode; diacritic input modal; HU/RO/EN switcher; recall-state visualization |
| `handshake.md` | Per-µservice API + per-tenant scoping | Each row names source tenant + target tenant + Cedar permit + cross-tenant recall handshake |
| `integration-test-plan.md` | Line-stop tests + recall workflow tests + cross-tenant notification tests + ISO-9001 audit reconstruction tests + diacritic fidelity tests | Each test names seed values + expected event chain + pass/fail thresholds |
| `schemas/openapi-quality-and-recall.json` | OpenAPI for line-stop + recall lifecycle endpoints | All 6 recall stages + customer notify + regulator notify |
| `schemas/cedar-policy.cedar` | Production line-stop + recall Cedar policy | Operator-level authority + cert checks + cross-tenant customer notify permit |
| `schemas/journey-messages.proto` | proto3 for all RPCs | Diacritic-safe string handling; ΔE2000 sample readings; recall state machine |
| `schemas/recall-state-machine.yaml` | 6-state recall lifecycle | Per-state Cedar guards + audit class + customer/regulator gates |
| `schemas/quality-defect-form-iso-9001.json` | Defect report schema (ISO-9001-compatible) | Required fields + photo evidence + root-cause cascade + correction + corrective + preventive |

## The five microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `quality-management` | Owns the inline ΔE2000 reading, the FOGRA-PSO tolerance gate, the operator-level line-stop permit, the defect classification | row 25 |
| `tasks` | Materializes the 14 atomic recall tasks; each carries photo + sample-ID + Cedar context | row 25 |
| `workflow-engine` | Drives the 6-state recall lifecycle; co-sign gates; customer + regulator notification | row 26 |
| `audit-chain` | Per-decision merkle anchor; ISO-9001-grade 7-year retention; reconstructible chain | row 28 |
| `messenger` | MLS-encrypted cross-tenant thread Tipografia ↔ Antibiotice; escalation channel to ANMDMR inspector if needed | row 26 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `identity` | Diana's passkey + diacritic-preserving name fields; manager cross-sign for the post-mortem |
| `tenancy` | Two tenants in scope (Tipografia Lazăr-Petrescu + Antibiotice); third tenant invited (ANMDMR-inspectorate) if recall escalates |
| `compliance` | Activates ISO-9001, FOGRA-PSO, ISO-12647-2, EU-GPSR-2023-988, RO-OUG-21-2021, EU-GDPR packs |
| `observability` | Captures the press telemetry stream (ink density, registration, sheet count, ΔE2000 history) |
| `production-planning` | Cancels the remaining schedule slot; re-plans the recovered run on the night shift; cascades to downstream batches |
| `notes` | Diana's bilingual RO/EN root-cause writeup; collaborative editing with her father Mihai |
| `learning-management` | Pulls Diana's FOGRA-PSO Operator Level 2 cert (issued 2024-09-18, valid through 2027-09-18) to populate the line-stop permit context |
| `crm` | Updates Antibiotice's account record with the open recall; SLA timer for response |
| `contract-lifecycle-management` | Surfaces the Tipografia ↔ Antibiotice MSA terms: defect liability clause §7.4 (pharma annex) |
| `plant-maintenance` | Logs the press's mid-run mechanical inspection (the suspected root-cause is dampener-roller cylinder #4 wear) |

## Pack overlays

| Pack | Activation reason |
|---|---|
| ISO-9001-QMS | Tipografia is ISO-9001 certified; full QMS retention + recall audit |
| ISO-12647-2 | Sheet-fed offset print process standard; defines ΔE2000 tolerance bands |
| FOGRA-PSO | Process-standard offset; Diana's operator certification + tolerance reference |
| EU-GPSR-2023-988 | General Product Safety Regulation; pharma-leaflet defects fall under Article 33 recall obligation |
| RO-OUG-21-2021 | Romanian consumer protection ordonanță; recall transparency requirements |
| EU-GDPR | Customer + employee personal data in recall correspondence; standard processing |
| EU-Falsified-Medicines-Directive-2011-62 | Pharma-leaflet is a "package leaflet" under FMD; leaflet defects can implicate FMD if they obscure tamper-evidence text |
| HU-Hungarian-locale-pack | Customer's RO subsidiary employs HU-speaking staff in Cluj region; messenger thread supports HU + RO + EN |

## Regulatory anchors

1. EU-GPSR Regulation (EU) 2023/988 Article 33 — Mandatory product safety recalls
2. RO-OUG nr. 21/2021 — Consumer protection ordonanță (recall transparency)
3. ANMDMR Hot Brand 2023/12 — Pharmaceutical packaging defect notification timelines (≤24 hr to regulator)
4. EU Directive 2001/83/EC + Falsified Medicines Directive 2011/62/EU — Package leaflet legal requirements
5. ISO 9001:2015 §10.2 Nonconformity + corrective action
6. ISO 12647-2:2013 §4.5 — Offset color tolerances
7. FOGRA PSO 2024 — Process Standard Offset operator authority schedules
8. ADR-0311 dual-tenant identity boundary (Tipografia tech vs Antibiotice customer)
9. ADR-0244 tenant scoping
10. ADR-0263 audit dual-seal on cross-tenant transitions

## Cell + certification matrix

| Cell | Certification | Journey use |
|---|---|---|
| `eu-bucharest-primary` | EU-GDPR + ISO 27001 + ISO 9001 | Primary cell for both Tipografia + Antibiotice tenants (RO data residency) |
| `eu-frankfurt-secondary` | EU-GDPR + ISO 27001 | DR replica |
| `eu-amsterdam-readonly-replica` | EU-GDPR | Cross-region read replica for analytics |

## Cedar production line-stop policy (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
// Operator-level line-stop authority — Cedar gates on cert + active shift + role
permit (
    principal == User::"diana.lazăr@tipografia-lazar-petrescu-ro",
    action in [
        Action::"quality.production_line_stop",
        Action::"quality.batch_quarantine",
        Action::"quality.defect_classify",
        Action::"workflow.recall_initiate"
    ],
    resource is ProductionLine
) when {
    resource.tenant_id == "tipografia-lazar-petrescu-ro" &&
    principal.has_certification_unexpired("FOGRA-PSO-Operator-Level-2") &&
    principal.has_certification_unexpired("ISO-12647-2-Trained") &&
    principal.role_in_tenant("tipografia-lazar-petrescu-ro") == "press_operator_day_shift" &&
    context.shift_active == true &&
    context.product_class in ["pharma_PIL", "food_label", "medical_device_label", "general_print"]
};

// CRITICAL: NO manager approval required for line stop on regulated product class
// This is a deliberate Cedar invariant — the operator's certification IS the authority
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J157-001 | Diana initiates line stop within 4 min of GMI alert; audit `EVT-J157-LINE-STOP-001` sealed in `tipografia-lazar-petrescu-ro` |
| AC-J157-002 | Press control system receives stop command + halts cleanly; sheets in transit are accounted for; audit `EVT-J157-PRESS-HALT-CONFIRMED-002` |
| AC-J157-003 | Recall workflow `recall-bch-2027-02-23-0612-pharma-leaflet-NSAID-RO` materializes 14 tasks; each carries Cedar context |
| AC-J157-004 | Defect classification via `quality-management` records ΔE2000 trajectory + 1.2mm registration shift + truncated-warning-text severity flag |
| AC-J157-005 | Cross-tenant customer notification to Antibiotice within 90 min of stop; audit `EVT-J157-CUSTOMER-NOTIFY-005` dual-sealed |
| AC-J157-006 | Customer response (Dr. Munteanu) recorded; recall scope confirmed; audit `EVT-J157-CUSTOMER-CONFIRM-006` |
| AC-J157-007 | Regulator notification path opened (ANMDMR template) — confirmed not yet triggered because batch never left QA quarantine; audit `EVT-J157-REGULATOR-PATH-PREPARED-007` |
| AC-J157-008 | Tipografia's correction + corrective-action + preventive-action (CAPA) plan filed; audit `EVT-J157-CAPA-FILED-008` |
| AC-J157-009 | Root-cause confirmed (dampener-roller cylinder #4 wear); plant-maintenance work order issued; audit `EVT-J157-ROOT-CAUSE-CONFIRMED-009` |
| AC-J157-010 | Diacritic fidelity tests pass — "Lazăr" never normalized to "Lazar" in any persisted field; ISO-9001 audit shows full diacritic preservation |
| AC-J157-011 | Shift handoff at 20:17 EET to night-shift operator Vladimir Csikós; full recall state visible; audit `EVT-J157-SHIFT-HANDOFF-011` |

## Cross-references

- Persona dossier: `docs/personas/print-operator-diana-lazar.md`
- MASTER-ROSTER §3.4 row 101
- Matrix §10 j157 recommendation
- Related: j155 (gray-collar dual-role), j156 (gray-collar after-hours emergency), j124 (supply-chain disruption emergency), j121 (B2B contract escalation)
- Pack roster: `packs/iso-9001/`, `packs/iso-12647-2/`, `packs/fogra-pso/`, `packs/eu-gpsr/`, `packs/ro-oug-21-2021/`, `packs/eu-fmd/`
- ADR-0244 tenant scoping
- ADR-0263 audit dual-seal
- ADR-0311 dual-tenant identity boundary

## Stop condition

This journey is complete when all 11 acceptance criteria pass on the seeded two-tenant fixture, the recall workflow reaches `closure_post_mortem`, the customer notification chain dual-seals, the CAPA plan files in the QMS, the diacritic preservation invariant holds across all persisted fields, and the ISO-9001 audit chain is reconstructible 7 years forward with deterministic merkle proofs.
