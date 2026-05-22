---
doc_class: Wave-Plan
plan_id: healthcare-decomposition-plan-2026-05-21
wave: Wave 15M (Healthcare Domain Decomposition)
sub_waves: [15M-A, 15M-B, 15M-C, 15M-D, 15M-E, 15M-F, 15M-G, 15M-H]
plan_date: 2026-05-21
plan_owner: council-architecture + axis-healthcare-integration + axis-emr (forming)
plan_authority_tier: 1
governing_adr: ADR-0332-healthcare-domain-decomposition.md
related_adrs: [ADR-0131, ADR-0132, ADR-0145, ADR-0244, ADR-0245, ADR-0247, ADR-0250, ADR-0251, ADR-0263, ADR-0316, ADR-0321, ADR-0322, ADR-0327, ADR-0328]
related_specs: [/specs/master-plan-sequencing.json, /specs/per-microservice-flat-layout.json, /specs/cell-certification-level-matrix.json, /specs/compliance-pack-schema.json]
canonical_anchors:
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0332-healthcare-domain-decomposition.md
  - /Users/jasonlee/oyatie/microservices/healthcare-integration/coherence-audit-2026-05-20.md
  - /Users/jasonlee/oyatie/microservices/healthcare-integration/feature-parity-matrix-2026-05-20.md
  - /Users/jasonlee/oyatie/microservices/healthcare-integration/REMEDIATION-NOTES-2026-05-21.md
  - /Users/jasonlee/oyatie/microservices/imaging/PRD.md
  - /Users/jasonlee/oyatie/microservices/imaging/REMEDIATION-NOTES-2026-05-21.md
  - /Users/jasonlee/oyatie/docs/standards/documentation-rigor.md
batch_ceiling: 8 codex agents per ADR-0328 §D-14
dispatch_convention: codex-only per ADR-0328 §D-14 + memory entry feedback_dispatch_ceiling_claude_only_2026_05_20
halt_condition: clean
---

# Wave 15M — Healthcare Domain Decomposition Plan

## §1 Plan summary

Per ADR-0332, decompose the existing `microservices/healthcare-integration/`
microservice (215 features × 14 domains in one µservice — violation of
ADR-0131 single-concern doctrine) into eight new domain-scoped single-
concern µservices and narrow the existing healthcare-integration µservice
to integration substrate concern only.

Eight new µservices: `emr`, `diagnostics`, `imaging`, `emergency`, `pharmacy`,
`patient-monitoring`, `clinical-decision-support`, `care-management`.

Total healthcare-domain µservice count after Wave 15M lands: 9 (eight new
+ one narrowed existing).

This plan owns the IP roster, parallelism strategy, acceptance criteria,
dependency ordering, and verification SLA for Wave 15M.

## §2 Wave 15M decomposition into sub-waves

### §2.1 Sub-wave 15M-A — Authorization (TODAY, 2026-05-21)

**Scope.** Author the three foundational deliverables:

1. `docs/decisions/ADR-0332-healthcare-domain-decomposition.md` (this ADR)
2. `.omc/plans/healthcare-decomposition-plan-2026-05-21.md` (this plan)
3. `microservices/healthcare-integration/REMEDIATION-NOTES-2026-05-21.md`
   (scope-narrowing notes for the existing µservice)

**Agent.** One agent (sole owner) authors all three files in one session.

**Acceptance.** All three files land at the specified paths. ADR-0332
parses as a valid Architecture-Decision-Record per
`docs/templates/adr-template.md`. The plan and remediation notes carry
frontmatter declaring sub-wave membership and anchor set.

**Verification.** `oya gate validate adr-frontmatter` exits 0 against the
new ADR. `oya gate validate plan-anchor-coverage` exits 0 against the
plan.

**Duration.** 1 session (today).

**Exit gate to 15M-B.** ADR-0332 lands in dev branch. The
healthcare-integration µservice carries the remediation-notes file in
its folder.

### §2.2 Sub-wave 15M-B — Eight µservice scaffolds (2026-05-22..2026-05-24)

**Scope.** Create eight new microservice folders with the minimum-viable
anchor set per ADR-0131 §"Canonical folder shape":

```
microservices/<ms>/
  manifest.json
  PRD.md
  ARCHITECTURE.md
  README.md
  CHANGELOG.md
  compliance.md
  contracts/
    openapi/<surface>.yaml (skeleton)
    asyncapi/<surface>.yaml (skeleton)
    proto/<surface>.proto (skeleton)
  slos/
    availability.openslo.yaml (skeleton)
    read-latency.openslo.yaml (skeleton)
    write-latency.openslo.yaml (skeleton)
    audit-emission-lag.openslo.yaml (skeleton)
  policies/  # Cedar skeleton
  catalog/  # empty per-crate row directory (filled in 15M-C)
  runbooks/  # empty (filled in 15M-C)
  iac/  # empty (filled in 15M-C)
  src/  # empty placeholder per ADR-0327 promotion bar (filled later)
  tests/  # empty placeholder
  evidence/multispectrum/  # empty (filled at first multispectrum review)
```

Also narrow the existing healthcare-integration µservice per the
remediation notes: manifest.json scope narrows; PRD.md scope narrows;
bounded-context list reduces; coverage_benchmarks confirmed as Redox /
Mirth Connect / Health Gorilla.

**Agent allocation (8 scaffold agents, then 1 narrow agent).**

| Agent | µservice | Scope |
|---|---|---|
| codex-15M-B-emr | microservices/emr/ | 6-anchor scaffold (manifest + PRD + ARCHITECTURE + compliance + contracts skeleton + slos skeleton + policies skeleton) |
| codex-15M-B-diagnostics | microservices/diagnostics/ | same shape |
| codex-15M-B-imaging | microservices/imaging/ | same shape; authoritative for imaging order, PACS/VNA, DICOM, radiology workflow, read reports, dose, and image AI |
| codex-15M-B-emergency | microservices/emergency/ | same shape |
| codex-15M-B-pharmacy | microservices/pharmacy/ | same shape |
| codex-15M-B-patient-monitoring | microservices/patient-monitoring/ | same shape |
| codex-15M-B-cds | microservices/clinical-decision-support/ | same shape |
| codex-15M-B-care-management | microservices/care-management/ | same shape |
| codex-15M-B-hci-narrow | microservices/healthcare-integration/ | scope narrow per remediation notes after the first scaffold batch clears |

8 scaffold agents run first (within ceiling of 8 codex per ADR-0328 §D-14);
the healthcare-integration narrow agent runs after one batch slot opens.

**Per-agent acceptance criteria.**

For each new µservice:

- `manifest.json` declares `microservice: <slug>`, `audience_type:
  tenant-b2b-healthcare`, `compliance_packs: [HIPAA-2024, SOC-2, ISO-
  27001, GDPR, +per-domain-specific-packs]`, `binding_adrs: [ADR-0105,
  ADR-0131, ADR-0132, ADR-0145, ADR-0244, ADR-0245, ADR-0250, ADR-0251,
  ADR-0263, ADR-0316, ADR-0322, ADR-0332]`, `depends_on_microservices:
  [audit-chain, consent-graph, tenancy, identity, cloud-iam,
  workflow-engine, ontology, governance, observability, healthcare-
  integration]` plus domain-specific peers per the ADR-0332 §D handoff
  matrix.
- `PRD.md` ≥ 400 lines bespoke. Names: bounded contexts (per ADR-0332
  §C), user stories (5+ personas × bounded contexts; each story
  bespoke — no template stamping), functional requirements (one FR per
  bounded-context × verb), non-functional requirements (SLO envelope
  per ADR-0332 §C), top-3 industry counterparts (per ADR-0332 §C),
  capability tier matrix doctrine note.
- `ARCHITECTURE.md` ≥ 700 lines. Names the bounded-context model,
  state machine per bounded context, cross-µservice handoff per the
  ADR-0332 §D matrix, cell residency model, multi-region awareness,
  break-glass posture (where applicable), audit-event class list.
- `compliance.md` ≥ 600 lines. Names HIPAA Security Rule mapping
  (§164.308 / §164.310 / §164.312), per-domain regulatory shape (e.g.,
  CLIA for diagnostics, EMTALA for emergency, DEA EPCS for pharmacy,
  FDA SaMD for clinical-decision-support, CMS Star for care-
  management), tenant-class behaviour table, BAA template binding (per
  ADR-0251).
- `contracts/openapi/<ms>-v1.yaml` skeleton with at least 3 endpoint
  stubs.
- `contracts/asyncapi/<ms>-v1.yaml` skeleton with at least 3 event
  stubs aligned with ADR-0332 §D handoff matrix.
- `contracts/proto/<ms>-v1.proto` skeleton with at least 3 RPC stubs.
- `slos/availability.openslo.yaml`, `slos/read-latency.openslo.yaml`,
  `slos/write-latency.openslo.yaml`, `slos/audit-emission-lag.openslo.
  yaml` skeletons aligned with the SLO envelope declared in
  ADR-0332 §C for that domain.
- `policies/cedar-default-deny-<concern>.cedar` skeleton with the
  PHI-in-demo_trial fragment (per ADR-0332 §F) baked in for every
  µservice.
- `README.md` declares status `reserved-wave-15m-anchor`, owner team
  (`axis-<ms>`), and entry-point links.

For healthcare-integration scope-narrow:

- Manifest `bounded_contexts` reduced to 3 broker-scoped values
  (`fhir-broker`, `hl7v2-broker`, `dicom-broker`).
- `coverage_benchmarks` confirmed as `[Redox, Mirth Connect, Health
  Gorilla]`.
- `compliance_packs` unchanged.
- PRD scope-narrow stub appended at top of PRD.md citing ADR-0332.
- Bounded-context-named files (patient-record, referral, clinical-
  consent) move OUT to the new µservices per ADR-0332 §H. Files remain
  on disk as RETIRED markers with redirect stubs per ADR-0138.

**Verification (per µservice).**

- `oya gate validate per-microservice-layout --microservice <slug>`
  exits 0
- `oya gate validate no-new-suite-bundles` exits 0
- `oya gate validate adr-citation-in-manifest --microservice <slug>`
  exits 0 (each manifest cites ADR-0332)
- `oya gate validate hipaa-pack-declared-on-healthcare-microservice
  --microservice <slug>` exits 0
- `cargo build --workspace` exits 0 (no new crates yet; src/ is empty
  placeholder)

**Duration.** 3 days (parallel dispatch with 8 codex agents).

**Exit gate to 15M-C.** All 8 new µservice scaffolds + 1 narrowed
existing healthcare-integration pass the verification above. ADR-0332
enforcement promotes from advisory to BLOCKER for downstream lanes.

### §2.3 Sub-wave 15M-C — Per-µservice IP roster (2026-05-25..2026-05-31)

**Scope.** Each new µservice gets its own 30-IP roster, mirroring the
shape of the existing healthcare-integration µservice's 30-IP roster
(IP-001..IP-030) but bespoke to the new domain.

**Per-µservice IP roster (universal 25 IPs + 5 domain-specific IPs).**

Universal 25 IPs (same across all 8 new µservices):

1. IP-001 tenant-scope-kernel
2. IP-002 cedar-default-deny
3. IP-003 ontology-projection
4. IP-004 workflow-template-library
5. IP-005 rest-contract-surface
6. IP-006 async-event-surface
7. IP-007 grpc-internal-surface
8. IP-008 policy-eval-library-binding
9. IP-009 credential-sidecar-binding
10. IP-010 multi-region-cell-layout
11. IP-011 observability-audit-events
12. IP-012 abuse-defence-edge-waf
13. IP-013 emergency-services-bypass (or domain-equivalent)
14. IP-014 marketplace-dealset-settlement
15. IP-015 data-residency-pack-overlays
16. IP-016 backfill-replay-worker
17. IP-017 cost-budget-enforcer
18. IP-018 capacity-admission-control
19. IP-019 sdk-client-generation
20. IP-020 catalog-layer-registration
21. IP-021 slo-gated-promotion
22. IP-022 chaos-drill-pack
23. IP-023 dpia-evidence-packet
24. IP-024 threat-model-control-map
25. IP-025 audit-findings-closeout

Domain-specific 5 IPs (varies per µservice):

**emr**:
- IP-026 patient-record-canonical-identity
- IP-027 encounter-state-machine
- IP-028 problem-allergy-immunization-vocabulary-binding
- IP-029 clinical-note-smarttext-substrate
- IP-030 emr-break-glass-justification-canonical

**diagnostics**:
- IP-026 lab-order-routing
- IP-027 lab-result-finalization-chain
- IP-028 lab-result-image-correlation-handoff
- IP-029 pathology-case-synoptic-reporting
- IP-030 reference-range-population-specific-binding

**imaging**:
- IP-026 imaging-order-routing
- IP-027 pacs-vna-object-custody
- IP-028 dicomweb-dimse-substrate-binding
- IP-029 radiologist-worklist-read-report
- IP-030 dose-tracking-image-ai-governance

**emergency**:
- IP-026 esi-triage-acuity-assignment
- IP-027 emtala-mse-gate-cedar-fragment
- IP-028 trauma-activation-protocol-engine
- IP-029 ems-handoff-nemsis-substrate
- IP-030 ed-tracking-board-real-time-substrate

**pharmacy**:
- IP-026 medication-order-interaction-engine
- IP-027 eprescribe-ncpdp-script-surescripts
- IP-028 dea-epcs-two-factor-biometric-binding
- IP-029 bcma-barcode-medication-administration
- IP-030 controlled-substance-custody-chain-audit

**patient-monitoring**:
- IP-026 vital-sign-acquisition-hl7v2-oru-r30
- IP-027 waveform-continuous-capture-substrate
- IP-028 alarm-fatigue-management-tjc-npsg-06-01-01
- IP-029 early-warning-score-computation-engine
- IP-030 rpm-home-device-ingress-substrate

**clinical-decision-support**:
- IP-026 cds-hooks-2.0-service-surface
- IP-027 fhir-clinical-reasoning-ig-binding
- IP-028 evidence-content-bundle-versioned-substrate
- IP-029 bpa-best-practice-advisory-engine
- IP-030 clinician-override-audit-binding

**care-management**:
- IP-026 care-plan-fhir-r5-canonical
- IP-027 care-transition-tcm-cpt-99495-99496-substrate
- IP-028 population-stratification-risk-score-engine
- IP-029 outreach-campaign-tcpa-consent-binding
- IP-030 quality-measure-hedis-mips-star-attribution

**Agent allocation.** Per `feedback_microservice_ownership_coherence_
2026_05_20`, ONE agent owns ONE µservice's 30-IP roster end-to-end
across the 7-day window. Eight agents in parallel.

**Per-IP acceptance criteria.**

- Each IP ≥ 14 KB substantive (per existing healthcare-integration IP
  shape).
- Each IP declares tenant scope, principal, purpose, audit-event
  class, runbook ref, SLO ref, rollback path, observability hooks.
- Each IP cites at least 3 ADRs (ADR-0332 + 2 universal substrate).
- Each IP names the cross-µservice handoff (where applicable) by
  Workflow event class per ADR-0332 §D matrix.
- Domain-specific IPs (026-030) cite the per-domain regulatory
  standard (HIPAA + CLIA / EMTALA / DEA EPCS / FDA SaMD / CMS Star).
- No template stamping per ADR-0322; each IP carries bespoke
  acceptance criteria.

**Verification.**

- `oya gate validate ip-shape --microservice <slug>` exits 0 for each
  of the 30 IPs.
- `oya gate validate cross-microservice-handoff-coherence` exits 0 —
  Workflow event names declared by each IP match the ADR-0332 §D
  matrix.

**Duration.** 7 days (1 agent per µservice, 30 IPs over 7 days,
~4 IPs per day per agent).

**Exit gate to 15M-D.** All 240 IPs (8 µservices × 30 IPs) pass
verification.

### §2.4 Sub-wave 15M-D — Capability-tier matrices (2026-06-01..2026-06-03)

**Scope.** Each new µservice gets a `capability-tiers/tier-matrix.md`
(Bronze / Silver / Gold / Platinum) projection per ADR-0316.

**Per-µservice tier-matrix shape (per ADR-0316).**

- Bronze tier: minimum-viable substrate (e.g., for emr: 100k patient
  record reads/day with 500ms p99; FHIR R4 read-only; no Care
  Everywhere integration).
- Silver tier: production-grade substrate (e.g., for emr: 1M patient
  record reads/day with 200ms p99; FHIR R4+R5 read/write; basic Care
  Everywhere; basic decision-support hooks).
- Gold tier: enterprise-grade substrate (e.g., for emr: 100M patient
  record reads/day with 100ms p99; FHIR R4+R5 full; multi-region
  active-active; full CDS Hooks 2.0; full TEFCA participation).
- Platinum tier: hyperscaler-grade substrate (e.g., for emr: 1B patient
  record reads/day with 50ms p99; FHIR R4+R5 with bring-your-own
  terminology pack; cross-cell longitudinal record assembly; full
  population analytics; full FHIR Bulk Data Export at scale).

**Acceptance.**

- Each tier-matrix ≥ 165 lines (same floor as existing healthcare-
  integration tier-matrix).
- Each tier-matrix names HIPAA pack version, version pinning per
  tenant, retention policy, audit-event class coverage, alarm-fatigue
  rule coverage (where applicable), TJC standard binding (where
  applicable).
- Each tier-matrix carries doctrine note: "capability tier (Bronze /
  Silver / Gold / Platinum) is orthogonal to tenant_class (demo_trial /
  paid). Tenant class gates PHI; capability tier gates feature surface
  and SLO envelope."

**Agent allocation.** 8 codex agents in parallel.

**Duration.** 3 days.

### §2.5 Sub-wave 15M-E — Cross-µservice handoff IP slices (2026-06-04..2026-06-07)

**Scope.** For each row in the ADR-0332 §D handoff matrix, author an
IP slice that wires the producer µservice's emission to the consumer
µservice's intake.

Total handoff rows: ADR-0332 §D row set, including imaging order, read-report,
image-correlation, imaging-charge, and imaging-CDS handoffs (per
ADR-0332 §D matrix).

**Per-handoff IP slice shape.**

- Producer-side IP slice (under producer µservice): emits the Workflow
  event with required schema, audit-event class binding, Cedar
  default-deny binding.
- Consumer-side IP slice (under consumer µservice): consumes the
  Workflow event, validates schema, enforces tenant-scope check,
  binds to audit emission.
- Schema declaration in `contracts/asyncapi/<surface>.yaml` (producer
  side) + `contracts/asyncapi/<surface>.yaml` (consumer side); schema
  must match across both sides.

**Acceptance.**

- Each handoff has matched producer-IP + consumer-IP.
- Schemas align across both sides.
- Workflow event names match the ADR-0332 §D matrix exactly.
- `oya gate validate cross-microservice-handoff-coherence` exits 0
  with the full ADR-0332 §D handoff row set registered.

**Agent allocation.** 4 codex agents in parallel (each agent owns
~8 handoffs).

**Duration.** 4 days.

### §2.6 Sub-wave 15M-F — HIPAA pack scaffold (2026-06-08..2026-06-14)

**Scope.** Author the HIPAA-2024 pack scaffold under
`microservices/governance/packs/HIPAA-2024/v1/` per ADR-0251 §D-2
Stage 1 directory shape.

**Pack scaffold contents.**

```
microservices/governance/packs/HIPAA-2024/v1/
  pack.yaml  # pack metadata + version + signature placeholder
  pack-manifest.json  # pack contents inventory
  agreements/
    baa-template.md  # Business Associate Agreement template
  workflows/
    breach-notification-workflow.yaml  # 60-day rule, HHS OCR notice
    training-acknowledgement-workflow.yaml  # workforce HIPAA training
    individual-rights-workflow.yaml  # §164.524 patient access right
  cedar/
    phi-in-demo-trial-deny.cedar  # PHI cannot exist in demo_trial
    cell-not-hipaa-certified-deny.cedar  # paid tenant on non-cert cell
    phi-cross-tenant-deny.cedar  # PHI cannot cross tenant boundaries
    minimum-necessary-default-deny.cedar  # §164.502(b)
    automatic-logoff-binding.cedar  # §164.312(a)(2)(iv)
  controls/
    administrative-safeguards.md  # §164.308 mapping
    physical-safeguards.md  # §164.310 mapping
    technical-safeguards.md  # §164.312 mapping
    organizational-requirements.md  # §164.314 mapping
    breach-notification.md  # §164.404 + HITECH §13402
  retention/
    audit-retention-six-years.yaml  # §164.316(b)(2)
  iac/
    fips-mode-binding.tf  # FIPS 140-2 cryptographic module declaration
  evidence/
    multispectrum-review/  # F1/F5/F6/F7/F11/A1/A2/A4/A6 facets per ADR-0251 §D-2 Stage 2
```

**Acceptance.**

- BAA template covers permitted uses + disclosures, safeguards
  (admin/phys/tech), reporting (60-day breach notification),
  subcontractor flow-down, termination, return/destroy of PHI.
- Breach-notification workflow names 60-day deadline, HHS OCR notice
  workflow, affected_phi_record_count field, breach_type field,
  mitigation_steps field, individual_notice_text field.
- Training-acknowledgement workflow covers HIPAA Security Awareness
  training cadence (annual minimum).
- Individual-rights workflow covers §164.524 patient access right
  with 30-day response SLO.
- Cedar fragments default-deny.
- Retention policy declares 6-year minimum from later of date of
  creation or date last in effect.
- FIPS mode IaC declares cryptographic module operating in FIPS
  140-2 validated mode (or maps to FIPS-validated provider).

**Agent allocation.** 2 codex agents (one on agreements/workflows,
one on cedar/controls/retention).

**Duration.** 7 days.

**Exit gate.** HIPAA-2024 pack v1 lands. All 9 healthcare-domain
µservices reference it in their manifest.compliance_packs and in their
compliance.md HIPAA section.

### §2.7 Sub-wave 15M-G — Per-domain compliance pack scaffolds (2026-06-15..2026-06-21)

**Scope.** Author per-domain regulatory packs that overlay the HIPAA
pack.

| Pack | Path | Scope | Owner µservice |
|---|---|---|---|
| ONC-170-315-g10-v2024 | `microservices/governance/packs/ONC-170-315-g10-v2024/v1/` | ONC §170.315(g)(10) Standardized API for Patient and Population Services | emr |
| CLIA-CAP-v2024 | `microservices/governance/packs/CLIA-CAP-v2024/v1/` | Clinical Laboratory Improvement Amendments + CAP accreditation | diagnostics |
| DICOM-IHE-ACR-MQSA-v2024 | `microservices/governance/packs/DICOM-IHE-ACR-MQSA-v2024/v1/` | DICOM conformance, IHE Radiology, ACR evidence, MQSA where mammography applies | imaging |
| EMTALA-v2024 | `microservices/governance/packs/EMTALA-v2024/v1/` | Emergency Medical Treatment and Labor Act 42 USC §1395dd | emergency |
| DEA-EPCS-21-CFR-1311-v2024 | `microservices/governance/packs/DEA-EPCS-21-CFR-1311-v2024/v1/` | DEA Electronic Prescriptions for Controlled Substances | pharmacy |
| FDA-SaMD-v2024 | `microservices/governance/packs/FDA-SaMD-v2024/v1/` | FDA Software as a Medical Device classification + September 2022 CDS guidance | clinical-decision-support + patient-monitoring (joint binding) |
| CMS-STAR-MIPS-HEDIS-v2024 | `microservices/governance/packs/CMS-STAR-MIPS-HEDIS-v2024/v1/` | CMS Star Ratings + MIPS + HEDIS reporting | care-management |
| 21CCA-INFO-BLOCKING-v2024 | `microservices/governance/packs/21CCA-INFO-BLOCKING-v2024/v1/` | 21st Century Cures Act information-blocking 45 CFR §171 | emr + healthcare-integration (joint binding) |
| TEFCA-v2024 | `microservices/governance/packs/TEFCA-v2024/v1/` | Trusted Exchange Framework and Common Agreement | healthcare-integration |
| TCPA-v2024 | `microservices/governance/packs/TCPA-v2024/v1/` | Telephone Consumer Protection Act outreach consent | care-management |
| KR-MED-LAW-v2024 | `microservices/governance/packs/KR-MED-LAW-v2024/v1/` | KR 의료법 + 의료정보보호 | all 9 healthcare-domain µservices (KR tenant overlay) |
| EU-MDR-v2024 | `microservices/governance/packs/EU-MDR-v2024/v1/` | EU Medical Device Regulation 2017/745 | clinical-decision-support + patient-monitoring (EU tenants) |
| EU-AI-ACT-HEALTHCARE-v2024 | `microservices/governance/packs/EU-AI-ACT-HEALTHCARE-v2024/v1/` | EU AI Act Annex III high-risk clinical AI | clinical-decision-support (EU tenants) |

**Acceptance.** Each pack scaffold mirrors the HIPAA-2024 pack shape
(pack.yaml + pack-manifest.json + agreements/ + workflows/ + cedar/ +
controls/ + retention/ + iac/ + evidence/multispectrum-review/) but
scoped to the domain regulation.

**Agent allocation.** 6 codex agents in parallel (each agent owns 2
packs).

**Duration.** 7 days.

### §2.8 Sub-wave 15M-H — Cleanup + RETIRED-stub removal (2026-06-22+)

**Scope.** Once all 15M-A..15M-G sub-waves land and the 9 healthcare-
domain µservices reach substance-bar substance, retire the RETIRED-
marker stub files that the scope-narrow remediation left behind. Per
ADR-0138 six-path deprecation pattern, RETIRED stubs deprecate AFTER
all callers update.

**Duration.** 1 day (cleanup only).

## §3 Parallelism Strategy

### §3.1 Within-sub-wave parallelism

Each sub-wave runs as many codex agents in parallel as possible
within the ADR-0328 §D-14 batch ceiling of 8.

| Sub-wave | Parallel agents | Max simultaneous |
|---|---|---|
| 15M-A | 1 | 1 (sole owner) |
| 15M-B | 8 | 8 |
| 15M-C | 8 | 8 (one per new µservice; each owns 30 IPs end-to-end) |
| 15M-D | 8 | 8 (one per new µservice) |
| 15M-E | 4 | 4 (handoff slicing) |
| 15M-F | 2 | 2 (HIPAA pack scaffold) |
| 15M-G | 6 | 6 (per-domain compliance packs) |
| 15M-H | 1 | 1 (cleanup) |

### §3.2 Across-sub-wave dependencies

```
15M-A
  ↓ exit gate
15M-B (parallel: 8 µservice scaffolds, then 1 scope-narrow)
  ↓ exit gate
15M-C (parallel: 8 µservice 30-IP rosters)
  ↓ exit gate (CAN start in parallel with 15M-D)
15M-D (parallel: 8 µservice tier matrices)
  ↓
15M-E (parallel: ~4 handoff slicing agents)
  ↓
15M-F (HIPAA pack scaffold)
  ↓ (CAN start in parallel with 15M-G after 15M-F midpoint)
15M-G (parallel: 6 per-domain compliance packs)
  ↓
15M-H (cleanup)
```

### §3.3 µservice authoring dependency ordering within 15M-B

Per ADR-0332 §G, emr is the foundation for cross-µservice handoffs
(patient identity is emr-owned; encounter is emr-owned). Within
15M-B, emr scaffold lands FIRST, then diagnostics + pharmacy +
emergency + patient-monitoring + clinical-decision-support + care-
management land in parallel (depending only on emr's patient identity
contract being available in skeleton form).

Within 15M-C, the same dependency: emr's IPs land first; the other
six µservices' IPs may reference emr's gRPC surface and Workflow
events.

## §4 Acceptance criteria per µservice

A new µservice is "scaffolded" (15M-B exit) when:

1. `microservices/<ms>/manifest.json` exists with the binding ADRs +
   tenant_class + audience_type + compliance_packs declared.
2. `microservices/<ms>/PRD.md` ≥ 400 lines bespoke.
3. `microservices/<ms>/ARCHITECTURE.md` ≥ 700 lines bespoke.
4. `microservices/<ms>/compliance.md` ≥ 600 lines bespoke with HIPAA
   Security Rule mapping + per-domain regulatory shape.
5. `microservices/<ms>/contracts/` skeleton present (openapi/asyncapi/
   proto each with ≥ 3 stubs).
6. `microservices/<ms>/slos/` skeleton present (≥ 4 OpenSLO files).
7. `microservices/<ms>/policies/` skeleton present (≥ 3 Cedar
   fragments including PHI-in-demo_trial deny).
8. `microservices/<ms>/README.md` declares reserved status, owner
   team, anchor links.
9. `microservices/<ms>/CHANGELOG.md` declares Wave 15M-B scaffold as
   first entry.

A new µservice is "IP-roster complete" (15M-C exit) when:

10. 30 IPs (IP-001..IP-030) authored per the 25-universal + 5-domain
    shape in §2.3.
11. Each IP ≥ 14 KB substantive.
12. Cross-µservice handoffs match the ADR-0332 §D matrix.

A new µservice is "tier-matrix complete" (15M-D exit) when:

13. `capability-tiers/tier-matrix.md` ≥ 165 lines authored with
    Bronze/Silver/Gold/Platinum projection.

A new µservice is "handoff-wired" (15M-E exit) when:

14. Per-handoff IP slices land for every row in the ADR-0332 §D
    matrix that names this µservice as producer or consumer.
15. AsyncAPI schemas align across producer + consumer sides.

A new µservice is "HIPAA-pack-bound" (15M-F exit) when:

16. manifest.json references HIPAA-2024 pack v1 at the new pack
    registry path.
17. compliance.md cross-references the pack's BAA template + breach-
    notification workflow + Cedar fragments + retention policy.

A new µservice is "per-domain-pack-bound" (15M-G exit) when:

18. manifest.json references the applicable per-domain pack(s) from
    §2.7 table.
19. compliance.md cross-references each per-domain pack's controls.

A new µservice is "substance-bar-ready for Phase 4 promotion" (post
15M-H + Phase 4 substance bar gate) when:

20. Multispectrum review evidence captured at
    `microservices/<ms>/evidence/multispectrum/`.
21. `oya gate validate microservice-coherence-audit --microservice
    <slug>` exits 0.
22. `oya gate validate substance-bar` exits 0.
23. `oya gate validate cross-ref-validity` exits 0.

## §5 Dependencies between µservices

### §5.1 Identity foundation

emr owns the canonical patient identity. The MPI substrate in
healthcare-integration matches external EHR identities to emr's
canonical patient identifier. All other domain µservices reference
emr's patient identifier in their bounded contexts.

Implication for Wave 15M-B ordering: emr's `manifest.json` +
PRD bounded-context-`patient` declaration MUST land before the other
six µservices declare their patient-referencing bounded contexts.

### §5.2 Encounter foundation

emr owns the canonical encounter (inpatient stay, outpatient visit,
ED visit, telehealth visit). diagnostics, pharmacy, patient-
monitoring, clinical-decision-support, and care-management reference
emr's encounter identifier.

emergency owns the ED-specific encounter sub-type but emits to emr's
canonical encounter (ED visit becomes an encounter in emr; emergency
provides the ED-specific lifecycle and tracking board on top).

### §5.3 Medication-list foundation

pharmacy owns the active medication list. clinical-decision-support
reads it (does not write it). emr reads it for the patient summary
view (does not write it). diagnostics reads it for TDM result-to-
dose-adjust loop.

### §5.4 Lab-result foundation

diagnostics owns the canonical lab result. clinical-decision-support
reads it for dose adjust + critical-value rules. emr reads it for
patient summary view. emergency reads it for STAT result routing.
care-management reads it for gap-in-care detection.

### §5.5 Vital-sign foundation

patient-monitoring owns the continuous vital-sign stream + waveform
storage. emr owns the point-in-time vital sign (manually entered by
nurse or read from patient-monitoring's rolled-up event). clinical-
decision-support reads the early-warning-score feed from patient-
monitoring.

### §5.6 Care-plan foundation

care-management owns the care plan. emr reads it for patient summary
view. patient-monitoring reads it for RPM enrollment context. The
care plan references emr's problem list (read).

### §5.7 Consent foundation

consent-graph (existing substrate µservice) owns the canonical
consent state. All 9 healthcare-domain µservices verify consent via
gRPC sync call to consent-graph. healthcare-integration enforces
consent segmentation at broker boundary.

### §5.8 Audit foundation

audit-chain (existing substrate µservice) owns the canonical
tamper-evidence audit log. All 9 healthcare-domain µservices emit
events to audit-chain per ADR-0263.

### §5.9 Identity / Cedar foundation

cloud-iam (existing substrate µservice) owns the principal binding
(user / agent / role / caregiver). care-management uses cloud-iam to
bind caregiver roles to patients. emr / emergency / pharmacy use
cloud-iam for clinician role binding.

policy-engine (existing substrate µservice) evaluates Cedar fragments
at every gate.

### §5.10 Workflow / Ontology foundation

workflow-engine (existing substrate µservice) routes the cross-
µservice Workflow events per ADR-0332 §D.

ontology (existing substrate µservice) holds the patient / encounter /
medication / lab / imaging / care-plan Object Types.

## §6 Verification SLA

Per ADR-0328 §D-10 verification SLA:

- Each sub-wave ends with a `done` claim only when ALL acceptance
  criteria for that sub-wave pass `oya gate validate ...`.
- Audit-event class emission proves the landing happened.
- A sub-wave that fails verification is not retried in place; a
  bounded fix IP authors against the affected deliverable.

### §6.1 Per-sub-wave verification

| Sub-wave | Verification command | Expected exit |
|---|---|---|
| 15M-A | `oya gate validate adr-frontmatter --adr ADR-0332` | 0 |
| 15M-A | `oya gate validate plan-anchor-coverage --plan healthcare-decomposition-plan-2026-05-21.md` | 0 |
| 15M-B | `oya gate validate per-microservice-layout --microservice <each>` | 0 |
| 15M-B | `oya gate validate no-new-suite-bundles` | 0 |
| 15M-B | `oya gate validate hipaa-pack-declared-on-healthcare-microservice --microservice <each>` | 0 |
| 15M-C | `oya gate validate ip-shape --microservice <each>` | 0 for all 30 IPs |
| 15M-C | `oya gate validate cross-microservice-handoff-coherence` | 0 |
| 15M-D | `oya gate validate capability-tier-matrix-coherence --microservice <each>` | 0 |
| 15M-E | `oya gate validate cross-microservice-handoff-coherence` | 0 with the full ADR-0332 §D row set |
| 15M-F | `oya gate validate hipaa-pack-readiness` | 0 |
| 15M-G | `oya gate validate per-domain-pack-readiness --pack <each>` | 0 |
| 15M-H | `oya gate validate cleanup-no-broken-references` | 0 |

### §6.2 Wave-level verification

After 15M-H lands, the Wave 15M aggregate verification runs:

- `oya gate validate microservice-coherence-audit --microservice <each
  of 9 healthcare-domain µservices>` exits 0.
- `oya gate validate substance-bar` exits 0 across all 9.
- `cargo build --workspace` exits 0.
- `cargo nextest run --workspace` exits 0.
- Wave 15M aggregate finding ledger: zero P0 contradictions; zero
  unresolved P1 substance-bar failures.

## §7 Risk + mitigation

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Codex agent dispatch ceiling exceeded mid-wave | Medium | Slows 15M-B | Strict batch ceiling enforcement; queue agents 9..N for next batch slot |
| µservice scaffold drift between agents (e.g., divergent manifest schemas) | Medium | Quality | Single authoritative template per ADR-0131; manifest schema validation on every push |
| HIPAA pack scaffold blocks 15M-F exit | Medium | High (downstream µservices wait) | Author HIPAA pack scaffold as a hard precondition; 2 dedicated agents on 15M-F |
| Cross-µservice handoff schema mismatch (producer + consumer drift) | High | Medium | AsyncAPI schema validation at producer + consumer; ADR-0332 §D matrix is normative reference |
| Per-domain regulatory pack misses key control | Medium | Medium (audit gap) | Each pack ships with named precedent citation per documentation-rigor §1.1; reviewer-agent enforces precedent presence |
| Substance bar regression — agent template-stamps under deadline pressure | Medium | High (P0 finding) | ADR-0322 substance-bar BLOCKER lane enforced on every PR; no template stamping allowed |
| Healthcare-integration scope-narrow leaves dangling references | High | Low (cosmetic) | 15M-H cleanup sub-wave dedicated to RETIRED stub removal + reference update |
| Behavioural health / surgery / anesthesia open questions force re-scope | Low | High (re-decomposition) | Defer per ADR-0332 §J Open Questions; future Wave 16+ decision; current 7-µservice scope locked |

## §8 Communication + escalation

- All sub-wave dispatches use codex-only per ADR-0328 §D-14 +
  memory entry `feedback_dispatch_ceiling_claude_only_2026_05_20`.
- Per-sub-wave halt-condition: a sub-wave HALTS CLEAN on completion
  of all acceptance criteria; otherwise HALTS WITH ESCALATION to
  council-architecture + axis-healthcare-integration.
- Escalation path: P0 contradiction in a new µservice → halt the
  sub-wave + dispatch a one-shot fix agent + re-verify.
- Per-µservice ownership: ONE codex agent owns ONE µservice across
  15M-B, 15M-C, 15M-D (per `feedback_microservice_ownership_
  coherence`). Handoff agents in 15M-E may overlap across µservices.

## §9 Halt condition

Wave 15M halts CLEAN when:

1. All 8 new microservice folders exist under `microservices/` with
   the full 15M-B..15M-G artifact set.
2. The existing `healthcare-integration` µservice scope is narrowed
   per the remediation notes.
3. All ADR-0332 §I verification gates exit 0.
4. Multispectrum review evidence captured for all 9 µservices.
5. Zero P0 findings unresolved.

End of Wave 15M plan.
