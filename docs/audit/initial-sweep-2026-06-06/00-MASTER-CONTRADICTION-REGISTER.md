# 00 — MASTER CONTRADICTION REGISTER (initial sweep 2026-06-06)

> **READ-ONLY MERGE. No re-audit, no new claims.** This register consolidates, dedupes, and
> cross-references the two completed, adversarially-verified audit artifacts. It cites only what they
> cite; every `path:line` here was opened-and-confirmed in one (or both) of their verification lanes.
> It proposes fixes; it applies nothing. Every mutating fix is gated on founder ratification
> (verify-at-every-step; never mutate on an unverified verdict).
>
> **Source artifacts merged:**
> 1. `accepted-adr-audit/00-ACCEPTED-ADR-AUDIT.md` — 41 confirmed contradictions (21 CRIT + 20 HIGH),
>    ~16 hyperscaler-lens problems, 8 anti-patterns, 169-ADR dispositions, 3 refuted.
>    Verified in `accepted-adr-audit/20-verified.md` (**CONFIRMED 41 · REFUTED 3**).
> 2. `contradiction-audit/00-SUPERSESSION-COMPLETENESS.md` — 12 supersession-completeness
>    contradictions (7 status-vs-edge §2A + 5 directive-without-edge P1–P5) + flat-crates FC-1..5.
>    Verified in `contradiction-audit/20-verified.md` (**all material findings CONFIRMED · 0 substantive refutations**).
>
> **ADR source tree:** `/Users/jasonlee/Developer/source/docs/decisions/` (349 ADR `.md` files).
> **Canon baseline:** `docs/audit/initial-sweep-2026-06-06/synthesis/decision-record-oyatie-canon.md`
> (cited `canon:NN`). **Machine index** `docs/machine-readable/decisions.json` = DRIFTED, NOT trusted.
>
> **Dedup convention.** Where the accepted-adr-audit (which owns the *contradiction* statement) and the
> supersession-completeness audit (which owns the *mechanical edge/status fix*) corroborate the same
> real defect, ONE row is kept here, citing BOTH. The accepted-audit explicitly names the
> supersession-audit as the mechanical-fix owner for C-12/13/17/18/19. The unique-id (e.g. `C-12 = P1`)
> mapping is shown in each such row.

---

## HEADLINE COUNTS

| Bucket | Count |
|---|---|
| **Confirmed contradictions (accepted-audit)** | **41** (21 CRITICAL + 20 HIGH) |
| **Supersession-completeness contradictions (contradiction-audit)** | **12** (7 status-vs-edge + 5 directive-without-edge) + flat-crates FC-1..5 |
| **Deduped distinct contradictions in §1** | **45** (5 overlaps collapsed: C-12=P1, C-13=P2, C-17=P5-adjacent/banner, C-18=P3, C-19=P4; FC-set folded into C-3; foundry-set C-11/C-14 cross-ref H-33) |
| Hyperscaler-lens problems (§2) | ~16 (4 SPOF/ceiling · 3 under-eng · 8 over-eng · 4 wrong own-vs-adopt; 0202 = the CRITICAL) |
| Anti-patterns (§3) | 8 systemic + 1 KEYSTONE FIX |
| Accepted ADRs dispositioned (§4) | **169** unique (KEEP ~108 / AMEND ~50 / SUPERSEDE 9 clean +3 bridge / ARCHIVE 5) |
| Refuted (carried, not dropped) | 3 (R-1 full · R-2/R-3 partial) |

**THE KEYSTONE FIX (one structural move, resolves the most):** add a **`superseded-on-cutover` /
`bridge-unratified`** value to the ADR `status` enum, plus a **no-dangling-ref / supersession-completeness
invariant** enforced by the **cross-artifact-agreement gate**. This makes Forgejo/Jenkins/Flagger/ArgoCD/
Zitadel READ as bridges instead of ratified canon, and makes a phantom edge un-mergeable. It mechanically
resolves **anti-patterns 1–3** and **C-12 / C-13 / C-16 / C-17 / C-18 / C-19 / C-20 / C-21 + H-21** at the
graph level (canon firewall `canon:166,177-184`; same class as the Phase-0 firewall ADR-0515 + 4 keystone gates).

**CRITICAL LIST (21, deduped — full detail in §1):** C-1 Accepted-on-Proposed foundation · C-2 foundry
load-bearing in 4 ADRs · C-3 flat-`crates/` + forbidden-vocab enum (0015; folds FC-1..5) · C-4 ADR-0057
dangling+colliding supersedes · C-5 flat-catalog vs D-PURESPLIT · C-6 dead grit/icm VCS mandated · C-7 Kafka
on critical path · C-8 autonomy-gate Accepted-on-Proposed + foundry-home · C-9 foundry-pipeline + Proposed-0111
· C-10 enforcement façade · C-11 foundry cluster as live µservice canon · **C-12 (=P1) 0160 Flagger vs D10** ·
**C-13 (=P2) 0187 Zitadel vs D5** · C-14 0192 Milvus dead-foundry + wrong layer · **C-15 0202 ArgoCD-as-owned-engine
(also the CRITICAL hyperscaler own-vs-adopt)** · C-16 phantom-0150 epidemic · **C-17 (=banner) 0239 no supersession
banner** · **C-18 (=P3) 0374 Jenkins+Forgejo as ratified canon** · **C-19 (=P4) 0380 rebuilds Jenkins+Forgejo** ·
C-20 phantom-edge epidemic · C-21 identity dual-bridge clash (0476 vs 0187).

---

## SECTION 1 — CONTRADICTIONS (deduped · severity-ranked CRITICAL→HIGH→MED · cited both sides · fix · owning A-lane)

> Each row: **id(s)** · the contradiction · cited `path:line` BOTH sides · the FIX · the owning A-lane.
> A-lane key: **A-STRUCT** flat-crates · **A-INTEGRITY** status-enum / supersession-edges / phantom-refs ·
> **A-CI** 0374/0380/0160/0202 · **A-FOUNDRY** 0335/0136/0137/0143/0099/0192 · **A-IDENTITY** 0187/0421 ·
> **A-DATA** 0059/0091 Kafka-critical-path · **A-TASTE** over-engineering.
> "BOTH" in the source column = corroborated by both artifacts; the unique-id map (`C-NN = P/FC`) is shown.

### CRITICAL (21)

| id(s) | Contradiction | Cited both sides | Fix | A-lane |
|---|---|---|---|---|
| **C-1** | Accepted ADRs built on a PROPOSED foundation (Cedar/tenant/identity/build). | `ADR-0007:3` `status: proposed` + `ADR-0002:3` proposed → load-bearing for `ADR-0001:81-82`, `ADR-0028:68`, `ADR-0034:20`, `ADR-0099:38`(→Proposed `ADR-0022:3`), `ADR-0116:50`(→Proposed `ADR-0111:2`), `ADR-0515:11`(→Proposed `ADR-0408`,`ADR-0392` `:3`). | Ratify 0002/0007/0022/0111/0392/0408 (reconciled to canon D5/D6) OR demote dependents until the foundation lands. D14 Proposed-ledger. | A-INTEGRITY |
| **C-2** | "Foundry" canon-dead but load-bearing in 4 Accepted ADRs — incl. the terminology-canon ADR. | `ADR-0512:57` "`foundry` name remains eradicated" (`canon:204` D-FOUNDRY-CLARIFY) vs `ADR-0001:80-82,87`; `ADR-0011:60,63,67`; `ADR-0017:54`; **`ADR-0018:45` glossary "| Foundry | Oyatie's AI agent runtime + control plane + engineering platform |"**. | AMEND all four (foundry→intelligence/governance/cloud-intelligence per the 3-way route); DELETE the 0018 glossary `Foundry` row. | A-FOUNDRY |
| **C-3** (folds **FC-1..5**) | Forbidden flat-`crates/` LOCATION + forbidden-vocab `<context>` enum (0015) — AND a live BLOCKER gate enforcing the dead depth-3 topology. | `ADR-0512:55-57,62` flat `crates/` "forbidden" / "rejected" vs `ADR-0015:38-39` `crates/oya-<context>-<role>` canonical + `:42` enum incl. `platform,workspace,vertical,foundry,cloud`. Status drift: `ADR-0015:3` `accepted` + `superseded_by:[ADR-0131]` `:5`. Live gate: `governance-lanes/flat-crates.md:7` Accepted, `:20` BLOCKER, `:42-44,:48` `if depth != 3 { … NestedCrate }` (rejects 0512's depth-5); ABSENT from `registry/quality/lanes.yaml`. Untracked code-empty `crates/` dir on disk (`git ls-files crates/`=0). | **(FC-1)** flip `ADR-0015` → `Superseded` (edge already exists). **(FC-2, HIGHEST SEVERITY)** retire `oya-governance-flat-crates`; fold intent into wired `lean-a1-architecture` (`lanes.yaml:485-493`, `validate architecture-boundaries`) per `ADR-0512:62`. **(FC-3)** delete the untracked `crates/` dir. **(FC-4)** sweep ~50 stale LOCATION/gate refs (keep the ~650 surviving-NAME headers). **(FC-5)** add ADR-INDEX rows for 0509/0512; flip `ADR-INDEX.md:37`(0015), `:317`(0357); back-pointer 0509→0512. | A-STRUCT |
| **C-4** | ADR-0057 dangling + colliding supersedes edge (points at a phantom v3 file; collides with real 0055). | `ADR-0057:32` "Supersedes: ADR-0055 (v3-era rename plan)" vs on-disk `ADR-0055-object-graph-renamed-to-ontology.md` (Accepted, different decision); the "v3-era" file is ABSENT (`ls`). | ARCHIVE 0057 (executed one-shot migration); fix/delete the supersedes edge first. D11(c). | A-INTEGRITY |
| **C-5** | Flat-`crates/` catalog canon + cloud cross-import exemption vs D-PURESPLIT two-tree rule. | `ADR-0058:33` "every feature/product is an independent microservice in `[workspace.metadata.oya.microservices]`" + `:144` "All crates flat under `crates/`"; `ADR-0056:242-256` `cloud public_layers` cross-import vs `canon:171-172` D-PURESPLIT "exactly two trees oya/+cloud/, ERADICATE everything else". | AMEND 0056/0058 topology → `{oya,cloud}` (keep à-la-carte product model + GTM-not-arch rule). | A-STRUCT |
| **C-6** | Dead VCS toolchain (grit/icm; ban direct git/gh) mandated live in two Accepted ADRs. | `ADR-0053:49` "no agent path may invoke git/gh" (`:138` "superseded by 0116") + `ADR-0103:38-50` "Direct git from agents → Banned" vs `ADR-0116:43` (retires tools) + `ADR-0363:16-17` ("use git as is"). Both Accepted; `ADR-0103` `superseded_by:[]`. `canon:192-193` D-AEC-DECLINE. | SUPERSEDE 0053 + 0103 by 0116/0363/0515; flip status (do not leave Accepted). | A-CI |
| **C-7** | Kafka on the critical consistency path in three Accepted ADRs. | `ADR-0059:112` "outbox → Kafka KRaft", `ADR-0062:71` "Outbox → Kafka KRaft" (+`:41` "Confluent Kafka (KRaft)"; corrected anchor :41 not :42), `ADR-0091:114` "bind Kafka producer to WriteGate" vs `canon:147` D-EVENT (Pulsar) + `canon:190` D-D1-TOPOLOGY (Kafka removed from consistency path). | AMEND Kafka→Pulsar (eventual fan-out only); adopt the D-D1 consistency-token model on the critical path. | A-DATA |
| **C-8** | Autonomy gate Accepted-on-Proposed-0022 + homed in dead foundry context. | `ADR-0099:38` calls Proposed `ADR-0022:3` a "mandate"; homes enforcement `:97` `namespace foundry::supervisor` + `:239` `oya-foundry-autonomy-ceiling-app` vs `canon:39` D16 (governance-owned hard gate) + `canon:34` D6 (PARC engine). | AMEND — re-home to governance, retarget engine to PARC, fix the Proposed-0022 dependency. | A-FOUNDRY |
| **C-9** | Foundry pipeline enthroned as sole canonical workflow + depends on Proposed 0111. | `ADR-0116:42` "Foundry pipeline (M01-P18) is the sole canonical workflow for concurrent agent work" + `:50` cites Proposed `ADR-0111:2` vs `ADR-0363:37` (substrate retired, 2/20 crates wired, never deployed). | AMEND 0116 — keep grit/icm/rtk/vox retirement, replace the Foundry-pipeline mapping with oya-ci/Forgejo per 0363/0515. | A-CI |
| **C-10** | Enforcement façade — "binding source of truth" while enforcement is advisory/planned. | `ADR-0128:22` `enforcement_status: advisory-until-product-prd-validator` + `:155` "planned `oya-governance-*` lanes … remain backlog items, not active required checks" vs canon firewall `canon:181,196`. | KEEP the invariant catalog; AMEND to wire real RED/GREEN-proven gates OR relabel honestly. | A-INTEGRITY |
| **C-11** | Foundry cluster declared a live µservice with six bounded contexts (4 Accepted ADRs, empty edges). | `ADR-0136:121` "Foundry is one µservice with six internal bounded contexts", `ADR-0137:46` "exactly six bounded contexts", `ADR-0138:45` Strangler INTO `microservices/foundry/`, `ADR-0143:68-93` `release/foundry-*` (corrected anchor :68 not :67) vs `canon:204` D-FOUNDRY-CLARIFY. Cross-ref **H-33**: 0335 supersedes only PRD/doc files, not 0137/0143. | SUPERSEDE 0136/0137/0143 into the cloud-intelligence framework ADR (salvage one-perimeter+six-BC reasoning); ARCHIVE 0138 (Strangler to a dead address; reuse template). | A-FOUNDRY |
| **C-12 = P1** | ADR-0160 Flagger canonical vs D10 Argo Rollouts; no supersede edge either direction. | **BOTH.** `ADR-0160:42` "adopts Flagger 1.x as the canonical progressive-delivery controller" + `:62` "Why Flagger over Argo Rollouts" + `:8` `superseded_by:[]` + `:151` cites Superseded `ADR-0124`. vs `canon:62` D10 ("Supersede Flagger 0160" → Argo Rollouts) + `ADR-0515:80,83` Argo-Rollouts canonical; **`ADR-0515:9` `supersedes` list OMITS 0160**. | SUPERSEDE 0160 by the D-CICD Argo-Rollouts ADR; **write the edge** (P1: `0160.superseded_by←[0515]` + add 0160 to `0515.supersedes`); flip status; keep SLO-gate ladder + auto-rollback shape. | A-CI |
| **C-13 = P2** | ADR-0187 Zitadel-canonical vs D5 (vendored bridge); **half-edge** — 0476 names the wrong successor id. | **BOTH.** `ADR-0187:37` "Zitadel … is the canonical IdP … the single issuer" + `:8` `superseded_by:[]` vs `canon:31` D5 (Zitadel = vendored bridge, demoted-as-endpoint by 0476). **Half-edge:** `ADR-0476:9` `supersedes:[ADR-0421]` (phantom) instead of `[ADR-0187]`; `ADR-0476:10` `superseded_by:[]`. | SUPERSEDE-as-endpoint: `0187.superseded_by←[0476]`, status→superseded-as-endpoint; **fix the 0421 mis-number** `0476.supersedes:[0421]→[0187]` (P2 DOUBLE write); keep 0187 operative as the Phase-1 bridge. Pairs with C-21. | A-IDENTITY |
| **C-14** | ADR-0192 Milvus owned by the dead foundry µservice + filed in the wrong layer. | `ADR-0192:47` "Milvus … owned by the `foundry` µservice" + `:128` `microservices/foundry/iac/helm/milvus/` vs D-FOUNDRY-CLARIFY (dead) + D4 (vector store is data-tier, not intelligence). | AMEND — re-home to cloud-data (D4), Pulsar-only; reconcile the 0046-closed-vs-Phase-2-reopen tension. | A-FOUNDRY |
| **C-15** | ADR-0202 ArgoCD declared the canonical owned engine vs D3/D-CICD bespoke oya-cd. **(= the CRITICAL hyperscaler own-vs-adopt, §2.)** | `ADR-0202:44` "Tier A — GitOps app deployment: ArgoCD" ("ArgoCD is the canonical Tier-A engine") vs `canon:169` D3/D-CICD (bespoke-Rust oya-cd adopting Argo *patterns*; D-EXEC ARCHIVE/DROP 0511, consolidate to 0515). | AMEND — reframe Tier-A ArgoCD as a vendored bridge → owned oya-cd (build-first-cutover-later); KEEP Tier-B (OpenTofu) + Tier-C (Cluster API); relate to 0515. | A-CI |
| **C-16** | Phantom-0150 epidemic — 0150 is Cursor-Pagination, mis-cited as Cedar/policy-engine in 4 sites. | `ADR-0150:1` "# ADR-0150: Cursor Pagination Canonical" vs `ADR-0239:49,92`, `ADR-0148:257`, `ADR-0182:165` all cite "ADR-0150 — Cedar policy engine". Real policy ADR = `ADR-0183`. | Repoint all four sites to 0183 (HIGH-priority dangling-edge cleanup, D11). | A-INTEGRITY |
| **C-17 = banner** | ADR-0239 ("foundry INTERNAL only") left Accepted with no supersession banner; 0335 marks it historical in PROSE only. | **BOTH.** `ADR-0239:3` "Accepted (amendment)" governing `:21` "`microservices/foundry/` is INTERNAL only" vs `ADR-0335:593-595` (historical in prose only; `0335.amends` includes 0239 at `:24`) — 0239's own status line is untouched. A 0239-only reader gets a dead topology. | SUPERSEDE/mark-historical 0239 by 0335 (status banner) + fix the phantom-0150 anchor. (Related: P5 owes 0335 its own superseded-on-cutover marker — §3 keystone.) | A-INTEGRITY |
| **C-18 = P3** | ADR-0374 Jenkins-as-orchestrator + Forgejo as ratified canon; excluded from 0515's supersedes. | **BOTH.** `ADR-0374:188` "Decision (founder): Jenkins-as-orchestrator" + `:55-56`/`:36` git+Jenkins+Forgejo substrate + `status: Accepted` `superseded_by:[]` (`:9`); **EXCLUDED from `ADR-0515:9` `supersedes:[0124,0349,0359,0361,0511,0513,0514]`** — appears only in `related` of the now-Superseded `ADR-0513:14,25`. vs `canon:153` D2 + `canon:207` D-FORGE-CLARIFY + D-CICD/0515. | SUPERSEDE by 0515 (**superseded-on-cutover**; keep the physical scaffold as the unratified bridge); **write the edge** (P3: add 0374 to `0515.supersedes` + `0374.superseded_by←[0515]` + flip status). | A-CI |
| **C-19 = P4** | ADR-0380 rebuilds the Jenkins CI farm + Forgejo-canonical; excluded from 0515's supersedes. | **BOTH.** `ADR-0380:21` "Re-establish the Jenkins CI farm on the Talos substrate" + `:38` "Forgejo-canonical" + `status: Accepted (amendment)` `superseded_by:[]` (`:9`); promised edge never landed (`ADR-0513:22-23` "formal supersession … lands at the Phase-1 cutover"; 0513 itself Superseded). vs `canon:78` D-EXEC (drop Jenkins/Argo debt). | SUPERSEDE by 0515 (**superseded-on-cutover**); **write the edge** (P4: add 0380 to `0515.supersedes` + `0380.superseded_by←[0515]` + flip status). | A-CI |
| **C-20** | Phantom-edge epidemic — supersedes/refs to ADR ids that do not exist on disk. | `ADR-0476:9` `supersedes:[ADR-0421]` (ABSENT); `ADR-0482:54` "oya-vcs (ADR-0409)" (ABSENT). Verified ABSENT via `ls`: 0409,0411,0416,0421,0428,0429,0434,0443,0457,0397,0477,0483,0484,0088,0012. Affects 0476/0478/0479/0480/0481/0482/0508/0069. | Repoint/renumber every phantom id; add a **no-dangling-ref invariant** to the integrity gate (D11/D13). See §3 keystone. | A-INTEGRITY |
| **C-21** | Identity dual-bridge clash — two Accepted IdP ADRs name DIFFERENT vendored bridges, no edge between them. | `ADR-0476:18` "Supersedes ADR-0421 (Keycloak)" (phantom; real IdP = 0187 Zitadel) + `:103` rejects Zitadel (inverting D5) + `:36-37` "Keycloak (0421) is the Phase-1 bridge" vs `ADR-0187:37` (Zitadel canonical); no 0187↔0476 edge. | AMEND 0476 — `supersedes:[0421]→[0187]`; flip Zitadel rejected→adopted Phase-1 bridge; long-term endpoint = bespoke oya-identity. Pairs with C-13. | A-IDENTITY |

### HIGH (20 from accepted-audit + the 7 status-vs-edge drifts + 2 SPOF-promoted; deduped)

| id(s) | Contradiction | Cited both sides | Fix | A-lane |
|---|---|---|---|---|
| **H-1** | KR minor-age contradiction across two regulator-facing Accepted ADRs. | `ADR-0008:69` `Minor // <14 KR` vs `ADR-0034:94` "under 18 in KR". | Reconcile (KR PIPA Art 22(6) child-consent <14 vs youth-protection "under 18" — disambiguate). | A-TASTE |
| **H-2** | Dangling ADR-0012. | `ADR-0008:197` "→ ADR-0012"; ABSENT (`ls`). | Repoint/remove. D11(c). | A-INTEGRITY |
| **H-3** | Advisory-not-enforced codified (false-green mechanism). | `ADR-0011:146` "`oya-check-contracts` is an advisory P0 lane reference until the crate exists". | Wire real enforcement or relabel honestly. D-DOCTRINE. | A-INTEGRITY |
| **H-4** | ADR-0017 GitHub slug framed permanent vs D2 bespoke-SCM ratchet. | `ADR-0017:26,40-41` retain `jason931225/oyatie` permanently vs D2/D-FORGE-CLARIFY (GitHub-interim → bespoke Sapling `cloud/cloud-scm`). | Add the GitHub-interim→bespoke ratchet. | A-CI |
| **H-5** | ADR-0006 self-rename tautology + missing temporal type (folds U-2). | `ADR-0006:11,22` `"Ontology" renamed to "Ontology"`; `:50-64` Entity has `schema_version` but no effective-dating/consistency-token vs `canon:190` D-D1 keystone. | Repair the rename text (D11(b)); add the effective-dating kernel temporal type + consistency token. | A-DATA |
| **H-6** | ADR-0029 12-app Workspace/M365 parity as one milestone, no sequencing gate. | `ADR-0029:38-53,154`. | In-scope but SEQUENCE per D8/D9 (M0 evidence-gate); don't present as a single milestone. | A-TASTE |
| **H-7** | ADR-0030 from-scratch search engine as a flat catalog µservice, no M0 gate. | `ADR-0030:32-96` (crawler/render-farm/inverted+vector index/KR morphology/RTBF/KG). | Sequence as its own vertical with an M0 evidence-gate (D8/D9); KEEP the KR-first morphology moat. | A-TASTE |
| **H-8** | Dead `foundry` context pervasive (≥8 Accepted ADRs; one touches a code enum). | `ADR-0062:130`, `ADR-0091:41`, `ADR-0067:97`, `ADR-0069:10`/`0065:10`/`0066:10` (owner `axis-foundry`), `ADR-0200:68` (`foundry-tool` sandbox class), `ADR-0258:99` (`oya.foundry.v2.CapabilityService`; corrected v2 not v1). | Batch foundry→intelligence/governance rename (canon WF2); the 0200 sandbox-class enum touches code — coordinate. | A-FOUNDRY |
| **H-9** | GitHub-Actions hard-wired as canonical CI + MASTERPLAN read as planned-set source (inverts D1). | `ADR-0063:185`, `ADR-0066:53`, `ADR-0067:64` (`.github/workflows`/`gh api`); `ADR-0063:95` reads `MASTERPLAN.md §2.1` vs `canon:9-10` D1 (ADRs SSOT; masterplan generated). | GH-Actions→oya-ci lane runner; read ADR front-matter not masterplan. | A-CI |
| **H-10** | ADR-0069 phantom-0088 + wrong filenames. | `ADR-0069:12,174` "ADR-0088 (foundry scaffolding)" (ABSENT); `:172,173` wrong filenames for 0056/0067. | Repair edges. D11(c). | A-INTEGRITY |
| **H-11** | ADR-0067 mega-service (~100-140 crates) vs flat-catalog single-concept + D-PURESPLIT. | `ADR-0067:159` "~20 BCs × 5-7 layer crates = ~100-140 crates" + `:139` subsumes 0065/0066. | Re-shape `ops` into properly-bounded services under the pure-split. | A-STRUCT |
| **H-12** | ADR-0062 day-1 100M-scale mandate vs M0-gated sequencing. | `ADR-0062:18` "100M+ user scale … mandatory from day one. No single-instance-only designs" vs `canon:116-117,121-122` D8/D-SEQ. | Reframe "from day one"→"M0-gated, scale-on-proven-demand"; keep the CI-enforced-quality spine. | A-TASTE |
| **H-13 / U-1** | ADR-0098 accepts silent power-loss data-loss to save one vetted dep. | `ADR-0098:71-78,184` documented non-durability to avoid `rustix`. | Flip durability default to `fsync(parent_dir)`; "zero net-new deps" as a hard goal is metric-worship. | A-TASTE |
| **H-14** | ADR-0119 dangling back-edge (partial-supersession not recorded on 0119). | `ADR-0131:10` partial-supersedes 0119 vs `ADR-0119:8` `superseded_by:[]`. | Add the back-edge. D11. | A-INTEGRITY |
| **H-15** | ADR-0123 dead-vcs forward-authority refs (content exemplary). | `ADR-0123:53,66` "HG-VCS"/`oya-vcs-admission` post-0363. | KEEP-with-amend: repoint vcs refs to oya-ci/governance. | A-CI |
| **H-16** | ADR-0173 stale vendor-doctrine SSOT (high-leverage; others cite it). | `ADR-0173:163-171` Forgejo/Woodpecker + "Foundry VCS (ADR-0113)"; `:187` "Kafka or NATS" (corrected :187 not :184); `:199` "OpenFeature + Flipt". | Keep the doctrine; fix every stale pick (Forgejo→oya-ci, Kafka/NATS→Pulsar, Flipt-vs-0159 reconcile, foundry→intelligence, 0113-vcs-retired). | A-CI |
| **H-17** | ADR-0159 vs ADR-0173 feature-flag clash. | `ADR-0159:42` owned `feature-flags` µservice vs `ADR-0173:199` "OpenFeature + Flipt". | Reconcile to one (owned OpenFeature server). | A-TASTE |
| **H-18** | Redis live vs Valkey ruling. | `ADR-0184:101-105` Redis REJECTED for Valkey vs `ADR-0191:46,68` + `ADR-0208:74` "Redis". `canon` D12. | AMEND 0191/0208 → Valkey. | A-DATA |
| **H-19** | Kafka-as-broker option / Kafka-Engine default + phantom-0397. | `ADR-0192:58` "Pulsar 4.2 or Kafka"; `ADR-0195:15,19,67` "Kafka Engine" default (substrate reconciled to Pulsar `:71-72`); `ADR-0377-kafka:22` cites phantom `ADR-0397`. D-EVENT Pulsar-only. | Name Pulsar (Kafka = wire-compat only); fix the phantom-0397 edge. | A-DATA |
| **H-20** | `microservices/` flat-catalog path residue fleet-wide. | `ADR-0184:144`, `0196:68`, `0202:75`, `0209:126`, `0143:89`, `0238:305` + every path-bearing Accepted ADR in 0133–0335. `canon:171-172` D-PURESPLIT. | Bulk additive path rename to `{oya,cloud}/<service>/` (mechanical, D13-AMENDED). | A-STRUCT |
| **H-21** | Kyverno hard-wired citing now-Superseded 0183 (resolved by the keystone graph fix). | `ADR-0183` Superseded (`superseded_by:[ADR-0379]`) vs `ADR-0338:229-233`, `ADR-0117:25-27` hard-wire Kyverno citing 0183 as live. | Repoint to 0379 (Kubewarden default). | A-INTEGRITY |
| **H-SE-0316** (drift §2A-2) | ADR-0316 `Proposed` while pointing a `superseded_by` at an Accepted ADR. | `ADR-0316:3` `Proposed` + `:28` `superseded_by:[ADR-0329]` (0329 Accepted). | Flip `Proposed`→`Superseded` (or resolve the Proposed: ratify/drop, then supersede). | A-INTEGRITY |
| **H-SE-0358** (drift §2A-3) | ADR-0358 `Proposed` + YAML block-list supersede edge (missed by inline-only scans). | `ADR-0358:3` `Proposed` + `:9-11` block-list `superseded_by:[ADR-0392,ADR-0408]`. | Flip `Proposed`→`Superseded` (§2 reversed by 0392/0408 per amendment_note). | A-INTEGRITY |
| **H-SE-0482** (drift §2A-4) | ADR-0482 `amended_by` points at a non-ADR token (dangling). | `ADR-0482:3` Accepted + `:13` `amended_by:[kubers-anchor-2026-05-28]` (non-ADR). | Fix the dangling amender to a real ADR id or convert to a tracked amendment record; set status per policy. | A-INTEGRITY |
| **H-SE-0052** (drift §2A-5) | ADR-0052 body status contradicts its own frontmatter status. | Frontmatter `ADR-0052:4` `Superseded` + `:11` `superseded_by:[ADR-0118]` vs BODY `:29` "Status: Accepted" + `:32` "Superseded-by: —". | Fix the BODY to match frontmatter (Superseded → ADR-0118). | A-INTEGRITY |
| **H-SE-0363** (drift §2A-6) | ADR-0363 Accepted + `amended_by` chain points at a stale (itself-Superseded) amender. | `ADR-0363:3` Accepted + `:10` `amended_by:[ADR-0510,ADR-0513]` (0513 Superseded), `superseded_by` empty `:9`. | Reconcile to a live amender or set `Amended`; also folds the false-"eradicated" + stale-Forgejo content fix. | A-CI |
| **H-SE-0054** (drift §2A-7) | ADR-0054 `deprecated` with the supersede edge only in the body, not frontmatter. | `ADR-0054:3` `deprecated` + body `:9,:13` "Superseded by ADR-0116". | Flip `deprecated`→`Superseded`; add frontmatter `superseded_by:[ADR-0116]`. | A-INTEGRITY |
| **H-22** (SPOF, promoted) | ADR-0031 literal-singleton ads-gate = fleet-wide SPOF. | `ADR-0031:56` "The gate is a singleton" + `:134` "gate outage = no ads served anywhere". (Cross-ref §2.) | "Logically-single policy authority, physically-replicated serving path." | A-TASTE |
| **H-23** (SPOF, promoted) | ADR-0028 same-provider primary+secondary SPOF. | `ADR-0028:36` "primary OCI KR-Seoul1; secondary OCI KR-Chuncheon; fail-open AWS" (only fail-open is cross-provider). (Cross-ref §2.) | Genuine multi-provider / multi-account control-plane isolation. | A-TASTE |

> **Note on P5 (the 5th supersession directive-without-edge).** P5 (`ADR-0335` foundry→intelligence,
> no marker at all; `canon:90` D-INTEL FINAL re-homes the engine into cloud/cloud-intelligence) is NOT a
> standalone §1 row because the accepted-audit dispositions 0335 as **AMEND** (it is the consolidating
> ADR, not a stale one). Its missing artifact is the **superseded-on-cutover marker** — which is exactly
> the KEYSTONE FIX in §3, and is tracked there + in the §4 disposition for 0335. C-17 carries 0335's
> banner obligation toward 0239/0220.

### REFUTED (carried, not dropped)

- **R-1** — "0335 keeps a `vcs-orchestrator` µservice." FULL REFUTE: `ADR-0335:224-227` "the principal
  namespace persists, **the µservice does not**." Residual LOW: the `oyatie.foundry.*` principal-namespace
  naming is a real but separate integrity nit.
- **R-2** — "Forgejo in **15** Accepted ADRs." COUNT REFUTE: `grep -lri forgejo` = 27 files (all statuses);
  the "15 Accepted" figure is unverified. Substance (Forgejo survives in multiple Accepted incl.
  `ADR-0515:76,96` and 0380) HOLDS.
- **R-3** — ads-gate SPOF dual-cited to `0028:18,136` + 0031. OVER-BROAD REFUTE: 0028:18 = "Cloud is the
  compute substrate"; no "gate outage" string in 0028. Only the 0031 half is valid (→ H-22); 0028 has its
  OWN separate same-provider SPOF (→ H-23).

---

## SECTION 2 — HYPERSCALER-LENS PROBLEMS (SPOF / ceilings · under-eng · over-eng · wrong own-vs-adopt)

### SPOFs / availability ceilings
- **[HIGH] ADR-0031 ads-gate fleet-wide SPOF** (= H-22). `ADR-0031:56` "the gate is a singleton";
  `:134` "gate outage = no ads served anywhere". **REC:** logically-single policy authority,
  physically-replicated/sharded serving path with a consistent policy snapshot (the Google-Ads pattern).
- **[HIGH] ADR-0028 same-provider primary+secondary SPOF** (= H-23). `ADR-0028:36` primary OCI KR-Seoul1 +
  secondary OCI KR-Chuncheon (same provider) + fail-open AWS (only this is cross-provider).
  **REC:** genuine multi-provider / multi-account control-plane isolation.
- **[MED] ADR-0029 shared CRDT runtime concentration.** `ADR-0029:64-69,155` one Yrs CRDT runtime under all
  of Docs/Sheets/Slides/Notes/Sites. **REC:** isolate per-surface or shard.
- **[MED] ADR-0067 mega-service blast radius.** `ADR-0067:160` self-flags "ops.oyatie.com outage takes down
  the operations surface". **REC:** decompose (pairs with H-11).

### Under-engineering / missing table-stakes
- **[HIGH] ADR-0098 accepts silent power-loss data-loss** to save one vetted dep (= H-13/U-1).
  **REC:** never trade durability for a dep-count vanity metric in a queue/DLQ path; `fsync(parent_dir)`.
- **[HIGH] ADR-0006 missing effective-dating + read-your-writes** (= U-2, folded into H-5). `ADR-0006:50-64`
  Entity has `schema_version` but no temporal/effective-dating type and no consistency-token contract — the
  very property `canon:190` D-D1 makes the keystone (payroll-close read-your-writes test).
  **REC:** add the effective-dating kernel temporal type + consistency token (net-new kernel build).
- **[MED] ADR-0184 "None planned" for owned Postgres** (`:169`) vs D4 owned-data-tier endpoint. D4 permits
  vendored-until-proven, so the literal "None planned" is the drift. **REC:** tag transitional.

### Over-engineering / premature complexity (pre-revenue, pre-M0)
- **[HIGH] ADR-0062 day-1 100M-scale mandate** (= H-12) — `ADR-0062:18`.
- **[HIGH] ADR-0030 from-scratch web search as a catalog µservice** (= H-7) — `ADR-0030:32-96`.
- **[HIGH] ADR-0029 full Workspace/M365 parity as one milestone** (= H-6) — `ADR-0029:38-53,154`.
- **[HIGH] ADR-0067 mega-service** 100-140 crates (= H-11) — `ADR-0067:159`.
- **[HIGH] ADR-0482 unbounded ambition.** `ADR-0482:114-117` "Any upfront investment is acceptable; timeline
  is the only constraint" (~30 parallel bespoke tracks) vs D8/D-SEQ capacity budget.
  **REC:** insert the D8 capacity-budget gate (oya-ci-prioritized, sequenced).
- **[HIGH] ADR-0508 owned-silicon.** `ADR-0508:79-92,143-152` OpenTitan SoC + manufacturing line.
  **REC:** bind to D8/D-SEQ M0 sequencing so it cannot be pulled forward.
- **[MED] ADR-0109 "automation cost ≈ 0"** unverified claim (`:33,179-188`) — robust-not-false risk.
- **[MED] ADR-0066/0067** ~15-extractor live-introspection daemon + nightly-rustdoc-JSON dep before first
  tenant. **REC:** sequence under M0.

### Wrong own-vs-adopt
- **[CRITICAL] ADR-0202 ArgoCD-as-owned-engine** (= C-15): adopt the *pattern*, reimplement in Rust.
  `ADR-0202:44` vs `canon:169` D3/D-CICD. **This is the one CRITICAL hyperscaler-lens item.**
- **[HIGH] ADR-0053/0103 adopt fragile niche `grit` as a HARD mandate** with a known upstream blocker
  (`ADR-0053:103`) — reinvent-plain-git in reverse; SUPERSEDED (= C-6).
- **[MED] ADR-0011 commits generated 4-language SDKs into the tree** (`:106`); hyperscalers generate SDKs as
  CI artifacts. **REC:** build-time generation.
- **[GOOD own-vs-adopt to KEEP]:** ADR-0139 ("adopt OSS leaders, own the differentiator"), ADR-0173
  (own-the-stack ratchet doctrine), ADR-0200 ("own the integration layer, not the runtime"), ADR-0506
  (mandatory feature-parity table), ADR-0209 (compliance-evidence 100% in-house — the differentiation).

---

## SECTION 3 — ANTI-PATTERNS (8 systemic) + THE KEYSTONE FIX

### The 8 systemic anti-patterns
1. **Incomplete supersession sets** (the dominant failure mode): a unifying ADR supersedes SOME of a
   cluster, leaving co-equal Accepted siblings live — CI/CD (C-18/C-19), identity (C-13/C-21), foundry
   (C-11, H-33), admission (H-21), progressive-delivery (C-12). → owned by
   `contradiction-audit/00-SUPERSESSION-COMPLETENESS.md` (P1–P5 + §2A).
2. **Dangling / phantom edges** (C-16, C-20, H-2, H-10): 0421/0409/0397/0088/0012/0150-as-policy +
   0331/0069/0476 filenames — no no-dangling-ref invariant is enforced.
3. **Bridge-vs-ratified encoded in prose, not the graph** (C-12/13/18/19, C-15): the status enum has no
   "superseded-on-cutover / bridge-unratified" value, so Jenkins/Forgejo/Flagger/ArgoCD/Zitadel read as
   fully ratified canon (`canon:166,177-184`). **The single highest-leverage structural fix.**
4. **Enforcement façade** (C-10, H-3): "binding spec, advisory enforcement" / advisory-lane-claiming-blocking
   (0128, 0011:146, un-wired `oya-governance-*`). → make enforcement REAL first (canon firewall).
5. **Un-propagated vendor rulings** (C-7, H-16/18/19/21): Forgejo/Kafka/Flagger/Kyverno/Redis still named
   live; rulings live only in the canon doc, never written into the Accepted corpus.
6. **Day-1-everything vs M0-gated sequencing** (H-6/7/11/12, 0482, 0508).
7. **Forbidden-term leakage as architecture** (C-2/C-3/H-8/H-20): foundry namespace; flat-`crates/`;
   `microservices/`.
8. **ADR-inflation / executed-migration-left-Accepted** (C-4): 0097 3-crate cosmetic rename; 0057 one-shot
   cutover; 0101 "temporary bypass" promoted to architecture (ARCHIVE set in §4).

### THE KEYSTONE FIX (one structural move)
> Add a **`superseded-on-cutover` / `bridge-unratified`** value to the ADR `status` enum, **plus a
> no-dangling-ref / supersession-completeness invariant enforced by the cross-artifact-agreement gate.**

- **Coupled-invariant rule (the procedure that, when skipped, IS the contradiction):** a later directive
  that moves away from an earlier ADR MUST, in the same act, (1) write BOTH directions of the
  supersession/amendment edge AND (2) flip the stale ADR's `status` off `accepted`/`Accepted`/`Proposed`/
  `deprecated`. Half-edges still fail (C-13/P2: 0476 named Zitadel's successor in prose but pointed
  `supersedes` at the wrong id). Ratchet/build-first moves still owe a **marker** (P5 = 0335) — "nothing at
  all" is the failure.
- **What it resolves at the graph level:** anti-patterns **1–3** + contradictions
  **C-12 / C-13 / C-16 / C-17 / C-18 / C-19 / C-20 / C-21 + H-21** — i.e. it makes the
  Forgejo/Jenkins/Flagger/ArgoCD/Zitadel bridges READ as bridges, and makes a phantom edge un-mergeable.
- This is the firewall-first move the charter targets (`canon:166,177-184`) — same class as the Phase-0
  firewall (ADR-0515 + the 4 keystone gates).

### Exemplary patterns to KEEP (cited; do not "fix")
- **Honest-claim / no-false-green:** ADR-0129 (active RED/GREEN gate `:30-33`), ADR-0135, ADR-0123
  (maturity-claim gate), ADR-0104 (no-stub build-graph reachability), ADR-0093, ADR-0204.
- **Seam / own-the-endpoint ratchet:** ADR-0173 (own-the-stack SSOT), ADR-0142 (CRDT ≥2-impls compile gate),
  ADR-0064 (seam>adapter>pack trichotomy), ADR-0482 (bridge-discipline checklist), ADR-0506 (parity table),
  ADR-0197 (BackupExecutor trait seam).
- **Type-safety / boundary:** ADR-0095, ADR-0083, ADR-0094, ADR-0056 (closed layer enum, inward-only deps).
- **Structure / monorepo doctrine:** ADR-0512 (founder-locked governing ADR), ADR-0131/0132, ADR-0362,
  ADR-0115.
- **Architecture altitude:** ADR-0145, ADR-0172 (narrow CQRS), ADR-0148/0182, ADR-0191, ADR-0158
  (rejects mandatory-active-active as malpractice), ADR-0376 (adr-challenge gate working).
- **Sovereign / regulated:** ADR-0240, ADR-0164, ADR-0241, ADR-0034, ADR-0008.
- **Integrity done right:** ADR-0179/0180 (honest `renumber_note`), ADR-0393 (clean-supersede), ADR-0118.

---

## SECTION 4 — PER-ADR DISPOSITION TABLE (all 169 Accepted)

> KEEP = substance sound, refs/path sweep only · AMEND = substance largely sound, named/integrity/canon
> fixes · SUPERSEDE = overruled by canon (salvage pattern, write edge, flip status) · ARCHIVE = target moot.
> "(minor)" = path/vocab/citation touch-ups only. The 9 clean SUPERSEDE + 3 bridge + 5 ARCHIVE carry a why.

| ADR | Disp | One-line why |
|---|---|---|
| 0001 | AMEND | foundry substrate owners + flat-catalog vs pure-split + Proposed-0007 dep |
| 0006 | AMEND | missing effective-dating/consistency-token (D-D1 keystone) + self-rename tautology |
| 0008 | KEEP | strongest privacy ADR; reconcile KR minor-age vs 0034; dangling 0012 |
| 0011 | AMEND | foundry owner + flat-crates paths + advisory-gate-claiming-enforced |
| **0015** | **SUPERSEDE** | forbidden flat-`crates/` LOCATION + forbidden-vocab enum; flip `accepted`→Superseded under 0512/0131 (edge exists; FC-1) |
| 0017 | AMEND | GitHub slug framed permanent vs D2 bespoke-SCM ratchet |
| 0018 | AMEND | glossary canonicalizes forbidden "Foundry"; delete the glossary row |
| 0028 | AMEND | same-provider primary+secondary SPOF; cloud-as-flat-peer vs D-LAYER |
| 0029 | AMEND | full Workspace/M365 parity as one milestone (no M0 gate) |
| 0030 | AMEND | from-scratch search engine as catalog µservice (no M0 gate) |
| 0031 | AMEND | literal-singleton ads-gate fleet-wide SPOF |
| 0034 | KEEP | exemplary data-flow guardrails; reconcile KR minor-age vs 0008 |
| 0051 | KEEP | minor: `W-Foundry-Preview` wave label + "14 verticals" count |
| **0053** | **SUPERSEDE** | dead grit/icm toolchain mandated live; by 0116/0363/0515 (flip status) |
| 0055 | KEEP | clean Ontology rename; the fix is on 0057's edge |
| 0056 | AMEND | flat-crate grammar + `public_layers` cross-import vs pure-split |
| **0057** | **ARCHIVE** | executed one-shot cutover; dangling+colliding supersedes to phantom 0055-v3 (fix edge first) |
| 0058 | AMEND | flat-`crates/` catalog topology vs D-PURESPLIT |
| 0059 | AMEND | Kafka-outbox on critical path vs D-EVENT/D-D1 |
| 0060 | KEEP | Bominal-inheritance ledger; amend inherited Kafka row |
| 0061 | KEEP | minor: Redis→Valkey; soften day-1-100M claim |
| 0062 | AMEND | day-1-hyperscale mandate + Kafka-as-benchmark + foundry |
| 0063 | AMEND | GitHub-Actions CI + MASTERPLAN-as-planned-set (inverts D1) + axis-foundry |
| 0064 | KEEP | exemplary seam/pack trichotomy; topology-path reconcile only |
| 0065 | AMEND | foundry owner + adds `docs` µservice that 0067 renames |
| 0066 | AMEND | GitHub-Actions + grit/ICM live data sources + 15-extractor premature build |
| 0067 | AMEND | `ops` mega-service (100-140 crates) + GH-Actions/grit + foundry |
| 0069 | AMEND | phantom ADR-0088 + wrong filenames; foundry owner |
| 0083 | KEEP | exemplary typed-error tiers |
| 0090 | KEEP | head Decision contradicts its own Amendment; mark head superseded-in-place |
| 0091 | AMEND | `oya-foundry-write-gate-*`→governance; Kafka→Pulsar |
| 0092 | KEEP | seam policy; reconcile one-workspace to pure-split |
| 0093 | KEEP | model honest-naming ADR; no canon conflict |
| 0094 | KEEP | typed Handler trait; no canon conflict |
| 0095 | KEEP | model boundary-type ADR; no canon conflict |
| 0096 | AMEND | foundry-supervisor (dead context); salvage Rust-vs-Node principle |
| **0097** | **ARCHIVE** | 3-crate cosmetic foundry rename; subsumed by the foundry sweep |
| 0098 | AMEND | foundry-supervisor + accepts power-loss data-loss to avoid `rustix` |
| 0099 | AMEND | Accepted-on-Proposed-0022 + foundry-home for autonomy gate vs D16/D6 |
| 0100 | AMEND | foundry-supervisor; extract zero-surface-change doctrine |
| **0101** | **ARCHIVE** | "temporary bypass" shortcut promoted to architecture; foundry |
| **0102** | **ARCHIVE** | foundry settings render; salvage atomic-render+sref pattern |
| **0103** | **SUPERSEDE** | grit/icm sanctioned-VCS + ban-git; by 0116/0363 (flip status) |
| 0104 | KEEP | exemplary no-stub build-graph reachability; amend dead-name examples |
| 0105 | KEEP | 13-layer enum BNF canon; amend broken math line |
| 0106 | KEEP | `application`→`usecase` rename; close 5 orphan crates |
| 0108 | KEEP | sunset lifecycle schema; amend foundry BNF gloss |
| 0109 | KEEP | lifecycle-automation; require RED/GREEN per lane before "enforced" |
| 0115 | KEEP | flat-singular registry consolidation; clean |
| 0116 | AMEND | keep grit/icm retirement; replace Foundry-pipeline mapping; Proposed-0111 dep |
| 0117 | KEEP | repo hygiene; amend `oya-vcs-admission` refs + Kyverno→0379 |
| 0118 | KEEP | exemplary anti-false-enforcement; reword Foundry-pipeline rationale |
| 0119 | AMEND | per-product-specs partially superseded by 0131; missing back-edge |
| 0122 | KEEP | Ontology crate rename; fix enum ref |
| 0123 | KEEP | exemplary maturity-claim gate; amend dead vcs refs |
| 0128 | KEEP | exemplary invariant catalog; enforcement is vapor — wire it or relabel |
| 0129 | KEEP | gold-standard active RED/GREEN honest-claims gate |
| 0130 | KEEP | KG→Ontology consolidation; add canonical YAML front-matter |
| 0131 | KEEP | per-service flat layout, pure-split-amended; verify 0119 back-edge |
| 0132 | KEEP | no-grouping forward-policy; amend `microservices/`→`{oya,cloud}` |
| 0133 | AMEND | conformance program; retarget Axis-2 to pure-split; Argo/Flagger reconcile |
| 0135 | KEEP | aspirational-enforcement gate (real, landed) |
| **0136** | **SUPERSEDE** | foundry-as-µservice → cloud-intelligence framework (salvage 6-BC reasoning) |
| **0137** | **SUPERSEDE** | foundry bounded contexts → intelligence-framework successor |
| **0138** | **ARCHIVE** | foundry Strangler to a dead address; reuse the template |
| 0139 | AMEND | resolve ledger self-contradiction; de-foundry; thread to oya-ci/cd |
| 0142 | KEEP | exemplary CRDT port + ≥2-impls compile gate; fix mislabeled 0135 edge |
| **0143** | **SUPERSEDE** | foundry per-BC release pointer → intelligence-framework successor |
| 0144 | AMEND | EU-AI-Act 5-tier (exemplary); fix wrong 0140-Cedar edge |
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
| **0160** | **SUPERSEDE** | Flagger vs D10 Argo Rollouts; write the edge (P1: 0160.superseded_by←[0515] + add to 0515.supersedes); fix 0124 ref |
| 0161 | AMEND | CSI/StorageClass; `storage_tier` namespacing; drop foundry edge |
| 0162 | AMEND | per-tenant audit-chain slicing; PARC split; drop foundry |
| 0163 | AMEND | environment tiers→stages (D12); de-foundry; PARC |
| 0164 | AMEND | sovereign/air-gap (STRONG); re-home inference foundry→cloud-intelligence |
| 0165 | KEEP | Chaos Mesh (founder-endorsed D10); add port/ratchet framing |
| 0166 | AMEND | schema registry Apicurio; add port + ratchet; flag JVM sprawl |
| 0167 | KEEP | tenant CLI `oya`; rename `oya foundry` verb → intelligence |
| 0168 | KEEP | public status page; rename foundry component |
| 0169 | AMEND | webhook DLQ (exemplary); Pulsar; de-foundry |
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
| **0187** | **SUPERSEDE** | Zitadel-canonical vs D5; demote-as-endpoint, superseded_by←[0476] (P2); keep as Phase-1 bridge |
| 0188 | KEEP | passkey/WebAuthn; downstream RP-home reword once 0187 demotes |
| 0189 | KEEP | step-up ACR (exemplary closed enum); drop foundry; PARC ref |
| 0190 | KEEP | SCIM 2.0; clean |
| 0191 | AMEND | exemplary boundary table; Redis→Valkey; Cedar→PARC |
| 0192 | AMEND | Milvus owned by dead foundry + wrong layer (data-tier D4); Pulsar |
| 0193 | AMEND | ClickHouse OLAP; Pulsar naming; D4 ratchet; path |
| 0194 | KEEP | TimescaleDB (license-fence exemplary); soften "out of scope forever" |
| 0195 | KEEP | stream processing (exemplary rubric); Pulsar-naming on Kafka-Engine default |
| 0196 | KEEP | SeaweedFS object store; path; tag D4 bridge |
| 0197 | KEEP | backup 3-prong (exemplary seam + maintainer hedge); path |
| 0198 | KEEP | Karpenter autoscaling (exemplary own-vs-buy altitude); path |
| 0199 | KEEP | FinOps OpenCost/FOCUS; path |
| 0200 | KEEP | Wasmtime (exemplary); foundry-naming in sandbox-class enum (coordinate w/ code) |
| 0201 | KEEP | email adapter (no-silent-failure); domain re-tag; path |
| 0202 | AMEND | ArgoCD-as-canonical-engine vs D3/D-CICD oya-cd (CRITICAL own-vs-adopt); keep Tier-B/C |
| 0203 | AMEND | doc-engine 3-tier; reconcile with D-DOCORG |
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
| **0239** | **SUPERSEDE/mark-historical (bridge)** | mark historical by 0335 (status banner currently absent); fix phantom-0150 (HIGH integrity) |
| 0240 | KEEP | sovereign per-pack (exemplary); path + allow-list `_TIER1` labels |
| 0241 | KEEP | DR tiers + drill receipts (exemplary); refresh foundry T1 example |
| 0258 | KEEP | versioning model (GA-class); amend `oya.foundry.v2.*` mesh package → intelligence |
| 0329 | KEEP | tier-retirement allow-list zero-residue lane; refresh foundry refs + absolute paths |
| 0330 | KEEP | tenant-class billing; fix dangling companion-doc filenames + absolute paths |
| 0331 | KEEP | tenant_class adoption template; fix dangling related-filenames (HIGH integrity) |
| 0332 | KEEP | healthcare decomposition; fix stale 0316 edge; add D-SAFETY edge |
| 0333 | KEEP | cell-µservice retired→pattern; fix 0263 dangling edge |
| 0334 | KEEP | shorts→social; path (real debt is on 0237/0238) |
| 0335 | AMEND | name cloud-intelligence endpoint; **add superseded-on-cutover marker (P5, currently absent)**; 3-way foundry-fitness→governance carve-out; supersession banners on 0220/0239 |
| 0350 | KEEP | UUIDv7; sweep 0150 dangling-ref + foundry vocab |
| 0351 | KEEP+AMEND | cell-rebalancer/lifecycle; foundry-principal rename; reconcile durable-workflow |
| 0362 | KEEP | grouping-retirement (robust-not-false exemplar) |
| 0363 | AMEND | false "eradicated" claim + stale Forgejo + stale amended_by chain (§2A-6); Forgejo→GitHub-interim |
| 0364 | KEEP | generative ADR template / masterplan-from-ADR keystone; cite D13-AMENDED deferral |
| 0365 | KEEP+AMEND | automated ADR lifecycle; §4 substrate → oya-ci/GitHub-interim |
| 0366 | AMEND | agentic pipeline; substrate refs → oya-ci |
| 0367 | AMEND | trustless pre-merge gateway (keystone D3); trusted-runner=oya-ci, auto-merge=Tide |
| 0368 | KEEP | self-governing north-star; fix "becomes ADR-0000" lines (D13-AMENDED deferral) |
| 0369 | AMEND | stacked-trunk change-flow; re-target Forgejo→GitHub-interim→oya-ci Tide |
| 0370 | AMEND | headline falsified by own verification; add supersession/reconcile edges to 0378 |
| 0371 | KEEP | Cloudflare Tunnel control-plane; note Cloudflare-on-critical-path vs own-edge ratchet |
| 0373 | KEEP | cloud-intelligence gateway design; clean |
| **0374** | **SUPERSEDE (superseded-on-cutover, bridge)** | Jenkins-as-orchestrator + Forgejo vs D-CICD; by 0515 (P3 — write the edge, keep scaffold as unratified bridge) |
| 0375 | KEEP | Talos+ClusterAPI+ArgoCD substrate (exemplar); drop "flips to Forgejo" |
| 0376 | KEEP | managed-K8s product (strongest hyperscaler reasoning in band) |
| 0377-kafka | KEEP+AMEND | Kafka→Pulsar-via-KoP (keep); renumber the duplicate-0377 id collision; fix phantom-0397 |
| 0378 | AMEND | local substrate vfkit+Talos; substrate refs → oya-ci; supersession edges to 0370 |
| 0379 | KEEP | Kubewarden default admission (supersedes 0183); clean |
| **0380** | **SUPERSEDE (superseded-on-cutover, bridge)** | rebuild-Jenkins+Forgejo vs D-EXEC; by 0515 (P4 — write the edge) |
| 0383 | KEEP | LGTM observability reconciliation; Jenkins cite → bridge; tag transitional |
| 0388 | AMEND | doc-axis convention; reconcile to single D-DOCORG Diátaxis topology |
| 0389 | KEEP+AMEND | cloud-intelligence Bedrock-on-Talos; fold D-INTEL FINAL engine-relocation |
| 0390 | KEEP | cloud-intelligence v1 + proof layer; "S3"→SeaweedFS-own-store wording |
| 0391 | KEEP+AMEND | N-lane safety proof + console; data sources Forgejo/Jenkins → oya-ci API |
| 0393 | AMEND | Leptos app-shell (clean-supersede exemplar); stale 0513→0515; Proposed-0394 dep |
| 0476 | AMEND | `supersedes:[0421]`→`[0187]`; Zitadel rejected→adopted bridge; Forgejo; phantom 0409 |
| 0478 | AMEND | phantom `supersedes:[0457]`; `microservices/` home; 0509→0512 |
| 0479 | AMEND | phantom `supersedes:[0429]`; home; ClickHouse-as-endpoint vs D4 |
| 0480 | AMEND | phantom `supersedes:[0443]`; pressure-test oya-cost-vs-oya-meter-subsystem (D8) |
| 0481 | AMEND | phantom `amends:[0428]`+0409/0434; Forgejo; Proposed-0408 BLOCKER dep |
| 0482 | AMEND | keystone bespoke-doctrine (KEEP); phantom Tier-1 rows; non-ADR `amended_by` (§2A-4); Forgejo; insert D8 capacity gate |
| 0506 | KEEP | aws-lc-rs crypto + parity table (best); de-hardcode absolute path |
| 0507 | KEEP | webauthn-rs RP; amend stale 7-crate scaffold → single-crate/BC per 0512 |
| 0508 | KEEP | OpenSK authenticator; fix dangling 0483/0484; bind silicon ambition to D8/D-SEQ |
| **0509** | **SUPERSEDE** | status drift — 0512 supersedes it but still Accepted (`superseded_by:[]`); flip + back-pointer; repoint 5 citers (FC-5/§2A) |
| 0512 | KEEP | best-engineered, founder-locked governing ADR; watch Proposed-0392/0408 dep; add ADR-INDEX row (FC-5) |
| 0515 | AMEND | ratified CI/CD canon; eradicate Forgejo (`:76,96`); ratify/gate Proposed-0408/0392; **add 0160/0374/0380 to supersedes (P1/P3/P4)** |

**Disposition totals (reconciled across lanes; 169 unique Accepted ids):**
- **KEEP** (incl. KEEP+minor-AMEND): **~108**
- **AMEND:** **~50**
- **SUPERSEDE (9 clean):** 0015, 0053, 0103, 0136, 0137, 0143, 0160, 0187, 0509
- **SUPERSEDE — bridge / superseded-on-cutover / mark-historical (3):** 0374 (P3), 0380 (P4), 0239 (banner)
- **ARCHIVE (5):** 0057, 0097, 0101, 0102, 0138

> NOTE on accounting: the bridge/historical class (0374/0380/0239) uses the **keystone**
> `superseded-on-cutover` status value (§3), not clean archival; their EDGES are owned by
> `contradiction-audit/00-SUPERSESSION-COMPLETENESS.md` P3/P4 + C-17. The strict clean-SUPERSEDE set is 9;
> the strict clean-ARCHIVE set is 5. The §2A status-vs-edge drifts (0316/0358/0482/0052/0363/0054) are
> integrity fixes folded into each ADR's disposition above, not separate disposition rows; 0316/0358 are
> Proposed (outside the 169 Accepted) and appear only as edge/status fixes.

---

## SECTION 5 — COVERAGE + CAVEATS

**Accepted ADRs on disk (verified):** `grep -ric "^status: accepted"` over the decisions dir = **169**
of **349** total. Lane 8's looser front-matter grep reported **171**; the 2-ADR delta is table-form
`- Status: Accepted` headers vs YAML `status:` — both were audited. The strict single-line count is **169**.
The contradiction-audit independently enumerated **347** `ADR-*.md` (two non-`ADR-*`-named files excluded).

**Per-lane audit (accepted-adr-audit):** lanes 0–8 over ranges 0001–0034 / 0051–0095 / 0096–0132 /
0133–0182 / 0184–0210 / 0222–0335 / 0350–0391 / 0393–0515 / cross-cutting. All `status:Accepted` read in
full unless capped (below). Verification re-opened all 41 CRITICAL/HIGH at cited lines (CONFIRMED 41,
REFUTED 3). The supersession-audit verification re-opened flat-crates (6 sections), §2A (7 drifts, scope
347, Case-B=0 independently reproduced), and P1–P5 + 3 exclusions (0 substantive refutations).

**Caveats carried forward (no silent caps):**
- **Read-capped ADRs (lane 5):** 0258 (→L794/1108), 0329 (→L1199/2570), 0330/0331/0332/0334 partial,
  0131 (lane 2, →L160). Findings are scoped to the read region and marked; tails were appendices/footprints.
- **Lane-8 cross-range blind spot:** lane 8 deep-read only the 5 vendor/identity/CI clusters; a *duplicate
  decision* outside those clusters and using none of {kafka, forgejo, istio} could be missed. Per-domain
  lanes 0–7 cover those ADRs individually, so the residual is **cross-ADR duplication**, not a missed ADR.
- **Sampled, not line-exhausted:** the ~50-entry stale-LOCATION/gate ref table (flat-crates §1) and the
  ~650 surviving-NAME comment-headers were sampled (7/7 accurate); the 374-file `microservices/` superset
  was not re-verified in the supersession lane (out of scope of those 3 lanes).
- **Machine index untrusted:** `docs/machine-readable/decisions.json` tops out at ADR-0392 (DRIFTED) —
  NOT used by either audit. `.claude/worktrees/**` excluded as stale clones.
- **Phantom/absent ids verified by `ls` (not assumed):** 0012, 0033, 0088, 0397, 0409, 0411, 0416, 0421,
  0428, 0429, 0434, 0443, 0457, 0477, 0483, 0484, + the 0055-rename-plan-v3 file.
- **The 6 refuted carried, not dropped:** R-1 (0335 vcs-orchestrator — full refute; residual LOW principal-
  namespace nit), R-2 (Forgejo "15 Accepted" count — refuted; 27-file substance holds), R-3 (0028:18/:136
  ads-gate half — refuted; 0031 half = H-22, 0028's own SPOF = H-23). The contradiction-audit logged 3
  citation-precision deltas (0374 status `:3` not `:6`; 0131 naming literally `oya-<ms>-<bc>-<layer>`;
  ADR-INDEX resolves at `docs/ADR-INDEX.md`) + the accepted-audit logged ≤3-line anchor drifts (0062:41,
  0062:130, 0143:68, 0173:187, 0258 v2) — none alter any verdict; corrected anchors are used above.

---

## APPENDIX — TRACEABILITY

- **CRITICAL ↔ supersession-P map:** C-12=P1 (0160) · C-13=P2 (0187 + 0476 half-edge) · C-18=P3 (0374) ·
  C-19=P4 (0380) · C-17=0239 banner (0335 prose-only) · P5=0335 superseded-on-cutover marker (→ §4 0335 AMEND).
- **flat-crates FC-set ↔ C-3:** FC-1 (0015 status flip) · FC-2 (governance-lanes/flat-crates gate, HIGHEST
  SEVERITY) · FC-3 (delete untracked `crates/`) · FC-4 (~50 LOCATION/gate refs) · FC-5 (ADR-INDEX + back-pointers).
- **§2A status-vs-edge drifts ↔ H-SE rows:** 0015 (=C-3/FC-1) · 0316 · 0358 · 0482 · 0052 · 0363 · 0054.
- **Both-artifact corroborated:** C-3/FC, 0015, 0160 (C-12/P1), 0187 (C-13/P2), 0374 (C-18/P3),
  0380 (C-19/P4), 0239 (C-17), phantom-0150 (C-16), 0509, foundry cluster (C-11/H-33), 0335 (P5).
