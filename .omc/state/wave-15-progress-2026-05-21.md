# Wave 15 Remediation Progress Snapshot — 2026-05-21 (pre-compact)

This file captures Wave 15 dispatch state at the pre-compact moment. Durable on disk + memory sibling `feedback_realignment_review_findings_2026_05_21.md`.

## Completed Wave 15 sub-waves (~46 agents)

### Doctrine ADRs (5 docs, 8,826 lines total)
| ADR | Lines | Status |
|---|---:|---|
| ADR-0329 tier-system-retired-replaced-by-tenant-class | 2,555 | ✅ LANDED |
| ADR-0330 tenant-class-demo-trial-vs-paid-composable-billing-components | 2,048 | ✅ LANDED |
| ADR-0331 cross-microservice-tenant-class-adoption-template | 1,137 | ✅ LANDED |
| ADR-0332 healthcare-domain-decomposition | 1,466 | ✅ LANDED |
| ADR-0333 cell-microservice-retired-pattern-not-service | 620 | ✅ LANDED |

### Big-8 + legal-complexity µservice rewrites (5 µservices, 305/357 P0 closure = 85%)
| µservice | P0 closed | Total lines authored |
|---|---|---:|
| crm | 78/94 (83%) | 3,790 |
| marketing-automation | ~70/103 (~68%) | 2,114 + 25 new IPs |
| contract-lifecycle-management | 100/100 (100%) | 6,500+ across 50 new files |
| itsm | 33/33 (100%) | 1,614 + 5 new Rust crates |
| performance-management | 24/27 (89%) | 30+ new capability YAMLs + 6 cross-handoff AsyncAPIs |

### Wave 15A-batch-2 µservice remediation (6 µservices)
| µservice | P0 closed | Notes |
|---|---|---|
| cloud-billing | 12/12 (spec-sprint) | PRD 786 + ARCH 1042 + README 418 + OpenAPI 993 + AsyncAPI 438 + proto 699 + 6 iac contexts + REMEDIATION-NOTES 659 |
| marketplace | 7/7 (100%) | 6-category surfaces (24 artifacts) + 11 rev-share IPs + 12/12 ADR-0331 tenant_class surfaces |
| identity | 4/4 (100%) | 7 OpenTofu modules + supported-oses.json + Cedar tenant_class binding + 1,900 lines |
| data-warehouse | 6/6 estimated | REMEDIATION-NOTES 407 lines |
| messenger | 3/3 (100%) | 6 iac contexts + iac/terraform/ removed + REMEDIATION-NOTES 82 lines |
| data-pipeline | 3/3 (100%) | IP-031..IP-037 all 7 substantive + REMEDIATION-NOTES 337 lines |

### Healthcare domain decomposition (6 new µservices, ADR-0132 enforcement)
| µservice | Lines authored | Bounded contexts | Capability rows |
|---|---:|---:|---:|
| emr (Electronic Medical Records) | 1,841 + 14 categories | 15 | 128 |
| diagnostics (lab + pathology — imaging stripped post-15M-reconcile) | 1,833 | 15 | 130 |
| emergency (ED-IS) | 1,761 | 17 | 113 |
| pharmacy (med mgmt + ePrescribe + DEA EPCS) | 1,958 | 20 | 150 |
| patient-monitoring (vital signs + RPM + ICU + ML) | 2,284 | 18 | 212 |
| imaging (DICOMweb + VNA + AI marketplace) | 7,720 across 57 files | 24 | 200 |
| **TOTALS** | **~17,400 lines** | **~109** | **~933** |

Plus: healthcare-integration NARROWED to 3 broker contexts (fhir-broker / hl7v2-broker / dicom-broker) per ADR-0332.

### Architectural retirements (3 retirements)
| Retirement | Mechanism | Output |
|---|---|---|
| Wave 15K (network → community merge) | Codex | network/RETIRED.md exists; LinkedIn-class content migrated into community 4-pillar |
| Wave 15L (cell µservice retire) | Codex | cell/RETIRED.md + ADR-0333 (620L) + oya-shuffle-sharding Rust crate authored; cell concerns absorbed by tenancy + cloud-iac + observability + api-gateway + audit-chain |
| Wave 15M-reconcile (imaging-diagnostics split) | Codex | diagnostics imaging contexts stripped; ADR-0332 amendment adds imaging as 8th healthcare µservice |

### Wave 15J tier-vocabulary scrub (21 µservices)
| Batch | µservices |
|---|---|
| Batch-1 (8) | global-trade / plant-maintenance / translate / tasks / design-collaboration / analytics / shorts / mail |
| Batch-2 (5) | workflow-engine / workflow-studio / ontology / api-gateway / intelligence |
| Batch-3 (8) | cloud-iam / cloud-iac / cloud-data / cloud-storage / tenancy / audit-chain / governance / observability |

All 21 had `capability-tiers/` directories deleted. capability_tiers manifest fields removed. tenant_class adoption per ADR-0331 substrate added.

### Wave 15 cumulative totals

| Metric | Count |
|---|---:|
| **Total Wave 15 agents complete** | **~46** |
| Lines authored | ~85,000+ |
| µservices touched | 47 of 77 (61%) |
| P0 findings closed | ~340+ of ~467 (~73%) |
| Tier-vocabulary scrubbed µservices | 21 (codex) + ~10 (Claude rewrites via tier-retirement compliance) = **~31 of 77 (40%)** |
| New µservices authored | 6 healthcare + 1 oya-shuffle-sharding Rust crate |
| Doctrine ADRs authored | 5 (0329 / 0330 / 0331 / 0332 / 0333) |
| Architectural retirements | 3 (network / cell / imaging-split) |

## Remaining Wave 15 work (queued)

### Wave 15J-batch-4 — tier-scrub remaining µservices (~25)

Targets NOT YET scrubbed:
- cloud-kms (Wave 2 audit)
- cloud-secrets (Wave 2)
- cloud-network (Wave 2)
- cloud-network-dns (Wave 2)
- cloud-billing-tax (Claude R1)
- cloud-k8s (Claude R1)
- compliance (Wave 3 B1)
- payments (Wave 3 B1)
- finops-portal (Wave 3 B1)
- application (Wave 3 B2)
- developer-sdk (Wave 3 B2)
- consent-graph (Wave 4 rolling)
- detection (Wave 4 rolling)
- drive / calendar / meet / recordings / notes / docs / sheets / slides / forms / connect / comms-email / community / shorts (NOTE: shorts done in batch-1) — most Wave 4 rolling
- social / sites / plugin-app-store / workplace-integration / ops-dashboard-control-center (Wave 4 rolling)
- incident-management / learning-management / contact-center / supply-chain-planning / production-planning / quality-management / treasury / healthcare-integration (Wave 4 rolling)
- real-estate / warehouse (Wave 4 rolling, codex R3)

Plan: 8-12 codex agents per batch, ~3 more batches needed to cover everything.

### Wave 15-IP-substance — corpus-wide stamped-IP conversion

Wave 4 audits found ~25 stamped 55-line IPs per µservice across many µservices. Wave 15 µservice rewrites addressed some inline (crm/marketing-automation/itsm have substantive new IPs); most others still have stamped IPs.

Plan: 8-15 codex agents to convert stamped IPs to substance across remaining µservices.

### Wave 15-CA-VERIFY — ADR-0105 13-layer compliance audit

Sample finding: many µservices declared only 9/13 of ADR-0105 layers. Wave 15 rewrite agents picked up 13-layer where prompted; new µservices used 12-layer or 13-layer mixed. Needs verification + remediation per µservice.

Plan: 1 Claude agent for corpus-wide compliance audit + per-µservice gap report.

### Wave 15I — foundry retirement + Hermes drop

`microservices/intelligence/` µservice retires per ADR-0247 + the prior session's foundry-absorption doctrine. Capability distributed to intelligence + workflow-engine + workflow-studio + ontology + governance/tenancy (already done in Wave 4 audits via foundry-absorption dimension).

Plan: 1 Claude agent for retirement + cross-reference update + Hermes terminology drop.

### Wave 15O — shorts → social absorption (UNCONFIRMED)

User asked "shorts is social?" earlier. I proposed Wave 15O merge but user never confirmed.

Status: AWAITING USER DECISION.

### Wave 14 — canonical aggregation polish

Promote running `.omc/state/wave-findings-aggregation-2026-05-21.md` → canonical Wave 14 deliverable at `.omc/state/wave-14-aggregation.md` with: per-phase rollup / P0-P3 prioritized remediation backlog / Wave 15A-L sub-wave routing / cross-µservice systemic patterns / tenant-class adoption plumbing IP catalog / counterpart-research database.

Plan: 1 Claude agent for polished canonical aggregation deliverable.

## Cross-cutting patterns confirmed during Wave 4 audits (durable)

1. **Template-stamping rate 58%** — 15 of 26 Claude audits showed industrial template-stamping (crm worst with 169+90+327+30 stamped rows). Lane 2 trace's "surface-wave coordination" hypothesis CONFIRMED.

2. **Tenant-class adoption gap was UNIVERSAL** — all 77 audited µservices flagged it. Wave 15 ADR-0331 + Wave 15A µservice plumbing partially addresses; Wave 15-IP-substance / Wave 15A-batch-N follow-ups close per-µservice gaps.

3. **Tier-vocabulary scope larger than projected** — original estimate 9,300 character-occurrences = 1,680 distinct call-sites. Codex audits found ~1,900 distinct sites in just 12 µservices = ~3,000+ corpus-wide. Wave 15J in 31 µservices so far; ~46 remain.

4. **Counterpart-mismatch sub-pattern** — 4 µservices used wrong counterparts (healthcare/learning-mgmt/quality-mgmt/treasury used ERP-suite vendors instead of domain best-of-breed). Wave 15A µservice rewrites corrected these.

5. **Strong-kernel-weak-docs sub-pattern** — 7+ µservices had hyperscaler-grade kernels with template-stamped docs (cloud-billing 1030L kernel + zero spec / data-pipeline / healthcare-integration / marketplace / quality-management / production-planning / treasury / contract-lifecycle-management). Wave 15B (cloud-billing) + Wave 15A µservice rewrites preserve kernel + replace stamped surfaces.

6. **Big-8 elevation pattern** — Big-8 family µservices (crm/itsm/incident-management/performance-management/learning-management/marketing-automation) had high P0 counts due to ADR-0328 §D-20.111-115 P0-elevation. Wave 15A µservice rewrites addressed the 5 highest.

## Post-compact resume protocol

1. Read this file + sibling `feedback_realignment_review_findings_2026_05_21.md` memory
2. Read `.omc/state/wave-findings-aggregation-2026-05-21.md` for per-µservice findings
3. Read `.omc/state/realignment-review-2026-05-21.md` for cross-cutting analysis
4. Check task list for `pending` Wave 15 sub-waves (#53, #55, #56)
5. Verify the 46 completed agents' outputs landed on disk (use `find microservices/ -name "REMEDIATION-NOTES-2026-05-21.md"`)
6. Dispatch remaining Wave 15J-batch-4 / Wave 15-IP-substance / Wave 15-CA-VERIFY / Wave 15I / Wave 14 polish per user direction

## Files this snapshot references (durable)

- `.omc/state/wave-findings-aggregation-2026-05-21.md` — per-µservice findings tally + cross-cutting patterns
- `.omc/state/realignment-review-2026-05-21.md` — mid-Wave-4 review analysis
- `.omc/specs/deep-dive-realign-oyatie-corpus-to-canonical.md` — original spec
- `.omc/plans/realign-oyatie-corpus-plan-2026-05-20.md` — original implementation plan
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` — canonical sequence (4,395 lines)
- `docs/decisions/ADR-0329-tier-system-retired-replaced-by-tenant-class.md` (2,555 lines)
- `docs/decisions/ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md` (2,048 lines)
- `docs/decisions/ADR-0331-cross-microservice-tenant-class-adoption-template.md` (1,137 lines)
- `docs/decisions/ADR-0332-healthcare-domain-decomposition.md` (1,466 lines)
- `docs/decisions/ADR-0333-cell-microservice-retired-pattern-not-service.md` (620 lines)
- `specs/master-plan-sequencing.json` (870 lines)
- `docs/standards/brief-template.md` (1,891 lines)
- 6 new healthcare µservice paths under microservices/{emr,diagnostics,emergency,pharmacy,patient-monitoring,imaging}/
- 5 P0-hotspot µservice REMEDIATION-NOTES at microservices/{crm,marketing-automation,contract-lifecycle-management,itsm,performance-management}/REMEDIATION-NOTES-2026-05-21.md
- 6 Wave 15A-batch-2 µservice REMEDIATION-NOTES at microservices/{cloud-billing,marketplace,identity,data-warehouse,messenger,data-pipeline}/REMEDIATION-NOTES-2026-05-21.md
- 2 retirement markers at microservices/{network,cell}/RETIRED.md
- crates/oya-shuffle-sharding/ (new Rust library for cellular shuffle-sharding)
