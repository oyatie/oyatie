# 00 — MASTER ACCEPTED-ADR AUDIT (initial sweep 2026-06-06)

> **Synthesis of VERIFIED findings only.** Every CRITICAL/HIGH item below was opened at its cited
> `path:line` and CONFIRMED verbatim against the actual files in
> `/Users/jasonlee/Developer/source/docs/decisions/` during the adversarial verification lane
> (`20-verified.md`: **CONFIRMED 41 · REFUTED 3**). The machine index
> `docs/machine-readable/decisions.json` is DRIFTED (tops out at ADR-0392) and was NOT trusted.
> **Canon baseline:** `docs/audit/initial-sweep-2026-06-06/synthesis/decision-record-oyatie-canon.md`
> (cited `canon:NN`). **READ-ONLY synthesis** — this document registers the findings + proposes the
> fix; it applies nothing. Every mutating fix is gated on founder ratification (verify-at-every-step;
> never mutate on an unverified verdict).
>
> **Cross-reference:** supersession-completeness / status-edge-drift / flat-crates blast-radius are
> owned by `contradiction-audit/00-SUPERSESSION-COMPLETENESS.md` (12 confirmed supersession-completeness
> contradictions). This document does NOT re-derive that register; it cites it where the two intersect
> (the supersession edges to write are the *mechanical fix* for several contradictions below).
>
> **Sources:** lanes 0–8 (`10-r*.md` + `10-cross-cutting.md`) → verification (`20-verified.md`).
> **Date:** 2026-06-06.

---

## SECTION 5 FIRST — COVERAGE (so every later count is grounded)

**Accepted ADRs on disk (verified):** `grep -ric "^status: accepted"` over
`/Users/jasonlee/Developer/source/docs/decisions/*.md` = **169 Accepted** of **349** total ADR files.
(Lane 8's corpus scan reported 171 by a looser front-matter grep; the strict single-line count is 169.
The 2-ADR delta is table-form `- Status: Accepted` headers vs YAML `status:` — both were audited.)

**Audited per lane (all `status:Accepted`, read in full unless noted):**

| Lane | Range | Audited | Notes / read caps |
|---|---|---|---|
| 0 | 0001–0034 | 12 | all full |
| 1 | 0051–0095 | 23 | all full |
| 2 | 0096–0132 | 25 | 0131 (35 KB) read to L160 — status + pure-split amendment confirmed |
| 3 | 0133–0182 | 45 | all full |
| 4 | 0184–0210 | 27 | all full |
| 5 | 0222–0335 | 17 | 0258→L794/1108, 0329→L1199/2570, 0330/0331/0332/0334 partial — findings scoped to read region |
| 6 | 0350–0391 | 25 | all full |
| 7 | 0393–0515 | 13 | all full |
| 8 | cross-cutting | (171 enumerated; ~24 deep-read; corpus greps over all) | the ~140 non-cluster Accepted ADRs were NOT individually deep-read by lane 8 — per-domain lanes own those |

**Total distinct Accepted ADRs given a disposition: ~187 disposition-rows across lanes**, reconciling to
**169 unique Accepted ids** (the over-count is the duplicate-id `0377` appearing twice + `0131`
double-listed in lane 2's recount; both are noted in their lanes). **No silent caps:** every range's
skipped (Proposed/Superseded/deprecated/Amended) ADRs are enumerated in the per-lane artifacts.

**Coverage gaps (declared):**
- Lane 8's cross-cutting scan deep-read only the 5 vendor/identity/CI clusters; a *duplicate decision*
  outside those clusters and using none of {kafka, forgejo, istio} could be missed. The per-domain
  lanes (0–7) cover those ADRs individually, so the residual risk is a **cross-ADR duplication** not
  visible to a single-range lane.
- Lane 5 read-caps on 5 large ADRs (0258/0329/0330/0331/0332/0334) — findings are scoped to the read
  region and marked; tails were appendices/footprints.
- ID gaps verified ABSENT (not silent): 0012, 0033, 0068, 0070–0082, 0084–0089, 0125–0127 (this range),
  plus the phantom-ADR set (§1 C-INTEGRITY).

---

## SECTION 1 — CONTRADICTIONS REGISTER (confirmed · cited · severity-ranked · with fix)

> Counts: **21 CRITICAL · 20 HIGH** confirmed (= the 41 from `20-verified.md`). REFUTED items are listed
> at the end so they are not silently dropped. Each row: finding → cited evidence (both sides) → FIX.

### CRITICAL (21)

**C-1 — Accepted ADRs built on PROPOSED foundation.** `ADR-0007` (Cedar) `status: proposed` (`:3`),
`ADR-0002` (tenant/identity) `proposed` (`:3`), yet load-bearing for Accepted `ADR-0001:81-82`,
`ADR-0028:68`, `ADR-0034:20`, `ADR-0099:38`(→Proposed `ADR-0022:3`), `ADR-0116:50`(→Proposed
`ADR-0111:2`), `ADR-0512:22`/`ADR-0515:11`(→Proposed `ADR-0392`/`ADR-0408:3`).
**FIX:** ratify 0002/0007/0022/0111/0392/0408 (reconciled to canon D5/D6) OR demote the dependents until
the foundation lands. Until then the "Accepted" status is not load-bearing. (D14 Proposed-ledger.)

**C-2 — "Foundry" canon-dead but load-bearing in Accepted ADRs.** Anchor `ADR-0512:57` "`foundry` name
remains eradicated"; canon D-FOUNDRY-CLARIFY `canon:204`. Live: `ADR-0001:80-82,87`; `ADR-0011:60,63`;
`ADR-0017:54`; **`ADR-0018:45` canonicalizes the glossary term** "| Foundry | Oyatie's AI agent
runtime…|" — the terminology-canon ADR is the home of a forbidden term.
**FIX:** AMEND all four (foundry→intelligence/governance/cloud-intelligence per the 3-way route);
DELETE the ADR-0018 glossary `Foundry` row.

**C-3 — Forbidden flat-`crates/` + forbidden-vocab enum (ADR-0015).** `ADR-0512:55` flat `crates/`
"forbidden"; `ADR-0015:39` declares `crates/oya-<context>-<role>` canonical and `:42` lists `<context>`
enum incl. `platform, workspace, vertical, foundry, cloud` — four 0018-forbidden / 0512-eradicated tokens.
**FIX:** SUPERSEDE 0015 cleanly under 0512 (status flip — see supersession-completeness FIX-FC-1);
migrate the surviving dep-direction rule into the 0131/0512 lineage. **NOTE:** the live
`oya-governance-flat-crates` GATE still enforces depth-3 and REJECTS 0512's depth-5 — highest-severity
downstream item, owned by SUPERSESSION-COMPLETENESS FIX-FC-2.

**C-4 — ADR-0057 dangling + colliding supersedes.** `ADR-0057:32` "Supersedes: ADR-0055 (v3-era rename
plan)"; on disk `ADR-0055` = "Object Graph renamed to Ontology" (Accepted, different decision); the
"v3-era" file does NOT exist.
**FIX:** ARCHIVE 0057 (executed one-shot migration); fix/delete the supersedes edge first (D11(c)).

**C-5 — Flat-`crates/` catalog canon vs D-PURESPLIT.** `ADR-0058:33` "Every feature and product is an
independent microservice registered in `[workspace.metadata.oya.microservices]`"; `:144` "All crates
flat under `crates/`." Canon D-PURESPLIT `canon:171-172` "exactly two trees oya/ + cloud/, ERADICATE
everything else." `ADR-0056:242-256` `cloud public_layers` cross-import exemption violates
"no oya→cloud internal dep."
**FIX:** AMEND 0056/0058 topology to `{oya,cloud}` (keep à-la-carte product model + GTM-not-arch rule).

**C-6 — Dead VCS toolchain mandated live.** `ADR-0053:49` + `ADR-0103:38-50` ban direct `git`/`gh`,
fix grit/icm as sole primitives; retired by `ADR-0116:43` + `ADR-0363:16-17` ("use git as is").
0053/0103 both `status: Accepted`. Canon D-AEC-DECLINE `canon:192-193`.
**FIX:** SUPERSEDE 0053 + 0103 by 0116/0363/0515; flip status (do not leave Accepted).

**C-7 — Kafka on the critical path.** `ADR-0059:112`, `ADR-0062:71`, `ADR-0091:114` ("bind Kafka
producer to WriteGate"). Canon D-EVENT `canon:147` (Pulsar) + D-D1-TOPOLOGY `canon:190` (Kafka removed
from consistency path).
**FIX:** AMEND Kafka→Pulsar (eventual fan-out only); adopt D-D1 consistency-token model on the critical path.

**C-8 — Accepted-on-Proposed + foundry-home for the autonomy gate.** `ADR-0099:38` calls Proposed
`ADR-0022` a "mandate"; homes enforcement in `foundry::supervisor` (`:97`) / `oya-foundry-autonomy-ceiling-app`
(`:239`). Canon D16 governance-owned hard gate (`canon:39`) + D6 PARC engine (`canon:34`).
**FIX:** AMEND — re-home to governance, retarget engine to PARC, fix the Proposed-0022 dependency.

**C-9 — Foundry pipeline enthroned + depends on Proposed 0111.** `ADR-0116:42` "Foundry pipeline
(M01-P18) is the sole canonical workflow for concurrent agent work"; `:50` cites Proposed `ADR-0111`;
`ADR-0363:37` retires the substrate (2/20 crates wired, never deployed).
**FIX:** AMEND 0116 — keep the grit/icm/rtk/vox retirement, replace the Foundry-pipeline mapping with
oya-ci/Forgejo per 0363/0515.

**C-10 — Enforcement façade.** `ADR-0128:22` `enforcement_status: advisory-until-product-prd-validator`;
`:155` "planned `oya-governance-*` lanes … remain backlog items, not active required checks" — declares
"binding source of truth" while enforcement is advisory/planned. Canon firewall `canon:181,196`.
**FIX:** KEEP the invariant catalog; AMEND to wire real RED/GREEN-proven gates OR relabel honestly.

**C-11 — Foundry cluster = live µservice canon.** `ADR-0136:121` "Foundry is one µservice with six
internal bounded contexts"; `ADR-0137:46` "exactly six bounded contexts"; `ADR-0138:45` Strangler INTO
`microservices/foundry/`; `ADR-0143:68-93` `release/foundry-*`. All Accepted, empty supersede edges.
Canon D-FOUNDRY-CLARIFY `canon:204`.
**FIX:** SUPERSEDE 0136/0137/0143 into the cloud-intelligence framework ADR (salvage the
one-perimeter+six-BC reasoning); ARCHIVE 0138 (Strangler to a dead address; reuse its template).

**C-12 — ADR-0160 Flagger vs D10.** `ADR-0160:42` "Flagger 1.x … canonical progressive-delivery
controller"; `:62` "Why Flagger over Argo Rollouts"; `:151` cites Superseded `ADR-0124`; `superseded_by:[]`.
Canon D10 `canon:62` ("Supersede Flagger (0160)" → Argo Rollouts), 0515 CD-face is Argo Rollouts (`:80`).
**FIX:** SUPERSEDE 0160 by the D-CICD Argo-Rollouts-patterned ADR; write the supersede edge
(SUPERSESSION-COMPLETENESS P1); keep the SLO-gate ladder + auto-rollback shape.

**C-13 — ADR-0187 Zitadel-canonical vs D5.** `ADR-0187:37` "Zitadel … is the canonical IdP … the
single issuer"; `:8` `superseded_by:[]`. Canon D5 `canon:31` (Zitadel = vendored bridge; demoted-as-
endpoint by 0476).
**FIX:** SUPERSEDE-as-endpoint (status → superseded-as-endpoint; `superseded_by:[0476]`); keep operative
as the Phase-1 bridge (SUPERSESSION-COMPLETENESS P2).

**C-14 — ADR-0192 Milvus owned by dead foundry + wrong layer.** `ADR-0192:47` "Milvus … owned by the
`foundry` µservice"; `:128` `microservices/foundry/iac/helm/milvus/`. Vector store is data-tier (D4),
not intelligence; foundry dead (D-FOUNDRY-CLARIFY).
**FIX:** AMEND — re-home to cloud-data (D4), Pulsar-only, reconcile the 0046-closed-vs-Phase-2-reopen tension.

**C-15 — ADR-0202 ArgoCD-canonical vs D3/D-CICD.** `ADR-0202:44` "ArgoCD is the canonical Tier-A engine."
Canon D3/D-CICD: bespoke-Rust oya-cd adopting Argo *patterns*; D-EXEC ARCHIVE/DROP 0511, consolidate to 0515.
**FIX:** AMEND — reframe Tier-A ArgoCD as vendored bridge → owned oya-cd (build-first-cutover-later);
KEEP Tier-B (OpenTofu) + Tier-C (Cluster API); relate to 0515.

**C-16 — Phantom-0150 epidemic.** `ADR-0150` on disk = "Cursor Pagination Canonical" (`:1`), mis-cited
as Cedar/policy-engine in `ADR-0239:49,92`, `ADR-0148:257`, `ADR-0182:165`. Real policy ADR = `ADR-0183`.
**FIX:** repoint all four sites to 0183 (HIGH-priority dangling-edge cleanup, D11).

**C-17 — ADR-0239 no supersession banner.** `ADR-0239:3` "Accepted (amendment)" governs `:21`
"`microservices/foundry/` is INTERNAL only"; 0335 marks it historical in PROSE only (`ADR-0335:593-595`),
not in 0239's own status. A 0239-only reader gets a dead topology.
**FIX:** SUPERSEDE/mark-historical 0239 by 0335 (status banner) + fix the phantom-0150 anchor — one of the
highest-priority integrity fixes in range.

**C-18 — ADR-0374 Jenkins+Forgejo as ratified canon.** `ADR-0374:188` "Decision (founder):
Jenkins-as-orchestrator"; `:36` Forgejo+git+Jenkins substrate; `status: Accepted`, `superseded_by:[]`;
EXCLUDED from `ADR-0515:9` supersedes set. Canon D-CICD/D2.
**FIX:** SUPERSEDE by 0515 (superseded-on-cutover; keep the physical scaffold as the unratified bridge);
write the edge (SUPERSESSION-COMPLETENESS P3).

**C-19 — ADR-0380 rebuilds Jenkins+Forgejo.** `ADR-0380:21` "Re-establish the Jenkins CI farm";
`:38` "Forgejo-canonical"; `Accepted (amendment)`, `superseded_by:[]`; EXCLUDED from 0515's supersedes.
Canon D-EXEC `canon:78` (drop Jenkins/Argo debt).
**FIX:** SUPERSEDE by 0515 (superseded-on-cutover); write the edge (SUPERSESSION-COMPLETENESS P4).

**C-20 — Phantom-edge epidemic.** `ADR-0476:9` `supersedes:[ADR-0421]` (ABSENT); `ADR-0482:54`
"oya-vcs (ADR-0409)" (ABSENT). Verified ABSENT via `ls`: 0409, 0411, 0416, 0421, 0428, 0429, 0434,
0443, 0457, 0397, 0477, 0483, 0484, 0088, 0012. Affects 0476/0478/0479/0480/0481/0482/0508/0069.
**FIX:** repoint/renumber every phantom id; add a no-dangling-ref invariant to the integrity gate (D11/D13).

**C-21 — Identity dual-bridge clash.** `ADR-0476:18` "Supersedes ADR-0421 (Keycloak)" (phantom; real IdP
= 0187 Zitadel); `:103` rejects Zitadel (inverting D5). Two Accepted IdP ADRs name DIFFERENT vendored
bridges (Zitadel vs Keycloak), no 0187↔0476 edge.
**FIX:** AMEND 0476 — `supersedes:[0421]`→`[0187]`; flip Zitadel rejected→adopted Phase-1 bridge; the
canonical long-term endpoint is bespoke oya-identity (canon-aligned). Pairs with C-13.

### HIGH (20)

**H-1 — KR minor-age contradiction.** `ADR-0008:69` `Minor // <14 KR` vs `ADR-0034:94` "under 18 in KR".
Both Accepted, same overlay, regulator-facing. **FIX:** reconcile (KR PIPA Art 22(6) child-consent <14
vs youth-protection "under 18" — disambiguate).

**H-2 — Dangling ADR-0012.** `ADR-0008:197` "→ ADR-0012"; absent. **FIX:** repoint/remove (D11(c)).

**H-3 — Advisory-not-enforced codified.** `ADR-0011:146` "`oya-check-contracts` is an advisory P0 lane
reference until the crate exists." **FIX:** wire real enforcement or relabel honestly (D-DOCTRINE).

**H-4 — ADR-0017 GitHub slug permanent vs D2.** `ADR-0017:26,40-41` retain `jason931225/oyatie` as
permanent ("filesystem migration cost exceeds brand purity"). Canon D2/D-FORGE-CLARIFY (GitHub-interim →
bespoke Sapling `cloud/cloud-scm`). **FIX:** add the GitHub-interim→bespoke ratchet.

**H-5 — ADR-0006 self-rename tautology.** `:11` `"Ontology" renamed to "Ontology"`; `:22` same.
**FIX:** repair the rename text (D11(b)); ALSO add the effective-dating temporal type the keystone needs
(see hyperscaler U-2). [CRITICAL vs D-D1 in lane 0; folded here as integrity + under-spec.]

**H-6 — ADR-0029 12-app parity, no sequencing gate.** `ADR-0029:38-53,154` full Workspace/M365 suite as
one M03 deliverable. **FIX:** in-scope but SEQUENCE per D8/D9 (M0 evidence-gate), don't present as a single
milestone.

**H-7 — ADR-0030 from-scratch search engine as flat µservice.** `ADR-0030:32-96` crawler/render-farm/
inverted+vector index/KR morphology/RTBF/KG, no M0 gate. **FIX:** sequence as its own vertical with an
M0 evidence-gate (D8/D9); KEEP the KR-first morphology moat.

**H-8 — Dead `foundry` context pervasive.** `ADR-0062:130`, `ADR-0091:41`, `ADR-0067:97`,
`ADR-0069:10`/`0065:10`/`0066:10` (owner `axis-foundry`), `ADR-0200:68` (`foundry-tool` sandbox class),
`ADR-0258:99` (`oya.foundry.v2.CapabilityService`). **FIX:** batch foundry→intelligence/governance
rename (831-file sweep, canon WF2); the 0200 sandbox-class enum touches code — coordinate carefully.

**H-9 — GitHub-Actions-as-canonical CI + MASTERPLAN-inverts-D1.** `ADR-0063:185`, `ADR-0066:53`,
`ADR-0067:64` hard-wire `.github/workflows`/`gh api`. ALSO `ADR-0063:95` reads `MASTERPLAN.md §2.1` as the
planned-set source — inverts D1 (ADRs SSOT; masterplan generated, `canon:9-10`). **FIX:** GH-Actions→oya-ci
lane runner; read ADR front-matter not masterplan.

**H-10 — ADR-0069 phantom-0088 + wrong filenames.** `:12,174` "ADR-0088 (foundry scaffolding)" (ABSENT);
`:172,173` wrong filenames for 0056/0067. **FIX:** repair edges (D11(c)).

**H-11 — ADR-0067 mega-service.** `:159` "~20 BCs × 5-7 layer crates = ~100-140 crates"; `:139` subsumes
0065/0066. Contradicts flat-catalog single-concept + D-PURESPLIT. **FIX:** re-shape `ops` into
properly-bounded services under the pure-split.

**H-12 — ADR-0062 day-1 hyperscale mandate.** `:18` "100M+ user scale … mandatory from day one. No
single-instance-only designs." Tension with D8/D-SEQ M0-gated sequencing. **FIX:** reframe "from day one"
→ "M0-gated, scale-on-proven-demand"; keep the CI-enforced-quality spine.

**H-13 — ADR-0098 accepts power-loss data-loss to avoid one dep.** `:71-78,184` documented non-durability
to avoid `rustix`. **FIX:** flip durability default to `fsync(parent_dir)`; "zero net-new deps" as a hard
goal is metric-worship — `rustix` is a vetted foundational dep.

**H-14 — ADR-0119 dangling back-edge.** `ADR-0131:10` partial-supersedes 0119 but `ADR-0119:8`
`superseded_by:[]`. **FIX:** add the back-edge (D11; SUPERSESSION-COMPLETENESS).

**H-15 — ADR-0123 dead-vcs forward-authority refs.** `:53,66` "HG-VCS"/`oya-vcs-admission` post-0363.
Content (maturity-claim gate) is exemplary → KEEP-with-amend. **FIX:** repoint vcs refs to oya-ci/governance.

**H-16 — ADR-0173 stale vendor-doctrine SSOT.** `:163-171` Forgejo/Woodpecker + "Foundry VCS (ADR-0113)";
`:187` "Kafka or NATS"; `:199` "OpenFeature + Flipt". The own-the-stack doctrine is exemplary but it is
the SSOT others cite → high-leverage. **FIX:** keep the doctrine, fix every stale pick (Forgejo→oya-ci,
Kafka/NATS→Pulsar, reconcile Flipt-vs-0159, foundry→intelligence, 0113-vcs-retired).

**H-17 — ADR-0159 vs ADR-0173 feature-flag clash.** `ADR-0159:42` owned `feature-flags` µservice vs
`ADR-0173:199` "OpenFeature + Flipt". **FIX:** reconcile to one (owned OpenFeature server).

**H-18 — Redis live vs Valkey ruling.** `ADR-0184:101-105` Redis REJECTED for Valkey; yet `ADR-0191:46,68`
+ `ADR-0208:74` "Redis". Canon D12. **FIX:** AMEND 0191/0208 → Valkey.

**H-19 — Kafka-as-broker option.** `ADR-0192:58` "Pulsar 4.2 or Kafka"; `ADR-0195:15,19,67` "Kafka Engine"
default (substrate reconciled to Pulsar at `:69-71`); `ADR-0377-kafka:22` cites phantom `ADR-0397`.
Canon D-EVENT Pulsar-only. **FIX:** name Pulsar (Kafka = wire-compat only); fix the phantom-0397 edge.

**H-20 — `microservices/` flat-catalog path residue (fleet-wide).** `ADR-0184:144`, `0196:68`, `0202:75`,
`0209:126`, `0143:89`, `0238:305`, + every path-bearing Accepted ADR in 0133–0335. Canon D-PURESPLIT.
**FIX:** bulk additive path rename to `{oya,cloud}/<service>/` (mechanical, D13-AMENDED).

**H-21 — Kyverno hard-wired citing Superseded 0183.** `ADR-0183` now Superseded (`superseded_by:[ADR-0379]`);
`ADR-0338:229-233`, `ADR-0117:25-27` still hard-wire Kyverno citing 0183 as live. **FIX:** repoint to 0379
(Kubewarden default).

**REFUTED (carried, not dropped):**
- **R-1** (lane5 H5) — "0335 keeps a `vcs-orchestrator` µservice." FULL REFUTE: `ADR-0335:224-227` says
  "the principal namespace persists, **the µservice does not**." Residual LOW: the `oyatie.foundry.*`
  principal-namespace naming is a real but separate integrity nit.
- **R-2** (lane8 X6) — "Forgejo in **15** Accepted ADRs." COUNT REFUTE: `grep -lri forgejo` = 27 files
  (all statuses); the "15 Accepted" figure is unverified. Substance (Forgejo survives in multiple
  Accepted incl. 0515/0380) HOLDS.
- **R-3** (lane0) — ads-gate SPOF dual-cited to `0028:18,136` + 0031. OVER-BROAD REFUTE: 0028:18 = "Cloud
  is the compute substrate"; no "gate outage" string in 0028. Only the 0031 half is valid (→ H-22 below);
  0028 has its OWN separate same-provider SPOF (→ H-23 below).

---

## SECTION 2 — HYPERSCALER-LENS PROBLEMS (over/under-engineering, wrong own-vs-adopt, SPOFs, ceilings, missing table-stakes)

### SPOFs / availability ceilings
- **H-22 [HIGH] — ADR-0031 ads-gate fleet-wide SPOF.** `ADR-0031:56` "The gate is a singleton";
  `:134` "gate outage = no ads served anywhere." Hyperscalers (Google Ads) get the same policy guarantee
  via a replicated/sharded gate with a consistent policy snapshot, NOT a literal singleton.
  **FIX:** "logically-single policy authority, physically-replicated serving path."
- **H-23 [HIGH] — ADR-0028 same-provider primary+secondary SPOF.** `ADR-0028:36` "primary OCI KR-Seoul1;
  secondary OCI KR-Chuncheon; fail-open AWS." Provider-level OCI failure takes out both; only fail-open is
  cross-provider. **FIX:** genuine multi-provider / multi-account control-plane isolation.
- **[MED] — ADR-0029 shared CRDT runtime concentration.** `:64-69,155` one Yrs CRDT runtime under all of
  Docs/Sheets/Slides/Notes/Sites — concentrated failure domain. **FIX:** isolate per-surface or shard.
- **[MED] — ADR-0067 mega-service blast radius.** `:160` self-flags "ops.oyatie.com outage takes down the
  operations surface." **FIX:** decompose (pairs with H-11).

### Under-engineering / missing table-stakes
- **U-1 [HIGH] — ADR-0098 accepts silent power-loss data-loss** to save one vetted dep (= H-13). A
  hyperscaler never trades durability for a dep-count vanity metric in a queue/DLQ path.
- **U-2 [HIGH] — ADR-0006 missing effective-dating + read-your-writes.** `ADR-0006:50-64` Entity model
  has `schema_version` but no temporal/effective-dating type and no consistency-token contract — the very
  property canon D-D1-TOPOLOGY makes the keystone (`canon:190`, payroll-close read-your-writes test).
  **FIX:** add the effective-dating kernel temporal type + consistency token (net-new kernel build).
- **[MED] — ADR-0184 "None planned" for owned Postgres** (`:169`) vs D4 owned-data-tier endpoint.
  D4 permits vendored-until-proven, so the literal "None planned" is the drift. **FIX:** tag transitional.

### Over-engineering / premature complexity (pre-revenue, pre-M0)
- **[HIGH] — ADR-0062 day-1 100M-scale mandate** (= H-12). **[HIGH] — ADR-0030 from-scratch web search**
  as a catalog µservice (= H-7). **[HIGH] — ADR-0029 full Workspace/M365 parity** as one milestone (= H-6).
- **[HIGH] — ADR-0067 mega-service** 100-140 crates (= H-11). **[HIGH] — ADR-0482 unbounded ambition.**
  `ADR-0482:114-117` "Any upfront investment is acceptable; timeline is the only constraint" (~30 parallel
  bespoke tracks) vs D8/D-SEQ capacity budget. **FIX:** insert the D8 capacity-budget gate (oya-ci-prioritized,
  sequenced). **[HIGH] — ADR-0508 owned-silicon** (`:79-92,143-152` OpenTitan SoC + manufacturing line) —
  bind to D8/D-SEQ M0 sequencing so it cannot be pulled forward.
- **[MED] — ADR-0109 "automation cost ≈ 0"** unverified claim (`:33,179-188`) — robust-not-false risk.
- **[MED] — ADR-0066/0067** ~15-extractor live-introspection daemon + nightly-rustdoc-JSON dep before
  first tenant. **FIX:** sequence under M0.

### Wrong own-vs-adopt
- **[CRITICAL] — ADR-0202 ArgoCD-as-owned-engine** (= C-15): adopt the *pattern*, reimplement in Rust.
- **[HIGH] — ADR-0053/0103 adopt fragile niche `grit` as a HARD mandate** with a known upstream blocker
  (`ADR-0053:103`) — reinvent-plain-git in reverse; SUPERSEDED (= C-6).
- **[MED] — ADR-0011 commits generated 4-language SDKs into the tree** (`:106`); hyperscalers generate
  SDKs as CI artifacts. **FIX:** build-time generation.
- **[GOOD own-vs-adopt to KEEP]:** ADR-0139 ("adopt OSS leaders, own the differentiator"), ADR-0173
  (own-the-stack ratchet doctrine), ADR-0200 ("own the integration layer, not the runtime"), ADR-0506
  (mandatory feature-parity table = the bespoke-doctrine artifact done best), ADR-0209 (compliance-evidence
  100% in-house "because this IS the differentiation").

---

## SECTION 3 — ANTI-PATTERNS TO FIX · EXEMPLARY PATTERNS TO KEEP

### Anti-patterns (systemic — drive the amendment lanes)
1. **Incomplete supersession sets** (the dominant failure mode): a unifying ADR supersedes SOME of a
   cluster, leaving co-equal Accepted siblings live (CI/CD X1, identity X2, foundry X3, admission X4,
   progressive-delivery X7). → owned by `00-SUPERSESSION-COMPLETENESS.md` (P1–P5 + §2A).
2. **Dangling/phantom edges** (0421/0409/0397/0088/0012/0150-as-policy; 0331/0069/0476 filenames) — no
   no-dangling-ref invariant is enforced.
3. **Bridge-vs-ratified encoded in prose, not the graph** — the status enum has no
   "superseded-on-cutover / bridge-unratified" value, so Jenkins/Forgejo/Flagger/ArgoCD read as fully
   ratified canon (canon:166,177-184). **The single highest-leverage structural fix.**
4. **Enforcement façade** — "binding spec, advisory enforcement" / advisory-lane-claiming-blocking
   (0128, 0011:146, un-wired `oya-governance-*`). → make enforcement REAL first (canon firewall).
5. **Un-propagated vendor rulings** — Forgejo/Kafka/Flagger/Kyverno/Redis still named live; rulings live
   only in the canon doc, never written into the Accepted corpus.
6. **Day-1-everything vs M0-gated sequencing** (0062/0029/0030/0067/0482/0508).
7. **Forbidden-term leakage as architecture** (foundry namespace; flat-`crates/`; `microservices/`).
8. **ADR-inflation / executed-migration-left-Accepted** (0097 3-crate cosmetic rename; 0057 one-shot
   cutover; 0101 "temporary bypass" promoted to architecture).

### Exemplary patterns to KEEP (cited, verbatim-worthy)
- **Honest-claim / no-false-green enforcement (the antithesis of the façade):** `ADR-0129` (real
  `enforcement_status: active` RED/GREEN-proven gate, `:30-33`), `ADR-0135` (aspirational-enforcement
  gate, blocks active-claims when the check crate is absent), `ADR-0123` (maturity-claim gate: the phrase
  "we are hyperscaler mature" is forbidden without fresh evidence), `ADR-0104` (no-stub build-graph
  reachability), `ADR-0093` (documentation-as-test honest-naming), `ADR-0204` (honest perf-wall disclosure).
- **Seam / own-the-endpoint ratchet (D-META charter):** `ADR-0173` (own-the-stack doctrine SSOT —
  elevate as charter), `ADR-0142` (CRDT port + ≥2 impls + CI-compile-the-alternates), `ADR-0064`
  (seam>adapter>pack trichotomy + canonical-base-neutrality CI gate), `ADR-0482` (bridge-discipline
  checklist + `bespoke_replacement_planned` convention), `ADR-0506` (mandatory per-primitive feature-parity
  table), `ADR-0197` (`BackupExecutor` trait seam + maintainer-transition hedge).
- **Type-safety / boundary discipline:** `ADR-0095` (`TenantId`/`TenantSlug` confused-deputy split),
  `ADR-0083` (typed-error tiers), `ADR-0094` (Handler associated `Error`), `ADR-0056` (closed layer enum +
  mechanically-enforced inward-only deps — Google/Meta monorepo doctrine).
- **Structure / monorepo doctrine:** `ADR-0512` (vertical-slice monorepo, crate=bounded-context, Buck2,
  hard no-concurrent-migration rule — the canonical, founder-locked governing ADR), `ADR-0131`/`ADR-0132`
  (per-service flat layout + no-grouping, already pure-split-aligned), `ADR-0362` (grouping-retirement
  robust-not-false), `ADR-0115` (no-parallel-canonical-trees hygiene).
- **Architecture altitude:** `ADR-0145` (ESB-2.0 → 3-invariants reshape), `ADR-0172` (narrow CQRS, not
  event-sourcing-everywhere), `ADR-0148`/`ADR-0182` (one-concern-per-layer mesh/gateway separation),
  `ADR-0191` ("If the answer is both, the design is wrong" boundary table), `ADR-0158` (per-µservice DR
  disposition, rejects mandatory-active-active as "engineering malpractice"), `ADR-0376` (Kamaji-vs-Gardener
  founder-challenge — the adr-challenge gate working).
- **Sovereign / regulated table-stakes (serve D9/D-KR):** `ADR-0240` (sovereign per-pack overlay),
  `ADR-0164` (air-gap pack overlay), `ADR-0241` (DR tiers with provable drill receipts), `ADR-0034`
  (structurally-impossible-by-construction data-flow guardrails), `ADR-0008` (orthogonal subject_class +
  purpose-permission matrix — beyond AWS/GCP defaults).
- **Integrity discipline done right:** `ADR-0179`/`ADR-0180` (honest `renumber_note`, no-id-reuse),
  `ADR-0393` (clean-supersede-not-nth-amendment), `ADR-0118` ("an always-empty runner would be false
  mechanical confidence").

---

## SECTION 4 — PER-ADR DISPOSITION TABLE (every Accepted ADR audited)

> KEEP = substance sound, refs/path sweep only · AMEND = substance largely sound, named/integrity/canon
> fixes · SUPERSEDE = overruled by canon (salvage the pattern, write the edge, flip status) · ARCHIVE =
> target/job moot (reuse template). "(minor)" = path/vocab/citation touch-ups only.

| ADR | Disp | One-line why |
|---|---|---|
| 0001 | AMEND | foundry substrate owners + flat-catalog vs pure-split + Proposed-0007 dep |
| 0006 | AMEND | missing effective-dating/consistency-token (D-D1 keystone) + self-rename tautology |
| 0008 | KEEP | strongest privacy ADR; reconcile KR minor-age vs 0034; dangling 0012 |
| 0011 | AMEND | foundry owner + flat-crates paths + advisory-gate-claiming-enforced |
| 0015 | SUPERSEDE | forbidden flat-`crates/` + forbidden-vocab enum; flip status under 0512/0131 |
| 0017 | AMEND | GitHub slug framed permanent vs D2 bespoke-SCM ratchet |
| 0018 | AMEND | glossary canonicalizes forbidden "Foundry"; behind canon D12 vocab |
| 0028 | AMEND | same-provider primary+secondary SPOF; cloud-as-flat-peer vs D-LAYER |
| 0029 | AMEND | full Workspace/M365 parity as one milestone (no M0 gate); subject-less prose |
| 0030 | AMEND | from-scratch search engine as catalog µservice (no M0 gate) |
| 0031 | AMEND | literal-singleton ads-gate fleet-wide SPOF |
| 0034 | KEEP | exemplary data-flow guardrails; reconcile KR minor-age vs 0008 |
| 0051 | KEEP | minor: `W-Foundry-Preview` wave label + "14 verticals" count |
| 0053 | SUPERSEDE | dead grit/icm toolchain mandated live; by 0116/0363/0515 |
| 0055 | KEEP | clean Ontology rename; the fix is on 0057's edge, add disambiguator |
| 0056 | AMEND | flat-crate grammar + `public_layers` cross-import vs pure-split; 12-vs-13 layer reconcile |
| 0057 | ARCHIVE | executed one-shot cutover; dangling+colliding supersedes to phantom 0055-v3 |
| 0058 | AMEND | flat-`crates/` catalog topology vs D-PURESPLIT; foundry first in list |
| 0059 | AMEND | Kafka-outbox on critical path vs D-EVENT/D-D1 |
| 0060 | KEEP | Bominal-inheritance ledger; amend inherited Kafka row + retired-0140 row |
| 0061 | KEEP | minor: Redis→Valkey; topology; soften day-1-100M claim |
| 0062 | AMEND | day-1-hyperscale mandate + Kafka-as-benchmark + foundry |
| 0063 | AMEND | GitHub-Actions CI + MASTERPLAN-as-planned-set (inverts D1) + axis-foundry |
| 0064 | KEEP | exemplary seam/pack trichotomy; topology-path reconcile only |
| 0065 | AMEND | foundry owner + adds `docs` µservice that 0067 immediately renames |
| 0066 | AMEND | GitHub-Actions + grit/ICM live data sources + 15-extractor premature build |
| 0067 | AMEND | `ops` mega-service (100-140 crates) + GH-Actions/grit + foundry |
| 0069 | AMEND | phantom ADR-0088 + wrong filenames; foundry owner |
| 0083 | KEEP | exemplary typed-error tiers; verify `oya-governance-error-boundary` is real |
| 0090 | KEEP | head Decision contradicts its own live Amendment; mark head superseded-in-place |
| 0091 | AMEND | `oya-foundry-write-gate-*` → governance; Kafka→Pulsar |
| 0092 | KEEP | seam policy; reconcile one-workspace to pure-split one-version-root |
| 0093 | KEEP | model honest-naming ADR; no canon conflict |
| 0094 | KEEP | typed Handler trait; no canon conflict |
| 0095 | KEEP | model boundary-type ADR; no canon conflict |
| 0096 | AMEND | foundry-supervisor (dead context); salvage the Rust-vs-Node principle |
| 0097 | ARCHIVE | 3-crate cosmetic foundry rename; subsumed by foundry sweep |
| 0098 | AMEND | foundry-supervisor + accepts power-loss data-loss to avoid `rustix` |
| 0099 | AMEND | Accepted-on-Proposed-0022 + foundry-home for autonomy gate vs D16/D6 |
| 0100 | AMEND | foundry-supervisor; extract the zero-surface-change doctrine |
| 0101 | ARCHIVE | "temporary bypass" shortcut promoted to architecture; foundry |
| 0102 | ARCHIVE | foundry settings render; salvage atomic-render+sref pattern |
| 0103 | SUPERSEDE | grit/icm sanctioned-VCS + ban-git; by 0116/0363 |
| 0104 | KEEP | exemplary no-stub build-graph reachability; amend dead-name examples |
| 0105 | KEEP | 13-layer enum BNF canon; amend broken math line + legacy role residue |
| 0106 | KEEP | `application`→`usecase` rename; close the 5 orphan crates |
| 0108 | KEEP | sunset lifecycle schema; amend foundry BNF gloss |
| 0109 | KEEP | lifecycle-automation; require RED/GREEN per lane before "enforced" |
| 0115 | KEEP | flat-singular registry consolidation; clean |
| 0116 | AMEND | keep grit/icm retirement; replace Foundry-pipeline mapping; Proposed-0111 dep |
| 0117 | KEEP | repo hygiene; amend `oya-vcs-admission` refs + Kyverno→0379 |
| 0118 | KEEP | exemplary anti-false-enforcement; reword Foundry-pipeline rationale |
| 0119 | AMEND | per-product-specs partially superseded by 0131; missing back-edge |
| 0122 | KEEP | Ontology crate rename; fix enum ref + Bominal cross-refs |
| 0123 | KEEP | exemplary maturity-claim gate; amend dead vcs refs |
| 0128 | KEEP | exemplary invariant catalog; enforcement is vapor — wire it or relabel |
| 0129 | KEEP | gold-standard active RED/GREEN honest-claims gate |
| 0130 | KEEP | KG→Ontology consolidation; add canonical YAML front-matter |
| 0131 | KEEP | per-service flat layout, already pure-split-amended; verify 0119 back-edge |
| 0132 | KEEP | no-grouping forward-policy; amend `microservices/`→`{oya,cloud}` |
| 0133 | AMEND | conformance program; retarget Axis-2 to pure-split; Argo/Flagger reconcile |
| 0135 | KEEP | aspirational-enforcement gate (real, landed) |
| 0136 | SUPERSEDE | foundry-as-µservice → cloud-intelligence framework (salvage 6-BC reasoning) |
| 0137 | SUPERSEDE | foundry bounded contexts → intelligence-framework successor |
| 0138 | ARCHIVE | foundry Strangler to a dead address; reuse the template |
| 0139 | AMEND | resolve ledger self-contradiction; de-foundry; thread to oya-ci/cd |
| 0142 | KEEP | exemplary CRDT port + ≥2-impls compile gate; fix mislabeled 0135 edge |
| 0143 | SUPERSEDE | foundry per-BC release pointer → intelligence-framework successor |
| 0144 | AMEND | EU-AI-Act 5-tier (exemplary); fix wrong 0140-Cedar edge; tier namespacing |
| 0145 | KEEP | exemplary ESB-reshape; amend Cedar→PARC refs |
| 0146 | AMEND | distroless base (sound); mechanical foundry/path/count sweep |
| 0148 | AMEND | exemplary mesh layering; FIX phantom-0150→0183 (HIGH); PARC split |
| 0149 | KEEP | idempotency keys table-stakes; mechanical sweep |
| 0150 | KEEP | cursor pagination (the genuine 0150); fix every doc that cites it as Cedar |
| 0151 | KEEP | X-Request-Id propagation; de-foundry decider |
| 0152 | AMEND | RPO/RTO; `dr_tier` namespacing + path |
| 0153 | AMEND | outbox; scope to notification/fan-out per D-D1; Pulsar |
| 0154 | AMEND | event-schema versioning; Pulsar channel list; reconcile registry vs 0166 |
| 0155 | KEEP | per-tenant quotas; path only |
| 0156 | KEEP | PII registry; path only |
| 0157 | AMEND | API gateway tier; reconcile Envoy-Gateway version with 0182; PARC split |
| 0158 | AMEND | exemplary per-µservice DR; re-home foundry row → intelligence |
| 0159 | AMEND | feature-flag µservice; reconcile owned-vs-Flipt with 0173 |
| 0160 | SUPERSEDE | Flagger vs D10 Argo Rollouts; write the edge (P1); fix 0124 ref |
| 0161 | AMEND | CSI/StorageClass; `storage_tier` namespacing; drop foundry edge |
| 0162 | AMEND | per-tenant audit-chain slicing; PARC split; drop foundry |
| 0163 | AMEND | environment tiers→stages (D12); de-foundry; PARC |
| 0164 | AMEND | sovereign/air-gap (STRONG, keep doctrine); re-home inference foundry→cloud-intelligence |
| 0165 | KEEP | Chaos Mesh (founder-endorsed D10); add port/ratchet framing |
| 0166 | AMEND | schema registry Apicurio; add port + ratchet; flag JVM sprawl |
| 0167 | KEEP | tenant CLI `oya`; rename `oya foundry` verb → intelligence |
| 0168 | KEEP | public status page; rename foundry component |
| 0169 | AMEND | webhook DLQ (exemplary); Pulsar; de-foundry; Rust term |
| 0171 | AMEND | multi-cluster federation; ArgoCD-as-bridge framing; parameterize repoURL |
| 0172 | KEEP | exemplary narrow-CQRS; fix 0141-superseded edge |
| 0173 | AMEND | own-the-stack doctrine SSOT (KEEP doctrine); fix all stale picks (HIGH leverage) |
| 0174 | AMEND | FinOps chargeback; re-home capability-cost foundry→cloud-intelligence |
| 0175 | AMEND | tenant-lifecycle saga; confirm 0222 ratification; reconcile D-D1 |
| 0176 | AMEND | brown-out API (exemplary); drop stale 0044 mesh edge |
| 0177 | AMEND | internal/external API split; drop stale 0044 edge |
| 0178 | AMEND | layered throttling (exemplary); drop 0044 edge; `tenant_class` |
| 0179 | KEEP | pgcat (D4 bridge); exemplary honest `renumber_note` |
| 0180 | KEEP | SLO composition (exemplary); retarget Flagger→Argo-Rollouts |
| 0181 | AMEND | image promotion; foundry-pipeline/GH-Actions→oya-ci; Flagger→Argo |
| 0182 | AMEND | exemplary gateway/mesh split; FIX phantom-0150→0183 (HIGH); version reconcile |
| 0184 | AMEND | storage tiers as destination vs D4 owned-endpoint; "None planned" Postgres |
| 0185 | KEEP | client stack; minor path + D8 sequencing note |
| 0186 | KEEP | LGTM observability (exemplary); drop "32"; path |
| 0187 | SUPERSEDE | Zitadel-canonical vs D5; demote-as-endpoint, `superseded_by:[0476]` (P2) |
| 0188 | KEEP | passkey/WebAuthn; downstream RP-home reword once 0187 demotes |
| 0189 | KEEP | step-up ACR (exemplary closed enum); drop foundry; PARC ref |
| 0190 | KEEP | SCIM 2.0; clean |
| 0191 | AMEND | exemplary boundary table; Redis→Valkey; Cedar→PARC |
| 0192 | AMEND | Milvus owned by dead foundry + wrong layer (data-tier D4); Pulsar |
| 0193 | AMEND | ClickHouse OLAP; Pulsar naming; D4 ratchet; path |
| 0194 | KEEP | TimescaleDB (license-fence exemplary); soften "Postgres replacement out of scope forever" |
| 0195 | KEEP | stream processing (exemplary rubric); Pulsar-naming on Kafka-Engine default |
| 0196 | KEEP | SeaweedFS object store; path; tag D4 bridge |
| 0197 | KEEP | backup 3-prong (exemplary seam + maintainer hedge); path |
| 0198 | KEEP | Karpenter autoscaling (exemplary own-vs-buy altitude); path |
| 0199 | KEEP | FinOps OpenCost/FOCUS; path |
| 0200 | KEEP | Wasmtime (exemplary); foundry-naming in sandbox-class enum (coordinate w/ code) |
| 0201 | KEEP | email adapter (no-silent-failure); domain re-tag; path |
| 0202 | AMEND | ArgoCD-as-canonical-engine vs D3/D-CICD oya-cd (CRITICAL); keep Tier-B/C |
| 0203 | AMEND | doc-engine 3-tier; reconcile with D-DOCORG doc-organization |
| 0204 | KEEP | canvas (exemplary honest perf-wall); path |
| 0205 | KEEP | CodeMirror 6; cosmetic foundry word |
| 0206 | KEEP | i18n Fluent/ICU; clean |
| 0207 | KEEP | a11y WCAG 2.2; clean |
| 0208 | AMEND | realtime transport (good design); Redis→Valkey |
| 0209 | KEEP | compliance-evidence in-house (correct own-the-moat); domain tag; path |
| 0210 | KEEP | OTel tail-sampling; drop "32"; citation cleanup |
| 0222 | KEEP | saga portfolio; path + drop foundry axis example |
| 0223 | KEEP | `oya git` drop-in (forge-agnostic by construction); none |
| 0234 | AMEND | stale Connect topology; add edges to 0334 + Wave-15 retirements |
| 0235 | KEEP | Connect core contracts; minor ref refresh |
| 0237 | KEEP+AMEND | Strangler pattern KEEP; re-derive trigger to post-merge set; fix paths |
| 0238 | AMEND | super-app table self-inconsistent vs its own verify block; reconcile to Wave-15 |
| 0239 | SUPERSEDE | mark historical by 0335 (no banner); fix phantom-0150 (HIGH integrity) |
| 0240 | KEEP | sovereign per-pack (exemplary); path + allow-list `_TIER1` labels |
| 0241 | KEEP | DR tiers + drill receipts (exemplary); refresh foundry T1 example |
| 0258 | KEEP | versioning model (GA-class); amend `oya.foundry.*` mesh package → intelligence |
| 0329 | KEEP | tier-retirement allow-list zero-residue lane; refresh foundry refs + absolute paths |
| 0330 | KEEP | tenant-class billing; fix dangling companion-doc filenames + absolute paths |
| 0331 | KEEP | tenant_class adoption template; fix dangling related-filenames (HIGH integrity) |
| 0332 | KEEP | healthcare decomposition; fix stale 0316 edge; add D-SAFETY edge |
| 0333 | KEEP | cell-µservice retired→pattern; fix 0263 dangling edge |
| 0334 | KEEP | shorts→social; path (real debt is on 0237/0238) |
| 0335 | AMEND | name cloud-intelligence endpoint; 3-way foundry-fitness→governance carve-out; vcs-orchestrator reconcile; supersession banners on 0220/0239 |
| 0350 | KEEP | UUIDv7; sweep 0150 dangling-ref + foundry vocab |
| 0351 | KEEP+AMEND | cell-rebalancer/lifecycle; foundry-principal rename; reconcile durable-workflow |
| 0362 | KEEP | grouping-retirement (robust-not-false exemplar) |
| 0363 | AMEND | false "eradicated" claim + stale Forgejo; fix claim; Forgejo→GitHub-interim |
| 0364 | KEEP | generative ADR template / masterplan-from-ADR keystone; cite D13-AMENDED deferral |
| 0365 | KEEP+AMEND | automated ADR lifecycle; §4 substrate → oya-ci/GitHub-interim |
| 0366 | AMEND | agentic pipeline; substrate refs → oya-ci (doctrine feeds oya-ci product) |
| 0367 | AMEND | trustless pre-merge gateway (keystone D3); trusted-runner=oya-ci, auto-merge=Tide |
| 0368 | KEEP | self-governing north-star; fix "becomes ADR-0000" lines (D13-AMENDED deferral) |
| 0369 | AMEND | stacked-trunk change-flow; re-target Forgejo→GitHub-interim→oya-ci Tide |
| 0370 | AMEND | headline falsified by own verification; add supersession/reconcile edges to 0378 |
| 0371 | KEEP | Cloudflare Tunnel control-plane; note Cloudflare-on-critical-path vs own-edge ratchet |
| 0373 | KEEP | cloud-intelligence gateway design; clean |
| 0374 | SUPERSEDE | Jenkins-as-orchestrator + Forgejo vs D-CICD; by 0515 superseded-on-cutover (P3) |
| 0375 | KEEP | Talos+ClusterAPI+ArgoCD substrate (exemplar); drop "flips to Forgejo" |
| 0376 | KEEP | managed-K8s product (strongest hyperscaler reasoning in band) |
| 0377-kafka | KEEP+AMEND | Kafka→Pulsar-via-KoP (keep); renumber the duplicate-0377 id collision; fix phantom-0397 |
| 0378 | AMEND | local substrate vfkit+Talos; substrate refs → oya-ci; supersession edges to 0370 |
| 0379 | KEEP | Kubewarden default admission (supersedes 0183); clean |
| 0380 | SUPERSEDE | rebuild-Jenkins+Forgejo vs D-EXEC; by 0515 superseded-on-cutover (P4) |
| 0383 | KEEP | LGTM observability reconciliation; Jenkins cite → bridge; tag transitional |
| 0388 | AMEND | doc-axis convention; reconcile to single D-DOCORG Diátaxis topology |
| 0389 | KEEP+AMEND | cloud-intelligence Bedrock-on-Talos; fold D-INTEL FINAL engine-relocation |
| 0390 | KEEP | cloud-intelligence v1 + proof layer; "S3" → SeaweedFS-own-store wording |
| 0391 | KEEP+AMEND | N-lane safety proof + console; data sources Forgejo/Jenkins → oya-ci API |
| 0393 | AMEND | Leptos app-shell (clean-supersede exemplar); stale 0513→0515; Proposed-0394 dep |
| 0476 | AMEND | `supersedes:[0421]`→`[0187]`; Zitadel rejected→adopted bridge; Forgejo; phantom 0409 |
| 0478 | AMEND | phantom `supersedes:[0457]`; `microservices/` home; 0509→0512 |
| 0479 | AMEND | phantom `supersedes:[0429]`; home; ClickHouse-as-endpoint vs D4 |
| 0480 | AMEND | phantom `supersedes:[0443]`; pressure-test oya-cost-vs-oya-meter-subsystem (D8) |
| 0481 | AMEND | phantom `amends:[0428]`+0409/0434; Forgejo; Proposed-0408 BLOCKER dep |
| 0482 | AMEND | keystone bespoke-doctrine (KEEP); phantom Tier-1 rows; Forgejo; insert D8 capacity gate |
| 0506 | KEEP | aws-lc-rs crypto + parity table (best); de-hardcode absolute path |
| 0507 | KEEP | webauthn-rs RP; amend stale 7-crate scaffold → single-crate/BC per 0512 |
| 0508 | KEEP | OpenSK authenticator; fix dangling 0483/0484; bind silicon ambition to D8/D-SEQ |
| 0509 | SUPERSEDE | status drift — 0512 supersedes it but still Accepted; repoint 5 citers (P/§2A) |
| 0512 | KEEP | best-engineered, founder-locked governing ADR; watch Proposed-0392/0408 dep |
| 0515 | AMEND | ratified CI/CD canon; eradicate Forgejo (`:76,96`); ratify/gate Proposed-0408/0392 |

**Disposition totals (reconciled across lanes, 169 unique Accepted ids):**
- **KEEP** (incl. KEEP+minor-AMEND): ~108
- **AMEND:** ~50
- **SUPERSEDE:** 9 — 0015, 0053, 0103, 0136, 0137, 0143, 0160, 0187, 0509 (+ 0374, 0380, 0239 =
  SUPERSEDE/mark-historical/superseded-on-cutover → **12 if counting the bridge/historical class**)
- **ARCHIVE:** 4 — 0057, 0097, 0101, 0102, + 0138 (foundry Strangler) = **5**

> NOTE on SUPERSEDE/ARCHIVE accounting: lanes used slightly different conventions for the
> "superseded-on-cutover" (0374/0380) and "mark-historical" (0239) class — these are bridge/historical
> dispositions, not clean archival, and their EDGES are owned by `00-SUPERSESSION-COMPLETENESS.md` (P3/P4
> + C-17). The strict per-lane clean-SUPERSEDE set is 9; the strict clean-ARCHIVE set is 5.

---

## SECTION 5 — COVERAGE (see top of doc for the grounded table)

- **169 Accepted ADRs on disk; all given a disposition** across lanes 0–8 (lane-overlap reconciled).
- **Deep-read in full:** ~165 (lane 5's 5 large ADRs read to a marked cap; lane 2's 0131 to L160).
- **Cross-ADR-duplication blind spot:** lane 8 deep-read only the 5 named clusters — a duplicate decision
  outside those clusters using none of {kafka,forgejo,istio} could be missed; per-domain lanes cover the
  ADRs individually, so the residual is cross-range duplication, not a missed ADR.
- **All non-Accepted skips enumerated** in per-lane artifacts (Proposed/Superseded/deprecated/Amended).
- **Phantom/absent ids verified by `ls`,** not assumed: 0012, 0033, 0088, 0397, 0409, 0411, 0416, 0421,
  0428, 0429, 0434, 0443, 0457, 0477, 0483, 0484, + the 0055-rename-plan-v3 file.

---

## APPENDIX — the one structural fix that resolves the most contradictions

Adding a **`superseded-on-cutover` / `bridge-unratified`** value to the ADR status enum + a
**no-dangling-ref invariant** to the integrity gate would mechanically resolve anti-patterns 1–3 and
contradictions C-12/C-13/C-16/C-17/C-18/C-19/C-20/C-21 + H-21 at the graph level — i.e. it makes the
Forgejo/Jenkins/Flagger/ArgoCD/Zitadel "bridges" READ as bridges instead of ratified canon, and makes a
phantom edge un-mergeable. This is the firewall-first move the charter targets (`canon:166,177-184`) and
is the same class of fix the Phase-0 firewall (ADR-0515 + the 4 keystone gates) is built to enforce.
