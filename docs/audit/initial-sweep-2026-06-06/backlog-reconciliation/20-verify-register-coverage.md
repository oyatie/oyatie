# 20 — VERIFY: Decision-Register Coverage (exist-vs-net-new) + Overlap with linux A5

**Lane:** backlog-reconciliation / register-coverage verification (READ-ONLY).
**Date:** 2026-06-06.
**Verdict mode:** evidence-based; every ADR status read directly from the live file's front-matter, not from the (drift-prone, generated) ADR-INDEX.

---

## 0. SOURCES OF TRUTH (located, verbatim)

The "DECISION REGISTER (#1..#21)" is NOT yet a standalone file under `backlog-reconciliation/` — it is the
**platform-readiness program backlog**, located in the **source** repo (this is exactly task #21's "source
backlog (platform-readiness program)"):

- Register home (canonical thematic list): `/Users/jasonlee/Developer/source/.omx/HANDOFF-platform-readiness-2026-06-01.md`
  - L44 (verbatim): *"...decision register (#1..#21, including bespoke cloud toolchain services, automation ratchet, and claim ceiling)..."*
  - L132 heading (verbatim): `## 4. DECISION REGISTER → propagate as plan-work (backlog §P)`
- Executable packet plan (seed IDs D01..D21): `/Users/jasonlee/Developer/source/.omx/plans/prd-platform-readiness.md` L334–349
  - L339 (verbatim): *"Seed packet IDs mirror backlog register #1..#21 as `P0.7-D01-d1-trinity`, `P0.7-D02-effective-dating`, ..., `P0.7-D17-dogfood-need-sequencing`, `P0.7-D18-merge-conflict-elimination`, `P0.7-D19-bespoke-cloud-toolchain-services`, `P0.7-D20-automation-ratchet`, and `P0.7-D21-claim-ceiling-no-empty-promises`..."*
- Evidence backlog (prose findings A–Q): `/Users/jasonlee/Developer/source/.omx/backlog/platform-readiness-backlog.md`
- Live ADR corpus scanned: `/Users/jasonlee/Developer/source/docs/decisions/ADR-*.md`

> **The register operates on the SOURCE/oyatie ADR namespace** (`source/docs/decisions/`, the 0001–0514 space), NOT the
> 26-ADR linux distributed-DB set (`linux/docs/decisions/`, ADR-0001..0026). This matters for §3 overlap analysis.

---

## 1. PER-ITEM (#1..#21): EXIST-vs-NET-NEW + LIVE STATUS

Item ordering follows the §P seed packet IDs D01..D21 (PRD L339–342) reconciled with the Section-4 thematic list
(HANDOFF L135–142). "Exists" = the named target ADR(s) to be **amended** already live; "NET-NEW" = the decision has
**no ADR file** and must be authored. Every status below was read from the live file front-matter.

| # | Register item (packet id) | Target ADR(s) named | Verdict | Live status (file front-matter) |
|---|---|---|---|---|
| 1 | D01 d1-trinity (+ A2a EntityMutated / A2b AgentAuthored protos) | **none** (new trinity ADR + proto schemas) | **NET-NEW** | no ADR file for trinity/EntityMutated; only prose (backlog §A2-RESOLVED L167–181) + specs. Related-but-not-same: 0035 (`proposed`), 0059 (`accepted`). |
| 2 | D02 effective-dating (ontology temporal kernel) | **none** (new effective-dating kernel ADR); touches 0006 | **NET-NEW** | no `effective-dating` ADR file; grep of decisions/ = 0 hits. Related: ADR-0006 ontology-typed-entity-layer (`accepted`). |
| 3 | D03 enforcement system / `oya` CLI retirement / Tide placement | **amend ADR-0513, ADR-0363** | **EXISTS (amend)** | 0513 oya-ci-bespoke-rust-prow `Accepted`; 0363 retire-agentic-vcs-foundry `Accepted`. |
| 4 | D04 pure-split + ADR-0131 reconcile | **amend ADR-0131, ADR-0512** | **EXISTS (amend)** | 0131 per-microservice-flat-layout `Accepted`; 0512 canonical-monorepo-pattern `Accepted`. |
| 5 | D05 packs (regional/localization pack roots) | **amend ADR-0064, ADR-0010** | **EXISTS (amend)** | 0064 canonical-base-and-localization-packs `accepted`; 0010 regional-pack-architecture `proposed`. |
| 6 | D06 multi-platform native clients | **none** (new multi-platform-client ADR) | **NET-NEW** | no `multi-platform` ADR file; only prose (HANDOFF L75–77 UniFFI/SwiftUI/WinUI3/Jetpack). |
| 7 | D07 parallel-dev | **amend ADR-0111, 0124, 0360, 0366** | **EXISTS (amend)** | 0111 merge-queue `Proposed`; 0124 own-merge-queue-webhook `accepted`; 0360 ci-pipeline-optimization `Proposed`; 0366 agentic-high-throughput-pipeline `Accepted`. |
| 8 | D08 verification / testing | **amend ADR-0139, 0346** | **EXISTS (amend)** | 0139 agentic-slo-gated-promotion `Accepted`; 0346 oya-verify-full-ci-mirror `Proposed`. |
| 9 | D09 frontend (Leptos canonical) | **ADR-0393** (source status now Accepted; index/migration evidence pending) | **EXISTS (amend/finish)** | 0393 leptos-canonical-app-shell `Accepted`. (Companion 0372 SolidJS `Superseded` — see FIX-set.) |
| 10 | D10 honest-claim (§K) | **none** (new honest-claim-contender ADR; §K) | **NET-NEW** | no `honest-claim` ADR file. Related-but-distinct: ADR-0129 changeset-plan-dag-**and-honest-claims-gate** (exists) and ADR-0123 hyperscaler-maturity-claim-gate — register §K/§21 are broader, still net-new. |
| 11 | D11 cross-artifact SSOT gate | **amend ADR-0365** | **EXISTS (amend)** | 0365 automated-adr-lifecycle-and-propagation `Accepted`. (Gate itself is new tooling under an amended 0365.) |
| 12 | D12 pure-Rust tooling (§Q allowlist) | **none** (new pure-Rust-tooling ADR; §Q) | **NET-NEW** | no `pure-rust`/`pure-Rust` ADR file; §Q is prose (HANDOFF L79). |
| 13 | D13 FIX: dup ADR-0377 (renumber one) | **ADR-0377 (x2)** | **EXISTS (fix)** | DUP confirmed: 0377-forgejo-board-git-ref-cas `Proposed (conditional)` AND 0377-kafka-to-pulsar-via-kop `Accepted` — two files, one number. |
| 14 | D14 FIX: ADR-0511 → superseded_by 0513 | **ADR-0511, ADR-0513** | **EXISTS (fix)** | 0513 `Accepted` (operative). 0511 named in plans as `Proposed`-Argo-orchestrator, not yet marked superseded (the fix). |
| 15 | D15 FIX: foundry eradication + 3-axis status enum + regenerate indexes | **amend ADR-0513/0365** (per PRD L451) + 0335 | **EXISTS (fix)** | 0335 foundry-retired-absorbed-by-intelligence `Accepted`; enforced via amended 0513/0365. |
| 16 | D16 (dogfood-need-sequencing / infra-sovereignty schedule) | spans existing CI/structure ADRs | **EXISTS (amend)** — process item over 0513/0509/0510 | 0509 hyperscaler-service-decomposition `Accepted`; 0510 scm-bespoke-cutover-trigger `Proposed`. |
| 17 | D17 dogfood-need-sequencing | (process; binds existing ADRs) | **EXISTS (amend)** | sequencing over the live CI/structure set; no new ADR named. |
| 18 | D18 merge-conflict elimination / tide generated-artifact registry + `git merge-tree` oracle | **ADR-0513 / cloud-ci (Tide)** | **EXISTS (amend)** | binds Tide into 0513 (`Accepted`) + 0111 (`Proposed`); registry is new spec, not new ADR. |
| 19 | D19 bespoke-cloud-toolchain-services | **none** (new ADR); reconcile 0349/0359/0361/0408/0511/0513/0514 | **NET-NEW** | spec exists (`specs/bespoke-cloud-toolchain-services.json`) but **no ADR file**. Reconciliation touches existing 0513 `Accepted` / 0514 `Proposed`. |
| 20 | D20 automation-ratchet | **none** (new ADR) | **NET-NEW** | only specs (`specs/phase0-automation-matrix.json`, fixtures); no ADR file. |
| 21 | D21 claim-ceiling / hyperscaler production-readiness claim contract | **none** (new ADR) | **NET-NEW** | spec exists (`specs/hyperscaler-production-readiness-claim-contract.json`) but **no ADR file**; gate is new. |

### 1a. Roll-up of the task's explicitly-named ADR list (all VERIFIED to EXIST)

All 23 register-named **existing** ADR numbers were confirmed present in `source/docs/decisions/` with these live statuses:

| ADR | Title (slug) | Status |
|---|---|---|
| 0513 | oya-ci-bespoke-rust-prow-cicd-platform | Accepted |
| 0514 | build-ci-cd-pipeline-target-architecture | Proposed |
| 0131 | per-microservice-flat-layout | Accepted |
| 0512 | canonical-monorepo-pattern | Accepted |
| 0064 | canonical-base-and-localization-packs | accepted |
| 0010 | regional-pack-architecture | proposed |
| 0111 | merge-queue-projected-state-fix-at-any-stage | Proposed |
| 0124 | own-merge-queue-webhook-driven | accepted |
| 0360 | ci-pipeline-optimization-program | Proposed |
| 0366 | agentic-high-throughput-self-enforcing-pipeline | Accepted |
| 0130 | deprecate-knowledge-graph-registry→ontology | Accepted |
| 0139 | agentic-slo-gated-promotion | Accepted |
| 0346 | oya-verify-must-run-full-ci-mirror | Proposed |
| 0365 | automated-adr-lifecycle-and-propagation | Accepted |
| 0247 | self-hosting-self-modification-doctrine | Proposed |
| 0335 | foundry-microservice-retired-absorbed-by-intelligence | Accepted |
| 0393 | leptos-canonical-app-shell-frontend | Accepted |
| 0372 | frontend-stack-solidjs (historical) | Superseded |
| 0255 | intelligence-as-two-layer-ai-substrate | Proposed |
| 0263 | observability-emission-contract | Proposed |
| 0296 | library-first-credential-sidecar | Proposed |
| 0509 | hyperscaler-service-decomposition-pattern | Accepted |
| 0510 | scm-bespoke-hyperscaler-destination-cutover-trigger | Proposed |
| 0363 | retire-agentic-vcs-foundry→intelligence-forgejo | Accepted |
| 0383 | observability-stack-reconciliation (LGTM/Grafana) | Accepted |
| 0377 (x2) | forgejo-board-git-ref-cas (Proposed-cond) **+** kafka-to-pulsar-via-kop (Accepted) | DUPLICATE NUMBER |

### 1b. The NET-NEW set (9 — no ADR file exists; author from scratch)

1. **D1-trinity / EntityMutated+AgentAuthored protos** (#1) — no ADR; backlog §A2a/§A2b prose only.
2. **effective-dating-kernel** (#2) — no ADR; ontology temporal primitive.
3. **multi-platform-client** (#6) — no ADR; shared-Rust-core native clients.
4. **honest-claim-contender / §K** (#10) — no ADR (0129/0123 are adjacent but not it).
5. **pure-Rust-tooling / §Q** (#12) — no ADR.
6. **cross-artifact-SSOT-gate** (#11) — gate is new tooling; lands as an **amendment to existing 0365**, but the gate
   mechanism itself has no prior ADR. Classified amend-of-0365 above; flagged here as new-mechanism.
7. **bespoke-cloud-toolchain-services** (#19) — spec exists, **no ADR**.
8. **automation-ratchet** (#20) — specs/fixtures exist, **no ADR**.
9. **claim-ceiling** (#21) — spec exists, **no ADR**.

> EVIDENCE for net-new: `ls source/docs/decisions/ | grep -iE 'trinity|effective-dat|honest-claim|claim-ceiling|automation-ratchet|cross-artifact|multi-platform|pure-rust|bespoke-cloud-toolchain|entitymutated'` returns **only** `ADR-0129-changeset-plan-dag-and-honest-claims-gate.md` (the adjacent, non-matching honest-claims-**gate** ADR). All nine register-new topics have zero matching ADR files.

---

## 2. EXIST-vs-NEW SUMMARY

- **Items resolved by AMENDING existing ADRs (12):** #3,#4,#5,#7,#8,#9,#11,#13,#14,#15,#16/#17,#18 — every target ADR confirmed live.
- **Items requiring a NET-NEW ADR (9):** #1,#2,#6,#10,#12,#19,#20,#21 (+ the cross-artifact gate *mechanism* in #11, formally an amend-of-0365).
- **FIX-class items (housekeeping over existing ADRs, no new decision):** #13 dup-0377 renumber, #14 0511→superseded_by-0513, #15 foundry-eradication + status-enum + regenerate-indexes.
- **No register item targets a NON-EXISTENT *numbered* ADR** — every numeric ADR the task listed is present. The "new" work is purely the 9 thematic net-new decisions, which carry no numbers yet (they will land additively in the live free block above 0514).

---

## 3. OVERLAP with linux A5 (the new-ADR lane)

**linux A5 lane definition (verbatim, `linux/docs/audit/initial-sweep-2026-06-06/UNIFIED-EXECUTION-PLAN.md` L52–56):**

> `A5  NEW / reshaped ADRs (ADDITIVE — into the live free block, NOT a renumber):`
> `oya-ci reshape (0513; supersede/relate 0511/0124, phase 0369/0367/0366) · unified safety-gate (D-SAFETY) ·`
> `KR EmploymentClassification enum as KR localization-pack model (D-KR) · infra-sovereignty ordered+M0 schedule (D-SEQ) ·`
> `domain-cohesion meta-ADR (D15) · masterplan-generated-wiring meta-ADR (D1) · data-engine-endpoint ADR (D-INTEL) ·`
> `amend cloud-intelligence docs to cite ADR-0389/0390`

**Critical fact:** linux A5 operates on the **SAME source/oyatie ADR namespace** (0513, 0511, 0124, 0366, 0389/0390 are
all `source/docs/decisions/` ADRs — confirmed live). It is NOT scoped to the 26-ADR linux distributed-DB set
(`linux/docs/adr-archive/ADR-0001-cohesion-thesis-one-product-flat-catalog.md which contains none of these numbers). Therefore A5 and the source DECISION
REGISTER write into one shared canon and **DO collide**. The overlap set:

| Overlap | linux A5 atom | source register item | Nature of collision |
|---|---|---|---|
| **O1 oya-ci reshape** | A5 "oya-ci reshape (0513; supersede/relate 0511/0124; phase 0369/0367/0366)" — D3 ruling | #3 enforcement/Tide (amend 0513) + #14 (0511→superseded_by 0513) + #7 parallel-dev (0124/0366) | **DIRECT, same ADRs.** Both lanes amend 0513 and resolve 0511-supersession + touch 0124/0366. Must be one edit, not two. |
| **O2 D1 masterplan-wiring meta-ADR** | A5 "masterplan-generated-wiring meta-ADR (D1)" | #11 cross-artifact-SSOT-gate (amend 0365 "automated-adr-lifecycle-and-propagation") | **STRONG.** A5-D1 (masterplan generated FROM ADRs) and register-#11 (cross-artifact-agreement/propagation gate) are the same SSOT-generation/propagation concern over 0365. Naming clash too: A5 calls it "D1", register calls its trinity item "D1/D01". |
| **O3 infra-sovereignty / D-SEQ** | A5 "infra-sovereignty ordered+M0 schedule (D-SEQ)" | #16/#17 dogfood-need-sequencing (over 0509/0510/0513) | **MODERATE.** Both are the build-first/cutover-trigger sequencing decision over 0509/0510 + cutover_trigger field. |
| **O4 D15 domain-cohesion meta-ADR** | A5 "domain-cohesion meta-ADR (D15)" | #4 pure-split + 0131/0512 reconcile (oya/cloud cohesion) | **MODERATE.** Domain-cohesion (one-product flat catalog, ADR-0001 lineage) and the pure-split #4 both govern service-tree cohesion/flattening; risk of two ADRs ruling the same boundary. |
| **O5 frontend** | A5 *implicitly* via "amend cloud-intelligence docs" set; deck=Leptos | #9 frontend (0393) | **WEAK/indirect.** Both touch the Leptos canonical decision (0393) + ADR-0513 deck SolidJS→Leptos flip. |
| **O6 D-INTEL data-engine-endpoint** | A5 "data-engine-endpoint ADR (D-INTEL); amend docs to cite 0389/0390" | (no direct register item; nearest = #1 trinity intelligence dispatch) | **WEAK.** A5-only net-new ADR; register #1 trinity touches intelligence-substrate but not the data-engine-endpoint specifically. Low collision, watch for boundary with trinity. |

**No-overlap A5 atoms (A5-unique, no register twin):** D-SAFETY (unified safety-gate) and D-KR (KR EmploymentClassification
localization enum) have **no** corresponding source-register item — these are linux-A5-only net-new ADRs.

**No-overlap register items (register-only, no A5 twin):** #2 effective-dating, #5 packs (0064/0010), #6 multi-platform-client,
#8 verification/testing (0139/0346), #10 honest-claim/§K, #12 pure-Rust-tooling/§Q, #13 dup-0377-renumber, #15 foundry-eradication,
#18 merge-conflict/tide-registry, #19 bespoke-cloud-toolchain, #20 automation-ratchet, #21 claim-ceiling.

**COLLISION RISK (load-bearing):** O1 and O2 are the dangerous ones — both lanes intend to *author/amend the same ADR
numbers* (0513, 0365) additively into "the live free block." If run independently, both will mint or edit overlapping
decisions → dual-canon / id-collision in the additive space. **They must be merged into a single authoring pass
(de-dupe D1/D15 meta-ADR naming, single 0513 amendment, single 0365 amendment).**

---

## 4. LIVE ADR / PROPOSED COUNTS (ground-truth file scan, not the generated index)

Scanned `source/docs/decisions/ADR-*.md` front-matter directly:

- **Total ADR files:** 346
- **Distinct ADR numbers:** 345 (delta of 1 = the **duplicate ADR-0377**, two files sharing one number — register #13).
- **Proposed-family** (any case, incl. "Proposed (conditional…)" / "Proposed (target: Accepted upon PR #143…)"): **131 files.**
- **Accepted-family** (Accepted/accepted, excluding amendment/superseded-marked lines): **172 files.**
- **Superseded:** **16 files.**
- Remainder: deprecated/Amended/Accepted(amendment)/non-standard multi-state status strings (`OK`, `completed-locally`, `Draft|Accepted|Shipped`, etc.) — a known status-vocab-incoherence finding (register #15 / backlog B-P1-1).

> **INDEX DRIFT FLAG (verify-lane finding):** `source/docs/ADR-INDEX.md` L17 status-counts line claims
> "Accepted 146, accepted 37 (=183); Proposed 91, proposed 34 (=125); Superseded 11 + superseded 1 (=12)." This
> **does not match** the ground-truth file scan (172 Accepted-family / 131 Proposed-family / 16 Superseded). The
> ADR-INDEX is a generated-from-stale-source artifact — itself one of the register FIX items (#15 "regenerate indexes",
> backlog L264/L281 generated-index drift). Trust the file scan, not the index.

---

## 5. RETURN-LEVEL FINDINGS (one-line)

- Register #1..#21 = **12 amend-existing + 9 net-new** (net-new: trinity/EntityMutated, effective-dating, multi-platform-client, honest-claim/§K, pure-Rust-tooling, cross-artifact-gate-mechanism, bespoke-cloud-toolchain-services, automation-ratchet, claim-ceiling). Plus 3 FIX-class (dup-0377, 0511→superseded, foundry/status-enum/regen).
- **Every numbered ADR the task listed EXISTS** in `source/docs/decisions/`; none point at a missing number. Net-new items carry no numbers yet (additive into the free block > 0514).
- **linux-A5 ↔ register overlap (6):** O1 oya-ci/0513+0511+0124 (DIRECT), O2 masterplan-wiring/0365 (STRONG), O3 infra-sovereignty/0509-0510 (MODERATE), O4 domain-cohesion/pure-split (MODERATE), O5 frontend/0393 (WEAK), O6 data-engine vs trinity (WEAK). A5-unique: D-SAFETY, D-KR. Merge O1+O2 authoring or risk dual-canon in the additive block.
- **Live counts:** 346 ADR files / 345 distinct numbers (dup-0377); **131 Proposed-family, 172 Accepted-family, 16 Superseded** (ground-truth scan; ADR-INDEX counts are drifted/stale).
