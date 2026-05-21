---
doc_class: CoherenceAudit
microservice: production-planning
audit_wave: wave-4-rolling
audit_date: 2026-05-20
authored_at: 2026-05-21
auditor_persona: muservice-ownership-coherence-audit-agent
owner_team: axis-production-planning + axis-erp-parity
phase: 4-enterprise-vertical
service_class: APS (Advanced Planning & Scheduling)
top_3_counterparts:
  - SAP APO (Advanced Planner & Optimizer) PP/DS + SAP S/4HANA PP/DS
  - Oracle Supply Chain Planning Cloud (Production Scheduling + Constraint-Based Optimization)
  - Kinaxis Maestro (production scheduling within RapidResponse / concurrent planning)
canonical_anchors:
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md §D-15..§D-20 (multi-context + IaC + OS + Rust + OCI Free + audit-agent dims)
  - /Users/jasonlee/oyatie/docs/standards/brief-template.md §3.4 per-muservice-ADR-author + §3.4.T/§3.4.C/§3.4.M (Tenant/Context/MRP substance gates)
  - /Users/jasonlee/oyatie/microservices/production-planning/PRD.md §A..§F + §C stories PP-001..PP-040
  - /Users/jasonlee/oyatie/microservices/production-planning/ARCHITECTURE.md §C bounded-context architecture + §D integration topology
  - /Users/jasonlee/oyatie/microservices/production-planning/manifest.json bounded_contexts + binding_adrs roster
related_adrs:
  - ADR-0105 13-layer enum
  - ADR-0131 per-microservice flat layout
  - ADR-0132 no-suite microservices
  - ADR-0145 inter-microservice direct gRPC
  - ADR-0243 Cedar universal gate
  - ADR-0244 tenant universal scoping primitive
  - ADR-0245 substrate vs product layering
  - ADR-0248 Amazon-shape cellular
  - ADR-0252 HLC default
  - ADR-0253 HTTP/3 + QUIC default
  - ADR-0263 observability emission contract
  - ADR-0314 marketplace settlement
  - ADR-0315 SAP module parity
  - ADR-0316 tenant-class activation (NOTE: tier doctrine retired per 2026-05-20 directive — tenant_class composable replaces tier deltas)
  - ADR-0328 substance bar + multi-context + IaC + OS + Rust + OCI Free
doctrine_notes:
  - tier_retired: tenant-class as a delta primitive is retired; tenant_class (demo/sandbox/trial/dev/paid/byo-cloud/self-hosted/multi-tenant) carries composable activation per specs/master-plan-sequencing.json#deployment_contexts
  - performance_target_model: single industry-leader target + deployment-context overlay + tenant-class overlay; no tier-delta numbers
  - aps_substance_required: this audit MUST exercise MRP, MPS, finite scheduling, dispatching, change-over optimization, bottleneck management beyond CRUD shape
---

# Coherence Audit: production-planning (Phase 4 APS)

## §1 Identity, Ownership, Scope, and Audit Posture

### §1.1 Service identity
`microservices/production-planning/` is the Phase 4 enterprise vertical microservice that implements the **Advanced Planning & Scheduling (APS)** layer above the supply-chain plan and below the shop-floor MES. It owns six bounded contexts: `bom-revision`, `mrp-run`, `capacity-calendar`, `routing-step`, `production-order`, and `shop-floor-release`. The service slug `production-planning` complies with BNF v4.1 kebab-case microservice slug grammar and the layer enum in ADR-0105 (13-layer set: api, rest, application, usecase, domain, kernel, adapter, worker, governance, plus three runtime-substrate layers — see ADR-0105). Generated crate names follow `oya-production-planning-<bounded-context>-<layer>` per the catalog roster (54 records cataloged).

### §1.2 SAP parity claim and Phase classification
The manifest declares `sap_module_parity.sap_code = "PP"` with surfaces `BOM`, `MRP`, `Capacity Planning`, `Shop Floor`, `Routing`. The PRD §A.3 declares: *"production-planning is equivalent to SAP PP module coverage for BOM, MRP, capacity planning, routings, production orders, and shop-floor release."* The Wave 4 audit doctrine requires that this be more than a CRUD shell wrapping document objects: this audit therefore measures **APS substance** — MRP explosion / MPS aggregation / finite scheduling / dispatching / change-over / bottleneck primitives — per the §3.4.M anchor in the brief template.

### §1.3 Top-3 counterparts
1. **SAP APO PP/DS + S/4HANA PP/DS (`/SAPAPO/CDPSC`, `CM27`/`CM28`)** — the dominant on-prem APS engine; PP/DS layer adds finite scheduling, detailed sequencing, setup matrix optimization, and pegging on top of MRP.
2. **Oracle Supply Chain Planning Cloud (Production Scheduling Cloud Service + Constraint-Based Optimization)** — the dominant cloud-native APS engine; rapid replan via in-memory engine with HLC-style time coordination.
3. **Kinaxis Maestro / RapidResponse** — the dominant concurrent-planning engine; collapses MPS+MRP+capacity+inventory into one always-on concurrent calculation surface with what-if scenario branching.

Out-of-the-Top-3 but referenced in IP-021: Siemens Opcenter APS, Dassault DELMIA Quintiq, PlanetTogether APS — these are inspected only when the §4 parity-line evidence cites them.

### §1.4 Scope of this audit (ownership)
- **In scope**: every file under `microservices/production-planning/`, including 25 IP slices, 6 Cedar policies, 4 OpenSLOs, 6 runbooks, 3 dashboards, 54 catalog records, manifest, PRD, ARCHITECTURE, README, threat model, DPIA, capacity model, cost budget, failure modes, multi-region, incident response, backfill replay, competitor parity matrix, SDK plan, compliance, contracts (OpenAPI + AsyncAPI), and `iac/` directory.
- **Audit only — no remediation**: per the dispatch brief, remediation is not authorized in this slice. Findings are documented; remediation is queued for a downstream IP.
- **Not in scope**: edits to ADR-0315, ADR-0244, ADR-0328, or any other doctrine ADR; edits to sibling microservices; edits to `tools/agent-skills/`; reorganization of the master plan.

### §1.5 Why this audit matters
Phase 4 of the master plan ships ERP-parity. Production planning is the single hardest Phase-4 service because APS engines are NP-hard in the general case (job-shop-scheduling is NP-hard, set-up-cost-optimal sequencing is NP-hard, finite-resource pegging is NP-hard) and because the cost of a wrong schedule is wall-clock manufacturing time — a missed bottleneck minute is a permanently lost minute. SAP charges six- to seven-figure annual subscriptions for PP/DS for this reason. If our coherence is template-stamped instead of substance-bearing, no manufacturing buyer will sign.

## §2 Nine-Dimension Audit Framework

This audit uses the canonical nine-dimension framework (Wave 4 spec) plus the three augmentation gates §3.4.T (tenant), §3.4.C (context), §3.4.M (MRP/MPS/finite-scheduling). Each dimension carries: scope, evidence inspected, finding, severity (P0/P1/P2/N/A), and remediation pointer.

The nine dimensions are: D1 Doc-suite coverage, D2 Architectural coherence with ADR roster, D3 Substance (no template-stamping), D4 Industry parity with top-3 counterparts, D5 Performance posture, D6 Compliance + pack overlays, D7 Observability + audit emission, D8 Deployment-context multi-context matrix, D9 Code-substrate alignment (Rust strictness + Cargo + layer enum).

Severity calibration: **P0** blocks ERP-parity claim; **P1** blocks Wave-4 phase exit; **P2** documentation gap that does not block runtime substance.

### §2.1 D1 — Doc-suite coverage
**Evidence inspected**: file listing returns **152 files** under the microservice tree. Mandatory artifact roster (per manifest `wave_3_g_follow_up.required_artifacts_not_in_anchor`): PHASE-NN, IP-NNN, README, CHANGELOG, threat-model, dpia, capacity-model, cost-budget, failure-modes, multi-region, incident-response, backfill-replay, competitor-parity-matrix, sdk-plan, policy/*.cedar, contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/*.proto, capabilities/*.yaml, dashboards/*.json, slos/*.openslo.yaml, catalog/*.yaml, iac/*, AUDIT-FINDINGS-<date>.json, scorecards/overrides.json.

**Presence verification**:
- PHASE-01-PRODUCTION-PLANNING-PARITY.md: present (400 lines). PASS-shape.
- IP-001..IP-025: present (25 slices). PASS-shape.
- README.md: present (201 lines). FINDING — see §3.1 (template-stamped).
- CHANGELOG.md: present (1061 bytes). PASS-shape.
- threat-model.md: present (261968 bytes). PASS-shape; depth verified §2.3.
- dpia.md: present (108239 bytes). PASS-shape.
- capacity-model.md: present (125987 bytes). PASS-shape.
- cost-budget.md: present (56850 bytes). PASS-shape.
- failure-modes.md: present (130558 bytes). PASS-shape.
- multi-region.md: present (61160 bytes). PASS-shape.
- incident-response.md: present (124023 bytes). PASS-shape.
- backfill-replay.md: present (70175 bytes). PASS-shape.
- competitor-parity-matrix.md: present (86728 bytes / 350 lines). FINDING — see §3.4 (350 rows of identical template assertion, zero APS substance per row).
- sdk-plan.md: present (109875 bytes). PASS-shape.
- policy/*.cedar: 6 of 6 expected (bom-revision, capacity-calendar, mrp-run, production-order, routing-step, shop-floor-release). PASS-shape.
- contracts/openapi-v1.yaml: present. PASS-shape.
- contracts/asyncapi-v1.yaml: present. PASS-shape.
- contracts/*.proto: declared in Cargo.toml as `production-planning-v1.proto` — **FINDING P1** — file not visible in the contracts/ directory listing (only openapi-v1.yaml + asyncapi-v1.yaml present). Cargo metadata references a file the listing does not show. See §3.7.
- capabilities/*.yaml: 3 present (bom-revision-command, capacity-calendar-export, mrp-run-reconcile). FINDING P2 — three more bounded contexts (routing-step, production-order, shop-floor-release) have no `capabilities/*.yaml` evidence. The manifest's `second_pass_doc_suite.categories.capabilities = 3` accepts this gap but the doctrine should require one capability per bounded context (six).
- dashboards/*.json: 2 present (bom-revision-health, production-planning-overview) plus 1 markdown (mrp-run-residency.md). FINDING P2 — should be JSON only; markdown dashboards are a code-smell.
- slos/*.openslo.yaml: 4 present (availability, latency-p99, throughput, bom-revision-success-rate). FINDING P1 — SLO file count covers cross-cutting metrics but does not name MRP-run-completion or schedule-recalc-latency SLOs; these are the headline APS user-visible SLOs and should appear by name. See §3.5.
- catalog/*.yaml: 54 present (9 layers × 6 bounded contexts). PASS-shape.
- iac/: present with k8s-deployment, helm-values, network-policy, ech-config, edge-waf, openbao-policy, pqc-cert, secret-bindings, terraform-module. FINDING P1 — no per-context `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/guest-on-oci/`, `iac/on-prem/`, `iac/colo/`, or `iac/oyatie-as-cloud-provider/` subdirectories. The directory uses generic Helm/k8s flat shape instead of the six-context shape required by ADR-0328 §D-15. See §3.8.
- AUDIT-FINDINGS-2026-05-21.json: present. PASS-shape; verdict claims `second-pass-authored` for six bounded-context doc-suite findings.
- scorecards/overrides.json: present. PASS-shape.

**D1 verdict**: PASS-shape (all artifacts present) with three FINDING flags carried to D3/D4/D8.

### §2.2 D2 — Architectural coherence with ADR roster
**Evidence inspected**: manifest `binding_adrs` declares ADR-0131, ADR-0132, ADR-0244, ADR-0245, ADR-0314, ADR-0315. PRD §A.3 quotes ADR-0315 (SAP-parity doctrine) and ADR-0316 (tenant-class activation), ADR-0244 (tenant scoping), ADR-0314 (DealSet settlement). ARCHITECTURE.md §H cites ADR-0131, ADR-0132. IP-024 D-2 cites ADR-0145 (inter-microservice direct gRPC).

**Coherence checks**:
- ADR-0131 (per-microservice flat layout): `src/` exposes `domain/`, `usecase/`, `adapter/` — flat. PASS.
- ADR-0132 (no-suite µservices): production-planning is single-concern APS, not a suite. PASS.
- ADR-0145 (direct gRPC): IP-024 declares AsyncAPI-over-Kafka for MES handshake; the integration topology in ARCHITECTURE.md §D names "API/event based" but does not specify gRPC for sibling Oyatie services. FINDING P2 — ARCHITECTURE.md §D should name the gRPC/AsyncAPI split per integration partner explicitly.
- ADR-0244 (tenant universal scoping): every bounded-context invariant in ARCHITECTURE.md §C declares "tenant scope required". Cedar policies all gate on tenant. PASS.
- ADR-0245 (substrate vs product): production-planning is correctly classified as `product` in manifest `keystone_adr_field_roster.substrate_or_product`. PASS.
- ADR-0248 (Amazon-shape cellular): manifest declares `cell_eligibility: [Tier 0, Tier 1, Tier 2, Tier 3]` and `failure_domain: cell-aware service eligible for Tier 0, Tier 1, Tier 2, Tier 3`. PASS.
- ADR-0252 (HLC default + TrueTime tier): manifest `time_coordination = "HLC default; TrueTime-compatible external evidence accepted when source system supplies it"`. IP-024 D-3.40 declares HLC + UTC timestamp with reconciliation drift ≤ ±2s for MES counterparts. PASS — this is unusually substantive.
- ADR-0253 (HTTP/3 + QUIC default): manifest `transport = "HTTP/3 default; fallback HTTP/2 then HTTP/1.1; TLS 1.3; ECH advertised; PQC hybrid offered where supported"`. README §Operating posture echoes this. PASS.
- ADR-0263 (observability emission contract): PRD §F.OM-01..OM-18 declares per-bounded-context SLO review fields (p50/p95/p99/burn/policy-deny/replay-lag/audit-seal-latency/ontology-projection-lag). PASS-shape (depth verified in §2.7).
- ADR-0314 (marketplace settlement): manifest `marketplace = "Marketplace settles all tenant deals per ADR-0314; this service records settlement refs only"`. PRD Story PP-013 covers settlement evidence ingest. PASS.
- ADR-0315 (SAP module parity): manifest `sap_module_parity.sap_code = "PP"` with named surfaces. PASS.
- ADR-0316 (tenant-class activation): **FINDING P0** — the manifest still lists `tenant_classes: ["T1"]` and the PRD still references "TenantClass" projections in PP-010 ("promote a tenant class"). Per the 2026-05-20 doctrine, **tier is retired**; activation must flow through `tenant_class` (multi-tenant / paid / byo-cloud / self-hosted / demo / sandbox / trial / dev) per specs/master-plan-sequencing.json#deployment_contexts. The persistence of TenantClass in PRD §B.7 and §C is a doctrine drift. See §3.2.
- ADR-0328 (substance bar + multi-context): see D8 below for context coverage.

**D2 verdict**: PARTIAL PASS with **P0 finding** on ADR-0316 tier-retirement drift; secondary P2 on ADR-0145 integration-topology specification.

### §2.3 D3 — Substance (anti-template-stamping)
**Evidence inspected**: PRD.md (1938 lines), README.md (201 lines), ARCHITECTURE.md (200 lines), competitor-parity-matrix.md (350 lines), capacity-model.md (300 lines), PHASE-01 (400 lines), 25 IP slices, 6 runbooks, 6 Cedar policies.

**Anti-pattern detection**:
- **README.md template-stamping** — lines 32..151 (120 evidence rows) are syntactically identical except for the bounded-context substring. Each row reads: "Production Planning.<context> links SAP PP, SAP PP Production Planning | Oracle Fusion Cloud Manufacturing | Workday Adaptive Planning production-capacity counterpart | NetSuite Manufacturing WIP and Routings | Microsoft Dynamics 365 Supply Chain Management, Cedar default deny, OpenBao secret reference, SLO evidence, dashboard evidence, and runbook recovery path." 120 rows × ~600 chars each = 72 KB of identical assertion. **P1 finding per ADR-0324 anti-template-stamping** — this is exactly the "vendor-variable swap" anti-pattern named in the brief template §2.8.
- **ARCHITECTURE.md template-stamping** — §H.Architecture trace 1..90 (lines 111..200) are 90 identical sentences modulo the bounded-context name and the trace number. The doctrine pattern: "production-planning.<context> must remain service-owned, tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, and independently deployable under ADR-0131 and ADR-0132." This is six bounded contexts cycled 15 times. **P1 finding** — line-count padding, vendor-irrelevant.
- **competitor-parity-matrix.md template-stamping** — §B 350 rows × six bounded contexts × two named counterparts (SAP PP, Oracle Fusion Cloud Manufacturing) cycled. Every row reads "Requires tenant scope, Cedar permit, audit-chain event, OpenAPI/AsyncAPI/proto parity, HTTP/3/ECH/PQC transport note, and pack overlay evidence before claiming parity". **P0 finding** — this is the headline parity claim of a Phase-4 APS microservice. It must carry vendor-specific endpoints, transactions (`CM27`/`CM28`/`/SAPAPO/CDPSC`/`MD01`/`MD02`/`MD04`), object types (`MARA`/`MARC`/`STPO`/`PLAF`/`AFKO`), and named differentiators. **It does none of this.** See §3.4 + the bespoke feature-parity-matrix-2026-05-20.md companion deliverable.
- **PRD.md user stories PP-001..PP-040** — 40 stories; each one is a structural mirror of PP-001 with the bounded-context substring and the action verb rotated through {create, amend, approve, reverse, archive, run-migration-dry-run, compare-source-system-rows, export-audit-evidence, resolve-policy-denied-mutation, promote-tenant-class, inspect-ontology-lineage, coordinate-cross-service-workflow, receive-settlement-evidence, handle-regional-failover, run-batch-reconcile, trace-source-system-discrepancy, apply-compliance-pack, review-SLO-burn, simulate-10x-volume-surge, deactivate-stale-pack}. **P1 finding** — these are governance-shape stories, not APS-substance stories. None of the 40 stories names "run MRP across 250k SKUs in <30s p95", "reschedule schedule after machine breakdown in <5s", "what-if branch a scenario with 10 setup-cost overrides", "dispatch 20 jobs into Mazak palette queue", "lock change-over matrix for SKU family X". The substance is missing.
- **PRD.md §F observability rows OM-01..OM-18** — 18 rows × 6 bounded contexts × 3 repetitions; identical assertion. **P2 finding** — observability fields are correct (p50/p95/p99/burn/deny/replay/seal/projection-lag) but listed 18 times.
- **PRD.md §G migration rows MR-01..MR-35** — 35 rows × 4 bounded contexts × multiple repetitions. **P2 finding** — same template-stamping shape.

**Substance evidence that IS present** (positive findings):
- **IP-021 (capacity-leveling-finite-scheduling-forward-backward-bottleneck)** — 357 lines of bespoke APS substance: declares NP-hardness reducibility to JSSP, names SAP `CM27`/`CM28` + `/SAPAPO/CDPSC` + Oracle Production Scheduling Cloud + Dynamics 365 SCM + Siemens Opcenter APS + Dassault DELMIA Quintiq + PlanetTogether APS, defines Forward + Backward + Bottleneck-Anchor strategies, encodes Drum-Buffer-Rope (DBR) per TOC-author's Theory of Constraints, ships Rust signatures (`schedule_forward`, `schedule_backward`, `schedule_bottleneck_anchor`), declares SQL schema for placements, provides AC-3 constraint propagation, ships benchmark p50/p95/p99 for 100/1k/10k operations, declares feature flag `production_planning_finite_scheduling_v1`. **PASS — this is the substance bar the rest of the suite should match.**
- **IP-024 (MES-handshake-bidirectional-event-flow-isa-95)** — declares ISA-95 / IEC 62264 standard, B2MML message types from MESA consortium, ISA-95 Level 3 (MES) ↔ Level 4 (Business Logistics Planning & Scheduling) mapping, drift detection every 5min with reconcile UC, ISA-95 hierarchy mapping `(tenant, plant, work_center, work_unit) ↔ (EnterpriseRef, SiteRef, AreaRef, WorkUnitRef)`, named vendor parallels (SAP DMC, SAP ME, Oracle MES Cloud, Siemens Opcenter Execution, DELMIA Apriso, Rockwell PlantPAx + FactoryTalk ProductionCentre, AVEVA MES, Critical Manufacturing CMF, Tulip). **PASS — this is genuinely interoperable APS substance.**
- **IP-018 (ddmrp-buffer-profile-authoring-and-daf-recalc)** — covers Demand-Driven MRP (Ptak/Smith) buffer profiles + Dynamic Adjustment Factor recalc; substance present.
- **IP-019 (sop-horizon-monthly-cycle-with-executive-signoff-gate)** — covers S&OP horizon cycle; substance present.
- **IP-020 (production-version-selection-with-co-product-yield-variance)** — covers production-version selection + co-product/by-product yield variance; substance present.
- **IP-022 (long-term-planning-versus-short-term-planning-split)** — covers LTP/SPT horizon split; substance present.
- **IP-023 (alternative-routing-engagement-decision-engine)** — covers alternative-routing engagement; substance present.
- **IP-025 (production-line-balancing-takt-time-workstation-load-smoothing)** — covers takt-time line balancing + workstation load smoothing; substance present.

**D3 verdict**: HYBRID. The IP slices IP-018..IP-025 carry strong bespoke APS substance. The strategic-doc tier (README, ARCHITECTURE, competitor-parity-matrix, PRD user stories) is **heavily template-stamped** and fails the substance bar of ADR-0322/ADR-0324. **P0 on competitor-parity-matrix; P1 on README + ARCHITECTURE + PRD user stories; P2 on PRD observability + migration rows.** Remediation queued via the bespoke feature-parity-matrix-2026-05-20.md and performance-benchmark-numbers-2026-05-20.md companions.

### §2.4 D4 — Industry parity with top-3 counterparts
This dimension is the headline of a Phase-4 audit. See companion deliverable `feature-parity-matrix-2026-05-20.md` for the UNION-coverage matrix against SAP APO + Oracle SCP + Kinaxis Maestro. Findings summary:

- **MRP coverage** (SAP `MD01`/`MD02`/`MD03`/`MD04` parallel): IP-002 declares mrp-run domain layer; IP-016 declares MRP-explosion-to-supply-chain-planning handoff. **PARTIAL** — net requirements calculation, lot-sizing rules (FX/PD/EX/SO), safety-stock policies, ATP-from-MRP, low-level-code algorithm are NOT named in PRD or IP slices.
- **MPS coverage** (SAP `MD41`/`MD43` parallel): IP-022 covers LTP vs short-term split; **PARTIAL** — explicit master-production-schedule generation, planning-time-fence, demand-time-fence, firm-planned-order semantics are NOT named.
- **Finite scheduling**: IP-021 is **PASS** — forward + backward + bottleneck-anchor + DBR + AC-3 constraint propagation all present.
- **Dispatching**: IP-006 covers shop-floor-release domain; IP-024 covers MES handshake outbound. **PARTIAL** — explicit dispatch list (SAP `CO04`/`CO05`/`COHV` parallel), priority sequencing rules (EDD/SPT/CR), and conveyor/palette integration are NOT named.
- **Change-over optimization**: **MISSING** — no IP slice covers setup-time matrix optimization, sequence-dependent setup costs, family-grouping, or SMED (single-minute-exchange-of-die). Kinaxis Maestro and Siemens Opcenter both have headline features here. P0 gap.
- **Bottleneck management**: IP-021 §D-3 covers bottleneck anchoring + DBR. **PASS** — TOC-author TOC pattern explicitly named.
- **What-if scheduling**: PRD Story PP-019 ("simulate a 10x volume surge") is template-stamped. No IP slice covers scenario branching, scenario merge, scenario lock, or scenario cost rollup. **P0 gap** — Kinaxis Maestro's headline differentiator is concurrent scenario branching; we have no comparable surface.
- **OEE tracking**: **MISSING** — no IP slice covers Overall Equipment Effectiveness (Availability × Performance × Quality). P1 gap.
- **ML anomaly detection on shop-floor signals**: **MISSING** — no IP slice covers anomaly detection on machine telemetry. P1 gap.
- **Real-time shop-floor integration**: IP-024 covers ISA-95 / B2MML handshake. **PASS** — though restricted to AsyncAPI-over-Kafka cadence (no OPC-UA / MQTT-Sparkplug-B native ingest).
- **Mobile**: **MISSING** — no IP slice covers mobile-supervisor / mobile-operator surfaces. SAP Asset Manager + Oracle Mobile Supply Chain are headline competitors. P1 gap.

**D4 verdict**: SUBSTANCE-MIXED. Finite scheduling + ISA-95 handshake + DBR + DDMRP are **above SAP APO parity** in clarity. MRP/MPS detail, change-over optimization, what-if scheduling, OEE, ML anomaly, and mobile are **below SAP APO / Oracle SCP / Kinaxis Maestro parity**. **P0 on change-over + what-if; P1 on MRP/MPS detail + OEE + ML + mobile.**

### §2.5 D5 — Performance posture
**Evidence inspected**: PRD §F.1063..1132 declares per-bounded-context command-latency histograms (p50<120ms, p95<300ms, p99<750ms for lightweight mutations); SLO files in `slos/` declare availability + latency-p99 + throughput + bom-revision-success-rate; capacity-model.md (300 lines) declares tier assumptions and hot partitions; IP-021 §H declares finite-scheduling benchmark table (50ms/110ms/220ms small, 480ms/1.1s/2.2s medium, 8s/18s/35s large for forward heuristic; bottleneck-anchor on 10k ops × 50 work centers in 8s/18s/35s).

**Findings**:
- **PRD §F latency target uniformity** — the same p50<120ms / p95<300ms / p99<750ms is declared for every bounded context, including MRP-run. **P0 finding** — an MRP run is NOT a lightweight mutation. SAP S/4HANA PP/DS publishes MRP-run benchmarks in the minutes-to-hours range for 100k-1M material masters. Declaring the same 120ms/300ms/750ms for `oya_production_planning_mrp_run_command_latency_ms` as for `oya_production_planning_bom_revision_command_latency_ms` is physically false. See companion `performance-benchmark-numbers-2026-05-20.md`.
- **Schedule recalc latency** — not declared as a named SLO. IP-021 §H gives heuristic-internal benchmarks but does not bind a customer-visible SLO surface. **P0 finding** — schedule recalc after a machine breakdown is the most-visible APS user-facing latency; Kinaxis publishes "concurrent planning sub-second response" as their differentiator. We have no named SLO.
- **What-if scenario fork latency** — not declared at all. **P0 finding**.
- **Shop-floor event ingestion** — IP-024 declares "drift detection every 5min" but no ingestion-rate SLO (events/sec/tenant). **P1 finding**.
- **Dashboard p99** — `dashboards/production-planning-overview.json` exists but no dashboard-render-latency SLO is declared. **P2 finding**.
- **Tier doctrine retirement impact** — the PRD §F.1129 baseline declares "300 ms p95 command at 1000 commands per second requires 300 concurrent worker slots before headroom." This is one number for the whole service. Per the 2026-05-20 doctrine, performance targets should be **single industry-leader target + deployment-context overlay + tenant-class overlay** — not flat. The current declaration does not exercise the overlay model.

**D5 verdict**: **P0 — MRP-run latency target is physically false** (uses lightweight-mutation budget); P0 on schedule recalc + what-if; P1 on shop-floor ingest; P2 on dashboard. Companion deliverable `performance-benchmark-numbers-2026-05-20.md` provides corrected numbers grounded in published industry-leader benchmarks.

### §2.6 D6 — Compliance + pack overlays
**Evidence inspected**: manifest `compliance_packs = [SOX-404, SOC-2, ISO-27001, GDPR, LGPD, KR-PIPA, jurisdictional-tax]`; `packs = [SOX-404, soc2, iso27001, gdpr, lgpd, kr-pipa, jurisdictional-tax, gdpr-eu, fedramp-high]`; compliance.md (24779 bytes); dpia.md (108239 bytes); policy/pack-overlay-authorization.cedar present; policy/data-residency.md present; policy/tenant-isolation.md present.

**Findings**:
- **Pack roster naming inconsistency** — manifest declares `compliance_packs` (mixed-case e.g., `SOX-404`, `SOC-2`) AND `packs` (lowercase e.g., `soc2`, `iso27001`) AND `compliance_packs_applicable` (lowercase). Three keys, three case conventions. **P2 finding** — pick one.
- **SOX-404 substance for manufacturing** — manufacturing-specific SOX-404 controls (inventory valuation, work-in-process roll-up, standard-cost-vs-actual variance) are NOT named in compliance.md (it carries pack-overlay scaffolding, not WIP-specific controls). **P1 finding**.
- **EU AI Act applicability** — the manifest does NOT carry the EU AI Act pack despite the service shipping a finite-scheduling engine that, under Article 6 + Annex III of the EU AI Act, may qualify as a "high-risk AI system" if it autonomously dispatches human shop-floor workers. **P1 finding** — pack roster gap; should include `eu-ai-act` at least for risk-classification evidence.
- **Industry-specific compliance** — FDA 21 CFR Part 11 (pharma manufacturing), ISO 13485 (medical device), AS9100 (aerospace), IATF 16949 (automotive) are NOT in the pack roster. Phase-4 ERP-parity in regulated manufacturing requires these. **P1 finding**.
- **KR-PIPA + tenant-class crossover** — KR-PIPA (Korea PIPA Article 29/30) is present but does not declare the tenant_class activation rule (which tenant_class values require KR-PIPA pack). **P2 finding**.

**D6 verdict**: PASS-shape, PARTIAL on substance. **P1 on industry-specific compliance roster (FDA / ISO 13485 / AS9100 / IATF 16949) + EU AI Act; P2 on key-case and tenant_class crossover.**

### §2.7 D7 — Observability + audit emission
**Evidence inspected**: manifest `audit_chain.seal_events = [EVT-PRODUCTION_PLANNING-BOM_REVISION-CHANGED, EVT-PRODUCTION_PLANNING-MRP_RUN-CHANGED, ..., EVT-PRODUCTION_PLANNING-SHOP_FLOOR_RELEASE-CHANGED]` (6 events); PRD §F.OM-01..OM-18 SLO review fields; dashboards/ directory; IP-024 §I declares additional event classes `EVT-PRODUCTION_PLANNING-MES-SCHEDULE_EMITTED`, `EVT-PRODUCTION_PLANNING-MES-PERFORMANCE_INGESTED`, `EVT-PRODUCTION_PLANNING-MES-RESPONSE_INGESTED`, `EVT-PRODUCTION_PLANNING-MES-STATE_DRIFT_DETECTED`.

**Findings**:
- **Audit-chain event roster mismatch** — manifest declares 6 seal events (one per bounded context, suffix `-CHANGED`). IP-024 ships 4 additional event classes (MES handshake events) that are not in the manifest's `seal_events` list. **P1 finding** — manifest is out of date relative to IPs.
- **Generic `-CHANGED` events** — the manifest's 6 events use the suffix `-CHANGED` (e.g., `EVT-PRODUCTION_PLANNING-MRP_RUN-CHANGED`). This is too generic for an APS audit trail. SAP's PP audit emits granular events: `MD41-MPS-CREATED`, `MD01-MRP-RUN-COMPLETED`, `MD04-PEGGING-REVISED`, `CO04-DISPATCH-LIST-PUBLISHED`. **P1 finding** — granularity gap.
- **Cedar policy gates** — 6 Cedar policies present (one per bounded context). PASS-shape; per-policy substance not exhaustively audited here.
- **Metric dimensions** — PRD §F dimensions are `tenant, tier, action, region, outcome, policy_decision`. The `tier` dimension is the tier-doctrine drift (see §2.2 ADR-0316 finding); should be `tenant_class` per the 2026-05-20 doctrine. **P0 finding** — every metric carries the retired dimension.

**D7 verdict**: PASS-shape, **P0 on tier-dimension drift** (every metric carries the wrong dimension); **P1 on audit event roster mismatch + granularity gap.**

### §2.8 D8 — Deployment-context multi-context matrix
**Evidence inspected**: per ADR-0328 §D-15, the six contexts are `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, `oyatie-as-cloud-provider`. The brief-template §3.9 anchor demands a `SUPPORTED_CONTEXTS` field per microservice.

**Findings**:
- **iac/ directory layout** — current shape: `iac/{ech-config, edge-waf, helm-values, k8s-deployment, network-policy, openbao-policy, pqc-cert, secret-bindings, terraform-module}/`. **P0 finding** — this is the pre-2026-05-20 flat shape; required shape is `iac/<context>/<files>` for each of the six contexts per ADR-0328 §D-15.X (Context N IaC target rules) + §D-16 (OpenTofu module per context).
- **manifest does not declare `supported_contexts`** — no `deployment_contexts` field or `supported_oses` field in the manifest. **P0 finding**.
- **on-prem applicability for Phase-4 ERP** — manufacturing is the canonical on-prem buyer (factories with intermittent connectivity to public cloud, sovereign data residency for IP). The dispatch decision tree step 7 in brief-template §3.9 says: "If the µservice is Phase 4 enterprise app surface, decide required contexts from target buyer expectations and data-residency posture." For manufacturing, on-prem is **required**. **P0 finding**.
- **Terraform vs OpenTofu** — `iac/terraform-module/` directory name uses "terraform" verbatim. Per ADR-0328 §D-16 and the brief-template §3.10 forbidden-pattern list, the engine name is **OpenTofu**, not Terraform. **P1 finding** — directory name should rename to `opentofu-module/` or be subsumed under per-context `iac/<context>/` directories.
- **Cargo workspace OS support** — Cargo.toml declares `rust-version = "1.95.0"` but the microservice manifest has no `supported-oses.json` file. Per ADR-0328 §D-17, Tier-1 OS roster must be explicit. **P1 finding**.

**D8 verdict**: **P0 — multi-context deployment matrix is not implemented**; iac/ uses pre-doctrine shape; manifest does not declare contexts; on-prem is required for Phase-4 manufacturing but no on-prem module exists; Terraform naming persists.

### §2.9 D9 — Code substrate alignment
**Evidence inspected**: `src/` exposes `domain/`, `usecase/`, `adapter/`, `lib.rs`, `main.rs`, `error.rs`, `config.rs`; `tests/integration.rs`; Cargo.toml declares `name = "oya-production-planning-mrp-app"` with `bounded_context = "mrp"` (single-bounded-context Cargo manifest); `clippy unwrap_used = deny`, `expect_used = deny`, `panic = deny` (strict lints).

**Findings**:
- **Single-crate vs six-bounded-context split** — Cargo.toml declares `bounded_context = "mrp"` (single). The architecture is six bounded contexts. **P0 finding** — either the Cargo manifest should be the workspace root with six member crates (one per bounded context × 9 layers = 54 crates per the catalog), or the manifest must be renamed to clarify it is the MRP-bounded-context only.
- **Crate name vs catalog** — catalog/ declares 54 catalog records named `oya-production-planning-<bounded-context>-<layer>.yaml`. Cargo.toml ships **one** crate named `oya-production-planning-mrp-app`. The 54 catalog records imply 54 crates; only 1 Cargo manifest is present. **P0 finding** — catalog/Cargo mismatch.
- **Layer enum compliance** — Cargo.toml metadata declares `layer = "app"` — but the ADR-0105 layer enum is `{api, rest, application, usecase, domain, kernel, adapter, worker, governance, ...}`. There is no `app` layer in ADR-0105. **P1 finding** — naming drift; `app` likely shorthand for `application`, but the catalog records use the canonical `application` token; the Cargo manifest should match.
- **Rust strict-only language policy** — no Python, JS, TS, Ruby, Perl, PHP, Java, Scala, Groovy, Go, F# detected in the microservice tree (verified by listing). PASS per ADR-0328 §D-18.
- **Authorized non-Rust extensions** — `.tf`, `.cedar`, `.yaml`, `.json`, `.proto`, `openapi.yaml`, `asyncapi.yaml`, `.openslo.yaml`, `.sql`, `.md` are all present and correctly classified. PASS.
- **Frontend allowlist** — no frontend code under the microservice tree (correct; frontend lives at repo-root `frontend/`). PASS.

**D9 verdict**: **P0 on Cargo/catalog mismatch (1 crate vs 54 catalog records); P1 on `app` vs `application` layer-enum drift.** Rust-strict policy PASS.

## §3 Detailed Findings (Severity-Ordered)

### §3.1 P1 — README evidence-row template-stamping (120 identical rows)
**Location**: `microservices/production-planning/README.md` lines 32..151.
**Description**: 120 evidence rows are syntactically identical modulo the bounded-context substring; each row repeats the same vendor parallel string and the same five-element evidence claim ("Cedar default deny, OpenBao secret reference, SLO evidence, dashboard evidence, runbook recovery path").
**Doctrine violated**: ADR-0322 (substance bar), ADR-0324 (anti-template/anti-script), brief-template §2.8 forbidden patterns.
**Substance bar a future intern should meet**: each bounded context should have ONE evidence paragraph that names the specific Cedar permit name, the specific OpenBao secret path, the specific SLO file, the specific dashboard JSON file, and the specific runbook by name. Six paragraphs total, not 120 rows.
**Remediation pointer**: IP-26 (proposed) — rewrite README with per-bounded-context evidence paragraphs.

### §3.2 P0 — Tier doctrine drift (ADR-0316 tenant_class retired)
**Location**: manifest.json `tenant_classes: ["T1"]` + `criticality_tier: "T1"`; PRD.md §B.7 + §C tier-projection fields; every metric dimension list in PRD §F (`tenant, tier, action, ...`).
**Description**: The 2026-05-20 doctrine retires `tenant_class` as a delta primitive. Activation flows through `tenant_class` (multi-tenant / paid / byo-cloud / self-hosted / demo / sandbox / trial / dev) per specs/master-plan-sequencing.json#deployment_contexts. The PRD persists tier semantics in user stories ("promote a tenant class") and in metric dimensions.
**Doctrine violated**: 2026-05-20 tier-retirement directive; specs/master-plan-sequencing.json#tenant_class composability.
**Substance bar a future intern should meet**: every reference to `tier` in user-facing dimensions (metrics, audit events, Cedar context) should be replaced by `tenant_class`; tenant-class activation logic should be re-grounded in deployment-context + tenant-class composition.
**Remediation pointer**: IP-26 (proposed) — manifest + PRD + Cedar + dashboards tier→tenant_class migration.

### §3.3 P1 — ARCHITECTURE.md trace-line template-stamping (90 identical traces)
**Location**: `microservices/production-planning/ARCHITECTURE.md` lines 111..200.
**Description**: 90 architecture-trace bullets are syntactically identical except for the bounded-context name and the trace number. Six bounded contexts × 15 cycles = 90 lines of vendor-irrelevant assertion.
**Doctrine violated**: ADR-0322, ADR-0324, brief-template §2.8.
**Substance bar a future intern should meet**: ARCHITECTURE.md §H should ship one paragraph per bounded context describing the unique integration topology (e.g., MRP-run reads from BOM-revision, writes to capacity-calendar, emits to ontology, lands in workflow-engine for human approval, settles via marketplace), not 15 cycles of the same sentence.
**Remediation pointer**: IP-26 (proposed) — rewrite ARCHITECTURE §H with per-bounded-context integration narrative.

### §3.4 P0 — competitor-parity-matrix.md template-stamping (350 identical rows)
**Location**: `microservices/production-planning/competitor-parity-matrix.md` lines 23..350.
**Description**: 350 parity rows all read "Requires tenant scope, Cedar permit, audit-chain event, OpenAPI/AsyncAPI/proto parity, HTTP/3/ECH/PQC transport note, and pack overlay evidence before claiming parity". No vendor-specific endpoint, no SAP transaction code, no Oracle SCM Cloud REST endpoint, no Kinaxis Maestro RapidResponse action name. This is the **headline parity document** of a Phase-4 APS microservice; it must carry SAP-PP / Oracle-SCP / Kinaxis-Maestro specifics.
**Doctrine violated**: ADR-0322 substance bar; ADR-0321 vendor dossier substance requirements; ADR-0324 anti-template.
**Substance bar a future intern should meet**: one parity row per feature × counterpart, naming the counterpart's transaction code (e.g., SAP `CM27`, Oracle `BIP_MRP_PROCESSING_JOB`, Kinaxis Maestro `RunNetting`), Oyatie's counterpart endpoint (`POST /v1/production-planning/mrp-run`), the gap or parity claim (yes / partial / no), and the specific differentiator.
**Remediation pointer**: This deliverable is replaced by the companion `feature-parity-matrix-2026-05-20.md`.

### §3.5 P0 — Performance SLO numbers are physically false for MRP-run
**Location**: PRD.md §F.1063..1132 — `oya_production_planning_mrp_run_command_latency_ms` declared as p50<120ms / p95<300ms / p99<750ms for "lightweight mutations".
**Description**: An MRP run on 100k SKUs is not a lightweight mutation; published industry-leader benchmarks (SAP S/4HANA PP/DS) put MRP-run completion in minutes-to-hours for 100k-1M material masters. Declaring 750ms p99 for MRP-run is physically false.
**Doctrine violated**: brief-template §2.5 substance requirements (real SLO numbers); ADR-0322 substance bar.
**Substance bar a future intern should meet**: the SLO file `slos/mrp-run-completion-latency.openslo.yaml` (to be created) should declare separate SLOs for: (a) command-acknowledgment latency (the ms the caller waits for the run to be accepted), (b) MRP-run-completion latency (the seconds/minutes for the run to complete end-to-end), keyed by tenant SKU count (`<10k`, `10k-100k`, `100k-1M`, `>1M`).
**Remediation pointer**: This deliverable is replaced by the companion `performance-benchmark-numbers-2026-05-20.md`.

### §3.6 P0 — Change-over / SMED / sequence-dependent setup optimization MISSING
**Location**: no IP slice covers change-over.
**Description**: SAP APO PP/DS, Kinaxis Maestro, and Siemens Opcenter APS all have headline change-over optimization features (sequence-dependent setup time matrices, family grouping, SMED reduction projects). We have 25 IP slices and none of them addresses this.
**Doctrine violated**: §3.4.M MRP/MPS/finite-scheduling primitive coverage requirement.
**Substance bar a future intern should meet**: IP-26 (proposed) "Change-over optimization with sequence-dependent setup time matrices" — declare the setup time matrix data structure (work-center × from-product-family × to-product-family → setup time), the optimization heuristic (greedy + 2-opt for ≤50 jobs; simulated annealing for >50; constraint propagation for ≥500), the SAP DELMIA Quintiq parallel, the Cedar gate, the SLO numbers, and the audit event class.
**Remediation pointer**: IP-26.

### §3.7 P1 — proto3 contract file missing from contracts/
**Location**: Cargo.toml metadata `protobuf = "contracts/production-planning-v1.proto"` references a file not visible in `contracts/` directory listing (only openapi-v1.yaml + asyncapi-v1.yaml present).
**Description**: Either (a) the proto file is missing and Cargo metadata is stale, or (b) the proto file exists in a location the directory listing did not surface.
**Doctrine violated**: ADR-0145 inter-microservice direct gRPC requires proto3 contracts to exist for cross-service gRPC.
**Substance bar a future intern should meet**: either ship `contracts/production-planning-v1.proto` with the per-bounded-context gRPC service definitions (BomRevisionService, MrpRunService, CapacityCalendarService, RoutingStepService, ProductionOrderService, ShopFloorReleaseService) OR remove the Cargo metadata reference.
**Remediation pointer**: IP-27 (proposed).

### §3.8 P0 — Six-context iac/ layout missing
**Location**: `microservices/production-planning/iac/` carries pre-doctrine flat shape.
**Description**: ADR-0328 §D-15 + brief-template §3.9 require `iac/<context>/` subdirectories per the six contexts `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, `oyatie-as-cloud-provider`. None of these subdirectories exists. The current iac/ is a generic Helm/k8s flat layout.
**Doctrine violated**: ADR-0328 §D-15..§D-16; brief-template §3.9 + §3.10.
**Substance bar a future intern should meet**: create six per-context module directories; for each, ship main.tf, variables.tf, outputs.tf, versions.tf, README.md (OpenTofu pinning + sigstore/cosign signing per §D-16); declare on-prem as a required context for Phase-4 manufacturing (sovereign IP residency); declare guest-on-oci with the Always-Free Ampere A1 module under `iac/oci-guest/always-free/`.
**Remediation pointer**: IP-28 (proposed) — six-context IaC migration.

### §3.9 P0 — Cargo workspace ↔ catalog mismatch (1 crate vs 54 catalog records)
**Location**: Cargo.toml declares one crate `oya-production-planning-mrp-app`; catalog/ declares 54 catalog records (6 bounded contexts × 9 layers).
**Description**: The catalog says the microservice has 54 crates. Cargo says it has 1. One of these is wrong.
**Doctrine violated**: ADR-0105 13-layer enum + per-bounded-context layering; ADR-0131 flat layout.
**Substance bar a future intern should meet**: either (a) split into 54 crates via a workspace Cargo.toml at the microservice root + per-bounded-context member directories, or (b) reduce the catalog to one record per bounded context and document that the bounded-context internally implements all 9 layers via modules not crates.
**Remediation pointer**: IP-29 (proposed) — Cargo/catalog reconciliation.

### §3.10 P0 — What-if scheduling MISSING (Kinaxis Maestro differentiator)
**Location**: no IP slice covers scenario branching / scenario merge / scenario lock.
**Description**: Kinaxis Maestro's #1 published differentiator is concurrent scenario branching: "fork a what-if scenario from production data; modify; merge or discard". Oracle SCP Cloud's analog is the Plan Inputs Diagnostics workflow. SAP APO has "alternative plan versions" via `MPS` planning books. Our PRD Story PP-019 ("simulate a 10x volume surge") is template-stamped (one row in a 40-row cycle); no IP slice expands it.
**Doctrine violated**: §3.4.M required APS primitives.
**Substance bar a future intern should meet**: IP-30 (proposed) "Scenario branching with concurrent-planning fork + merge". Declare scenario data model (scenario id, parent scenario id, divergence point HLC, divergence inputs delta, merge resolution policy), scenario fork latency SLO (Kinaxis published target: sub-second), scenario lock semantics (no merge across locked scenarios), Cedar gate (only scenario-owner principal can merge), audit event class, ontology projection.
**Remediation pointer**: IP-30.

### §3.11 P1 — OEE (Overall Equipment Effectiveness) tracking MISSING
**Description**: no IP slice covers OEE = Availability × Performance × Quality. SAP DMC, Oracle MES Cloud, AVEVA MES all ship OEE. Without OEE, production-planning is "open-loop" (publishes schedule; never measures plan-vs-actual).
**Remediation pointer**: IP-31 (proposed).

### §3.12 P1 — ML anomaly detection on shop-floor signals MISSING
**Description**: no IP slice covers anomaly detection on machine telemetry (vibration / temperature / cycle-time drift). Siemens Industrial Edge + AWS Lookout for Equipment + Azure Industrial IoT Hub all ship this. The Oyatie `detection` microservice exists; production-planning should declare the integration handshake.
**Remediation pointer**: IP-32 (proposed).

### §3.13 P1 — Mobile supervisor / operator surface MISSING
**Description**: no IP slice and no frontend artifact covers a mobile shop-floor surface. SAP Asset Manager, Oracle Mobile Supply Chain, and Aptean MES Mobile all ship one. Per the language policy in ADR-0328 §D-18, frontend mobile lives at `frontend/ios=Swift` + `frontend/android=Kotlin` — these would be a sibling repo, but production-planning's PRD should declare the contract surface that backs them.
**Remediation pointer**: IP-33 (proposed) — declare REST + AsyncAPI surfaces consumed by mobile supervisors / operators.

### §3.14 P1 — FDA 21 CFR Part 11 / ISO 13485 / AS9100 / IATF 16949 compliance packs MISSING
**Description**: Phase-4 ERP-parity in manufacturing requires industry-specific compliance packs. The current pack roster (SOX-404, SOC-2, ISO-27001, GDPR, LGPD, KR-PIPA, jurisdictional-tax, gdpr-eu, fedramp-high) covers cross-industry corporate compliance but not manufacturing-domain compliance.
**Remediation pointer**: IP-34 (proposed) — add `fda-21-cfr-part-11`, `iso-13485-medical-device`, `as9100-aerospace`, `iatf-16949-automotive` packs.

### §3.15 P1 — EU AI Act applicability MISSING
**Description**: Article 6 + Annex III of the EU AI Act may classify a finite-scheduling engine that autonomously dispatches human shop-floor workers as a "high-risk AI system". The manifest does not carry `eu-ai-act` as a compliance pack.
**Remediation pointer**: IP-35 (proposed) — risk-classification evidence + EU AI Act Article 12 (logging) + Article 14 (human oversight) controls.

### §3.16 P2 — Pack roster naming-case inconsistency
**Description**: manifest declares `compliance_packs` (mixed-case) + `packs` (lowercase) + `compliance_packs_applicable` (lowercase). Three keys, three case conventions. Should pick one.
**Remediation pointer**: IP-36 (proposed) — manifest normalization.

### §3.17 P2 — Dashboard markdown vs JSON
**Description**: `dashboards/mrp-run-residency.md` exists alongside JSON dashboards. Dashboards should be JSON-only.
**Remediation pointer**: IP-37 (proposed) — convert MD → JSON.

### §3.18 P2 — Capabilities roster gap (3 of 6 bounded contexts)
**Description**: `capabilities/*.yaml` covers bom-revision, capacity-calendar, mrp-run only. Routing-step, production-order, shop-floor-release have no capability YAML.
**Remediation pointer**: IP-38 (proposed) — add three capability files.

### §3.19 P1 — Audit event roster mismatch (manifest vs IP-024)
**Description**: manifest declares 6 seal events; IP-024 ships 4 additional MES handshake events not in manifest. Manifest out of date.
**Remediation pointer**: IP-39 (proposed) — manifest event roster reconciliation.

### §3.20 P1 — Audit event granularity gap (`-CHANGED` is too generic)
**Description**: events use the suffix `-CHANGED`. SAP PP audit emits granular events like `MD41-MPS-CREATED`, `MD01-MRP-RUN-COMPLETED`. We should match granularity.
**Remediation pointer**: IP-40 (proposed) — event class enumeration per state transition.

### §3.21 P1 — Cargo layer drift (`app` vs `application`)
**Description**: Cargo metadata `layer = "app"` but ADR-0105 enum uses `application`.
**Remediation pointer**: IP-41 (proposed).

### §3.22 P1 — Terraform vs OpenTofu naming
**Description**: `iac/terraform-module/` uses retired engine name.
**Remediation pointer**: IP-42 (proposed) — rename to `opentofu-module/` or subsume under per-context dirs.

## §4 §3.4.T — Tenant Substance Gate

The §3.4.T (tenant) substance gate from the brief template requires that every Phase-4 audit verify tenant scoping flows end-to-end through Cedar gates, ontology projections, dashboards, audit events, and Cargo-level type signatures.

### §4.1 Cedar tenant scoping
- `policy/bom-revision-authorization.cedar`, `policy/mrp-run-authorization.cedar`, `policy/capacity-calendar-authorization.cedar`, `policy/routing-step-authorization.cedar`, `policy/production-order-authorization.cedar`, `policy/shop-floor-release-authorization.cedar` — 6 policies present. Per ARCHITECTURE.md §C, every bounded-context invariant says "tenant scope required". PASS-shape; cedar substance not exhaustively re-audited here but the file count matches the bounded-context count.

### §4.2 Manifest tenant_scope
- Manifest `tenant_scope: {required: true, source: ADR-0244, identity_continuity_required: true}`. PASS.

### §4.3 PRD tenant claim
- PRD §F.1241 names source systems "SAP MARA/MARC/STPO/PLAF/AFKO extracts; Oracle work definition exports; MES route history; CSV BOM packs". Migration rows MR-01..MR-35 declare per-row source-id + source-version + tenant + state + owner + policy-context + ontology-type + workflow-template + audit-stream + replay-key as the mapping requirement. PASS — tenant is in the per-row mapping requirement.

### §4.4 Ontology projection tenant scope
- PRD §D.1..§D.6 declare 6 ontology object projections (BomRevision, MrpRun, CapacityCalendar, RoutingStep, ProductionOrder, ShopFloorRelease) each linking to `Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet, TenantClass`. **FINDING (§3.2 P0)** — `TenantClass` should be `TenantClass`.

### §4.5 Metric dimension tenant scope
- PRD §F.OM-01..OM-18 declare dimensions `tenant, tier, action, region, outcome, policy_decision`. **FINDING (§3.2 P0)** — `tier` should be `tenant_class`.

### §4.6 Tenant_class composition (the NEW model)
Per the 2026-05-20 doctrine, tenant_class composes with deployment_context to produce the effective activation surface. For production-planning:
- `multi-tenant` × `oyatie-public-cloud` → shared MRP engine pool, tenant-pinned schedule namespace, common SLO budget across tenants
- `paid` × `guest-on-aws` → dedicated MRP engine instance, customer-owned AWS S3 state backend, customer-pinned SLO budget
- `byo-cloud` × `guest-on-oci` → dedicated MRP engine instance, customer-owned OCI Object Storage state backend; if `tenant_class = byo-cloud` AND `oci_always_free = true`, Ampere A1 module applies
- `self-hosted` × `on-prem` → customer-owned k8s + Cloud Hypervisor, sovereign data residency, customer-pinned audit-chain anchor
- `self-hosted` × `colo` → same as on-prem with facility-level seam
- `demo`/`sandbox`/`trial`/`dev` × `guest-on-oci` (Always Free) → resource-budgeted Ampere A1 + Autonomous DB; throttled MRP cadence

**§4 verdict**: tenant scoping is structurally PRESENT but uses retired `tier` semantics; tenant_class composability is NOT YET ENCODED in PRD or manifest. P0 on §3.2; PASS on §4.1, §4.2, §4.3.

## §5 §3.4.C — Deployment-Context Substance Gate

See D8 in §2.8 + finding §3.8 (P0 six-context iac/ layout missing). Detailed context applicability for production-planning:

### §5.1 Context 1 — `oyatie-public-cloud` — REQUIRED
SaaS manufacturers (especially mid-market discrete manufacturing) buy SaaS APS; this is the default context. Required deliverable: `iac/oyatie-public-cloud/` OpenTofu module backing the Oyatie cell topology.

### §5.2 Context 2 — `guest-on-aws` — REQUIRED
Enterprise manufacturing buyers with AWS-anchored ERP (e.g., Coca-Cola Bottling, GE Aviation pilots) require Oyatie-on-AWS guest mode. Required deliverable: `iac/guest-on-aws/` OpenTofu module + S3+DynamoDB state backend.

### §5.3 Context 3 — `guest-on-oci` — REQUIRED + Always-Free sub-profile
OCI is Oracle's manufacturing cloud (Fusion SCP / SCM Cloud). Buyer affinity is real. Required deliverable: `iac/guest-on-oci/` standard + `iac/oci-guest/always-free/` Ampere A1 demo/sandbox/trial/dev module per ADR-0328 §D-19.

### §5.4 Context 4 — `on-prem` — REQUIRED (Phase-4 manufacturing canonical)
Manufacturing buyers have intermittent factory-floor connectivity, sovereign IP residency requirements, and decade-long capital-equipment lifecycles. On-prem is **the** canonical Phase-4 manufacturing context. Required deliverable: `iac/on-prem/` OpenTofu module + MinIO state backend + k8s + Cloud Hypervisor.

### §5.5 Context 5 — `colo` — REQUIRED
Tier-1 manufacturers (auto OEMs, aerospace primes) use colocation. Required deliverable: `iac/colo/` OpenTofu module + Equinix Metal / Cyxtera provider integration.

### §5.6 Context 6 — `oyatie-as-cloud-provider` — REQUIRED
Per ADR-0328 §D-15.81..§D-15.99, every Phase-4 product running on Oyatie's IaaS surface must declare this context. Required deliverable: `iac/oyatie-as-cloud-provider/` consuming cloud-* µservices natively.

**§5 verdict**: all six contexts are REQUIRED for production-planning; zero of six are currently implemented; manifest does not declare them. P0 finding §3.8.

## §6 §3.4.M — MRP / MPS / Finite-Scheduling Primitive Coverage

### §6.1 MRP (Material Requirements Planning) coverage
- IP-002 (mrp-run domain layer): aggregate root `mrp_run_document`, invariants tenant-scope/version-monotonic/source-system-provenance/destructive-correction-forbidden. **PASS-shape**.
- IP-008 (mrp-run usecase layer): command handlers + read models. **PASS-shape**.
- IP-016 (MRP-explosion-to-supply-chain-planning handoff): explosion handoff. **PASS-shape**.
- **MISSING**: explicit net-requirements calculation, lot-sizing rules (Fixed Quantity FX / Period of Supply PD / Economic Order EOQ / Lot-for-Lot LFL / Wagner-Whitin / SM heuristic), safety-stock policies (static / dynamic / service-level-driven), ATP (Available-To-Promise) from MRP output, low-level-code algorithm (BOM depth-first traversal with cycle detection), pegging (single-level / multi-level pegging). **P0 finding under §3.4.M.**

### §6.2 MPS (Master Production Schedule) coverage
- IP-022 (long-term-planning-versus-short-term-planning-split): horizon split. **PASS-shape**.
- IP-019 (sop-horizon-monthly-cycle-with-executive-signoff-gate): S&OP horizon cycle. **PASS-shape**.
- **MISSING**: explicit master-production-schedule generation, planning-time-fence (PTF) and demand-time-fence (DTF) enforcement, firm-planned-order (FPO) semantics, MPS-to-MRP cascade, MPS lock and unlock workflow. **P0 finding under §3.4.M.**

### §6.3 Finite-scheduling coverage
- IP-021 (capacity-leveling-finite-scheduling-forward-backward-bottleneck): forward + backward + bottleneck-anchor strategies, DBR, AC-3 constraint propagation, JSSP reducibility named, benchmark p50/p95/p99 ship per scale. **PASS — substance bar met.**

### §6.4 Dispatching coverage
- IP-006 (shop-floor-release-domain) + IP-012 (shop-floor-release-usecase) + IP-017 (shop-floor-release-to-warehouse-staging-handoff): release domain + handoff. **PASS-shape**.
- IP-024 (MES-handshake-bidirectional-event-flow-isa-95): outbound dispatch via ISA-95 B2MML. **PASS — substance bar met for the integration plane**.
- **PARTIAL — MISSING**: explicit dispatch list (SAP `CO04`/`CO05`/`COHV` parallel), priority-sequencing rules (EDD Earliest-Due-Date / SPT Shortest-Processing-Time / CR Critical-Ratio / Slack), conveyor/palette integration semantics. **P1 finding under §3.4.M.**

### §6.5 Change-over / sequence-dependent setup coverage
- **MISSING**: no IP slice covers change-over. SAP DELMIA Quintiq, Siemens Opcenter, Kinaxis Maestro all ship sequence-dependent setup matrices and family-grouping heuristics. **P0 finding under §3.4.M; see §3.6.**

### §6.6 Bottleneck management coverage
- IP-021 §D-3 covers bottleneck identification + DBR anchoring. **PASS — TOC-author TOC explicitly named.**

### §6.7 What-if scheduling coverage
- **MISSING**: no IP slice covers scenario branching. **P0 finding under §3.4.M; see §3.10.**

### §6.8 OEE coverage
- **MISSING**: no IP slice covers OEE = A × P × Q. **P1 finding under §3.4.M; see §3.11.**

### §6.9 Real-time shop-floor integration coverage
- IP-024 (MES handshake) ships ISA-95 / B2MML / AsyncAPI-over-Kafka. **PASS-shape**.
- **MISSING**: native OPC-UA, MQTT-Sparkplug-B, EtherCAT, PROFINET. These are the field-bus protocols on factory floors; without them, production-planning depends on a MES vendor adapter (which is the IP-024 path). **P2 finding** — if Oyatie wants to displace MES vendors, native field-bus is required.

### §6.10 ML anomaly detection on shop-floor signals
- **MISSING**. **P1 finding under §3.4.M; see §3.12.**

### §6.11 Mobile coverage
- **MISSING**. **P1 finding under §3.4.M; see §3.13.**

**§6 verdict**: 4 PASS (finite scheduling, bottleneck, MES handshake, dispatching-integration), 4 P0 (MRP detail, MPS detail, change-over, what-if), 4 P1 (dispatching priority rules, OEE, ML anomaly, mobile), 1 P2 (native field-bus).

## §7 Cross-Microservice Boundary Audit

### §7.1 Outbound: production-planning → supply-chain-planning
- IP-016 (mrp-explosion-to-supply-chain-planning-handoff) declares the handoff. PASS-shape.
- ARCHITECTURE.md §D names `supply-chain-planning` as a downstream integration? — actually integration_points in manifest names `ontology, workflow-engine, warehouse, quality-management, finops-portal, marketplace`. **FINDING P1** — supply-chain-planning is named in IP-016 but NOT in manifest.integration_points. Manifest is stale.

### §7.2 Outbound: production-planning → warehouse (shop-floor-release → warehouse staging)
- IP-017 (shop-floor-release-to-warehouse-staging-handoff). PASS-shape.
- manifest.integration_points includes `warehouse`. PASS.

### §7.3 Outbound: production-planning → quality-management
- manifest.integration_points includes `quality-management`. PASS.
- **FINDING P2** — no IP slice covers the QM handshake (QC inspection result → MRP demand release). Should be one.

### §7.4 Outbound: production-planning → finops-portal
- manifest.integration_points includes `finops-portal`. PASS.

### §7.5 Outbound: production-planning → marketplace
- manifest.integration_points includes `marketplace`. PASS.
- ADR-0314 declares marketplace settles tenant deals. PRD Story PP-013 covers settlement evidence. PASS.

### §7.6 Outbound: production-planning → ontology
- manifest.integration_points includes `ontology`. PASS.
- PRD §D.1..§D.6 declares 6 ontology object projections. PASS.

### §7.7 Outbound: production-planning → workflow-engine
- manifest.integration_points includes `workflow-engine`. PASS.

### §7.8 Outbound: production-planning → MES (external)
- IP-024 declares ISA-95 / B2MML handshake. PASS — see §6.4.

### §7.9 Outbound: production-planning → plant-maintenance (NEW)
- **FINDING P1** — manifest does not include `plant-maintenance` in integration_points. SAP PM is the canonical plant-maintenance counterpart and `microservices/plant-maintenance/` exists in the repo. Production-planning MUST handshake with plant-maintenance for downtime windows that block work-center capacity (preventive maintenance, break-fix). Without this handshake, capacity-calendar is fictional.

### §7.10 Inbound: supply-chain-planning → production-planning
- IP-016 declares this as outbound from MRP to SCP. Inbound is forecast → MPS in the reverse direction. **FINDING P1** — no IP slice covers inbound forecast from SCP.

## §8 Doctrine Adherence Roll-Up

| Doctrine | Reference | Production-Planning Status |
|---|---|---|
| Tier retired → tenant_class composable | 2026-05-20 directive | **P0 DRIFT** — PRD + manifest still use `tenant_class` |
| Performance: industry-leader + context overlay + tenant_class overlay | 2026-05-20 directive | **P0 DRIFT** — flat numbers |
| Rust-only backend | ADR-0328 §D-18 | PASS |
| OpenTofu-only IaC | ADR-0328 §D-16 | **P1 DRIFT** — `terraform-module/` dir name |
| Six-context deployment | ADR-0328 §D-15 | **P0 DRIFT** — flat iac/ shape |
| OS Tier-1 manifest | ADR-0328 §D-17 | **P1 DRIFT** — no supported-oses.json |
| HTTP/3 default | ADR-0253 | PASS |
| HLC default | ADR-0252 | PASS — IP-024 substantively |
| Cedar universal gate | ADR-0243 | PASS-shape (6 policies) |
| Tenant universal scoping | ADR-0244 | PASS-shape; **P0 DRIFT** via tier dimension |
| Audit-chain emission | ADR-0263 | **P1 DRIFT** — roster mismatch + granularity |
| Marketplace settlement | ADR-0314 | PASS |
| SAP module parity | ADR-0315 | **P0 DRIFT** — competitor parity matrix template-stamped |
| Substance bar | ADR-0322 + ADR-0328 §D-1..§D-2 | **P0 DRIFT** — 4 strategic docs template-stamped |
| Anti-template/anti-script | ADR-0324 | **P0 DRIFT** — same 4 docs |
| Per-microservice flat layout | ADR-0131 | PASS |
| No-suite microservices | ADR-0132 | PASS |
| 13-layer enum | ADR-0105 | **P1 DRIFT** — Cargo metadata uses `app` not `application` |
| Direct gRPC inter-microservice | ADR-0145 | **P1 DRIFT** — proto file missing or unlisted |
| Amazon cellular | ADR-0248 | PASS |

## §9 Severity Roll-Up

- **P0 (blocks ERP-parity claim)**: 8 findings
  - §3.2 tier doctrine drift
  - §3.4 competitor parity matrix template-stamping
  - §3.5 MRP-run latency target false
  - §3.6 change-over MISSING
  - §3.8 six-context iac/ layout MISSING
  - §3.9 Cargo/catalog mismatch
  - §3.10 what-if scheduling MISSING
  - §6.1 + §6.2 MRP/MPS substance MISSING

- **P1 (blocks Wave-4 phase exit)**: 12 findings
  - §3.1 README template-stamping
  - §3.3 ARCHITECTURE template-stamping
  - §3.7 proto3 file missing
  - §3.11 OEE MISSING
  - §3.12 ML anomaly MISSING
  - §3.13 mobile MISSING
  - §3.14 manufacturing compliance packs MISSING
  - §3.15 EU AI Act MISSING
  - §3.19 audit event roster mismatch
  - §3.20 audit event granularity
  - §3.21 Cargo `app` vs `application`
  - §3.22 Terraform vs OpenTofu
  - §6.4 dispatching priority rules MISSING
  - §7.1 supply-chain-planning not in manifest.integration_points
  - §7.9 plant-maintenance not in manifest.integration_points
  - §7.10 inbound SCP forecast IP MISSING

- **P2 (documentation gap)**: 7 findings
  - §3.16 pack roster case inconsistency
  - §3.17 dashboard MD vs JSON
  - §3.18 capabilities roster gap (3 of 6)
  - §6.9 native field-bus MISSING (OPC-UA / MQTT-Sparkplug-B)
  - §7.3 QM handshake IP MISSING

**Total**: 8 P0 + 16 P1 + 5 P2 = 29 findings.

## §10 Audit Verdict and Path to Production

### §10.1 Verdict
**APS-substance assessment**: HYBRID. The 25-IP slice tier carries APS-substance comparable to or exceeding SAP APO PP/DS at the unit level (finite scheduling, DBR, JSSP reducibility, ISA-95 / B2MML handshake, DDMRP, S&OP, alternative routing). The strategic-doc tier (README, ARCHITECTURE, competitor-parity-matrix, PRD §F observability rows) is **template-stamped** and fails the substance bar. The compliance-pack roster, deployment-context iac/ layout, and Cargo/catalog reconciliation are **pre-doctrine** relative to the 2026-05-20 directive.

### §10.2 Phase-4 ERP-parity gate: NO-GO
Per the §3.4.M MRP/MPS/finite-scheduling primitive coverage requirement, this microservice cannot claim Phase-4 ERP-parity until:
1. The 8 P0 findings are remediated.
2. The competitor-parity-matrix is rewritten with bespoke per-counterpart per-feature substance (see companion `feature-parity-matrix-2026-05-20.md`).
3. The performance SLO numbers are rewritten with industry-leader + context-overlay + tenant_class-overlay grounding (see companion `performance-benchmark-numbers-2026-05-20.md`).
4. Six-context iac/ subdirectories are created with OpenTofu modules.
5. Tier-doctrine drift is purged from PRD + manifest.

### §10.3 Wave-4 phase exit: NO-GO
The 16 P1 findings block phase exit. Top three blockers: (a) audit event granularity (`-CHANGED` is too generic for an APS audit trail buyer), (b) manufacturing compliance packs (FDA 21 CFR Part 11 / ISO 13485 / AS9100 / IATF 16949 + EU AI Act), (c) plant-maintenance handshake (capacity-calendar is fictional without it).

### §10.4 Strengths to preserve
- **IP-021 finite-scheduling** is the paid_core-standard slice of this microservice; future authoring should match its substance level.
- **IP-024 MES handshake / ISA-95 / B2MML** is unusually well-grounded — declares the standards body (MESA), the message types (B2MML), the named ISA-95 levels (3 ↔ 4), the wall-clock reconciliation (HLC + UTC drift ≤ ±2s), the vendor adapter abstraction, and the state-machine drift detection. This is genuinely interoperable.
- **HLC + TrueTime tier** declaration in manifest is substantively right for a manufacturing service that crosses time zones and that ingests external MES wall-clock signals.
- **Marketplace settlement reference** (ADR-0314 + Story PP-013) is correctly delegated to the marketplace microservice; production-planning does not re-implement settlement.

### §10.5 Next-IP roster (proposed; remediation deferred to dispatch)
- IP-26: README + ARCHITECTURE + PRD-user-story de-template-stamping + tier→tenant_class migration.
- IP-27: contracts/production-planning-v1.proto authoring.
- IP-28: six-context iac/ migration.
- IP-29: Cargo/catalog reconciliation.
- IP-30: scenario branching / what-if (Kinaxis differentiator).
- IP-31: OEE tracking.
- IP-32: ML anomaly detection handshake with `detection` microservice.
- IP-33: mobile supervisor/operator REST + AsyncAPI surface.
- IP-34: manufacturing compliance packs (FDA / ISO 13485 / AS9100 / IATF 16949).
- IP-35: EU AI Act risk classification.
- IP-36: manifest normalization (case, integration_points, event roster).
- IP-37: dashboard MD → JSON.
- IP-38: capabilities for routing-step + production-order + shop-floor-release.
- IP-39: manifest audit event roster reconciliation (MES handshake events).
- IP-40: granular per-state-transition audit event class enumeration.
- IP-41: Cargo `app` → `application` rename.
- IP-42: `terraform-module/` → `opentofu-module/` rename (or subsumption).
- IP-43: MRP net-requirements + lot-sizing rules + safety-stock + ATP + low-level-code + pegging.
- IP-44: MPS generation + planning-time-fence / demand-time-fence + firm-planned-order + lock workflow.
- IP-45: dispatching priority rules (EDD / SPT / CR / Slack).
- IP-46: change-over / sequence-dependent setup matrices.
- IP-47: plant-maintenance handshake (downtime → capacity-calendar block).
- IP-48: supply-chain-planning inbound forecast IP.
- IP-49: quality-management handshake IP.
- IP-50: native field-bus (OPC-UA / MQTT-Sparkplug-B) — P2; defer to post-Wave-4.

### §10.6 Anti-recurrence rule
Future authoring dispatch into production-planning MUST:
1. Cite IP-021 + IP-024 as the substance bar (not the README or ARCHITECTURE §H).
2. Carry SAP transaction code + Oracle endpoint + Kinaxis Maestro action name in every parity row.
3. Carry industry-leader benchmark number + deployment-context overlay + tenant-class overlay in every performance assertion.
4. Use `tenant_class`, never `tier`, in metric dimensions, ontology projections, Cedar context, and audit-event dimensions.
5. Use `iac/<context>/` paths, never flat `iac/<tool>/` paths.

## §11 Evidence Roster (auditable file paths)

Inspected files (absolute paths):
- /Users/jasonlee/oyatie/microservices/production-planning/manifest.json
- /Users/jasonlee/oyatie/microservices/production-planning/README.md
- /Users/jasonlee/oyatie/microservices/production-planning/ARCHITECTURE.md
- /Users/jasonlee/oyatie/microservices/production-planning/PRD.md
- /Users/jasonlee/oyatie/microservices/production-planning/PHASE-01-PRODUCTION-PLANNING-PARITY.md
- /Users/jasonlee/oyatie/microservices/production-planning/competitor-parity-matrix.md
- /Users/jasonlee/oyatie/microservices/production-planning/capacity-model.md
- /Users/jasonlee/oyatie/microservices/production-planning/AUDIT-FINDINGS-2026-05-21.json
- /Users/jasonlee/oyatie/microservices/production-planning/Cargo.toml
- /Users/jasonlee/oyatie/microservices/production-planning/IP-001..IP-025 (25 files)
- /Users/jasonlee/oyatie/microservices/production-planning/policy/*.cedar (6 files)
- /Users/jasonlee/oyatie/microservices/production-planning/slos/*.openslo.yaml (4 files)
- /Users/jasonlee/oyatie/microservices/production-planning/contracts/openapi-v1.yaml
- /Users/jasonlee/oyatie/microservices/production-planning/contracts/asyncapi-v1.yaml
- /Users/jasonlee/oyatie/microservices/production-planning/catalog/*.yaml (54 files)
- /Users/jasonlee/oyatie/microservices/production-planning/iac/{ech-config, edge-waf, helm-values, k8s-deployment, network-policy, openbao-policy, pqc-cert, secret-bindings, terraform-module}.yaml
- /Users/jasonlee/oyatie/docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md (anchor §D-15..§D-20)
- /Users/jasonlee/oyatie/docs/standards/brief-template.md (anchor §3.4, §3.9..§3.12)

Companion deliverables in this audit:
- /Users/jasonlee/oyatie/microservices/production-planning/feature-parity-matrix-2026-05-20.md
- /Users/jasonlee/oyatie/microservices/production-planning/performance-benchmark-numbers-2026-05-20.md

## §12 HALT-CLEANLY Conditions (none triggered)

No HALT-CLEANLY condition triggered in this audit:
1. All five canonical anchors were readable.
2. No concurrent claim on the audit deliverable files.
3. Audit-only mode; no remediation attempted.
4. Substance bar met for the audit deliverables themselves (no fabricated vendor / regulatory / Cedar / SLO claim).
5. No scripting / template substitution used for audit body content.
6. No hard contradiction between authority-tier peers (ADR-0316 vs the 2026-05-20 directive is resolved in favor of the directive per the user's instruction; this is recorded as the §3.2 finding).
7. Verification: all 152 files were either directly read or evidence-sampled via grep; AUDIT-FINDINGS-2026-05-21.json verdict is acknowledged but does not overrule this audit's findings (it predates the 2026-05-20 doctrine directive).

## §13 Closing Note

production-planning is the hardest service in Phase 4. The good news: IP-021 + IP-024 prove the team can write APS substance at the SAP APO PP/DS level when given the right brief. The bad news: the strategic-doc tier is template-stamped, the tier doctrine is retired but still encoded, and the six-context iac/ layout has not landed. With the 29 findings remediated and the 25-slice IP roster expanded to ~50 (per §10.5), this microservice can credibly displace SAP APO + Oracle SCP + Kinaxis Maestro for the mid-market discrete-manufacturing buyer segment. Without remediation, it ships as a CRUD shell with APS branding — which is exactly what every Kinaxis salesperson will demonstrate against in a buyer evaluation.

The verdict is NO-GO for Phase-4 ERP-parity until the 8 P0 items are remediated. The verdict is NO-GO for Wave-4 phase exit until the 16 P1 items are remediated. The verdict is GO for continued authoring of the IP-26..IP-50 roster.

End of audit.
