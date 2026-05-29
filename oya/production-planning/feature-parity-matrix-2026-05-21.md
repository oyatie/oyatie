---
doc_class: FeatureParityMatrix
microservice: production-planning
audit_wave: wave-4-rolling
audit_date: 2026-05-20
authored_at: 2026-05-21
top_3_counterparts:
  - sap-apo-pp-ds: SAP APO Advanced Planner & Optimizer Production Planning and Detailed Scheduling + S/4HANA PP/DS
  - oracle-scp: Oracle Supply Chain Planning Cloud — Production Scheduling Cloud Service + Constraint-Based Optimization
  - kinaxis-maestro: Kinaxis Maestro on RapidResponse — concurrent planning with scenario branching
companion_docs:
  - microservices/production-planning/coherence-audit-2026-05-20.md
  - microservices/production-planning/performance-benchmark-numbers-2026-05-20.md
related_adrs:
  - ADR-0315 SAP module parity
  - ADR-0145 inter-microservice direct gRPC
  - ADR-0244 tenant universal scoping primitive
  - ADR-0328 substance bar
no_tier_deltas: true
authoring_doctrine: tier retired; tenant_class composable; one industry-leader target + deployment-context overlay + tenant-class overlay
---

# Feature Parity Matrix: production-planning vs SAP APO + Oracle SCP + Kinaxis Maestro

## §1 Purpose and Counterpart Identity

This matrix is the **UNION-coverage** parity matrix between Oyatie `production-planning` and the three top APS counterparts. UNION-coverage means: every feature that ANY of the three counterparts ships becomes a row. Oyatie's parity claim is then YES / PARTIAL / NO / N/A per row, with a named differentiator or gap.

### §1.1 Counterpart 1 — SAP APO PP/DS + S/4HANA PP/DS
- **Vendor**: SAP SE
- **Product line**: SAP APO PP/DS (legacy on-prem), SAP S/4HANA PP/DS (current cloud + on-prem), SAP IBP for Response & Supply (cloud successor)
- **Engine type**: in-memory finite-scheduling engine with constraint propagation; pegging engine; PP/DS heuristic stack
- **Headline transactions / endpoints** (selected, named): `MD01` (MRP single-level), `MD02` (MRP single-item single-level), `MD03` (MRP single-item multi-level), `MD04` (stock/requirements list), `MD41` (MPS single-item), `MD43` (MPS multi-item), `CM21` (capacity planning table), `CM27`/`CM28` (interactive capacity leveling), `/SAPAPO/CDPSC` (PP/DS detailed scheduling planning board), `/SAPAPO/RRP3` (PP/DS interactive horizon), `CO01` (production order create), `CO02` (production order change), `CO04` (production order print), `CO05` (production order release), `COHV` (mass order processing), `CR01` (work-center create), `CA01` (routing create), `CS01` (BOM create)
- **Auth**: SAP user role concept (PFCG roles), OAuth 2.0 for S/4HANA Cloud APIs
- **Data shape**: ABAP tables `MARA` (material master), `MARC` (plant material), `STPO` (BOM item), `PLAF` (planned order), `AFKO` (production order header), `AFPO` (production order item), `AFRU` (confirmation), `AUFK` (order master)

### §1.2 Counterpart 2 — Oracle SCP Cloud
- **Vendor**: Oracle Corporation
- **Product line**: Oracle Supply Chain Planning Cloud (release 24A/24B/24C/24D + 25A/25B), Production Scheduling Cloud Service, Constraint-Based Optimization
- **Engine type**: in-memory constraint solver; scenario-based replanning; integrated with Oracle Fusion Cloud Manufacturing
- **Headline endpoints / actions** (selected, named): Plan Inputs Diagnostics, Run Plan, Release Plan, Compare Plan Versions, Manage Items, Manage BOMs, Manage Routings, Manage Work Definitions, Manage Resources, Manage Calendars, Production Scheduling Workbench, Demand Management Workbench, Sales and Operations Planning Workbench, Inventory Planning Workbench, Replenishment Planning Workbench
- **REST surface**: `/fscmRestApi/resources/{version}/productionSchedules`, `/fscmRestApi/resources/{version}/supplyOrders`, `/fscmRestApi/resources/{version}/workOrders`, `/fscmRestApi/resources/{version}/workDefinitions`
- **Auth**: Oracle IDCS / OCI IAM OAuth 2.0
- **Data shape**: Oracle Fusion tables `EGP_SYSTEM_ITEMS_B`, `INV_ORG_PARAMETERS`, `WIS_WORK_ORDER_HEADERS`, `WIS_WORK_ORDER_OPERATIONS`, `MSC_PLAN_BUCKETS`, `MSC_DEMANDS`, `MSC_SUPPLIES`

### §1.3 Counterpart 3 — Kinaxis Maestro on RapidResponse
- **Vendor**: Kinaxis Inc.
- **Product line**: Kinaxis Maestro (formerly RapidResponse), concurrent planning surface; Maestro Applications (Demand Planning, Supply Planning, Capacity Planning, Inventory Planning, S&OP, Order Fulfillment); Maestro Resource Library and Bookmark stack
- **Engine type**: concurrent-planning in-memory engine; scenario-branching graph; always-on calculation; algorithmic stack includes optimization (CPLEX-backed for some applications) + heuristics
- **Headline actions / scripts** (selected, named): `RunNetting` (MRP), `RunCapacity` (capacity scheduling), `RunForecast` (demand forecast), `RunInventoryPolicy` (inventory replenishment), `CreateScenario` (scenario fork), `MergeScenario` (scenario merge), `CompareScenarios`, `Publish`, `RapidResponse Authoring Environment` (RAE) for workbook authoring, Sankey / Pegging viewers, AlertDefinition / AlertSubscription for exception management
- **REST surface**: `/REST/Data`, `/REST/Bookmark`, `/REST/Scenario`, `/REST/Calculation`, `/REST/Worksheet`
- **Auth**: Kinaxis SSO via SAML 2.0 + OAuth 2.0
- **Data shape**: in-memory dataset with workbook overlays; canonical entities Part, Site, Operation, Resource, Order, Demand, Supply, Constraint

### §1.4 Audit method
For each row: feature description, counterpart-specific surface (transaction or endpoint), Oyatie counterpart endpoint (proposed or actual), parity verdict (YES / PARTIAL / NO / N/A / GAP-FIXABLE-IN-IP-NN), differentiator note. Tier deltas are NOT used; tenant_class + deployment_context overlay is named when relevant.

## §2 Feature Group A — BOM Management

### A.1 Single-level BOM definition with engineering-change-order versioning
- **SAP**: `CS01`/`CS02`/`CS03` create/change/display BOM; `CC01` engineering change number; ECN-driven version control; effectivity dates
- **Oracle SCP**: Manage Bill of Materials task in Manufacturing; ECO (Engineering Change Order) workflow in Product Lifecycle Management
- **Kinaxis Maestro**: `BillOfMaterial` table with effectivity date columns; ECO via PLM integration (not native)
- **Oyatie**: IP-001 (bom-revision domain) + IP-007 (bom-revision usecase); BomRevision aggregate root with version monotonic, source-system provenance immutable, ECO via amend/approve/reverse commands
- **Verdict**: PARTIAL. Oyatie ships ECO via state-machine, but no explicit ECO number entity or effectivity date pair. Gap: add ECO number + effectivity from/to per BOM revision. **IP-43-ECO** (proposed).

### A.2 Multi-level BOM explosion (low-level code computation)
- **SAP**: `CS11`/`CS12`/`CS15` (multi-level BOM listing); `BOM_M_VARIANTEN` (variant BOM); low-level code in `MARC-DISLS`
- **Oracle SCP**: Manage Item Structures (multi-level); Low Level Code field on item
- **Kinaxis Maestro**: `BillOfMaterial` table joined recursively in scripts; low-level-code derived by `RunNetting`
- **Oyatie**: IP-016 (MRP-explosion-to-supply-chain-planning handoff) declares the explosion handoff but does NOT name the low-level-code algorithm explicitly. Gap.
- **Verdict**: PARTIAL. Need: explicit low-level-code algorithm (BOM depth-first traversal + cycle detection + max-depth tracking), per-tenant cache, recompute trigger. **IP-43-LLC** (proposed).

### A.3 Phantom (logical) assembly BOM items
- **SAP**: special procurement type `50` (phantom); `MARC-SOBSL`
- **Oracle SCP**: BOM Component Type = "Phantom"
- **Kinaxis Maestro**: `BillOfMaterial.UsageType = Phantom`
- **Oyatie**: NOT NAMED in any IP. Gap.
- **Verdict**: NO. **IP-43-PHANTOM** (proposed).

### A.4 Variant / configurable BOM
- **SAP**: Variant Configuration (`CU40`/`CU41` configuration profile); class-characteristic system
- **Oracle SCP**: Configure-to-Order (CTO) with Configurator Cloud
- **Kinaxis Maestro**: Configured Part via Maestro Applications
- **Oyatie**: NOT NAMED. Gap.
- **Verdict**: NO. **IP-43-VARIANT** (proposed). This is a P1 buyer-blocker for any discrete-manufacturing buyer with CTO products.

### A.5 Co-product and by-product BOM
- **SAP**: BOM item category `M` (intra-material); joint production via `MD61` flexible planning
- **Oracle SCP**: Co-Product and By-Product columns in Work Definition
- **Kinaxis Maestro**: `BillOfMaterial.UsageType = Coproduct`
- **Oyatie**: IP-020 (production-version-selection-with-co-product-yield-variance) covers co-product + by-product + yield variance. **PASS**.
- **Verdict**: YES. Differentiator: co-product yield variance is modeled as a first-class IP slice with explicit yield-variance handling, which neither SAP APO nor Kinaxis Maestro ship as a top-level concept.

### A.6 BOM change-effectivity (date-effective and serial-number-effective)
- **SAP**: ECN with `valid-from` and `valid-to` dates; serial-effective via `iPart` integration
- **Oracle SCP**: ECO `Effective From` / `Effective To`; serial-effective via Item Configurator
- **Kinaxis Maestro**: `BillOfMaterial.EffectiveDate` / `ExpirationDate`
- **Oyatie**: NOT NAMED. Gap.
- **Verdict**: NO. **IP-43-EFFECTIVITY** (proposed).

## §3 Feature Group B — MRP (Material Requirements Planning)

### B.1 Single-level MRP run
- **SAP**: `MD01` (total planning), `MD02` (single-item single-level); planning mode 1/2/3/NEUPL
- **Oracle SCP**: Run Plan with Plan Type = Material Plan
- **Kinaxis Maestro**: `RunNetting` script
- **Oyatie**: IP-002 (mrp-run domain) + IP-008 (mrp-run usecase). PASS-shape.
- **Verdict**: PARTIAL. The IP slices declare MRP-run state machine but not the netting algorithm. Gap. **IP-43-MRP-CORE** (proposed): net-requirements = gross-requirements - on-hand - in-transit - scheduled-receipts + safety-stock, with lot-sizing rule application.

### B.2 Multi-level MRP run
- **SAP**: `MD01` with planning mode = multilevel
- **Oracle SCP**: Plan Type = Material Plan covers multi-level
- **Kinaxis Maestro**: `RunNetting` is inherently multi-level
- **Oyatie**: IP-016 declares MRP-explosion-to-supply-chain-planning handoff; does NOT name multi-level netting algorithm
- **Verdict**: PARTIAL. **IP-43-MRP-MULTI** (proposed).

### B.3 Lot-sizing rules: Fixed Quantity (FX)
- **SAP**: lot-sizing procedure `FX`
- **Oracle SCP**: Order Modifier Method = Fixed Order Quantity
- **Kinaxis Maestro**: `Part.LotSize` field
- **Oyatie**: NOT NAMED. **IP-43-LOTSIZE-FX** (proposed).
- **Verdict**: NO.

### B.4 Lot-sizing rules: Period of Supply (PD)
- **SAP**: `PD` (period of supply); planning calendar
- **Oracle SCP**: Order Modifier Method = Period of Supply
- **Kinaxis Maestro**: `Part.PeriodOfSupply`
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-43-LOTSIZE-PD**.

### B.5 Lot-sizing rules: Economic Order Quantity (EOQ)
- **SAP**: `EX` (exact lot for lot, EOQ via cost calculation)
- **Oracle SCP**: Order Modifier Method = Economic Order Quantity
- **Kinaxis Maestro**: EOQ via Maestro Applications Inventory Optimization
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-43-LOTSIZE-EOQ**.

### B.6 Lot-sizing rules: Lot-for-Lot (LFL)
- **SAP**: `EX` exact lot
- **Oracle SCP**: Order Modifier Method = Lot for Lot
- **Kinaxis Maestro**: default lot rule
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-43-LOTSIZE-LFL**.

### B.7 Lot-sizing rules: Wagner-Whitin optimal lot sizing
- **SAP**: `WB` Wagner-Whitin
- **Oracle SCP**: Wagner-Whitin available in Plan Options
- **Kinaxis Maestro**: optimization-based via Inventory Optimization application
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-43-LOTSIZE-WW**.

### B.8 Lot-sizing rules: SM lot-sizing heuristic
- **SAP**: `SM` SM heuristic
- **Oracle SCP**: SM heuristic available
- **Kinaxis Maestro**: via Maestro Applications scripting
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-43-LOTSIZE-SM**.

### B.9 Safety stock: static (fixed quantity)
- **SAP**: `MARC-EISBE` (safety stock); `MARC-EISLO` (safety stock level dynamic)
- **Oracle SCP**: Item.Safety Stock Quantity
- **Kinaxis Maestro**: `Part.SafetyStock`
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-43-SAFETY-STATIC**.

### B.10 Safety stock: dynamic (service-level driven)
- **SAP**: `MRP4` safety stock days; service-level via Demand Planning
- **Oracle SCP**: Safety Stock Method = Service Level
- **Kinaxis Maestro**: Inventory Optimization with service-level target
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-43-SAFETY-SERVICE-LEVEL**.

### B.11 ATP (Available To Promise) from MRP output
- **SAP**: `CO09` ATP check; `MD04` ATP integration
- **Oracle SCP**: Global Order Promising; ATP check on sales order line
- **Kinaxis Maestro**: `RunOrderFulfillment` with ATP calculation
- **Oyatie**: NOT NAMED in MRP IPs; should integrate with supply-chain-planning ATP surface
- **Verdict**: NO. **IP-43-ATP** (proposed) — cross-microservice handoff to supply-chain-planning.

### B.12 Pegging (single-level)
- **SAP**: `MD09` pegged requirements; `/SAPAPO/PEG1` PP/DS pegging
- **Oracle SCP**: Plan Pegging report
- **Kinaxis Maestro**: Pegging viewer (native)
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-43-PEGGING-SINGLE**.

### B.13 Pegging (multi-level)
- **SAP**: `/SAPAPO/PEG2` multi-level pegging
- **Oracle SCP**: Multi-level pegging via Plan Pegging
- **Kinaxis Maestro**: Multi-level pegging (native)
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-43-PEGGING-MULTI**.

### B.14 Net-change MRP run (only changed material masters)
- **SAP**: `MD01` planning mode `NETPL`; net-change indicator on `MARC-DISPO`
- **Oracle SCP**: Run Plan with Refresh Mode = Net Change
- **Kinaxis Maestro**: Default behavior — concurrent planning recomputes only changed branches
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-43-NET-CHANGE**.

### B.15 Net-change planning horizon scoping
- **SAP**: planning horizon in days; `MARC-DZEIT`
- **Oracle SCP**: Planning Horizon in plan options
- **Kinaxis Maestro**: Horizon scope per scenario
- **Oyatie**: IP-022 declares LTP vs short-term horizon split. **PASS-shape** — but does not name the horizon-scoping configuration entity.
- **Verdict**: PARTIAL.

### B.16 MRP exception messages
- **SAP**: `MD06`/`MD07` MRP list with exception messages 10/15/20/...
- **Oracle SCP**: Exception Management Workbench
- **Kinaxis Maestro**: AlertDefinition framework
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-43-EXCEPTIONS**.

### B.17 DDMRP (Demand-Driven MRP) buffer profiles
- **SAP**: not native; via SAP Integrated Business Planning + Demand Driven Replenishment add-on
- **Oracle SCP**: Demand-Driven MRP via Demand Management Cloud
- **Kinaxis Maestro**: DDMRP via Maestro Applications Demand Planning
- **Oyatie**: IP-018 (ddmrp-buffer-profile-authoring-and-daf-recalc). **PASS** — substance present.
- **Verdict**: YES. Differentiator: DDMRP is a first-class IP slice, not an add-on.

## §4 Feature Group C — MPS (Master Production Schedule)

### C.1 Single-item MPS run
- **SAP**: `MD41`
- **Oracle SCP**: Run Plan with Plan Type = Production Plan
- **Kinaxis Maestro**: `RunCapacity` with master plan flag
- **Oyatie**: IP-019 (sop-horizon-monthly-cycle-with-executive-signoff-gate) covers S&OP, NOT MPS directly. Gap.
- **Verdict**: PARTIAL. **IP-44-MPS-SINGLE**.

### C.2 Multi-item MPS run
- **SAP**: `MD43`
- **Oracle SCP**: Production Plan covers multi-item
- **Kinaxis Maestro**: `RunCapacity` is multi-item by default
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-44-MPS-MULTI**.

### C.3 Planning Time Fence (PTF)
- **SAP**: `MARC-FXHOR` planning time fence (days)
- **Oracle SCP**: Planning Time Fence in Plan Options
- **Kinaxis Maestro**: `Part.PlanningTimeFence`
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-44-PTF**.

### C.4 Demand Time Fence (DTF)
- **SAP**: `MARC-EISLO` + planning calendar (effective)
- **Oracle SCP**: Demand Time Fence in Plan Options
- **Kinaxis Maestro**: `Part.DemandTimeFence`
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-44-DTF**.

### C.5 Firm Planned Order (FPO)
- **SAP**: `MARC-DISMM` planned order flag F (firm); `MD12` firm planned order
- **Oracle SCP**: Firm = Yes flag on supply order
- **Kinaxis Maestro**: `IndependentSupply.Firmed = true`
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-44-FPO**.

### C.6 MPS lock / unlock workflow
- **SAP**: planning version; `MD61` flexible planning lock
- **Oracle SCP**: Plan Status workflow (Approved / Released / Locked)
- **Kinaxis Maestro**: Scenario lock / unlock
- **Oyatie**: NOT NAMED. **IP-44-LOCK**.
- **Verdict**: NO.

### C.7 Rough-cut capacity planning (RCCP)
- **SAP**: `CM01` work-center capacity check
- **Oracle SCP**: Rough-Cut Capacity in Plan Options
- **Kinaxis Maestro**: `RunCapacity` with rough-cut flag
- **Oyatie**: NOT NAMED in IPs explicitly; capacity-calendar exists but not RCCP-specific
- **Verdict**: PARTIAL. **IP-44-RCCP**.

### C.8 MPS-to-MRP cascade trigger
- **SAP**: planning mode auto-cascade
- **Oracle SCP**: Cascade Plan in Plan Options
- **Kinaxis Maestro**: Concurrent planning auto-cascades
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-44-CASCADE**.

## §5 Feature Group D — Finite Scheduling

### D.1 Forward scheduling from earliest start
- **SAP**: `CM27` forward leveling; PP/DS forward
- **Oracle SCP**: Production Scheduling Workbench forward mode
- **Kinaxis Maestro**: `RunCapacity` forward direction
- **Oyatie**: IP-021 `schedule_forward` Rust function. **PASS**.
- **Verdict**: YES.

### D.2 Backward scheduling from due date
- **SAP**: `CM27` backward leveling
- **Oracle SCP**: Production Scheduling backward mode
- **Kinaxis Maestro**: `RunCapacity` backward direction
- **Oyatie**: IP-021 `schedule_backward` Rust function. **PASS**.
- **Verdict**: YES.

### D.3 Bottleneck-anchor scheduling (DBR / Drum-Buffer-Rope)
- **SAP**: not first-class; bottleneck via `/SAPAPO/CDPSC` constraint solver
- **Oracle SCP**: Bottleneck scheduling via Production Scheduling
- **Kinaxis Maestro**: Bottleneck via `RunCapacity` with bottleneck flag
- **Oyatie**: IP-021 `schedule_bottleneck_anchor` with DBR + TOC-author TOC explicit. **PASS — above SAP parity** (SAP does not name DBR explicitly).
- **Verdict**: YES. **Differentiator**: explicit TOC-author TOC pattern with named DBR algorithm.

### D.4 Constraint propagation (AC-3 or equivalent)
- **SAP**: PP/DS heuristic stack; not named publicly
- **Oracle SCP**: Constraint-Based Optimization (CBO)
- **Kinaxis Maestro**: not publicly named
- **Oyatie**: IP-021 names AC-3 constraint propagation explicitly. **PASS — above SAP parity**.
- **Verdict**: YES. **Differentiator**: named AC-3 algorithm.

### D.5 Setup-time-aware scheduling (sequence-independent)
- **SAP**: `OPR1` operation setup time
- **Oracle SCP**: Operation setup time
- **Kinaxis Maestro**: `Operation.SetupTime`
- **Oyatie**: IP-021 places operations with `setup_time`; **PASS-shape**.
- **Verdict**: YES.

### D.6 Setup-time-aware scheduling (sequence-dependent matrix)
- **SAP**: setup matrix in PP/DS (`/SAPAPO/MAT_SETUP_GRP`)
- **Oracle SCP**: Setup Matrix in Production Scheduling
- **Kinaxis Maestro**: SetupMatrix in Maestro Applications
- **Oyatie**: NOT NAMED. **P0 gap.**
- **Verdict**: NO. **IP-46-SETUP-MATRIX** (proposed).

### D.7 Family grouping for changeover minimization
- **SAP**: Product Group; group setup-time reduction
- **Oracle SCP**: Item Category / Item Family
- **Kinaxis Maestro**: Part.Family
- **Oyatie**: NOT NAMED. **P0 gap**.
- **Verdict**: NO. **IP-46-FAMILY**.

### D.8 SMED (Single-Minute Exchange of Die) project tracking
- **SAP**: not native; via PLM
- **Oracle SCP**: not native
- **Kinaxis Maestro**: not native
- **Oyatie**: NOT NAMED.
- **Verdict**: N/A (no counterpart ships native). Skip — leave as future differentiation.

### D.9 Drum-Buffer-Rope sizing
- **SAP**: not native
- **Oracle SCP**: Buffer sizing via Constraint-Based Optimization
- **Kinaxis Maestro**: Buffer profiles
- **Oyatie**: IP-021 ships DBR placement but does not ship explicit drum/buffer/rope sizing algorithm. Gap.
- **Verdict**: PARTIAL. **IP-46-DBR-SIZE**.

### D.10 Multi-resource scheduling (work center + secondary resource)
- **SAP**: `CR01` work center + secondary resource via `OPR2`
- **Oracle SCP**: Resource Hierarchy
- **Kinaxis Maestro**: `Resource.Type = Secondary`
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-46-MULTI-RESOURCE**.

### D.11 Calendar-aware scheduling (working time, shifts, holidays)
- **SAP**: `CA01` routing with shift calendar; `OY05` factory calendar
- **Oracle SCP**: Manage Calendars
- **Kinaxis Maestro**: Calendar entity
- **Oyatie**: IP-003 (capacity-calendar-domain) + IP-009 (capacity-calendar-usecase). PASS-shape.
- **Verdict**: PARTIAL. Need explicit shift / holiday / overtime modeling. **IP-46-CALENDAR-DETAIL**.

### D.12 Operation overlap (lap phasing)
- **SAP**: operation overlap percent in routing
- **Oracle SCP**: Operation Overlap Type
- **Kinaxis Maestro**: `Operation.Overlap`
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-46-OVERLAP**.

### D.13 Operation splitting (parallel processing)
- **SAP**: operation splitting in routing
- **Oracle SCP**: Operation Quantity Split
- **Kinaxis Maestro**: `Operation.Split`
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-46-SPLIT**.

## §6 Feature Group E — Dispatching

### E.1 Production order release
- **SAP**: `CO05` order release; `COHV` mass release
- **Oracle SCP**: Release Plan from Production Scheduling
- **Kinaxis Maestro**: `Publish` action
- **Oyatie**: IP-006 (shop-floor-release domain) + IP-012 (shop-floor-release usecase). **PASS-shape**.
- **Verdict**: YES.

### E.2 Dispatch list generation (work-center sequence)
- **SAP**: `CO04` print shop papers; `CR06` capacity dispatch
- **Oracle SCP**: Dispatch List report
- **Kinaxis Maestro**: Maestro Applications Production Scheduling dispatch sheet
- **Oyatie**: NOT NAMED as a dispatch-list entity. Gap.
- **Verdict**: PARTIAL. **IP-45-DISPATCH-LIST**.

### E.3 Priority sequencing rule: Earliest Due Date (EDD)
- **SAP**: PP/DS sequencing heuristic; `H001` EDD
- **Oracle SCP**: Sequencing Rule = Earliest Due Date
- **Kinaxis Maestro**: `RunCapacity` with priority = EDD
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-45-PRIORITY-EDD**.

### E.4 Priority sequencing rule: Shortest Processing Time (SPT)
- **SAP**: `H002` SPT
- **Oracle SCP**: Sequencing Rule = Shortest Processing Time
- **Kinaxis Maestro**: priority = SPT
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-45-PRIORITY-SPT**.

### E.5 Priority sequencing rule: Critical Ratio (CR)
- **SAP**: `H003` CR
- **Oracle SCP**: Sequencing Rule = Critical Ratio
- **Kinaxis Maestro**: priority = CriticalRatio
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-45-PRIORITY-CR**.

### E.6 Priority sequencing rule: Slack-based
- **SAP**: `H004` slack
- **Oracle SCP**: Sequencing Rule = Slack Time
- **Kinaxis Maestro**: priority = Slack
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-45-PRIORITY-SLACK**.

### E.7 MES dispatch handshake (outbound schedule)
- **SAP**: B2MML `ProductionSchedule` from SAP DMC
- **Oracle SCP**: REST `productionSchedules` API
- **Kinaxis Maestro**: `Publish` with B2MML export
- **Oyatie**: IP-024 ISA-95 / B2MML `mes.production-schedule.v1`. **PASS**.
- **Verdict**: YES. **Differentiator**: explicit ISA-95 / B2MML standard reference.

### E.8 Production order printing (shop papers)
- **SAP**: `CO04` shop papers (routing card, operation card, pick list, material withdrawal)
- **Oracle SCP**: Shop Floor Reports
- **Kinaxis Maestro**: Maestro Applications report templates
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-45-SHOP-PAPERS**.

### E.9 Pick list and material staging
- **SAP**: `MB1B` material movement; pick-list integration with WM
- **Oracle SCP**: Pick Slip generation via Inventory
- **Kinaxis Maestro**: not native; via WMS integration
- **Oyatie**: IP-017 (shop-floor-release-to-warehouse-staging-handoff). **PASS-shape**.
- **Verdict**: YES.

### E.10 Conveyor / palette / AGV integration semantics
- **SAP**: via SAP DMC integration; not first-class
- **Oracle SCP**: via Oracle MES
- **Kinaxis Maestro**: via custom integration
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-45-AUTOMATION** — defer to post-Wave-4 (factory-automation-specific, not all buyers need).

## §7 Feature Group F — Bottleneck Management

### F.1 Bottleneck identification (utilization-based)
- **SAP**: `CM01` work-center utilization; PP/DS bottleneck identification
- **Oracle SCP**: Constraint-Based Optimization bottleneck identification
- **Kinaxis Maestro**: `RunCapacity` bottleneck report
- **Oyatie**: IP-021 §D-3 identifies bottleneck as work-center with highest utilization vs capacity. **PASS**.
- **Verdict**: YES.

### F.2 Drum scheduling (bottleneck operations first)
- **SAP**: not first-class
- **Oracle SCP**: via CBO
- **Kinaxis Maestro**: via priority + sequencing
- **Oyatie**: IP-021 `anchor_bottleneck` Rust function. **PASS — above SAP parity**.
- **Verdict**: YES. **Differentiator**: explicit TOC-author TOC drum step.

### F.3 Rope (upstream feeding) scheduling
- **SAP**: not first-class
- **Oracle SCP**: via CBO upstream propagation
- **Kinaxis Maestro**: via pegging + sequencing
- **Oyatie**: IP-021 §schedule_bottleneck_anchor step 3 "pre-schedule (rope) upstream operations to feed bottleneck on time". **PASS**.
- **Verdict**: YES.

### F.4 Buffer (drum protective) scheduling
- **SAP**: not first-class
- **Oracle SCP**: Buffer sizing
- **Kinaxis Maestro**: Buffer profiles
- **Oyatie**: IP-021 step 4 ships downstream drum follow but does not name explicit buffer-size calculation.
- **Verdict**: PARTIAL. **IP-46-DBR-BUFFER**.

### F.5 Bottleneck shift detection
- **SAP**: `CM01` longitudinal
- **Oracle SCP**: Plan comparison
- **Kinaxis Maestro**: Bottleneck shift alert
- **Oyatie**: IP-021 §I event `production-planning.schedule.bottleneck-shift.v1`. **PASS**.
- **Verdict**: YES.

## §8 Feature Group G — What-If Scheduling / Scenario Branching

### G.1 Scenario fork (create scenario from production)
- **SAP**: APO planning version; `/SAPAPO/MVM` copy
- **Oracle SCP**: Scenario Copy
- **Kinaxis Maestro**: `CreateScenario` — **headline differentiator**
- **Oyatie**: NOT NAMED. **P0 gap.**
- **Verdict**: NO. **IP-30-SCENARIO-FORK**.

### G.2 Scenario delta inputs (override demand / supply / capacity)
- **SAP**: planning version overrides
- **Oracle SCP**: Scenario Inputs override
- **Kinaxis Maestro**: Scenario worksheet overlays
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-30-SCENARIO-DELTA**.

### G.3 Scenario merge (resolve scenario back into production)
- **SAP**: planning version copy back
- **Oracle SCP**: Approve Scenario
- **Kinaxis Maestro**: `MergeScenario` — **headline differentiator**
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-30-SCENARIO-MERGE**.

### G.4 Scenario comparison (delta KPIs)
- **SAP**: `/SAPAPO/PLAN_VERSIONS_COMP`
- **Oracle SCP**: Compare Plan Versions
- **Kinaxis Maestro**: `CompareScenarios`
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-30-SCENARIO-COMPARE**.

### G.5 Scenario locking (read-only after publication)
- **SAP**: planning version lock
- **Oracle SCP**: Plan Status = Locked
- **Kinaxis Maestro**: Scenario lock
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-30-SCENARIO-LOCK**.

### G.6 Concurrent planning (always-on, no batch)
- **SAP**: NO — batch-mode MRP
- **Oracle SCP**: PARTIAL — interactive mode + batch
- **Kinaxis Maestro**: YES — **headline differentiator**, "always-on" calculation
- **Oyatie**: NOT NAMED. PRD §F.1129 declares 1000 commands/sec baseline; not concurrent-planning semantics.
- **Verdict**: NO. **IP-30-CONCURRENT-PLANNING**. This is the hardest gap to close; Kinaxis took ~15 years to perfect. Should be a Phase 5 or 6 stretch goal.

## §9 Feature Group H — OEE (Overall Equipment Effectiveness)

### H.1 OEE = Availability × Performance × Quality computation
- **SAP**: via SAP DMC + SAP Plant Connectivity
- **Oracle SCP**: via Oracle MES
- **Kinaxis Maestro**: NOT NATIVE (out of scope)
- **Oyatie**: NOT NAMED. **P1 gap**.
- **Verdict**: NO. **IP-31-OEE**.

### H.2 Availability calculation (planned production time / actual run time)
- **SAP**: via SAP DMC
- **Oracle SCP**: via Oracle MES
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-31-OEE-AVAILABILITY**.

### H.3 Performance calculation (ideal cycle time × actual count / actual run time)
- **SAP**: via SAP DMC
- **Oracle SCP**: via Oracle MES
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-31-OEE-PERFORMANCE**.

### H.4 Quality calculation (good count / total count)
- **SAP**: via SAP QM + DMC
- **Oracle SCP**: via Oracle MES + QM
- **Oyatie**: NOT NAMED. Cross-microservice handoff to quality-management required.
- **Verdict**: NO. **IP-31-OEE-QUALITY**.

### H.5 OEE benchmarking (world-class > 85%, acceptable > 60%)
- **SAP**: dashboard
- **Oracle SCP**: dashboard
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-31-OEE-BENCHMARK**.

### H.6 Six Big Losses categorization
- **SAP**: via SAP DMC
- **Oracle SCP**: via Oracle MES
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-31-OEE-SIX-LOSSES**.

## §10 Feature Group I — Machine Learning Anomaly Detection

### I.1 Vibration anomaly detection (rotating equipment)
- **SAP**: via SAP Predictive Asset Insights
- **Oracle SCP**: via Oracle IoT
- **Kinaxis Maestro**: NOT NATIVE
- **Oyatie**: NOT NAMED in production-planning IPs; should integrate with `detection` microservice. **P1 gap**.
- **Verdict**: NO. **IP-32-ML-VIBRATION**.

### I.2 Temperature anomaly detection
- **SAP**: via SAP PAI
- **Oracle SCP**: via Oracle IoT
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-32-ML-TEMPERATURE**.

### I.3 Cycle-time drift detection
- **SAP**: via SAP DMC
- **Oracle SCP**: via Oracle MES
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-32-ML-CYCLE-DRIFT**.

### I.4 Predictive maintenance trigger (RUL — Remaining Useful Life)
- **SAP**: via SAP PAI
- **Oracle SCP**: via Oracle IoT Asset Monitoring
- **Oyatie**: NOT NAMED. Cross-microservice handoff to plant-maintenance required.
- **Verdict**: NO. **IP-32-RUL** + plant-maintenance handshake.

### I.5 Schedule-deviation prediction (will-this-run-late?)
- **SAP**: not native
- **Oracle SCP**: via predictive cloud
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-32-SCHED-DEVIATION-PRED**.

## §11 Feature Group J — Real-Time Shop-Floor Integration

### J.1 ISA-95 / IEC 62264 level-3 ↔ level-4 handshake
- **SAP**: via SAP DMC
- **Oracle SCP**: via Oracle MES + Production Scheduling
- **Kinaxis Maestro**: via custom integration
- **Oyatie**: IP-024 explicit ISA-95 / IEC 62264 reference. **PASS — above all three**.
- **Verdict**: YES. **Differentiator**: standards body (MESA) explicitly named.

### J.2 B2MML (Business To Manufacturing Markup Language) message types
- **SAP**: B2MML supported via SAP DMC
- **Oracle SCP**: B2MML via Oracle MES
- **Kinaxis Maestro**: via custom integration
- **Oyatie**: IP-024 B2MML ProductionSchedule + ProductionPerformance + ProductionResponse. **PASS**.
- **Verdict**: YES.

### J.3 OPC-UA (IEC 62541) native ingest
- **SAP**: via SAP Plant Connectivity
- **Oracle SCP**: via Oracle IoT
- **Kinaxis Maestro**: via custom
- **Oyatie**: NOT NAMED. P2 gap.
- **Verdict**: NO. **IP-50-OPC-UA** (post-Wave-4).

### J.4 MQTT Sparkplug-B native ingest
- **SAP**: via SAP Plant Connectivity
- **Oracle SCP**: via Oracle IoT
- **Kinaxis Maestro**: via custom
- **Oyatie**: NOT NAMED. P2 gap.
- **Verdict**: NO. **IP-50-MQTT-SPARKPLUG**.

### J.5 Bidirectional state drift detection (Oyatie order state vs MES execution state)
- **SAP**: native via SAP DMC
- **Oracle SCP**: via Oracle MES
- **Kinaxis Maestro**: via custom
- **Oyatie**: IP-024 §D-2.AC-5 drift detector every 5min. **PASS**.
- **Verdict**: YES.

### J.6 HLC + UTC time-synchronization with reconciliation
- **SAP**: not first-class; UTC only
- **Oracle SCP**: UTC + Oracle DB time
- **Kinaxis Maestro**: UTC
- **Oyatie**: IP-024 §D-2.40 HLC + UTC drift ≤ ±2s. **PASS — above all three**. Differentiator: HLC explicitly named, drift bound named.
- **Verdict**: YES.

### J.7 Tenant-pin + ISA-95 hierarchy mapping
- **SAP**: SAP DMC enterprise/site/area
- **Oracle SCP**: Oracle Org Hierarchy
- **Kinaxis Maestro**: Site / Resource
- **Oyatie**: IP-024 §D-2.AC-6 `(tenant, plant, work_center, work_unit) ↔ (EnterpriseRef, SiteRef, AreaRef, WorkUnitRef)`. **PASS**.
- **Verdict**: YES.

## §12 Feature Group K — Mobile

### K.1 Mobile supervisor (schedule view + reassign)
- **SAP**: SAP Asset Manager + Fiori
- **Oracle SCP**: Oracle Mobile Supply Chain
- **Kinaxis Maestro**: Kinaxis Mobile
- **Oyatie**: NOT NAMED. **P1 gap**.
- **Verdict**: NO. **IP-33-MOBILE-SUPERVISOR**.

### K.2 Mobile operator (confirm operation + record quantity)
- **SAP**: SAP Plant Connectivity + Fiori
- **Oracle SCP**: Oracle Mobile Supply Chain
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-33-MOBILE-OPERATOR**.

### K.3 Mobile material handler (pick list + WM)
- **SAP**: SAP Mobile Asset Management
- **Oracle SCP**: Oracle Mobile Supply Chain Pick
- **Oyatie**: NOT NAMED.
- **Verdict**: NO. **IP-33-MOBILE-MATERIAL**.

### K.4 Mobile maintenance (work order)
- **SAP**: SAP Asset Manager
- **Oracle SCP**: Oracle Maintenance Mobile
- **Oyatie**: cross-microservice to plant-maintenance.
- **Verdict**: NO (out of production-planning scope).

### K.5 Native mobile platform binding
- **SAP**: Fiori (web + mobile hybrid)
- **Oracle SCP**: Oracle Mobile Application Framework (hybrid)
- **Kinaxis Maestro**: Kinaxis Mobile (native iOS + Android)
- **Oyatie**: ADR-0328 §D-18 mandates `frontend/ios=Swift` + `frontend/android=Kotlin` (native). Not yet implemented. **Differentiator**: native-by-policy when implemented.
- **Verdict**: NO (not yet implemented).

## §13 Feature Group L — Cross-Cutting Concerns

### L.1 Multi-tenancy
- **SAP**: single-tenant by default (S/4 Cloud: tenant per environment)
- **Oracle SCP**: tenant per pod
- **Kinaxis Maestro**: tenant per dataset
- **Oyatie**: tenant scope required on every aggregate. **PASS** — multi-tenant from day one.
- **Verdict**: YES. **Differentiator**.

### L.2 Tenant_class composability (replaces tier deltas)
- **SAP**: NO
- **Oracle SCP**: NO (subscription tier only)
- **Kinaxis Maestro**: NO
- **Oyatie**: per 2026-05-20 directive — should support `multi-tenant / paid / byo-cloud / self-hosted / demo / sandbox / trial / dev`. Currently P0 drift (§3.2 of audit).
- **Verdict**: PARTIAL (post-remediation: PASS). **Differentiator** once remediated.

### L.3 Six-context deployment matrix
- **SAP**: on-prem + private cloud + S/4 Cloud
- **Oracle SCP**: OCI only
- **Kinaxis Maestro**: AWS only
- **Oyatie**: six contexts per ADR-0328 §D-15. Currently P0 drift (§3.8 of audit).
- **Verdict**: PARTIAL (post-remediation: PASS). **Differentiator**.

### L.4 Marketplace-settled commercial model
- **SAP**: SAP Store
- **Oracle SCP**: Oracle Cloud Marketplace
- **Kinaxis Maestro**: Kinaxis Marketplace
- **Oyatie**: ADR-0314 marketplace settlement via marketplace microservice; PRD Story PP-013. **PASS**.
- **Verdict**: YES.

### L.5 Audit-chain emission
- **SAP**: change documents (`CDHDR`/`CDPOS`)
- **Oracle SCP**: Fusion Audit Trail
- **Kinaxis Maestro**: ChangeLog
- **Oyatie**: manifest `audit_chain.seal_events` 6 events + IP-024 4 MES events. PASS-shape; P1 drift on granularity (§3.20 of audit).
- **Verdict**: PARTIAL (post-remediation: PASS).

### L.6 Cedar universal authorization gate
- **SAP**: SAP authorization (PFCG roles)
- **Oracle SCP**: Oracle role-based
- **Kinaxis Maestro**: Kinaxis role-based
- **Oyatie**: 6 Cedar policies per bounded context per ADR-0243. **PASS**. **Differentiator**: declarative + ADR-0243-uniform.
- **Verdict**: YES.

### L.7 HTTP/3 + QUIC transport
- **SAP**: HTTP/1.1 + HTTP/2
- **Oracle SCP**: HTTP/2
- **Kinaxis Maestro**: HTTP/2
- **Oyatie**: HTTP/3 default per ADR-0253 + manifest. **PASS — above all three**.
- **Verdict**: YES. **Differentiator**.

### L.8 Post-quantum crypto readiness
- **SAP**: NO (announced for 2027)
- **Oracle SCP**: NO
- **Kinaxis Maestro**: NO
- **Oyatie**: PQC hybrid offered per manifest. **PASS — above all three**.
- **Verdict**: YES. **Differentiator**.

### L.9 ECH (Encrypted Client Hello)
- **SAP**: NO
- **Oracle SCP**: NO
- **Kinaxis Maestro**: NO
- **Oyatie**: ECH advertised per manifest + iac/ech-config.yaml.
- **Verdict**: YES. **Differentiator**.

### L.10 SemVer for REST + event + gRPC + SDK
- **SAP**: SAP API Hub versioning (API-versioned)
- **Oracle SCP**: REST versioned by URI segment
- **Kinaxis Maestro**: REST versioned
- **Oyatie**: SemVer per manifest. **PASS**.
- **Verdict**: YES.

### L.11 Open standards: OpenAPI 3.2.0 + AsyncAPI 3.1.0 + proto3
- **SAP**: OpenAPI 3.0
- **Oracle SCP**: OpenAPI 3.0
- **Kinaxis Maestro**: REST + scripting only
- **Oyatie**: manifest declares OpenAPI 3.2.0 + AsyncAPI 3.1.0 + proto3. **PASS — above all three**.
- **Verdict**: YES. **Differentiator**.

### L.12 Sovereign cell / data-residency override
- **SAP**: data residency via region selection
- **Oracle SCP**: data residency via OCI region
- **Kinaxis Maestro**: AWS region only
- **Oyatie**: per ADR-0248 Amazon-shape cellular; manifest `cell_eligibility: [Tier 0, Tier 1, Tier 2, Tier 3]`. **PASS**.
- **Verdict**: YES.

### L.13 Industry-specific compliance pack (FDA 21 CFR Part 11)
- **SAP**: SAP for Life Sciences (separate license)
- **Oracle SCP**: Oracle Cloud Manufacturing for Life Sciences
- **Kinaxis Maestro**: native
- **Oyatie**: NOT NAMED. **P1 gap**. **IP-34**.
- **Verdict**: NO.

### L.14 Industry-specific compliance pack (ISO 13485 medical device)
- **SAP**: SAP for Life Sciences
- **Oracle SCP**: Oracle Cloud Manufacturing for Med Device
- **Oyatie**: NOT NAMED. **IP-34**.
- **Verdict**: NO.

### L.15 Industry-specific compliance pack (AS9100 aerospace)
- **SAP**: SAP for Aerospace & Defense
- **Oracle SCP**: Oracle Cloud Manufacturing for A&D
- **Oyatie**: NOT NAMED. **IP-34**.
- **Verdict**: NO.

### L.16 Industry-specific compliance pack (IATF 16949 automotive)
- **SAP**: SAP for Automotive
- **Oracle SCP**: Oracle Cloud Manufacturing for Automotive
- **Oyatie**: NOT NAMED. **IP-34**.
- **Verdict**: NO.

### L.17 EU AI Act compliance
- **SAP**: not yet
- **Oracle SCP**: not yet
- **Kinaxis Maestro**: not yet
- **Oyatie**: ADR-0328 baseline + EU AI Act pack (proposed). **IP-35**.
- **Verdict**: PARTIAL (post-remediation: YES + differentiator).

### L.18 KR PIPA + tenant_class composability
- **SAP**: KR localization pack
- **Oracle SCP**: KR localization pack
- **Kinaxis Maestro**: not specifically
- **Oyatie**: KR-PIPA pack in manifest; per-tenant_class activation rule TBD. **PASS-shape**, P2 drift (§3.16).
- **Verdict**: PARTIAL.

## §14 Differentiator Roll-Up

### §14.1 Oyatie strict wins (above all three counterparts when claimed)
1. **HLC + UTC drift ≤ ±2s reconciliation** (IP-024 / ADR-0252) — no counterpart names HLC.
2. **DBR / TOC-author TOC explicit naming** (IP-021) — SAP does this implicitly via PP/DS but never names DBR.
3. **AC-3 constraint propagation explicit** (IP-021) — no counterpart names the algorithm.
4. **ISA-95 / B2MML / MESA standards-body reference** (IP-024) — counterparts ship the support but rarely cite the standard.
5. **HTTP/3 + QUIC default + PQC hybrid + ECH** — no counterpart ships any of the three.
6. **Six-context deployment matrix (Oyatie public + AWS + OCI + on-prem + colo + Oyatie-as-cloud)** — no counterpart spans all six.
7. **Marketplace-settled tenant deals via dedicated marketplace microservice** (ADR-0314) — counterparts have marketplaces but not policy-gated settlement.
8. **Cedar universal authorization gate** (ADR-0243) — counterparts use role-based auth, not declarative policy.
9. **Substrate vs product layering** (ADR-0245) — clean separation; counterparts mix.
10. **DDMRP as first-class IP slice** (IP-018) — counterparts ship as add-on or via partner.

### §14.2 Oyatie strict losses (counterpart wins)
1. **MRP / MPS algorithmic depth** — SAP and Oracle ship 8+ lot-sizing rules; Oyatie ships 0 named. P0 gap.
2. **Change-over / sequence-dependent setup matrix** — all three counterparts ship; Oyatie ships none. P0 gap.
3. **What-if scenario branching** — Kinaxis Maestro's headline differentiator; Oyatie ships none. P0 gap.
4. **Concurrent planning (always-on)** — Kinaxis Maestro's headline differentiator; Oyatie batch-mode only. P0 gap (long-horizon).
5. **OEE tracking** — SAP DMC and Oracle MES native; Oyatie ships none. P1 gap.
6. **ML anomaly detection on shop-floor signals** — SAP PAI and Oracle IoT native; Oyatie cross-microservice. P1 gap.
7. **Mobile** — all three counterparts ship; Oyatie ships none. P1 gap.
8. **Industry-specific compliance packs** (FDA / ISO 13485 / AS9100 / IATF 16949) — counterparts ship via separate licenses; Oyatie ships none. P1 gap.
9. **Variant configuration** — SAP and Oracle ship; Oyatie ships none. P1 gap.
10. **Phantom assembly, multi-level pegging, ECN with effectivity** — counterparts ship; Oyatie ships none. P1 gap each.

### §14.3 Parity (counterpart and Oyatie both ship at comparable depth)
1. Forward / backward / bottleneck-anchor finite scheduling.
2. Production order release.
3. ISA-95 / B2MML MES handshake (Oyatie above).
4. Calendar-aware scheduling (shape only; depth gap noted).
5. Tenant scoping (Oyatie above on multi-tenancy).
6. Audit emission (Oyatie shape PASS; granularity P1 gap).
7. SemVer + OpenAPI / AsyncAPI / proto3 (Oyatie above).
8. Marketplace integration (Oyatie above on architectural cleanliness).

## §15 Counterpart-Specific Migration Hazards

### §15.1 SAP customers migrating to Oyatie
- **Hazard 1**: ABAP-based custom enhancements (`MV45AFZZ` user exits, BAdIs, `Z*` Z-tables) — Oyatie ships pluggable Workflow Studio + Workflow Engine; migration path is to re-express ABAP logic as workflow templates.
- **Hazard 2**: SAP Variant Configuration with thousands of class characteristics — Oyatie does not yet ship variant config (IP-43-VARIANT P1 gap). **Mitigation**: defer migration of CTO-heavy customers until IP-43-VARIANT lands.
- **Hazard 3**: SAP MARC-level material extension with thousands of fields — Oyatie's tenant-pinned ontology projection (PRD §D) absorbs this via ontology object versioning.
- **Hazard 4**: PFCG role-based authorization — must remap to Cedar permits. Provide a one-shot extract + Cedar-author tool.
- **Hazard 5**: PP/DS heuristic stack (40+ heuristics like H001..H040) — Oyatie ships 3 strategies (forward/backward/bottleneck-anchor) and AC-3. Customers using uncommon SAP heuristics need IP-46 extensions.

### §15.2 Oracle SCP customers migrating to Oyatie
- **Hazard 1**: Oracle DB stored procedures — Oyatie is Rust-strict (ADR-0328 §D-18); migration is workflow templates + cedar.
- **Hazard 2**: Fusion Org Hierarchy depth — map to Oyatie tenant-context fields.
- **Hazard 3**: Oracle Constraint-Based Optimization — Oyatie ships AC-3 + DBR; customers using deep CBO need IP-46 extensions.

### §15.3 Kinaxis Maestro customers migrating to Oyatie
- **Hazard 1**: Maestro scripts (RAE workbooks) — Oyatie ships Workflow Studio (visual editor + engine); migration is workbook-to-workflow.
- **Hazard 2**: Concurrent planning model — Oyatie batch-mode by default; need IP-30-CONCURRENT-PLANNING.
- **Hazard 3**: Scenario-branching workflows — Oyatie ships none; need IP-30. This is the largest migration-blocker for Kinaxis customers.
- **Hazard 4**: AlertDefinition framework — map to Oyatie exception-management primitives (currently NOT NAMED; **IP-43-EXCEPTIONS** required).

## §16 Verdict Summary by Feature Group

| Group | Total Features | YES | PARTIAL | NO | Differentiator-Above |
|---|---|---|---|---|---|
| A. BOM Management | 6 | 1 | 1 | 4 | 1 (co-product yield variance) |
| B. MRP | 17 | 1 | 4 | 12 | 1 (DDMRP first-class) |
| C. MPS | 8 | 0 | 2 | 6 | 0 |
| D. Finite Scheduling | 13 | 5 | 2 | 6 | 2 (DBR explicit, AC-3 explicit) |
| E. Dispatching | 10 | 3 | 1 | 6 | 1 (ISA-95 standards-body) |
| F. Bottleneck Management | 5 | 4 | 1 | 0 | 1 (TOC-author TOC explicit) |
| G. What-If Scheduling | 6 | 0 | 0 | 6 | 0 |
| H. OEE | 6 | 0 | 0 | 6 | 0 |
| I. ML Anomaly | 5 | 0 | 0 | 5 | 0 |
| J. Shop-Floor Integration | 7 | 5 | 0 | 2 | 2 (HLC + standards-body) |
| K. Mobile | 5 | 0 | 0 | 5 | 0 |
| L. Cross-Cutting | 18 | 9 | 4 | 5 | 6 (HTTP/3, PQC, ECH, marketplace, cedar, OpenAPI/AsyncAPI/proto3) |
| **TOTAL** | **106** | **28** | **15** | **63** | **14 differentiators** |

**Parity score**: (28 + 15×0.5) / 106 = **33.5%** weighted parity. Excluding P2 deferrals (15 features) → **39.0%** weighted parity over P0+P1 in-scope.

**Differentiator count**: 14 strict-wins above counterparts.

**Strict-loss count**: 10 P0/P1 gaps (§14.2).

## §17 Path to ≥80% Parity

Implementing IP-26 through IP-50 (per coherence-audit §10.5) brings parity to:
- IP-43 (MRP detail) closes 12 NOs in Group B → 17 YES → +12 = 23 YES
- IP-44 (MPS detail) closes 6 NOs in Group C → 8 YES → +6 = 8 YES
- IP-45 (dispatching priority) closes 4 NOs in Group E → 7 YES
- IP-46 (change-over + setup matrix) closes 6 NOs in Group D → 11 YES (8 + 3 net of conflict)
- IP-30 (scenario branching) closes 5-6 NOs in Group G → 5-6 YES
- IP-31 (OEE) closes 6 NOs in Group H → 6 YES
- IP-32 (ML anomaly) closes 5 NOs in Group I → 5 YES
- IP-33 (mobile) closes 4 NOs in Group K (1 cross-microservice) → 4 YES
- IP-34 (industry compliance packs) closes 4 NOs in Group L → 4 YES
- IP-35 (EU AI Act) closes 1 PARTIAL → YES + differentiator
- Audit + manifest remediation closes the 4 PARTIAL → YES in Group L

After full remediation: ~95 YES / 106 = **≈90% weighted parity**, with **15-16 differentiators above counterparts**.

## §18 Reading Guidance for Future Authoring

- **Reusable across counterparts** — Group F (bottleneck management) is fully covered; future authoring should not redo this.
- **Substrate-level** — Group J (shop-floor integration) is the foundation for Group H (OEE) and Group I (ML); IP-31 and IP-32 will depend on IP-24's substrate.
- **Sequential dependency** — IP-43 (MRP detail) must precede IP-44 (MPS detail) because MPS-to-MRP cascade is defined in MRP terms.
- **Migration-driven** — IP-30 (scenario branching) and IP-43-VARIANT (variant config) are the two largest buyer-blockers for migration from Kinaxis and SAP respectively; prioritize accordingly.

End of feature-parity matrix.
