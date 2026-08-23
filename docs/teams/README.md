---
doc_status: published
---

# Oyatie Engineering Teams — Team-of-Teams Overview

> **Status:** Draft v0.1 — 2026-05-09.
> **Owner:** `council-architecture` (meta-supervisor); each team owns its own `CHARTER.md`.
> **Total active teams:** 37 (38 defined; 1 retired — see §6).
> **Scope update (2026-05-09):** `axis-foundry` consolidated into `axis-foundry`. Total drops from 38 to 37. See §6.

---

## 1. Team roster

### 1.1 Platform & Cross-cutting (5 teams)

| Team ID | Name | Wave first active | Charter |
|---|---|---|---|
| `platform-tenancy-identity` | Platform — Tenancy & Identity | W-Foundation | [CHARTER.md](platform-tenancy-identity/CHARTER.md) |
| `platform-audit-evidence` | Platform — Audit & Evidence | W-Foundation | [CHARTER.md](platform-audit-evidence/CHARTER.md) |
| `platform-privacy-dub` | Platform — Privacy & Data Use Boundary | W-Foundation | [CHARTER.md](platform-privacy-dub/CHARTER.md) |
| `platform-eventing-og` | Platform — Eventing & Object Graph | W-Foundation | [CHARTER.md](platform-eventing-og/CHARTER.md) |
| `platform-api-sdk` | Platform — Public API & SDK | W-SaaS-Preview | [CHARTER.md](platform-api-sdk/CHARTER.md) |

### 1.2 Axis (5 teams — Foundry consolidated into Foundry)

| Team ID | Name | Axis | Wave first active | Charter |
|---|---|---|---|---|
| `axis-saas` | Axis — SaaS Multi-Tenant Platform | Axis 1 | W-SaaS-Preview | [CHARTER.md](axis-saas/CHARTER.md) |
| `axis-foundry` | Axis — Foundry (Agent Runtime + Foundry) | Axes 3+4 | W-Foundry-Preview | [CHARTER.md](axis-foundry/CHARTER.md) |
| `axis-cloud` | Axis — Cloud Provider | Axis 5 | W-Cloud-Preview | [CHARTER.md](axis-cloud/CHARTER.md) |
| `axis-search` | Axis — Search Engine | Axis 6 | W-Search-Preview | [CHARTER.md](axis-search/CHARTER.md) |
| `axis-ads-analytics` | Axis — Ads & Analytics | Axis 7 | W-Ads-Preview | [CHARTER.md](axis-ads-analytics/CHARTER.md) |

> **Note:** `axis-foundry` (formerly a separate team) is **consolidated into `axis-foundry`** as of 2026-05-09. The Foundry team now owns both the agent runtime (capability registry, autonomy ceiling, evidence chain, provider adapters) and the Foundry surfaces (repoctl, catalog, claim-ceiling, fitness functions, supply-chain). See `axis-foundry/CHARTER.md` for the full consolidated scope.

### 1.3 Vertical — Deep (6 teams)

| Team ID | Name | Wave first active | Charter |
|---|---|---|---|
| `vertical-corporate` | Vertical — Corporate (HR/Payroll/GL/Mail/Comms) | W-Vertical-Pilot | [CHARTER.md](vertical-corporate/CHARTER.md) |
| `vertical-healthcare` | Vertical — Healthcare (Clinical/Ambulatory/HL7-FHIR) | W-Vertical-Fan-Out | [CHARTER.md](vertical-healthcare/CHARTER.md) |
| `vertical-industrial` | Vertical — Industrial (MES/OEE/ISA-95/OPC UA) | W-Vertical-Fan-Out | [CHARTER.md](vertical-industrial/CHARTER.md) |
| `vertical-logistics` | Vertical — Logistics (Shipment/Dock/EDI/Route) | W-Vertical-Fan-Out | [CHARTER.md](vertical-logistics/CHARTER.md) |
| `vertical-fintech` | Vertical — Fintech (PG/Open-Banking/KYC/AML) | W-Vertical-Fan-Out | [CHARTER.md](vertical-fintech/CHARTER.md) |
| `vertical-legal` | Vertical — Legal (Regulated Corpus/Contracts) | W-Vertical-Fan-Out | [CHARTER.md](vertical-legal/CHARTER.md) |

### 1.4 Vertical — Skeleton (8 teams, activate at W-Vertical-Fan-Out)

| Team ID | Name | Charter |
|---|---|---|
| `vertical-retail` | Vertical — Retail (POS/Inventory/Promotions) | [CHARTER.md](vertical-retail/CHARTER.md) |
| `vertical-education` | Vertical — Education (LMS) | [CHARTER.md](vertical-education/CHARTER.md) |
| `vertical-public-sector` | Vertical — Public Sector (Forms/조달청/Global Gov) | [CHARTER.md](vertical-public-sector/CHARTER.md) |
| `vertical-hospitality` | Vertical — Hospitality (PMS) | [CHARTER.md](vertical-hospitality/CHARTER.md) |
| `vertical-construction` | Vertical — Construction (Project Management) | [CHARTER.md](vertical-construction/CHARTER.md) |
| `vertical-real-estate` | Vertical — Real Estate (Leasing) | [CHARTER.md](vertical-real-estate/CHARTER.md) |
| `vertical-agriculture` | Vertical — Agriculture (Traceability) | [CHARTER.md](vertical-agriculture/CHARTER.md) |
| `vertical-food` | Vertical — Food (Supply-Chain Compliance) | [CHARTER.md](vertical-food/CHARTER.md) |

### 1.5 Operations (5 teams)

| Team ID | Name | Charter |
|---|---|---|
| `ops-sre-reliability` | Ops — SRE & Reliability | [CHARTER.md](ops-sre-reliability/CHARTER.md) |
| `ops-security` | Ops — Security | [CHARTER.md](ops-security/CHARTER.md) |
| `ops-compliance` | Ops — Compliance | [CHARTER.md](ops-compliance/CHARTER.md) |
| `ops-dr-capacity` | Ops — DR & Capacity | [CHARTER.md](ops-dr-capacity/CHARTER.md) |
| `ops-finops` | Ops — FinOps | [CHARTER.md](ops-finops/CHARTER.md) |

### 1.6 GTM (4 teams)

| Team ID | Name | Charter |
|---|---|---|
| `gtm-sales-se` | GTM — Sales & Solutions Engineering | [CHARTER.md](gtm-sales-se/CHARTER.md) |
| `gtm-customer-success` | GTM — Customer Success | [CHARTER.md](gtm-customer-success/CHARTER.md) |
| `gtm-marketing` | GTM — Marketing | [CHARTER.md](gtm-marketing/CHARTER.md) |
| `gtm-partnerships` | GTM — Partnerships | [CHARTER.md](gtm-partnerships/CHARTER.md) |

### 1.7 Special (4 active, 1 retired)

| Team ID | Name | Type | Charter |
|---|---|---|---|
| `council-architecture` | Council — Architecture | Permanent governance | [CHARTER.md](council-architecture/CHARTER.md) |
| `council-privacy` | Council — Privacy | Permanent governance | [CHARTER.md](council-privacy/CHARTER.md) |
| `crew-adr-promotion` | Crew — ADR Promotion | Time-boxed crew | [CHARTER.md](crew-adr-promotion/CHARTER.md) |
| `tactical-first-vertical-pilot` | Tactical — First Vertical Pilot | Time-boxed coordination | [CHARTER.md](tactical-first-vertical-pilot/CHARTER.md) |
| ~~`tactical-m3-launch`~~ | ~~Tactical — M3 Launch~~ | **RETIRED 2026-05-09** | See §6 |

---

## 2. Team-of-teams operating model

Oyatie uses a **federated team-of-teams** model, not a hierarchical command structure. The operating model has four layers:

### Layer 0 — Founder (north-star arbiter)
Jason Lee is the north-star arbiter for decisions that exceed council authority or involve commercial commitments above delegated thresholds. The founder is not in the day-to-day engineering loop — escalation to founder is a last resort.

### Layer 1 — Councils (governance)
Two permanent councils hold cross-cutting authority:
- **`council-architecture`**: cross-axis contract authority, wave-gate sign-off, RACI, doc-catalog meta-supervisor, quarterly contradiction audit.
- **`council-privacy`**: Data Use Boundary ADR authority, consent-taxonomy ratification, privacy incident review, DSR cascade protocol.

Councils do not write code. They hold decision authority on the *seams* between teams.

### Layer 2 — Axis + Platform teams (product builders)
Five platform teams (cross-cutting) and five axis teams (product surfaces) build and own the canonical Oyatie product. Axis teams consume platform team contracts; platform teams are shared infrastructure that every axis inherits.

The key structural decision (2026-05-09): **`axis-foundry` owns both the agent runtime and the Foundry**. This means:
- Every PR in the repo passes through `axis-foundry`'s fitness functions and CI gates.
- Every capability invocation in every axis goes through `axis-foundry`'s autonomy ceiling.
- The team that builds the product substrate is the same team that enforces the quality substrate. No split incentive.

### Layer 3 — Vertical + Ops + GTM teams (domain specialists and operators)
Vertical teams consume axis + platform contracts and add domain-specific entities, workflows, and regulatory compliance. Ops teams ensure the system runs reliably, securely, and compliantly. GTM teams translate the product into commercial reality.

### Layer 4 — Time-boxed crews and councils
`crew-adr-promotion` and `tactical-first-vertical-pilot` are time-boxed. They dissolve at mission completion. Their responsibilities are absorbed into the permanent team layer.

### Cross-team coordination norms
1. **Cross-axis PR class label**: any PR touching a row in DESIGN §10 gets the `cross-axis` label. The relevant axis tech leads are required reviewers.
2. **Monthly cross-axis contract audit**: `council-architecture` runs this. All axis tech leads participate.
3. **Wave-gate readiness review**: council-architecture assembles the evidence pack; all teams contribute their slice.
4. **Monthly ADR batch**: `crew-adr-promotion` prepares the agenda; `council-architecture` votes.
5. **Dependency matrix for team waves**: when parallel workers share deliverables, the lead publishes a dependency matrix before spawn (per CLAUDE.md "Team Worker Brief Standards" (formerly "CUG Worker Brief Standards"; renamed 2026-05-09)).

---

## 3. RACI overlay

The full RACI lives in `RACI-OWNERSHIP.md`. This section provides the structural overlay — which teams hold which RACI role for cross-cutting decisions.

| Decision type | Responsible | Accountable | Consulted | Informed |
|---|---|---|---|---|
| Cross-axis contract change (DESIGN §10 row) | Owning axis team | `council-architecture` | Consuming axis teams | All teams |
| `Tenant` kernel shape change | `platform-tenancy-identity` | `council-architecture` | All 7 axis teams | All teams |
| Data Use Boundary ADR amendment | `platform-privacy-dub` | `council-privacy` | All data-touching axes | All teams |
| Wave-gate pass/fail decision | Relevant axis teams (evidence) | `council-architecture` | `ops-compliance`, `ops-security`, `ops-sre-reliability` | All teams |
| Autonomy ceiling policy change | `axis-foundry` | `council-architecture` + ADR-0050 governance | `ops-security`, all regulated axes | All teams |
| New vertical approved | `council-architecture` | Founder | All axis teams | All teams |
| New regional pack approved | `council-architecture` | Founder | `axis-cloud`, `platform-tenancy-identity`, `ops-compliance` | All teams |
| Security-class incident response | `ops-security` | `ops-sre-reliability` | Affected axis team | All teams |
| Privacy incident response | `platform-privacy-dub` (secretariat) | `council-privacy` | `ops-compliance`, `ops-security` | All teams |
| ADR promotion (cross-cutting) | `crew-adr-promotion` (facilitation) | `council-architecture` | Owning axis team | All teams |
| Design-partner pilot blocker | `tactical-first-vertical-pilot` | `council-architecture` | All pilot-axis leads | Founder |
| Product pricing / packaging | `gtm-sales-se` (input) | Founder | `ops-finops`, `council-architecture` | GTM teams |
| Partner agreement | `gtm-partnerships` | Founder | `axis-cloud` (technical), `ops-compliance` (regulatory) | GTM teams |

---

## 4. On-call rotation summary

Full rotation schedules live in the SLO catalog and PagerDuty configuration (owned by `ops-sre-reliability`). The structural model:

| Tier | Who is on-call | Coverage |
|---|---|---|
| **P0 — Foundation** | `platform-tenancy-identity` + `platform-audit-evidence` rotating | 24×7 |
| **P1 — Axis runtime** | `axis-foundry` (daemon + capability registry) + `axis-cloud` (cell + IAM) rotating | 24×7 |
| **P1 — Data** | `axis-search` (index + RAG) + `axis-ads-analytics` (auction) rotating | 24×7 |
| **P2 — Platform** | `platform-eventing-og` + `platform-api-sdk` rotating | 24×5 + pager |
| **P2 — SaaS** | `axis-saas` rotating | 24×5 + pager |
| **P3 — Vertical** | Per-vertical team (primary + secondary) | Business hours + pager |
| **Security** | `ops-security` (dedicated security on-call) | 24×7 |
| **CS escalation** | `gtm-customer-success` (design-partner Sev-1 acknowledgment) | Business hours + emergency pager |

**On-call rotation principles:**
- Every production surface has a named primary + secondary on-call. Zero gaps.
- Skeleton vertical teams have no on-call until activated at W-Vertical-Fan-Out.
- `ops-sre-reliability` owns the meta-on-call: when a Sev-1 war-room is declared, SRE coordinates across all axis on-call owners.
- Design-partner tenants get a 30-minute `gtm-customer-success` acknowledgment SLA for Sev-1 events.

---

## 5. Wave-gate team activation schedule

Teams activate as their wave approaches. Skeleton vertical teams have 0 FTE until W-Vertical-Fan-Out.

| Wave | Teams first activated / reaching full capacity |
|---|---|
| **W-Foundation** | `platform-tenancy-identity`, `platform-audit-evidence`, `platform-privacy-dub`, `platform-eventing-og`, `council-architecture`, `council-privacy`, `crew-adr-promotion` |
| **W-Foundry-Preview** | `axis-foundry` (full capacity), `ops-security` |
| **W-Foundry-Preview** | *(merged into W-Foundry-Preview via consolidation)* |
| **W-Cloud-Preview** | `axis-cloud`, `ops-dr-capacity`, `ops-sre-reliability`, `ops-finops` |
| **W-SaaS-Preview** | `axis-saas`, `platform-api-sdk` |
| **W-Search-Preview** | `axis-search` |
| **W-Vertical-Pilot** | `vertical-corporate` (full capacity), `gtm-customer-success`, `gtm-sales-se`, `tactical-first-vertical-pilot` |
| **W-Vertical-Fan-Out** | `vertical-healthcare`, `vertical-industrial`, `vertical-logistics`, `vertical-fintech`, `vertical-legal` + all 8 skeleton teams activate, `gtm-marketing`, `gtm-partnerships` |
| **W-Ads-Preview** | `axis-ads-analytics` (full capacity) |
| **W-Region-Fan-Out** | `ops-compliance` expands per new regional pack; `axis-cloud` adds regional engineers |

**Time-boxed team dissolution schedule:**
- `crew-adr-promotion`: dissolves when Proposed ADR backlog ≤ 5 (target: W-Foundry-Preview)
- `tactical-first-vertical-pilot`: dissolves at W-Vertical-Pilot gate pass + 2-week handover

---

## 6. Retired teams

| Team ID | Name | Retirement date | Reason | Replacement |
|---|---|---|---|---|
| `tactical-m3-launch` | Tactical — M3 Launch | 2026-05-09 | "M3" milestone vocabulary dropped per PRD §3.1 vocabulary update 2026-05-09. Milestone numbering (M0/M1/M2/M3/MVP) replaced with wave sequencing language (`Foundation → Substrate → Axis-Preview → …`). Any open work items referencing `tactical-m3-launch` must be re-triaged against W-Vertical-Pilot scope or the owning axis team. | `tactical-first-vertical-pilot` |
| `axis-foundry` | Axis — Foundry | 2026-05-09 | Consolidated into `axis-foundry`. Foundry surfaces (repoctl, catalog, claim-ceiling, fitness functions, supply-chain, scorecards, plane-gated CI lanes, branch-protection-as-code, signed commits) and Agent Runtime surfaces are now owned by a single team because they share the capability registry and autonomy ceiling as a single contract surface. Splitting them would diverge the ground truth. | `axis-foundry` |

---

## 7. Team scope contradictions identified

The following scope tensions were detected during charter authorship and are flagged for `council-architecture` resolution:

### 7.1 `platform-api-sdk` vs. per-axis public API ownership
**Tension:** `platform-api-sdk` owns the OpenAPI spec format, stability tier, and gateway infrastructure. Each axis team authors its own API slice. The boundary between "platform owns the contract format" and "axis owns the contract content" can create disputes when an axis team wants to publish an API before the gateway is stable.
**Recommendation:** Clarify in DESIGN §10 that `platform-api-sdk` owns the *stability tier gate* (ADR-0040), not the *content* of each axis's OpenAPI slice. Content ownership stays with the axis. Gate ownership stays with platform-api-sdk.

### 7.2 `axis-foundry` Foundry fitness functions vs. axis team autonomy
**Tension:** `axis-foundry` owns all fitness functions (including ones that check other axes' code). An axis team that disagrees with a fitness function result has no recourse except `council-architecture`. This creates a power asymmetry.
**Recommendation:** Establish a fitness-function dispute process: a consuming team can file a dispute with `council-architecture`; the fitness function is paused for that team's PRs for ≤ 5 business days while the dispute is resolved. Document this in the Foundry CHARTER and the RACI.

### 7.3 `vertical-agriculture` ↔ `vertical-food` traceability chain handoff
**Tension:** Both teams own a slice of the farm-to-fork traceability chain. The handoff point (farm → processor) is not owned by either team; it is a cross-vertical contract that has no named owner.
**Recommendation:** Define a `TraceabilityHandoff` contract owned by `council-architecture` in DESIGN §10. Both vertical teams implement the handoff; the council owns the schema.

### 7.4 `axis-cloud` IAM vs. `platform-tenancy-identity` IAM
**Tension:** Two IAM surfaces exist: `cloud-iam-kernel` (cloud-customer-facing) and `platform-identity-kernel` (SaaS-facing). DESIGN §10 notes these are "two ADRs in lockstep." The lockstep requirement is a coordination burden that could drift.
**Recommendation:** Monthly IAM sync between `axis-cloud` and `platform-tenancy-identity` is already in both charters. Additionally, add a fitness function `governance-iam-lockstep` that verifies the two IAM kernel shape versions are compatible after any IAM change. This is better than a process-only mitigation.

### 7.5 `council-privacy` secretariat vs. `platform-privacy-dub` team
**Tension:** `platform-privacy-dub` is the secretariat of `council-privacy`, meaning the team that drafts ADRs also manages the governance process that ratifies them. This is a soft conflict of interest.
**Recommendation:** The council chair should NOT be from `platform-privacy-dub`. The chair should rotate annually among non-privacy-team council members. The secretariat (meeting notes, agenda) can remain with `platform-privacy-dub`.

---

## 8. Recommended team mergers (if headcount is constrained)

These mergers are **optional** — they are only appropriate if early-wave headcount is constrained. They should be revisited when the team reaches the relevant wave.

| Option | Merge | Rationale | Risk |
|---|---|---|---|
| M-1 | `ops-dr-capacity` into `ops-sre-reliability` | DR and SRE have overlapping tooling; DR drills are facilitated by SRE today | SRE bandwidth risk; DR gets deprioritized if SRE is firefighting |
| M-2 | `crew-adr-promotion` dissolved early, ADR promotion embedded in `council-architecture` monthly | If the backlog is cleared faster than expected | Loss of dedicated bandwidth; backlog may creep back |
| M-3 | `gtm-marketing` + `gtm-partnerships` into a single `gtm-market-development` team | Both are relationship-heavy functions with limited headcount at early wave | Brand and partner disciplines are different; dilution risk |
| M-4 | `platform-eventing-og` Object Graph half folded into `axis-saas` | OG is SaaS-primary; eventing backbone stays as a separate platform team | OG is a cross-axis contract; folding into `axis-saas` risks SaaS team being the gatekeeper for all-axis OG changes |

**Recommendation:** Do not merge any P0-foundation teams (`platform-tenancy-identity`, `platform-audit-evidence`, `platform-privacy-dub`). These own hard invariants and need dedicated focus. M-1 is the least risky merger if headcount is tight.

---

## 9. Sources scanned

PRD.md (all sections), DESIGN.md §10 (cross-axis contract table), §11 (contradiction audit), DOC-CATALOG.md (all sections), products/README.md, CLAUDE.md (Team Worker Brief Standards [formerly "CUG Worker Brief Standards"], Code Review rules), all 37 team charters authored 2026-05-09.
