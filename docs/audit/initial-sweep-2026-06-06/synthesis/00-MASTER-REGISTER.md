# 00 — MASTER REGISTER (consensus-ready synthesis — MERGE-FINAL)

> **Two-repo ADR audit — initial sweep 2026-06-06.** SYNTHESIS-MERGE lead rollup over the **three complete per-ADR extracts** (`synthesis/_partial-{1,2,3}.md` = full 372-block coverage of all 26 LINUX + 346 SOURCE ADRs), the keystone canonical-posture-and-supersession map, the 5 cross-tension digests, the 2 hyperscaler-lens passes, the ideas/promotion register, and the prior partial synthesis (all prior findings carried forward).
> **READ-ONLY pass.** No audited doc was amended. This is the consensus vehicle for the founder `/deep-interview`. It is an INDEX over the durable companion artifacts; nothing load-bearing lives only here.
> **Companion artifacts:** `01-ADR-DISPOSITION-TABLE.md` (every ADR, both sides — 231 rows, 100% coverage) · `02-DECISION-ATOM-LEDGER.md` (the masterplan backfill spec) · `03-PROPOSED-RESOLUTION-LEDGER.md` (all Proposed → ratify/drop + door-class — *no unaccounted proposals*) · `04-DOMAIN-TAXONOMY.md` (closed enum + per-domain read-set index).

---

## 1. EXECUTIVE SUMMARY

**State of the two corpora.** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`) carries **346 ADR files on disk = 345 unique ids + 1 duplicate-numbered (dup-0377)** (range 0001–0514, documented gaps). LINUX (`~/Developer/linux`, the substrate PILOT/staging) carries **26 ADRs** (0001–0026), all `Accepted`/`accepted-with-reservations`, every one tagged to renumber into the source sequence (→ 0515+) on merge. The two are **deliberately divergent, not in error**: SOURCE *assembles* a best-of-breed OSS substrate and owns-when-proven; LINUX *owns from the kernel up* but writes everything to the Linux syscall ABI so it runs ON the source stack today and only replaces it under a proven-over-a-span ratchet. The shared seam (Linux ABI) plus a shared own-when-proven ratchet (LINUX 0019/0020/0022 ≈ SOURCE 0173/0211) means **most cross-side tensions are reconcilable overlaps, not flat contradictions** — the genuine forks are a small set of founder calls in §7.

**Coverage is now 100% (the MERGE delta, grep-verified against disk).** The prior synthesis flagged source chunks 2/4/9 as `[GAP]` and 0187–0514 as `[POSTURE]`-only, and under-counted the corpus at "205 rows." The three complete extracts + a disk diff close both: the disposition table now carries **all 345 on-disk source ids + 26 LINUX = 372 rows** (verified by `comm` of on-disk-ids vs table-ids → zero missing, zero phantom, after adding the two stragglers ADR-0381 + ADR-0482). **Every on-disk ADR id carries an explicit decision_atom + truth_flag + disposition.** No `[GAP]` rows remain. The only residual is a *branch-locality* caveat — the 0476–0482 bespoke-service/roadmap ADRs supersede/relate Phase-1 predecessors (0421/0457/0429/0443/0428 + the 0397–0451/0483–0508 band) that live partly on `origin/dev`, not this branch.

**The auto-reconciliation (wm4gkcey5) is NOT "plain wrong."** Every theme that re-verified the recent LINUX edits found them internally coherent and self-aware (each carries a `review_note`; ADR-0018 honestly records `consensus=FALSE` and that the framekernel boots as a QEMU guest; ADR-0021 removed phantom citations; ADR-0001/0020 walk back "eliminate Postgres" toward "own the differentiator layer"). The **one residual defect**: LINUX ADR-0001's Postgres reconciliation is **half-applied** — line 36 still reads "eliminate external DB dependencies" while lines 38/115/136 say "PostgreSQL+Citus is retained." Finish that scrub.

**Headline disposition counts — TRUE counts over 100% coverage (full table in `01`):**

| Side | rows | KEEP | AMEND | ARCHIVE | SUPERSEDE/MERGE | RATIFY (Proposed→accept) | DROP |
|---|---:|---:|---:|---:|---:|---:|---:|
| SOURCE | 346 (345 ids + dup-0377) | ~110 | ~150 | ~30 | ~7 | (≈122 of the KEEP/AMEND rows are also Proposed→RATIFY, see `03`) | 3 |
| LINUX | 26 | 18 | 7 | 1 (0016→0015 dup) | — | — | 0 |
| **Total** | **372** | | | | | | |

Net: the corpus is **overwhelmingly sound decisions wearing dead vocabulary.** The dominant AMEND driver is mechanical — retired `foundry` brand, `Redis`→Valkey, `Kafka`→Pulsar, superseded cross-refs, stale `proposed` status, missing masterplan binding — not architectural error. True ARCHIVEs are the agentic-VCS/GitHub-automation cluster (0110/0112/0113/0124), the grit/icm cluster (0052/0053/0054/0103), and clean supersessions (0042→0383, 0046→0192, 0121/0120→0375, 0183→0379, 0140/0141→0145, 0107→0105, 0170→0394, 0359→0511, 0372→0393). The **3 true DROPs** are 0325 (prices a retired primitive), 0316 (superseded by 0329), 0349 (Jenkins-half only). (ADR-0352 is AMEND-MANDATORY, not a DROP.)

**Biggest risks (ranked).**
1. **Masterplan authority is undecided in the human projection but ALREADY DECIDED on disk.** `MASTERPLAN.md` still declares itself a `compatibility_projection` (not authority) and the two design docs contradict — yet **ADR-0364 AND ADR-0365 are both `Accepted`** and explicitly build the masterplan as a GENERATED projection from accepted `planning_impact:true` ADRs, with a drift gate that *fails on a hand-edited masterplan*. The fork the keystone calls "open" is **resolved generated-from-ADRs in the ratified ADR log.** See §2 — single highest-leverage item; it reframes the founder's question.
2. **Data-integrity corruption from un-guarded find-replace sweeps (founder "plain wrong" warning, CONFIRMED).** (a) `MVP→minimum-shippable-tier` corrupted the Korean regulatory term **KCMVP → `KCminimum-shippable-tier`** in 8 files / 31 occurrences, including the GLOSSARY that *defines* the term and HSM/secrets ADR-0043 (×12) + ADR-0002. (b) ADR-0006 carries "Ontology renamed to Ontology" (×2, a destroyed Object-Graph→Ontology rename). (c) ADR-0025 `oya-foundry-* governed by same team as oya-foundry-*` tautology; ADR-0352 "Tenant RBAC view plus Tenant RBAC view." A corpus-wide grep-and-restore for `*-shippable-tier` and self-referential renames MUST run before any backfill.
3. **Phantom / dangling canonical anchors poison any generated-from-ADRs masterplan.** The entire SOURCE Cedar-engine chain (0243/0246/0294 + 0144) `amends`/cites **`ADR-0150 cedar-policy-engine`** — but on-disk **ADR-0150 is cursor-pagination** (**≥47 files cite the phantom ADR-0150-cedar-policy-engine anchor**); there is **no `Accepted` Cedar-engine ADR anywhere** (the real separation ADR is 0183→0379). ADR-0476 `supersedes` non-existent 0421 + four phantom parents; the Cedar-as-`ADR-0083` mis-cite propagates across 0476/0478/0479/0480/0481 (0083 is rust-error-handling); ADR-0057 `supersedes` non-existent `ADR-0055-v3`; ADR-0069 cites non-existent 0088 ×2; ADR-0377-kafka cites non-existent 0397/0436. Under generated-from-ADRs every dangling edge is a build failure.
4. **The one hard live contradiction: identity build-vs-buy.** ADR-0187 (Zitadel, Accepted, "rejects self-built IdP") vs ADR-0476 (oya-identity bespoke Rust, Accepted + founder-locked, "rejects Zitadel by name"). Two Accepted ADRs make opposite rulings; 0187 is never marked superseded; 0394 still lists it "open" and doesn't cite 0476. Founder-locked 0476 governs → 0187 must be marked `superseded_by:[0476]`, Zitadel demoted to a Phase-1 bridge.
5. **Masterplan binding is ~8.8% today.** The vast majority of true, load-bearing decisions carry no `masterplan_ref`/`planning_impact`, and many header-only ADRs (0093/0094/0095/0130/0146/0149/0150/0151) carry no YAML front-matter at all — so under EITHER reading they are invisible to the masterplan. This is the bulk of the backfill, enumerated in `02`.
6. **Decision-debt: ~131 Proposed ADRs, now fully resolved.** `03-PROPOSED-RESOLUTION-LEDGER.md` accounts for every one (RATIFY ≈122 · DROP 3 · AMEND-MANDATORY 1 · RENUMBER-then-RATIFY 1 · KEEP-as-Proposed-by-design 1) with door-class. **Zero unaccounted proposals.**
7. **Number collisions guaranteed on merge.** Duplicate source ADR-0377 (kafka-to-pulsar Accepted vs forge-board Proposed); all 26 LINUX ids collide with source 0001–0026. Renumber LINUX → 0515+; renumber the forge-board 0377.

---

## 2. THE PIVOTAL FORK — masterplan AUTHORED-as-SSOT vs GENERATED-from-ADRs

This is the keystone §4 open founder question and it gates HOW every true decision is backfilled.

- **Option A — masterplan-as-AUTHORED-authority** (`planning-ssot-drift-prevention.md`): `masterplan.json` *is* the one planning authority; ADRs + specs **bind into it** via a `planning-ssot-coverage` gate (`masterplan_ref`, bidirectional, supersession-aware). Found only **8.8% ADR binding** today. The founder's phrasing ("masterplan = single source of truth; backfill it") superficially reads this way.
- **Option B — masterplan-GENERATED-from-ADRs** (`planning-ssot-consolidation.md` + the *ratified* ADR-0364/0365): ADRs are the authored, immutable SSOT (append-only; supersede, never edit); `oya gen masterplan` reads accepted `planning_impact:true` ADRs, topo-sorts by `depends_on`/`supersedes`, emits the masterplan; status is *derived* from `verified_by` gate output; a masterplan-drift gate fails on any hand-edited masterplan. Precedent: Kubernetes KEP. Proposes re-founding the log from ADR-0000 with `consolidates:` provenance.

**Decisive evidence (verified on disk): the fork is already resolved toward B in the ratified ADR log.** `ADR-0364` is **`status: Accepted`** ("Make the masterplan a GENERATED projection of the ADR decision log… eliminating the parallel hand-maintained planning sources that drift"); `ADR-0365` is **`status: Accepted`** (the propagation engine + `propagation-drift` gate). ADR-0327 (status-derived-from-gates) and ADR-0280 (DAG-spec-as-authority, the cleanest worked example) vote the same way. The only reason the keystone could call it "open" is that **`MASTERPLAN.md` still self-declares `shape: compatibility_projection` and `planning-ssot-drift-prevention.md` was never reconciled to 0364/0365** — the human projection lags the ratified decision. ADR-0352 (frozen-stack handoff) is the live counter-pull.

**RECOMMENDATION: ratify Option B (generated-from-ADRs) — it is already the accepted ADR-0364/0365 design — and make "backfill the masterplan" concrete as "author/clean the ADR front-matter so the generator produces the right masterplan."**

**What B implies (the backfill precondition gate):**
1. **Front-matter hygiene is mandatory, not cosmetic.** Every KEEP/AMEND ADR needs `planning_impact`, `domain` (closed enum, see `04`), `depends_on`/`supersedes`, `status` derivable from gates. The ~91% unbound + header-only ADRs get YAML front-matter *first*.
2. **No-dangling / no-reuse ID invariant becomes load-bearing.** Every phantom edge (risk #3) is a generator build failure → fix or remove.
3. **Stale `status: proposed` on canonical keystones must be resolved** (`03` does this — all ~131 → ratify/drop with door-class).
4. **The re-founding (ADR-0000+) is in-scope and already designed.** LINUX 0001–0026 fold in renumbered to 0515+.

> **DECISION-NEEDED-FROM-FOUNDER (FORK, door:one-way):** *"ADR-0364/0365 are already Accepted and make the masterplan a generated projection of the ADR log. Confirm B (generated-from-ADRs) is canonical, retire MASTERPLAN.md's `compatibility_projection` framing and the contradicting `planning-ssot-drift-prevention.md` authored-authority design, and treat 'backfill the masterplan' as 'clean the ADR front-matter so the generator is correct'? Or override 0364/0365 back to authored-authority?"*

---

## 3. CONTRADICTION REGISTER (cross-side + intra-side), ranked by leverage

Each row: positions · which governs · surgical resolution · founder call. "Surface, do not resolve" items flagged FOUNDER-CALL.

### C-1 — Forge: GitHub vs Forgejo vs bespoke-VCS (THREE-way) — **FOUNDER-CALL, highest leverage**
- **Positions:** Founder directive = **GitHub `jason931225/oyatie`** (echoed by ADR-0017 slug retention + `migration/source-consolidation-plan.md §0.4` "GitHub IS canonical; Forgejo refs are stale"). Source canon = **Forgejo** self-hosted canonical host, GitHub mirror (ADR-0363/0374/0387/0369), ADR-0363 explicitly rejecting GitHub-as-substrate. Long-horizon = **bespoke hyperscaler monorepo-VCS** declared destination, Forgejo transitory (ADR-0510, numeric-trigger cutover).
- **Governs:** Unresolved by design. The founder's directive settles the **host** layer; source canon retired GitHub as the **automation substrate** — different layers; the conflict is real at the gate-sink + merge-substrate.
- **Resolution:** One masterplan "forge" node with explicit layers: host = founder's call; automation-substrate = forge-neutral gates (not `gh api`/Actions). Every gate/merge/status ADR (0041/0124/0139/0170/0171/0173/0374/0387) resolves off this one ruling. ADR-0124 ARCHIVEs either way (salvage projected-merge-state→Tide). ADR-0387 (GitHub sink) vs ADR-0374 (Forgejo sink) for the *same* `ci-webhook-gateway` reconcile to one sink. ADR-0221's "oya vcs canonical; git forbidden" must be retired vs plain-git canon.
- **DECISION-NEEDED (door:one-way):** *"Is the canonical forge GitHub (host directive), Forgejo-transitory (0363/0510), or bespoke-VCS-destination — and is the CI automation substrate forge-neutral or GitHub-pinned? What replaces the dead ADR-0113/0124 merge-gate?"*

### C-2 — CI destination: Argo Workflows vs bespoke `oya-ci` Prow — **FOUNDER-CALL**
- **Positions:** ADR-0511 (Proposed) names **Argo Workflows** the destination and *defers* a bespoke controller; ADR-0513 (**Accepted, founder-locked**, 1 day later) builds exactly that bespoke `oya-ci` Prow *as* the destination and never cites Argo. Different destinations, no cross-ref. (Hyperscaler-lens: 0513 is the own-too-early risk — bespoke Prow before the OSS substrate provably fails.)
- **Governs:** ADR-0513 (only Accepted/founder-locked). Add reconciliation cross-ref; ADR-0511 demotes to the adopt-CNCF alternative or transitory orchestrator. Founder Q: is oya-ci gated behind a numeric trigger (like 0510 gates bespoke-VCS) or day-0?
- **DECISION-NEEDED (door:one-way):** *"Argo Workflows (adopt-CNCF, 0511) or bespoke-Rust oya-ci Prow (0513, founder-locked) as the masterplan CI node — and is the bespoke build day-0 or numeric-trigger-gated?"*

### C-3 — Data tier: LINUX owned-engine vs SOURCE Postgres+best-of-breed — **FOUNDER-CALL** (keystone fault #1)
- **Positions:** LINUX ADR-0001 (owned Rust multi-model engine) vs SOURCE ADR-0045/0179/0184 (Postgres+Citus OLTP, pgcat pooling, Postgres 18.4 Tier-1 SoT). pgcat (0179) is the concrete artifact that breaks if "eliminate Postgres" governs. ADR-0194 ("replacing Postgres is out of scope for any plausible roadmap") is the sharpest restatement.
- **Governs:** **Reconcilable, and LINUX already walked it back on disk** — ADR-0001 lines 38/115/136 retain Postgres+Citus as OLTP, scope the owned engine to the *differentiator/etcd-replacement* layer. **One residual defect:** line 36 still says "eliminate external DB dependencies" — finish the scrub. (ADR-0001's cited spec `staged-ownership-roadmap-canonical.json:245` does NOT exist in the LINUX tree.)
- **DECISION-NEEDED (door:one-way):** *"Own the entire data tier eventually (Postgres retired) or own only cloud-data with Postgres+Citus as the permanent OLTP substrate of record? Does cloud-data also absorb the vector tier + FTS, or do Milvus/search stay permanent separate substrates? Where exactly is the boundary?"*

### C-4 — Identity build-vs-buy: Zitadel vs oya-identity — **the one hard live contradiction**
- **Positions:** ADR-0187 (Zitadel, Accepted, rejects self-built IdP) vs ADR-0476 (oya-identity bespoke Rust, **Accepted + founder-locked**, rejects Zitadel by name). ADR-0394 lists it "open," doesn't cite 0476; ADR-0507 "bespoke WebAuthn RP" conflicts with 0187's "Zitadel is the WebAuthn RP." The crypto cluster (0506/0507/0508; founder-locked 2026-05-28, door:two-way) hangs off the 0476 resolution.
- **Governs:** Founder-locked **ADR-0476**; Zitadel/Keycloak demoted to Phase-1 bridges.
- **Resolution:** Mark 0187 `superseded_by:[0476]`; fix 0476's dangling `supersedes:[ADR-0421]` (non-existent — was Keycloak ever an ADR, or should it be `supersedes:[0187]`?) + phantom parents + Cedar-0083 mis-cite; retarget 0394's identity pre-req to 0476.
- **DECISION-NEEDED (door:one-way):** *"Confirm oya-identity (0476, founder-locked) is canonical with Zitadel/Keycloak as bridges, and 0476 supersedes 0187? And does 0476's `supersedes:[ADR-0421]` mean Keycloak (mis-numbered) or should it be 0187?"*

### C-5 — Policy: LINUX owned compile-to-Rust engine vs SOURCE Cedar — **reconcilable; needs cross-ref + a phantom fix**
- **Positions:** LINUX ADR-0021 (owned, typed, compile-to-Rust, *explicitly Cedar-compatible*, vendors `cedar-policy` as day-0 adapter + differential oracle) vs SOURCE Cedar-as-universal-gate (0243/0246) + Kubewarden admission (0379).
- **Governs:** **Not a flat contradiction — both converge on "Cedar = external-standard contract, owned PARC = the engine behind it."** The only open question is the ratchet *trigger threshold*.
- **Resolution:** One masterplan policy atom + merge cross-ref. **DATA-INTEGRITY:** the canonical `ADR-0150-cedar-policy-engine` that 0243/0246/0294 `amend` does not exist (on-disk 0150 = cursor-pagination). Re-author the missing engine pick or fold into 0243; do NOT archive 0150-cursor-pagination.
- **DECISION-NEEDED (door:one-way):** *"Permanent owned compile-to-Rust evaluator behind the Cedar contract (LINUX 0021), or vendored Cedar engine forever with ownership limited to the asset layer (0183/0243)? If owned, what fires the ratchet? And re-author the phantom Cedar-engine anchor."*

### C-6 — Isolation: LINUX framekernel/own-the-host vs SOURCE Talos+Kata+containerd — **FOUNDER-CALL** (keystone fault #3)
- **Positions:** LINUX framekernel/Capsule/owned-VMM (0014/0017/0018) vs SOURCE Talos+Kata+Cloud-Hypervisor+wasmtime (0375/0147/0338/0200).
- **Governs:** **SOURCE governs now** (Accepted, dogfood-deployed). LINUX ADR-0018 itself concedes H1 (yrs 1–3) runs the flagship on Linux with an external hypervisor and records `consensus=FALSE`; "we are the host" is true only at uncommitted H2. LINUX is the gated successor. Sub-tension **T-4 (the DEFAULT):** LINUX 0023 assume-breach microVM-for-all (authorship NOT a trust axis) vs SOURCE 0338 runc-for-first-party (kill the 30–40% Kata density tax) — opposite defaults for ~60 µservices. Sub-tension **T-8:** own full L0–L8 (0017, incl. L7 BuildKit-class build engine) vs reuse containerd+runc behind CRI (0014) — 0017 itself flags L7 as "a second mountain."
- **DECISION-NEEDED (door:one-way):** *"Is framekernel/owned-VMM the committed REPLACEMENT for Talos+Kata (a masterplan TARGET) or a research track that must win the conformance scorecard first? Fleet isolation default for first-party: runc (0338, density) or assume-breach-microVM (0023, security)? And is the L7 build engine owned day-0 or DEFER_VENDORED (reuse BuildKit)?"*

### C-7 — Progressive-delivery + chaos: Flagger vs Argo-Rollouts; Chaos Mesh vs Litmus — **intra-source, must reconcile**
- ADR-0160 picks **Flagger** and rejects Argo-Rollouts; keystone §3 + ADR-0040 name **Argo-Rollouts** canonical. ADR-0165 (Chaos Mesh) vs Litmus (better Argo integration). Governs: Argo-Rollouts (the stack is Argo-centric). **DECISION-NEEDED:** *"Flagger (0160) or Argo-Rollouts (§3/0040)? Chaos Mesh or Litmus (0165)?"*

### C-8 — Search engine: pgroonga vs Meilisearch — **intra-source, one must archive**
- ADR-0047 pins pgroonga→Tantivy; Accepted ADR-0184 pins Meilisearch-1.9→Tantivy and never mentions pgroonga. Governs: 0184. Archive pgroonga core of 0047; re-home 0048 KR-morphology onto Meilisearch/Tantivy. (advisory — 0184 governs.)

### C-9 — Eventing-backbone status drift — **resolved by supersession, needs the edge**
- ADR-0005 (Kafka, still `proposed`) superseded-in-fact by ADR-0377-kafka-to-pulsar (Pulsar 4.x + Oxia, Accepted) but never marked superseded; 0003/0004/0154/0166/0169/0172/0195/0246/0249/0252/0356 carry stale Kafka deps. Outbox/CloudEvents/partitioning survive (0153). Mark 0005 `superseded_by`; mechanical Kafka→Pulsar repoint. (0377-kafka itself dangles 0397/0436 — fix.)

### C-10 — Admission engine: Kyverno vs Kubewarden — **resolved, mechanical repoint**
- ADR-0183 (Kyverno separation) Superseded by ADR-0379 (Kubewarden default; Cedar/admission separation principle retained). Repoint stale citers (0039/0117/0181/0182/0191/0200/0202/0250/0251/0361). Mechanical.

### C-11 — Observability authority: ADR-0383 vs ADR-0186 — **establish authority**
- ADR-0383 (map-canonical, superseded 0042) vs ADR-0186 (finer 5-stage layering) compete without cross-referencing; 0186 also mis-cites 0130 as "agentic-SLO-promotion" (means 0139). Make 0383 authoritative; 0186 layers under it.

### C-12 — Workflow engine: bespoke FSM+DAG vs adopt Temporal — **FOUNDER-CALL** (hyperscaler-flagged)
- ADR-0035 builds a bespoke hybrid FSM+DAG engine rejecting Temporal day-0. **False-contradiction note (do not log as a conflict):** ADR-0035 rejects "Argo Workflows" as a *business-process* engine while ADR-0511 adopts Argo Workflows as the *CI/CD* orchestrator — different layers, both correct. **DECISION-NEEDED:** *"Own a bespoke workflow engine day-0 (0035) or adopt a durable-execution substrate (Temporal-class) behind a port and own only the per-tenant-versioning + jurisdiction overlay until proven?"*

### C-13 — Time/clock: LINUX HLC-no-TSO vs SOURCE HLC+TrueTime-Tier-4 — **clarification, not policy**
- LINUX ADR-0006 "do not reserve a TSO path" vs SOURCE ADR-0252 "TrueTime provider at Tier-4." Same HLC family. AMEND ADR-0006 to keep the Clock port swappable (consistent with its own hexagonal design) so a Tier-4 TrueTime adapter is addable without a kernel rewrite. Founder Q: does the owned engine ever run inside a Tier-4 IL5/financial cell?

### C-14 — Internal/self-contradictions to fix at amend time (intra-doc; not founder calls)
- **ADR-0139** asserts BOTH "no git-tracked JSONL ledger; Mimir recording-rules ARE the ledger" AND a canonical `registry/promotion-eligibility.jsonl` — one must be canonical.
- **ADR-0109** §Decision-6 schedules a migration its §Migration-policy forbids (two-pass authoring).
- **ADR-0147** post-amendment body still says "gVisor by default / three RuntimeClass objects" while the amendment makes Cloud-Hypervisor primary.
- **ADR-0098** best-effort fsync on audit-adjacent JSONL contradicts ADR-0096's crash-atomic mandate on the same files (also hyperscaler MISALIGNED — §4).
- **ADR-0335 vs ADR-0363** "foundry eradicated" claim is FALSE: `oya-foundry-*` survives across 59 ADR files. AMEND 0363's "eradicated"→"sequenced/pending"; promote ADR-0347 (Proposed→Accept) as the bulk-rename mechanism.
- **ADR-0348/0351 vs 0333** absorb-then-re-extract churn (cell=pattern vs cell-rebalancer/cell-lifecycle re-created as µsvc) — boundary-thrash; lock ownership to 0351.
- **ADR-0391** mandates a SolidJS DevOps console while ADR-0393 retires SolidJS→Leptos — stale-on-arrival.

### C-15 — Number/title-drift corrections (data-integrity; load-bearing for any generated index)
- **ADR-0150 = cursor-pagination** on disk (NOT Cedar/Kyverno separation as the map + 0148 + 0182 claim). Re-key the supersession graph: policy-engine-separation = ADR-0183→0379. Do NOT archive 0150.
- **ADR-0153** cited under two titles (outbox-pattern [correct] vs "observability backplane" in 0148).
- **ADR-0055 number-reuse:** 0057/0056 point at a phantom `ADR-0055-v3-rename`; the only on-disk 0055 is object-graph→ontology (live, NOT superseded).
- **Cedar mis-cited as ADR-0083** across 0476/0478/0479/0480/0481 (0083 = rust-error-handling; real Cedar = 0099/0243/0246).
- Duplicate **ADR-0377** (kafka-to-pulsar Accepted vs forge-board Proposed) — renumber the forge-board one.
- **Bominal foreign-corpus dependency** (root ADR-0060; 0059/0061–0065/0092 cite unindexed "Bominal ADR-####") — incompatible with a self-contained generated masterplan; absorb-or-track ruling needed.

### C-16 — Autonomy-ceiling semantic authority + ownership home — **FOUNDER-CALL, BLOCKER** (restored from losslessness gap)
- **Positions:** ADR-0007 (Cedar + autonomy ceiling) treats T1–T4 as an **advisory-centric** persona/autonomy ceiling — guidance enforced by policy review. ADR-0022 (autonomy ceiling, Cedar per-invocation) treats T1–T4 as **execution-centric runtime-enforced** — a hard gate checked at every invocation. These are meaningfully different semantics for the same T1–T4 ladder. Additionally, post-ADR-0335 (foundry-absorption), the ownership home of the ceiling mechanism is unclear: does it live in `intelligence` (the absorbed foundry, which ran the per-invocation Cedar checks) or `governance` (the policy authority)?
- **Governs:** Unresolved. Both ADRs carry `Proposed` status; neither explicitly defers to the other on semantics. The cross-tension digest `cross-tension/policy-authz-autonomy-governance.md` (on disk) surfaces this as a real DECISION-NEEDED that was not carried into the main synthesis gate.
- **Resolution:** Founder ruling required before ratifying either 0007 or 0022.
- **DECISION-NEEDED (door:one-way):** *"Are the T1–T4 autonomy tiers advisory-centric (ADR-0007: persona guidance, enforced by policy review) or execution-centric runtime-enforced (ADR-0022: hard Cedar gate at every invocation)? And does the ceiling mechanism's ownership home live in `intelligence` (post-0335 foundry-absorption) or `governance`? Confirm which ADR's semantics governs — gates 0007/0022/0139/0144 and the policy-engine decision (#6)."*

---

## 4. HYPERSCALER-CHALLENGE FINDINGS (would Google/AWS/Azure do this?)

**The DOCTRINE is hyperscaler-correct; the DAY-0 PORTFOLIO is where the hubris risk lives.** The own-when-proven ratchet (SOURCE 0173/0211 value-anchored triggers + LINUX 0019/0020/0022 four-axis no-cherry-pick scorecard) is *exactly* what AWS/Google/Azure do; ADR-0211 even rejects "build everything in-house day-1." **Highest-value backfill: reconcile the two near-duplicate ratchets (OWN_DAY0/DEFER vs Class-A/B/C vs Tier-I/II/III) into ONE rubric** — kills a guaranteed merge conflict and resolves most "questionable" verdicts mechanically.

**MISALIGNED (a hyperscaler would not do this) — AMEND/scope-down:**
- **Jenkins as destination CI** (ADR-0359) — already self-corrected to transitory by 0511. (MASTERPLAN.md L126 still names stale "Jenkins required checks.")
- **GitHub-native automation substrate** (0041/0124/0139/0170/0171) — binding merge-gate logic to `gh api`/Actions is the exact SPOF 0359 fled. Forge-neutral gates; host = founder's GitHub call, substrate ≠ GitHub.
- **Best-effort fsync on audit-adjacent JSONL** (ADR-0098) — compliance gap; full durability (contradicts 0096).
- **Programmatic consumer-subscription auth** (ADR-0020/0384 — Claude Pro/ChatGPT Plus driven programmatically) — rejected on ToS/fragility; scope-down or archive that sub-decision.

**Two FACTUAL corrections:**
- **The absolute "no custom silicon" anti-scope (ADR-0032) is wrong** — Google (TPU), AWS (Graviton/Nitro), Azure (Cobalt/Maia) all built silicon. Soften "never"→"not in the day-0 horizon."
- **"Eliminate PostgreSQL" overstates ADR-0001's own clarification** (already reduced to a differentiator layer; Postgres+Citus reused) — materially more aligned than the keystone-flagged headline.

**QUESTIONABLE — all collapse to one axis (own-day-0 vs own-when-proven); FOUNDER scope calls:**
- LINUX **ADR-0015** full Rust k8s control-plane rewrite — *the most hubristic single bet*; EKS/AKS run upstream k8s. AMEND: own the cellular + owned-datastore differentiator under an etcd-v3 adapter, NOT the apiserver/scheduler rewrite.
- LINUX **ADR-0017** L0–L8 container platform (BuildKit-class L7 = "second mountain") — DEFER L7.
- SOURCE **ADR-0032** DCIM built before owning a DC — re-sequence to Phase-2.
- SOURCE **ADR-0035** bespoke workflow engine rejecting Temporal day-0 — adopt-then-own the overlay.
- SOURCE **ADR-0185** five fully-native client stacks — Google built Flutter to avoid exactly this; stage to 1–2.
- SOURCE **ADR-0058/0321/verticals** (medical/pharmacy/banking/insurance/ads/manufacturing as owned first-class µsvc) — no hyperscaler builds verticals; ISVs do. Founder scope ruling.
- SOURCE **ADR-0315** full SAP S/4HANA parity + 9 new µsvc — highest-stakes; vs founder GOAL ORDER "Linux-parity kernel FIRST."
- SOURCE **ADR-0249** 8 marketplace substrates day-one; **0300/0301** Tor-SecureDrop/survivor-safety universal substrates; **0027** SC4-AV robotics; **0028** greenfield mega-DC; **0029** 12-app suite; **0293** ICANN-grade HSM ceremony to gate internal self-mod.
- SOURCE **ADR-0133** one mega-BLOCKER conformance lane vs N per-axis scorecards.
- **The structural gap:** the ratchet is per-component but there is **no portfolio-level capacity gate** — ADR-0001/0015/0017/0025 are each *separately* Accepted with multi-year day-0 starts; summed, day-0 is NOT small. No hyperscaler built all of these in parallel. **NOTE — the founder already authored the doctrine that should carry this gate:** **ADR-0482 (Accepted, founder-locked, door:one-way) "Bespoke Substrate Roadmap"** declares "OSS = bridges not destinations; anything acceptable for bespoke, *timeline it appropriately*, keep bridges," with phased Tier-1/2/3 + per-component OSS bridge + quality-gated cutover (no hard-deadline). This IS the apex own-when-proven sequencing doctrine — but it asserts the *multi-decade ambition* without yet binding a **concurrent day-0 capacity budget** (which ONE substrate gets the senior team first). FOUNDER decision #8 = make 0482's "timeline it" concrete as a portfolio gate, not a new doctrine.

**CONFIRMED ALIGNED (the founder's "confirm the right ones" — KEEP + promote to named masterplan invariants):** own-when-proven ratchet; cellular arch + bounded blast radius (LINUX 0012/SOURCE 0248); assume-breach/strength-by-blast-radius (LINUX 0023 = BeyondProd/Nitro/NIST-800-207); evidence-gated promotion + claims-gate (0128/0129/0133/0135/0367); distroless/immutable node (0146/0375); Zitadel→oya-identity IdP path; Milvus; Pulsar+Oxia; **Valkey-not-Redis** (the literal hyperscaler Redis-relicense response); LGTM observability; Cedar+Kubewarden separation; Talos+CAPI+ArgoCD; Argo Workflows + Buck2; in-house AI substrate (0026/0220 = Bedrock pattern, eval-gated, NOT a frontier lab); owned VMM (0014 = Firecracker precedent, proof-gated); node-OS (0025 = Bottlerocket precedent). **The wasmtime/Firecracker/Kata "mix" the founder flagged is CORRECT** — different latency/attack-surface tiers (Lambda=Firecracker, Cloudflare=wasmtime), not a redundant pick. **ADR-0018 H2 kernel-replacement is a MODEL for carrying a moonshot honestly** (`consensus=FALSE`, time-boxed, budgeted, go/no-go-gated).

---

## 5. IDEAS / PLANS — promote or remove

**35 docs scanned. The corpus is overwhelmingly already-promoted** — the CI/Buck2/cloud-intelligence idea cluster maps to existing ADRs. Source's idea→ADR pipeline (ADR-0365) mostly worked; only **2 true new ADRs are owed** (both founder-gated).

**MUST-PROMOTE (grep-verified zero ADR coverage):**
1. **`agent-execution-controller.md` (PR #605)** — agent *execution* (run a CLI agent as a K8s Job → sealed `evidence-bundle.v1`), distinct from the inference gateway (0384) and intelligence substrate (0255). Founder gate: promote-as-narrower vs decline per ADR-0116/0363. **Carry forward:** ADR-0113's per-changeset cost budgets + override-frequency alarms before that superseded ADR is removed.
2. **`affected-gated-migration-engine.md` ("Sweep")** — risk-classed mass-transform Workflow + auto-quarantine + auto-merge-on-green; scope as a Tide client (ADR-0513); promote-narrow or fold-into-0513.

**FOLD-into-existing (9):** the amendment ADRs 0239/0353/0354/0355/0356 are **legitimate `amends:` decisions, NOT noise**; the rest fold to governing ADRs (best-of-both/oauth-pool→0384; agentic-slo→0139; buck2-native-ci-gate→0392/0408; nativelink→0514; oya-ci-prow→0513; pipeline-review/revamp/optimization→0511/0513/0514).

**REMOVE/ARCHIVE (11):** 3 `ideas/archive/*` (self-superseded by 0389/0390/0391, leave frozen); `single-bootstrap-omni-talos` (→0375, mis-filed live); both `rename-plan-*` + `cutover-cross-cutting-amendments` + `M01-foundation-cc-01-cutover/` (cutover executed); `hyperscaler-gap-closure-plan` + `post-cutover-program` (M01 residue); `IP-INTEL-MIGRATE-CANONICAL` (execution IP under 0509).

**KEEP-as-research (8)** (incl. the two SSOT design docs — the FORK literature) and **PILOT-SCAFFOLD/retire-at-integration (2 LINUX migration docs)** — note `source-consolidation-plan.md` carries the **founder's GitHub-is-canonical forge directive in writing** (the C-1 founder side).

---

## 6. THE "NOT NEEDED" LIST — content that is not a true decision for the masterplan

Per the founder's binding rule ("if it is not part of the masterplan, it is not needed") and the README ORPHAN class:

- **Executed one-shot cutover-mechanics records** (no surviving decision atom): ADR-0052/0053 (grit/icm inventory + sanctioned-primitives), 0103 (grit cutover), 0057 (Rename Plan v4 after Shard-1 landed), 0107 (tools-dir suffix, absorbed by 0105), 0138 (foundry six-path Strangler), 0143 (foundry per-BC pointer). *Salvage atoms first* (banned-direct-git from 0053; destructive-migration manifest from 0052; deprecation-lane from 0138→0139).
- **Cleanly superseded substrate ADRs** (decision lives in the successor): 0042→0383, 0046→0192, 0121/0120→0375, 0183→0379, 0140/0141→0145, 0359→0511, 0170→0394, 0110/0112/0113→0363, 0124→0363/0513, 0372→0393. Keep as frozen history; not masterplan nodes.
- **DROPs from `03`:** ADR-0325 (prices the retired tier ladder); ADR-0316 (superseded by 0329); ADR-0349 Jenkins-half (ArgoCD-half survives). (ADR-0352 is AMEND-MANDATORY per `03` — not a DROP.)
- **Ideas that are M01-execution residue** (§5 REMOVE list).
- **Bominal foreign-corpus** (root 0060) — absorb into native ADRs or fully archive; not a masterplan node as a foreign dependency.
- **MASTERPLAN.md's own stale lines** (`foundry` in FD-001 surface list; "Jenkins required checks") — not decisions; auto-corrected once the generator (B) runs.

**NOT on the "not needed" list (explicitly true decisions, common misread):** the amendment ADRs 0239/0353/0354/0355/0356; ADR-0150-cursor-pagination; the header-only API-hygiene ADRs 0093/0094/0095/0146/0149/0151; ADR-0134 (honest non-binding backlog — KEEP-as-Proposed by design); PR #605 agent-execution-controller; the crypto cluster 0506/0507/0508. These are real and must be captured.

---

## 7. FOUNDER DECISIONS REQUIRED (the gate — nothing amends until these are ruled)

1. **MASTERPLAN FORK (the keystone, door:one-way).** ADR-0364/0365 are already Accepted and make the masterplan a GENERATED projection of the ADR log. **Confirm generated-from-ADRs (Option B) is canonical** — retire MASTERPLAN.md's `compatibility_projection` framing + the contradicting authored-authority design, and treat "backfill the masterplan" as "clean the ADR front-matter so the generator is correct." (If overriding to authored-authority, say so — it contradicts two Accepted ADRs.) *Gates §2 + all backfill.*
2. **FORGE ruling (door:one-way).** GitHub `jason931225/oyatie` (host directive) vs Forgejo-transitory (0363/0510) vs bespoke-VCS-destination — and is the CI **automation substrate** forge-neutral or GitHub-pinned? *Gates C-1, ADR-0041/0124/0139/0170/0171/0173/0374/0387/0221/0377-forge.*
3. **CI destination (door:one-way).** Argo Workflows (0511) or bespoke `oya-ci` Prow (0513, founder-locked) — and day-0 or numeric-trigger-gated? *Gates C-2.*
4. **DATA TIER (door:one-way).** Own the entire data tier (Postgres eventually retired) or own only cloud-data with Postgres+Citus permanent OLTP? Does cloud-data absorb vector+FTS or do Milvus/search stay separate? Where is the boundary? *Gates C-3, LINUX 0001 final scrub.*
5. **IDENTITY (door:one-way).** Confirm oya-identity (0476, founder-locked) canonical with Zitadel/Keycloak as bridges; 0476 supersedes 0187; resolve 0476's `supersedes:[0421]` (Keycloak mis-number vs should-be-0187). *Gates C-4 + crypto cluster 0506/0507/0508.*
6. **POLICY ENGINE (door:one-way).** Permanent owned compile-to-Rust evaluator behind the Cedar contract (LINUX 0021) or vendored Cedar engine forever (0183/0243)? If owned, what fires the ratchet? Re-author the phantom Cedar-engine anchor (0150). *Gates C-5.*
7. **ISOLATION END-STATE + DEFAULT (door:one-way).** Is framekernel/owned-VMM the committed replacement for Talos+Kata (masterplan TARGET) or research-gated? Fleet isolation default for first-party: runc-for-density (0338) or assume-breach-microVM-for-all (0023)? Is the L7 build engine owned day-0 or DEFER_VENDORED? *Gates C-6.*
8. **OWN-WHEN-PROVEN — reconcile the two ratchets into ONE rubric**, and add a **portfolio-level capacity gate** (which ONE from-scratch substrate is the crown-jewel day-0; the rest DEFER_VENDORED). The founder's **ADR-0482 (Accepted, door:one-way) "Bespoke Substrate Roadmap"** already supplies the doctrine ("OSS = bridges; timeline it; keep bridges") — make its "timeline it appropriately" concrete as a binding concurrent-day-0 budget. *Resolves most §4 "questionable" verdicts mechanically.* **Sub-question — six shared substrates (ADR-0001 `consensus_needed: yes`):** is the "six shared substrates" count frozen as the canonical organizing invariant, or resolved-by-0335 (foundry-absorption collapsed foundry into intelligence+governance, altering the substrate boundary count)? If 0335 already settles it, confirm explicitly; otherwise add as a founder ruling.
9. **SCOPE/BREADTH bets** (each a yes/no, door:one-way): workflow engine own-day-0 vs adopt-Temporal (0035); DCIM day-0 vs Phase-2 + no-custom-silicon soften (0032); five native clients vs 1–2 (0185); verticals as owned µsvc vs substrate+ISV (0058); SAP-parity + authorize-9-new-µsvc (0315); 8 marketplace substrates day-one (0249); detection 8-family day-0 (0307); LINUX k8s control-plane full rewrite vs etcd-adapter differentiator (0015); L7 build engine own vs defer (0017); marketplace M&A/JV over-claim (0314); ICANN-grade self-mod HSM ceremony (0293).
10. **PROGRESSIVE DELIVERY + CHAOS.** Flagger (0160) or Argo-Rollouts (§3/0040); Chaos Mesh or Litmus (0165). *Gates C-7.*
11. **DATA-INTEGRITY SWEEP authorization** (mechanical but required before backfill): (a) restore `KCMVP`/`KISA` from `*-shippable-tier` corruption (8 files/31 occ incl. GLOSSARY + 0043 + 0002); (b) fix self-referential renames (0006 "Ontology→Ontology", 0025/0352 tautologies); (c) fix all dangling supersedes/amends edges (0150-cedar phantom, 0476→0421, 0057→0055-v3, 0069→0088, 0377→0397/0436, Cedar-as-0083 ×5); (d) authorize the `foundry`→cloud-intelligence/governance bulk-rename wave (ADR-0347 Proposed→Accept; AMEND 0363's false "eradicated" claim).
12. **VOCABULARY namespacing** (backfill precondition): "tier" is 4–5-way overloaded (autonomy T1–T4 / EU-AI-Act 0–4 / retired tenant-tier→tenant-class / storage-tier / env-tier) — confirm namespacing (`autonomy_tier`/`eu_ai_act_risk_tier`/`dr_tier`) rather than supersession; rename ADR-0163 "environment tiers"→"environment stages."
13. **ID DISCIPLINE.** Mandate strict no-reuse + no-dangling-ref ADR-id invariant (required if B governs); renumber LINUX 0001–0026 → 0515+ on merge; renumber the duplicate forge-board ADR-0377. Confirm the ADR-0000 re-founding scope (LINUX folds in; Bominal corpus absorb-or-track, root 0060).
14. **PROPOSED-RESOLUTION LEDGER sign-off.** All ~131 source `Proposed` → ratify/drop per `03` (RATIFY ≈122 · DROP 3 · AMEND-MANDATORY 1 · RENUMBER 1 · KEEP-as-Proposed 1); founder confirms the door:one-way subset (the FORK/forge/data-tier/identity/self-mod/scope-breadth/license/substrate clusters). Includes the self-modification ceiling (0247: can autonomous workflows author ADRs/policy-root at all? — door:one-way).
15. **DOMAIN ENUM + cohesion gate.** Approve the closed 16-domain `domain` taxonomy (`04`) as the cohesion-gate key, the `ci-cd-build`/`governance-process` SPLITS, and the `hardware-firmware`→`node-os` / `comms-notify`→`compliance-residency` merges, with the `domain-cohesion` contradiction gate at decision time. *Enables no-contradiction-by-construction backfill.*
16. **AUTONOMY CEILING (door:one-way).** ADR-0007 treats the persona/autonomy ceiling as advisory-centric; ADR-0022 treats T1–T4 as execution-centric runtime-enforced. Confirm the canonical semantics (advisory vs hard runtime gate) AND the ownership home of the autonomy-ceiling mechanism — `intelligence` (post-0335 foundry-absorption) vs `governance`. *Gates ADR-0007/0022/0139/0144 and the policy-engine decision (#6).*

---

## 8. COVERAGE / HONESTY NOTES (MERGE-FINAL)

- **100% per-ADR coverage (disk-verified).** All 372 rows (26 LINUX + 346 SOURCE) carry an explicit per-ADR audit. A `comm -23`/`comm -13` diff of on-disk source ids (`~/Developer/source/docs/decisions/`: 346 files / 345 unique ids) against the table's row-ids returned **zero missing + zero phantom** after adding the two stragglers **ADR-0381** (Kaniko→BuildKit + multi-node Talos cells, Proposed) and **ADR-0482** (Bespoke Substrate Roadmap, Accepted founder-locked). The prior `[GAP]` (chunks 2/4/9) and `[POSTURE]`-only (0187–0514) classifications are RESOLVED — every row has a decision_atom + truth_flag + disposition. **No coverage gap remains on this branch.**
- **Residual (branch-locality, not an audit gap):** ADRs 0476–0481 supersede Phase-1 predecessors (0421/0457/0429/0443/0428) + depend on the 0397–0451 infra band that live on `origin/dev`, not `main`. A masterplan generated from this branch alone has broken edges there — re-resolve at merge.
- **Corpus size (normalized):** **346 ADR files on disk = 345 unique ids + 1 duplicate-numbered (dup-0377)**. README's "346" matches the file count; the earlier "345 unique" formulation was the deduplicated-id count. Both are correct at different levels; the dup-0377 pair (kafka-to-pulsar Accepted + forge-board Proposed) accounts for the difference. Immaterial to dispositions.
- **No audited doc was modified.** This MERGE-FINAL synthesis must pass a SEPARATE verifier lane (not this author) against all three extracts + the keystone map + real-ADR spot-checks before the consensus gate, per `README.md` §"Verification gate." The verifier should spot-check: (1) the 3 DROP verdicts (0325/0316/0349) + the 1 AMEND-MANDATORY (0352); (2) the phantom-edge list (#3); (3) the KCMVP corruption count; (4) the 0377 duplicate; (5) that no `01` row contradicts its source extract.
