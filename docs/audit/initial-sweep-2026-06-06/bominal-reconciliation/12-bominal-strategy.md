---
doc_class: BominalStrategyReconstruction
title: Bominal strategic intent + backlog themes — recovered from _recover-bominal + GitHub issues
status: synthesized
date: 2026-06-06
context: >
  oyatie WAS bominal — renamed + migrated; the migration churned/lost context. This file
  recovers oyatie's OWN strategic intent (mission/thesis, moat strategy, sequencing,
  high-value-areas + strategic Moves, kill/maturity framing) from the recovered bominal
  working tree at /Users/jasonlee/Developer/_recover-bominal and from the live GitHub
  backlog (jason931225/bominal, 250 issues sampled).
inputs:
  - _recover-bominal/CONSTITUTION.md, CONTEXT.md, README.md
  - _recover-bominal/portfolio/strategy/{product-arm-theses,platform-moat-strategy,sequencing-rationale}.md
  - _recover-bominal/portfolio/{kill-criteria,maturity,launch-readiness,commercial-risks}/
  - gh issue list jason931225/bominal (EPICs / Moves / High-value-areas / milestones)
relationship_to_canon: >
  COMPLEMENTS docs/audit/.../synthesis/decision-record-oyatie-canon.md (founder-ruled
  identity/forge/data-tier/scope decisions — NOT re-surfaced here) and the legacy-recovery
  00-RECOVERY-REGISTER.md (7 recovered .Trash items — NOT duplicated here). This file is the
  STRATEGY layer: the investment/portfolio plane + backlog program, which the prior two did
  not cover.
---

# Bominal strategy reconstruction

## 0. What this adds (vs. existing canon)

The decision-record canon settled **what oyatie owns** (own-everything ratchet, forge,
data-tier, identity bridge, framekernel host, maximal vertical scope incl. defense +
power-grid). The legacy-recovery register settled **what got dropped in the .Trash** (KR
HR/payroll packs, First Proof Slice, 4-lane closure model). Neither captured the
**portfolio / capital-allocation strategy** or the **live backlog program**. That is this
file: the mission framing, the five-moat compounding thesis, the build-sequencing rationale,
the in-flight "Maturity at Full Scale" 8-move program + 10 high-value areas, and the
kill/maturity/launch-readiness governance instruments. All of it is oyatie's own — recovered,
not imported.

---

## 1. Mission / thesis

**Mission (CONSTITUTION.md):** build toward the stated mission with *verified, documented,
high-impact* work — **mission impact over motion**. Ten Do / ten Avoid rules; the load-bearing
ones: verify before assuming, research when facts are stale/risky, plan before irreversible
changes, prefer working+tested over speculative architecture, keep changes typed/reviewed/
reversible, no undocumented behavior, no duplicated sources of truth, blameless RCA. (This is
the same founder "verify at each step / no phantom findings" rule already in project memory —
the constitution is its bominal-side codification.)

**Product thesis (README.md + CONTEXT.md):** Bominal is a **multi-arm enterprise platform** —
one tenant-aware substrate spanning workplace operations, regulated-industry workflows,
communications, healthcare, financial rails, and emerging hospitality/lifestyle — governed
through an **Authority Map** of named single-sources-of-truth rather than one duplicated
mega-document. (This is the same "Ecosystem-as-a-Service / Oyatie-is-itself-a-tenant"
framing flagged as FOLD-worthy in legacy-recovery row 4 — here it survives as the live
multi-arm thesis.)

**Wedge thesis (portfolio/strategy):** the entry wedge is **Korean SMB payroll** inside the
`corporate` arm — monthly tax filing is mechanical, the buyer is the HR/admin lead, and the
employee/contract/payment/statutory-filing data graph compounds messaging, documents, notify,
and downstream healthcare-billing. Payments rails are the substrate everything else settles
against. (Directly continuous with the recovered KR HR/payroll packs + First Proof Slice.)

---

## 2. The moat strategy — five compounding moats

Source: `portfolio/strategy/platform-moat-strategy.md`. The strategy answers one question:
**why does building a fifth arm make the first four better, not worse?** Five moats, each a
platform property that gets *harder to copy as more arms ship*:

| # | Moat | ADR | Why it compounds |
|---|------|-----|------------------|
| 1 | **Catalog** | ADR-0017 + ADR-0226 | Everything (capability/deployable/surface/domain/module/runtime/connector/webhook) is a catalog record = single SoT for what exists / who owns / what state / who references. Nth arm's records are cheaper than (N-1)th because references already exist. |
| 2 | **Trust** | ADR-0225 | Data classification, audit emission, isolation plans, restore drills, break-glass, Trust Center pack, procurement packets. Pay the trust tax once (corporate statutory filing) → healthcare-billing inherits PHI isolation for free. Lets us enter regulated markets single-product competitors can't. |
| 3 | **Ecosystem** | ADR-0227 | One integration shape for all external surfaces (APIs/webhooks/connectors/SDKs/MCP). Breaking-change reviews (`BC-NNNN`) gate every removal. **AI/MCP no-bypass:** agents follow the same gates as humans — can't be circumvented by an "AI shortcut" partner. |
| 4 | **Data + AI** | ADR-0228 | Every arm emits structured data → normalized into evals + metrics; AI rides evals, never raw PHI without isolation. Nth arm's data exhaust = (N+1)th arm's training signal. **Intelligence is not a product; it is the multiplier** on every other arm's data graph. |
| 5 | **Builder OS** | ADR-0229 + ADR-0230 | Councils own decision rights; PR classes gate change types; lifecycle vocabulary governs deprecation/removal; quarterly portfolio review governs kill/fund/defund. New arms inherit governance without reinventing it. |

**Rejection rule (explicit moat discipline):** a new arm that does not compound a moat is
opportunity cost. Five rejections (greenfield silo / one-off compliance / closed-loop no
integration / no structured data / governance vacuum) = a kill, recorded under
`quarterly-portfolio-reviews/` naming which moats it failed to compound.

Authority spine: ADR-0017 (catalog), 0222 (architecture target-state), 0223 (Proof Ladder),
0224 (deploy consolidation), 0225 (trust), 0226 (product control), 0227 (ecosystem),
0228 (data+AI), 0229 (builder OS), 0230 (evolution/simplification), 0231 (portfolio plane).

---

## 3. Sequencing rationale — the compounding spine

Source: `portfolio/strategy/sequencing-rationale.md`. Build order is chosen so each arm rides
the data graph + trust posture of its predecessors:

```
platform → payments → corporate → messaging → documents → notify → healthcare-billing → intelligence
```

`A → B` = "A's existence makes B cheaper / faster / higher-trust." Why this order:
1. **platform first** — Object Graph, design system, deploy helpers, gitops, contracts; without it every arm re-invents scaffolding (ADR-0222 platform-first target-state).
2. **payments second** — domestic KR rails are the settlement substrate; every downstream billing event becomes a compounding signal (failure mode / settlement latency / fraud heuristic) no competitor can replicate.
3. **corporate third (current wedge)** — KR SMB payroll, HR/admin buyer-of-record; once they trust monthly tax filing every other arm ships through them with no separate adoption motion.
4. **messaging fourth** — workspace-grade engagement surface; standalone messaging without payroll is a category they explicitly *do not enter*.
5. **documents fifth** — templated contracts / statutory filings / signed records; compounds corporate + healthcare; retention+audit story (ADR-0225) is a non-trivial moat.
6. **notify sixth** — cheapest to hold at L5, highest leverage (every product event is a notify event); maintain-level, not a growth arm.
7. **healthcare-billing seventh** — high-trust adjacency; PHI isolation is the moat only trust-tax-payers can ship.
8. **intelligence eighth** — not a product; the leverage multiplier on every arm's data graph (ADR-0228).

**Investment levels (product-arm-theses.md):** corporate/payments/messaging/documents/
healthcare-billing = `growth`; notify/platform = `maintain`; intelligence = `seed`. Current
rungs run L3→L4 (messaging, documents, healthcare-billing, intelligence) up to L5→L6 (platform).

**Deferred (named + reasoned):** `dental-clinic-hr` (low compounding into the spine — defer
till corporate-payroll hits L5); `generic-international-payroll` (regulatory complexity exceeds
capacity — hits a kill criterion before the wedge is proven); `consumer-marketing` (opportunity
cost; no consumer wedge).

**Parked (pre-L1, idea-only, NOT killed; `#1450` scope-kill):** `dining`, `cellar`, `pos`,
`retail`, `hospitality-ops`, `fashion`, `career` — each gated by an `allowed_work` /
`forbidden_work` / `promotion_requires` envelope so parked work can't accidentally graduate to
production posture.

Sequencing changes only in the **quarterly portfolio review** (13-week cadence); any state /
investment-level edit cites the review's decision id.

---

## 4. Maturity framing — Proof Ladder L0..L7

Source: `portfolio/maturity/rungs-l0-to-l7.md` (binds ADR-0223 to the investment plane). The
**Proof Ladder is the cross-cutting product-readiness axis** every catalog artifact carries:

```
L0 Idea          → narrative exists
L1 Cataloged     → catalog record w/ owner/domain/capability/module/data-plane/lifecycle
L2 Scaffolded    → generated crates/routes/contracts + passing boundary checks
L3 Executable    → ≥1 thin use-case path CI-green end-to-end
L4 Governed      → permissions + trust classification + audit emission + runbook + rollback
L5 Externalizable→ API docs + sandbox + SDK/example + event/webhook + changelog
L6 Sellable      → onboarding + pricing hypothesis + support model + import path + customer-evidence
L7 Enterprise    → restore drill + isolation plan + Trust Center evidence + procurement packet + burn-in
```

Three gates carry the strategy: **L4→L5 = market-evidence gate** (kill if evidence absent),
**L5→L6 = commercial-packaging gate** (cite pricing hypothesis), **L6→L7 = launch-readiness
gate** (the 9-item pack, below). Rules: **demotion permitted, skipping is not**; rung (what was
proven) is orthogonal to lifecycle (still on an active path); **AI/MCP gets no fast track to L5**.

---

## 5. Kill criteria + commercial-risk taxonomy (the honesty instruments)

These are the live continuation of the recovered **4-lane closure model / false-closure
validators** (legacy-recovery #3) — same founder DNA ("no phantom findings, verify at each step").

**Kill triggers (ADR-0231 §5; `kill-criteria/triggers.md`).** Every investment declares **≥4
kill triggers, one per class**; any firing schedules a kill decision at the next quarterly
review. Classes:
1. `regulatory` — regulatory complexity exceeds team capacity (named regulation + control gap + resource estimate).
2. `reliability` — reliability ceiling unreachable (SLO history + restore-drill log + architectural blocker).
3. `market_evidence` — absent at the L4→L5 gate (customer-evidence record ids + trailing-window summary + thesis-claim diff).
4. `opportunity_cost` — dominated by a higher-return investment (named competitor for capital + resource contention + council ledger).

A `kill` record is incomplete without a populated **migration_path** (customers / capabilities /
deployables / data-retention / evolution-stage). Decision recorded *regardless of outcome* —
continuing past a fired trigger without a record is a governance violation.

**Commercial-risk classes (closed enum; `commercial-risks/risk-classification.md`):** `market`,
`regulatory`, `competitive`, `operational`, `reputational`. Severity `low|medium|high|
catastrophic`; `catastrophic` forces a quarterly-review kill check regardless of likelihood.
Closed enum on purpose — free-form classes let the same risk be re-labeled to dodge a trigger.

**Launch-readiness (ADR-0231 §4; `launch-readiness/checklist.md`):** L6→L7 needs **all 9
mandatory items** — onboarding flow, pricing artifact, support runbook, import path,
customer-evidence pack (≥3 customers, ≥2 segments), restore-drill log (`passed`),
isolation-plan, Trust Center pack, procurement packet. Closed list; "overall complete" is
mechanical; reviewers MUST verify before the L6→L7 transition.

**Decision forum for all of the above:** the **quarterly portfolio review**, 13-week cadence,
record-kept under `portfolio/quarterly-portfolio-reviews/`.

---

## 6. The live backlog program — High-value areas + strategic Moves

The dominant strategic thread in the GitHub backlog is the **"Maturity at Full Scale"** program
(milestone `2026-Q3..2027-Q1`, EPIC **#1557**): transform Bominal from a documentation-rich +
sample-record state into an **AWS-like governed / operable / measurable / customer-verifiable**
platform. Anchored to AWS Well-Architected pillars + SaaS control-plane-vs-application-plane
separation. **The maturity program IS the refactor** — not feature work then refactor later;
a **feature-work moratorium** governs the foundation tranche (Wave-0 + Move #0 + Moves #1–#4,
per the Q5 user resolution).

### 10 High-value areas (#1570–#1579)
1. Tenant isolation audit · 2. Identity & authorization model · 3. Tenant onboarding /
activation factory · 4. Backup, restore & DR proof · 5. Observability & operational readiness ·
6. Contract compatibility & deprecation · 7. Metering, quotas & unit economics · 8. Data & AI
governance · 9. Import / migration factory · 10. Platform governance as executable checks.

### 8-move program (#1557 parent; each Move is its own EPIC + ralplan cycle)
- **Move #0 — Tenancy foundation (#1558)** — tenant identity / data ownership / authorization /
  topology / audit / isolation **impossible-to-bypass at the runtime boundary**. Framed as **the
  biggest costly pitfall**: fixing tenancy later touches every table, query, entitlement check,
  backup/restore path, analytics dataset, AI context pack, import/export flow, audit trail, and
  compliance claim. Wave-0+1 prerequisite. (ADR-0018 multi-tenant, 0225 trust zones, 0226 entitlements.)
- **Move #1 — Proof Ladder as release gate (#1559)** — every catalog record carries L0..L7; repoctl enforces evidence before claims rise.
- **Move #2 — Product Control runtime resolver/enforcer (#1560)** — entitlement lookup, topology resolution, quota/rate-limit, metering, capability lifecycle.
- **Move #3 — Ops contracts as operational evidence (#1561)** — every deployable carries SLO/RTO/RPO/dashboard/alert/runbook/rollback/restore-drill (docs → evidence).
- **Move #4 — Trust runtime enforcement chain (#1562)** — single enforcement chain merging Trust + Product Control + Ecosystem.
- **Move #5 — Plane separation (control / data / analytics) (#1563)** — isolation, trust zone, runtime, storage class, replication, analytics export, AI/knowledge-pack eligibility.
- **Move #6 — Ecosystem contracts cloud-provider-grade (#1564)** — OpenAPI/proto + compat checks + SDK/example + sandbox + webhook/event shape + changelog + deprecation.
- **Move #7 — Builder OS executable (#1565)** — CODEOWNERS generation, PR-class validation, council gates, on-call lookup, doc freshness, scorecards.
- **Move #8 — Architecture fitness functions + complexity budgets (#1566)** — dependency direction, domain seams, doc freshness, deprecated-record sunset, complexity budgets, consolidation triggers.
- **Wave-0 prerequisite** — ADR promotion sweep (6 still-`Proposed` ADRs 0222–0233 → Accepted; #1556) + centralized engineering-practices registry + maturity scorecard (#1580).

### Open user-decision EPICs gating the program
- **#1567 (Q5)** — refactor-first posture + feature-work moratorium ratification.
- **#1568 (Q6)** — Move #0 tenancy aggressiveness scope: forward-only vs migrate-existing.
- **#1569 (Q7)** — AWS Well-Architected adoption shape: explicit framework vs inspiration.

(These mirror the founder "one-way door / explicit sign-off" gating discipline — strategic
posture is ratified by the user, not assumed by agents.)

---

## 7. Roadmap milestone landscape (backlog evidence)

Issue milestones, by volume (250-issue sample) — the de-facto delivery roadmap:

| Issues | Milestone | Theme |
|---|---|---|
| 58 | 2026-Q2 Core Work/Business MVP | the corporate/work MVP (Bench app shape #1284, dev surface #1228) |
| 34 | 2026-Q2 Program Foundation | KR policy-intelligence ingestion/regression (#1335–#1337); fintech-foundations M0 (#1137) |
| 27 | 2026-Q2 M3 KR Group Payroll + Mail Launch | **the wedge launch** — KR group payroll + corporate mail prod gate (#1198, #1219) |
| 26 | Maturity at Full Scale (Q3..2027-Q1) | the 8-move program above |
| 19 | 2028 Infrastructure Moat | deferred orbital/geospatial intelligence (see below) |
| 11 / 2 / 1 | Foundation / MVP / Audit (12r) | early program scaffolding |
| 8 | 2026-Q2 Partner-Launched Capabilities | plugins-as-partner-capabilities, accounting pilot (#1506) |
| 2 each | 2027-Q1 Marketplace+Profiling · 2026-Q3 Health Entry · 2026-Q4 Industry Entry I (Logistics+Transport+Warehouse) | post-wedge expansion arms |
| 1 | 2026-Q3 Design Partner Alpha | post-M3 MVP-plus follow-up (#1218) |

**Forward / deferred verticals in the backlog** (relevant to the founder's maximal-vertical-scope
canon, incl. defense + power-grid):
- **2026-Q4 Fintech Foundations (#1136)** — PG/payment-gateway first slice → open banking → corporate AP/AR → ledger/reconciliation → payouts → 2027 escrow/insurance-marketplace → 2028+ capital-markets. Explicit **product-boundary discipline**: start as software/orchestration layer over *regulated partners*; holding funds / lending / broker-dealer / money-transmission require separate legal gating.
- **2028 Infrastructure Moat (#1119, #1128)** — deferred **Realtime Global Tracking + Orbital Intelligence** (ships/flights/satellites, AIS/ADS-B/TLE-OMM, orbit prediction) and **Satellite Imagery + Geospatial Intelligence** (imagery-to-3D, pipeline/infrastructure monitoring, change detection, financial intelligence). Justified as Map-backed shared modules feeding logistics / public-safety / **defense** / insurance / marketplace — i.e., the long-horizon edge of the maximal-vertical-scope thesis.

---

## 8. Backlog hygiene signal (not strategy, but shapes it)

A large share of non-strategic issues are **CUG-program (multi-agent "concurrent update group")
hygiene**: orphan-worktree cleanup (#1588), EPIC title/template drift (#1543), wiki-stub
residuals (#1542), drift-backlog operationalization (#1177), label-taxonomy consolidation. This
is the operational tail of running the platform via parallel agent swarms — consistent with the
CONTEXT.md "Drift Backlog / Doc Drift Gate / Label Taxonomy" governance vocabulary. Strategically
relevant only as evidence that **governance-as-executable-checks (High-value area #10 / Move #7/#8)
is the program's self-correction mechanism**.

---

## 9. One-paragraph digest

Bominal (= oyatie pre-rename) is a **single tenant-aware multi-arm enterprise platform** whose
strategy is **moat-compounding sequencing**: build platform → payments → corporate(KR-payroll
wedge) → messaging → documents → notify → healthcare-billing → intelligence, so each arm rides
the prior arms' data graph + trust posture. Five moats (catalog, trust, ecosystem, data+AI,
builder-OS) make each new arm cheaper and higher-trust than the last; an arm that compounds no
moat is killed. Readiness is measured on the **Proof Ladder L0..L7** with three strategic gates
(market-evidence at L4→L5, packaging at L5→L6, 9-item launch-readiness at L6→L7); **kill triggers
(4 classes)** + a **closed commercial-risk enum** + the **quarterly portfolio review** are the
capital-allocation governance — the live continuation of the recovered 4-lane / false-closure
honesty instrument. The dominant in-flight program is **"Maturity at Full Scale"**: an
AWS-Well-Architected-anchored **8-move refactor (Move #0 tenancy-first being the biggest pitfall)
+ 10 high-value areas**, executed under a **feature-work moratorium** and gated by explicit
founder user-decisions (Q5/Q6/Q7). Near-term delivery centers on the **2026-Q2 KR group-payroll +
corporate-mail launch (M3)**; long-horizon backlog reserves fintech foundations (2026-Q4) and a
deferred 2028 orbital/geospatial "infrastructure moat" feeding the defense/logistics edge of the
maximal-vertical-scope thesis.
