# ADR Audit — SOURCE, Chunk 7

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** 7
- **Slice requested:** lines 43–49 of sorted `docs/decisions/ADR-*.md`
- **Range:** ADR-0045 … ADR-0051
- **ADRs actually reviewed (7):** ADR-0045, ADR-0046, ADR-0047, ADR-0048, ADR-0049, ADR-0050, ADR-0051
- **Auditor posture:** READ-ONLY. Keystone map consulted; later canonical ADRs (0179/0184/0192/0193/0194, 0030, 0173, 0211) cross-read to verify supersession/drift.

This is a tight **data-plane + search + automation + clients cluster**, all dated 2026-05-09, all part of the "pack-of-19 → 50-ADR consolidation." Five of seven are still `proposed` and **none carry `masterplan_ref` front-matter** (0% binding in this chunk vs the 8.8% corpus average per the map §4). The defining finding: a later **Accepted** storage/search reframe (the 0179–0194 cluster, 2026-05-18) has drifted past 0045/0046/0047/0048 without any of those four updating their front-matter — classic stale-supersession drift the map §6 warns about.

---

### ADR-0045 — Database tier strategy (Postgres+Citus OLTP / ClickHouse-fork OLAP / Iceberg+DataFusion lakehouse)

- **decision_atom:** Canonical data tier = PostgreSQL+Citus for OLTP (per-tenant per-cell shard by `tenant_id`), a columnar OLAP engine, and an Iceberg+DataFusion lakehouse — one engine family per tier, license-clean, residency-bound.
- **current_status:** `proposed` (front-matter) / "Proposed" (body). `supersedes:-`, `superseded_by:-`.
- **disposition:** AMEND. OLTP-as-Postgres+Citus is still TRUE canon, but every other pin has been overtaken by Accepted successors and must be reconciled: OLAP→ADR-0193 (ClickHouse 26.3 LTS), tenant-TS→ADR-0194 (TimescaleDB), pooling PgBouncer→ADR-0179 (pgcat), vector pgvector→ADR-0192 (Milvus, via 0046's chain), and the whole picture recast by ADR-0184 (four-tier OLTP/read-replica/cache/search). Keep the OLTP atom; amend the rest to reference governors.
- **governing:** Not superseded as a whole, but partially governed by ADR-0184 (storage layering), ADR-0193 (OLAP), ADR-0194 (tenant TS), ADR-0179 (pooling), ADR-0192 (vector). The map §3 data row cites 0192/0196/0193/0194/0179 as current canon — 0045 is the historical pin.
- **truth_flag:** PARTIAL. OLTP=Postgres+Citus TRUE; OLAP/pooling/vector pins STALE; one factual WRONG: ADR-0045 asserts "Citus extension is Apache-2 (clean)" but ADR-0184 L130 records Citus **columnar = AGPL3** (only the sharding extension is MIT). That license claim is plainly wrong and matters under the OSI-strict posture (map §3 license row).
- **in_masterplan:** NO. No `masterplan_ref`; no planning front-matter; `proposed` and unbound. Under "masterplan-as-authority" reading the OLTP atom should backfill; under "generated-from-ADRs" reading the stale front-matter would generate wrong OLAP/license facts.
- **tensions:** (a) LINUX ADR-0001 wants a from-scratch Rust multi-model engine that **eliminates Postgres** — direct own-vs-assemble conflict (map fault-line §1); this ADR is the sharpest source-side anchor of "assemble proven OSS." (b) Self-vs-0184: 0045's PgBouncer + ClickHouse-fork vs 0184/0193's pgcat + ClickHouse-26.3 LTS. (c) License contradiction with 0184 on Citus. (d) Owner `foundry` is RETIRED vocab (→ governance/intelligence per ADR-0335/0347).
- **hyperscaler_challenge:** Aligned-but-dated. Google/AWS/Azure absolutely standardize one OLTP engine + a columnar OLAP tier + an object-store lakehouse (Spanner/BigQuery/Iceberg-class). Postgres+Citus is a credible managed-OSS analog. Verdict: aligned in shape; the per-tier-fork license bookkeeping (ClickHouse fork tracking) is the un-hyperscaler-like cost — argues for AMEND toward 0193's straight-Apache-2 ClickHouse LTS, not a self-maintained fork.
- **ai_slop:** Mild. Fabricated-precision risk in the per-axis SLA table (5ms/20ms P95 numbers with no derivation) and "clickhouse-bundle-oss" fork named speculatively. The "Citus is Apache-2 clean" line is an internal-contradiction-with-corpus, not just slop.
- **refinement:** Add `superseded_by`/`amended_by` edges to 0184/0192/0193/0194/0179; fix the Citus license statement; drop the unsourced SLA numbers or cite ADR-0042 SLO catalog; rename owner `foundry`→`governance`.
- **consensus_needed:** YES. "Does Postgres+Citus remain the canonical OLTP engine, or does LINUX ADR-0001's eliminate-Postgres own-engine thesis win for the substrate pilot?" — load-bearing cross-repo data-tier ruling.

---

### ADR-0046 — Vector store strategy (pgvector day-1 / in-house Rust HNSW-IVF / FAISS adapter-only)

- **decision_atom:** Vector search is a substrate concern: pgvector day-1 inside the OLTP tier, in-house Rust HNSW/IVF at billion-scale, all external engines (FAISS/Milvus/Qdrant/Pinecone) permitted only as adapters behind a port.
- **current_status:** `Superseded`, `superseded_by:[ADR-0192]` (both front-matter and body, cleanly cross-linked).
- **disposition:** ARCHIVE. Correctly superseded; this is the one ADR in the chunk whose front-matter is honest about its own death.
- **governing:** **ADR-0192** (Milvus canonical vector DB, >10M vectors; pgvector retained ≤10M). Map §1.1 confirms.
- **truth_flag:** STALE (by design — superseded). The retained kernel principle ("adapter-only, never primary; own at scale") is still TRUE and survives into 0192; the specific "pgvector is the day-1 primary, in-house Rust is the destination" decision is superseded.
- **in_masterplan:** NA (superseded ADR; should be archived, not backfilled). The surviving principle is what 0192 carries into canon.
- **tensions:** Note the **reversal of stance**: 0046 said "Milvus/Qdrant adapter-only, never primary"; ADR-0192 then makes **Milvus the primary**. That is a genuine doctrine flip (own-it-ourselves → adopt-Milvus), not just a scale threshold — worth flagging as a real decision change, not a clarification. Also touches LINUX ADR-0020, which flags Milvus as an UNSAFE deferral with a hard vector-count gate (map §5.1).
- **hyperscaler_challenge:** The 0046 original ("don't depend on an external vector engine, build in-house at scale") is the *anti*-hyperscaler-consumer stance; the 0192 successor (adopt Milvus) is what AWS/GCP actually do (managed vector service). Verdict on 0046: misaligned-with-industry, correctly archived in favor of 0192. Reinforces ARCHIVE.
- **ai_slop:** Low. Code blocks (HnswIndex/IvfIndex structs) are illustrative-but-fabricated implementation detail typical of this corpus; harmless in a superseded doc.
- **refinement:** None needed beyond archival; ensure 0192 explicitly records that it *reverses* 0046's adapter-only-Milvus stance so the provenance is honest.
- **consensus_needed:** NO. Cleanly governed by 0192. (The own-vs-adopt principle is contested, but that ruling lives at 0192/LINUX-0020, not here.)

---

### ADR-0047 — Search backend strategy (pgroonga day-1 / Tantivy in-Rust / OpenSearch adapter / in-house long-horizon)

- **decision_atom:** Search backend is a substrate concern: pgroonga in-OLTP day-1 (LGPL legal-isolation), Tantivy in-Rust at scale, OpenSearch Apache-2 adapter-only, Elasticsearch-SSPL forbidden, in-house long-horizon.
- **current_status:** `proposed`. `supersedes:-`, `superseded_by:-`. **Stale** — no edge despite a contradicting Accepted successor.
- **disposition:** AMEND (leaning ARCHIVE/SUPERSEDE). ADR-0184 (Accepted, 2026-05-18) makes **Meilisearch 1.9 the canonical Tier-4 search engine** with **Tantivy as the Phase-2 in-house path** — and never mentions pgroonga. The "pgroonga day-1" core of 0047 is contradicted, not refined. The durable parts (Tantivy long-horizon, OpenSearch-as-adapter-shape, SSPL-forbidden) survive; pgroonga does not.
- **governing:** **ADR-0184** (storage-tier layering, Tier-4 = Meilisearch; Phase-2 Tantivy). Also ADR-0173 (vendor-lock-in/stack-ownership) and ADR-0211 (in-house tech-stack policy) re-cite Meilisearch+Tantivy as the search canon. ADR-0030 (search microservice architecture) is the parent.
- **truth_flag:** STALE/PARTIAL. SSPL-forbidden + Tantivy-as-owned-target = TRUE and survived. pgroonga-as-day-1-engine = STALE (superseded by Meilisearch). The whole four-stage pgroonga→Tantivy trajectory is now wrong at the first stage.
- **in_masterplan:** NO. No `masterplan_ref`; unbound; and worse — it would generate a *wrong* search posture (pgroonga) if masterplan were generated from this ADR's front-matter today.
- **tensions:** (a) Direct contradiction with ADR-0184 Tier-4 (Meilisearch vs pgroonga). (b) Internal tension within source's own later canon: 0184 *defers* Tantivy/Quickwit ("not rejected") and picks Meilisearch first, whereas 0047 makes Tantivy the destination and never considers Meilisearch — the two ADRs disagree on the day-1 engine AND the destination's framing. (c) ADR-0048 depends on 0047's pgroonga-mecab path, so 0047's drift orphans 0048's KR-morphology-in-search rationale.
- **hyperscaler_challenge:** Questionable. AWS/Azure ship managed OpenSearch/Cognitive Search; none would run an LGPL Postgres-extension (pgroonga) as the day-1 search engine for a multi-tenant SaaS — the legal-isolation overhead is exactly what a hyperscaler avoids. Verdict: misaligned on pgroonga; aligned on "Rust-native search you can own" (Tantivy/Meilisearch). Argues for ARCHIVE-the-pgroonga-pin, keep the ownership principle.
- **ai_slop:** Moderate. The seven-engine "per-engine matrix" with crisp per-cell ceilings ("~100M docs/cell" repeated verbatim from 0046) is fabricated-precision; "W+18/W+30 removal" timeline is invented certainty. None of it survives contact with 0184.
- **refinement:** Add `superseded_by:[ADR-0184]` (or `amended_by`); reconcile to Meilisearch-day-1 + Tantivy-Phase-2; strike pgroonga or demote to "rejected alternative"; rename owner `axis-search` if that role survives the 32-µservice manifest.
- **consensus_needed:** YES. "Is the canonical search engine pgroonga-day-1→Tantivy (ADR-0047) or Meilisearch-1.9→Tantivy-Phase-2 (ADR-0184)? One must be archived." — a real internal source contradiction, not a cross-repo one.

---

### ADR-0048 — Korean morphology + multilingual tokenization (`Tokenizer` trait, mecab-ko+khaiii FFI day-1, in-house Rust long-horizon)

- **decision_atom:** One `Tokenizer` trait per language family is a substrate concern: mecab-ko(LGPL)+khaiii(Apache-2) via FFI for KR day-1, per-pack impls for JP/ZH/EN/Indic/Arabic, in-house Rust ports long-horizon — no per-axis tokenizer drift.
- **current_status:** `proposed`. `supersedes:-`, `superseded_by:-`.
- **disposition:** AMEND. The trait-as-substrate principle is sound and uncontested, but the ADR is **orphaned by ADR-0047's drift**: it assumes pgroonga/Tantivy own the tokenizer pipeline, whereas the canonical Tier-4 is now Meilisearch (which carries its own tokenization). The KR-morphology requirement is TRUE; its integration point needs re-homing onto the Meilisearch/Tantivy-Phase-2 path.
- **governing:** Parent ADR-0030 (search architecture) + ADR-0047/ADR-0184 (engine). ADR-0013 (license policy) governs the mecab-ko LGPL isolation. Not superseded by a specific ADR — drifted, not replaced.
- **truth_flag:** PARTIAL/TRUE-principle. The "Korean needs morphological tokenization, not ICU whitespace" fact is TRUE and durable. The specific FFI-into-search-engine wiring is STALE relative to the Meilisearch decision. Watch the JP row claiming "MeCab-ja (BSD-style)" — MeCab is actually tri-licensed (GPL/LGPL/BSD); calling it flatly "BSD-style" is a fabricated-clean license claim (same pattern as 0045's Citus error).
- **in_masterplan:** NO. No `masterplan_ref`; unbound.
- **tensions:** (a) Orphan-dependency on ADR-0047 (which is itself drifted). (b) License-claim drift (MeCab-ja "BSD-style"; mecab-ko/Farasa LGPL isolation recurring cost vs OSI-strict posture). (c) The 10-language table is far beyond GA scope (Q4 admits "KR+EN+JP+ZH at GA") — scope/precision mismatch.
- **hyperscaler_challenge:** Aligned-in-principle. Google/Apple/AWS all maintain per-language tokenizer/analyzer stacks (CJK analyzers are standard). A unified trait is exactly right. But a hyperscaler would not hand-roll 10 language FFI shims pre-GA; they'd lean on a library (ICU + per-CJK analyzers) and own incrementally. Verdict: aligned on the trait, over-scoped on the day-0 10-language matrix — argues AMEND (narrow to GA locales).
- **ai_slop:** Moderate-high. The full 10-row locale table with per-engine license annotations is fabricated breadth (the ADR's own Q4 contradicts it). Code structs are illustrative-fabricated.
- **refinement:** Re-home the tokenizer onto the Meilisearch/Tantivy path (reconcile with 0184); narrow the locale table to GA scope with the rest marked "long-horizon"; fix the MeCab-ja license claim; add `related: ADR-0184`.
- **consensus_needed:** NO (principle uncontested) — but it inherits the 0047 search-engine consensus question; resolve that first.

---

### ADR-0049 — Cross-region replication + residency (per-pack default class, opt-in cross-region per consent, immutable post-create)

- **decision_atom:** Residency is a per-tenant property (not per-axis): per-pack default residency class (`strict_kr`/`kr_with_us_failover`/`global`), cross-region replication opt-in per consent/data-class, residency immutable post-create (change = recreate-tenant + DSR-cascade-old-cell).
- **current_status:** `proposed`. `supersedes:-`, `superseded_by:-`. Owner `council-architecture` (a live role, not retired vocab).
- **disposition:** KEEP (with light AMEND for binding). This is the strongest, most defensible ADR in the chunk: regulator-grounded (PIPA Art 28-8, GDPR Art 44-49, Schrems, KR FSC/FSS, 242-FZ), internally consistent, and not contradicted by any later ADR I found. The recreate-not-mutate residency rule is a genuinely good auditability decision.
- **governing:** None supersedes it. It consumes ADR-0008 (DUBO), ADR-0028 (cells), ADR-0043 (HSM/KMS), ADR-0038 (trust portal) — all live. It is a candidate **keystone** for the tenancy/residency masterplan section.
- **truth_flag:** TRUE. The regulatory citations are specific and correct (Art 28-8 exists; Schrems II/III; 242-FZ Russia localization; EHDS/DORA). No fabricated-precision of the kind seen in 0045/0047/0048.
- **in_masterplan:** PARTIAL/NO. Not bound via `masterplan_ref`, but the *content* aligns with the map §3 tenancy/residency canon (tenant = universal scoping primitive; tenant-class). Strong backfill candidate under either masterplan reading — it carries real, immutable, regulator-defensible decisions.
- **tensions:** (a) Minor: residency-class enum here predates the **tenant-class** vocabulary (ADR-0329, `demo_trial`/`paid`) — confirm residency-class and tenant-class are orthogonal axes (they appear to be; map §2 warns against conflating tier/tenant-class but residency is a third axis). (b) `PerPack(eu_only/jp_only)` "onboarded via ADR amendment" presumes the authored-ADR model — collides with the generated-from-ADR masterplan design if that wins (map §4). (c) Cites ADR-0007 as "Cedar" — consistent with current Cedar canon (0243/0246), good.
- **hyperscaler_challenge:** Strongly aligned. AWS (Regions/data-residency + Local Zones), GCP (data-residency + Assured Workloads), Azure (sovereign clouds, EU Data Boundary) all do exactly this: per-region defaults, explicit cross-border consent, immutable-region-at-create. The recreate-to-change-residency rule mirrors AWS's "you can't move a resource's region." Verdict: aligned — KEEP. If anything it's *more* rigorous than typical (most hyperscalers allow some in-region replication flexibility).
- **ai_slop:** None material. Dense but substantive; the regulator tables are real-world-grounded, not filler.
- **refinement:** Promote `proposed`→`accepted` (it reads accepted-grade); add `masterplan_ref`; explicitly state residency-class ⟂ tenant-class ⟂ autonomy-tier orthogonality to pre-empt the conflation the map warns about.
- **consensus_needed:** YES (one narrow point). "When residency changes require recreate+DSR-cascade, who bears that operational cost and is it acceptable for enterprise tenants?" — and the structural question: "Are new residency classes onboarded by amending this ADR (authored model) or by regenerating from data (generated model)?" — ties to the open masterplan question.

---

### ADR-0050 — Automation-first pipeline (Google+Amazon doctrine, sccache+RE, affected-graph testing, Foundry-driven PR triage)

- **decision_atom:** Adopt a Google+Amazon "what can be automated must be" doctrine with the supporting machinery: sccache+remote-execution, per-affected-graph testing, per-agent worktree isolation, merge-queue, auto-rebase/review-bot/merge gates, per-lane CI budgets, agentic PR triage, flaky-quarantine, nightly affected-rebuild, per-PR blast-radius classification.
- **current_status:** `proposed`. Owner `foundry` (RETIRED vocab).
- **disposition:** AMEND. The doctrine + most machinery survive, but the ADR is saturated with **retired vocabulary and superseded CI tooling** that must be reconciled: "Foundry-driven" everywhere (→ intelligence/governance per ADR-0335/0347), `oya-foundry-test-quarantine` crate name (→ `oya-governance-*` per ADR-0347), "Bazel-style remote execution" (→ Buck2 per ADR-0392), and the implicit GH-Actions/Jenkins-era lane model superseded by the Buck2+Argo-Workflows canon (ADR-0408/0511/0513).
- **governing:** Not superseded as a decision, but the CI substrate it assumes is governed by ADR-0392 (Buck2), ADR-0408 (Buck2 CI/CD), ADR-0511 (Argo Workflows destination), ADR-0347 (oya-foundry-*→oya-governance-* rename), ADR-0335 (Foundry retired→intelligence). The lane table already uses `oya-governance-*` prefixes — so the ADR is *internally half-migrated* (governance lanes) yet still says "Foundry-driven" in prose: an internal-inconsistency.
- **truth_flag:** PARTIAL. Doctrine (automation-first, affected-graph testing, blast-radius routing, flaky-quarantine) = TRUE and durable. Tooling references (Foundry brand, Bazel RE, crate names) = STALE. The `oya-governance-*` lane names are already-correct (post-0347), which is good evidence the corpus was partially swept.
- **in_masterplan:** NO. No `masterplan_ref`; the automation doctrine is a strong engineering-operating-model backfill candidate but currently unbound and brand-contaminated.
- **tensions:** (a) Retired "foundry" brand throughout (map §2). (b) "Bazel-style RE" vs ADR-0392 Buck2 reversal. (c) Per-agent worktree isolation cites `~/.claude/skills/superpowers/using-git-worktrees` — a local dev-env path leaking into a corporate ADR (fabricated-authority / non-portable reference). (d) Auto-merge "+ at least one human approval" vs the broader agentic-VCS retirement (ADR-0363) and merge-queue model (ADR-0041) — needs reconciliation with current forge canon (Forgejo/transitory, map §5).
- **hyperscaler_challenge:** Strongly aligned (doctrine), questionable (specific machinery). Google (Blaze/Bazel→ the actual lineage, TAP affected-testing, Critique, Rosie) and Amazon ("you build it, you run it", Pipelines) literally invented this; the doctrine is textbook-correct. But hyperscalers would NOT hand-roll sccache-on-S3 + a bespoke affected.mjs + a custom flaky-quarantine crate when Buck2 RBE + Argo already provide these — argues AMEND to ride the Buck2/Argo canon rather than the 2026-05-09 bespoke machinery.
- **ai_slop:** Moderate. Invented precision (≥80% sccache hit target, 5%/7d flaky threshold, per-lane minute budgets, merge-queue cap 5) presented as decided constants without derivation; `.mjs` script names (`affected.mjs`, `blast-radius.mjs`) in a Rust monorepo is an oddly JS-flavored fabrication. The doctrine prose itself is solid.
- **refinement:** Strip "Foundry" brand→"intelligence"/"governance"; rename `oya-foundry-test-quarantine`→`oya-governance-*`; replace "Bazel-style RE"→Buck2 RBE (ADR-0392); remove the `~/.claude/...superpowers` path; reconcile auto-merge/merge-queue with ADR-0041/0363 forge canon; add `masterplan_ref`.
- **consensus_needed:** NO on the doctrine (uncontested). YES-adjacent only insofar as it must be reconciled against the Buck2+Argo CI canon — but that ruling lives at 0392/0408/0511, not here.

---

### ADR-0051 — Mobile and Native Client Strategy

- **decision_atom:** Web is the canonical surface and conformance reference for every capability; native iOS/Android land at W-Workspace-Stable (parity-gated), per-product PRD owns tech selection, all clients consume the same canonical contracts/gateway/audit-chain with no native-only API.
- **current_status:** `accepted` (the ONLY Accepted ADR in the chunk). Supersedes the legacy mobile-clients cluster per `ADR-LEGACY-REGRESSION-MAPPING.md`.
- **disposition:** KEEP (light AMEND for retired wave-name vocab). Well-formed, scoped honestly ("decides scope/sequencing/ownership rather than tech selection"), and correctly defers tech choice to per-product PRD — a mature decision shape.
- **governing:** None supersedes it; it *supersedes* the legacy mobile cluster. Consumes ADR-0001/0010/0017/0044/0008/0011/0003 — all live. Self-consistent with web-canonical doctrine.
- **truth_flag:** TRUE (with stale wave-name leakage). The decision is sound; the contamination is the **retired milestone vocabulary**: "W-Foundry-Preview" is a dead wave name (map §2: "Foundry-Preview wave name now anachronistic post-0335; M0–M3 retired"). Also names a `oya-governance-mobile-native` lane (correctly post-0347 prefix — good).
- **in_masterplan:** PARTIAL. Accepted + supersedes-legacy gives it real authority weight, but no `masterplan_ref`. Good backfill candidate; the web-canonical-surface rule is a clean masterplan invariant.
- **tensions:** (a) "W-Foundry-Preview" retired wave name (map §2). (b) Cites "Leptos for engineering surfaces; SvelteKit for tenant-facing" — needs cross-check vs ADR-0372 (frontend SolidJS+Rust/WASM, now Superseded per map §1.1) and ADR-0394 (bespoke-Rust IDP Leptos): the Leptos half aligns with 0394, but "SvelteKit for tenant UIs" may conflict with the SolidJS/superseded frontend line — a frontend-stack tension to verify. (c) "bind to capability registry per ADR-0021 + autonomy ceiling per ADR-0022" — note LINUX ADR-0021 is *owned-policy* (different doc, collision on merge per map §6.4); the source ADR-0021 here is the foundry-capability-registry — number-collision hazard flagged for merge.
- **hyperscaler_challenge:** Aligned. Google (web-first PWA + native where the surface demands), Microsoft (web-canonical M365 + native shells), AWS (console web-first + mobile app for a subset) all do web-canonical + selective native + no native-only API. Per-product tech-selection deferral mirrors how big orgs avoid one-mobile-stack mandates. Verdict: aligned — KEEP.
- **ai_slop:** Low. Concrete and decision-dense. The "127 legacy ADRs → 50-ADR pack" provenance and "Codex Round 2 verdict" citation are specific (verifiable against `ADR-LEGACY-REGRESSION-MAPPING.md`), not filler. Minor over-enumeration of store policies (ONE Store/Galaxy/AppGallery/Naver) but that's substantive, not slop.
- **refinement:** Replace retired wave names (W-Foundry-Preview→current wave vocabulary); verify SvelteKit-vs-SolidJS frontend line against ADR-0372/0394; flag the ADR-0021 number-collision for the merge renumber; add `masterplan_ref`.
- **consensus_needed:** NO. Web-canonical + per-PRD-tech is a stable, well-reasoned decision. (Only the frontend-framework cross-ref needs a quick verify, not a founder ruling.)

---

## Chunk notes for synthesis

**1. The dominant pattern: a Proposed 2026-05-09 data/search cluster overtaken by an Accepted 2026-05-18 cluster, with zero front-matter updates.** ADR-0045/0046/0047/0048 (all 2026-05-09, mostly `proposed`) were superseded or drifted by the **0179–0194 storage cluster** (all `Accepted`, 2026-05-18): pgvector→Milvus (0192), PgBouncer→pgcat (0179), ClickHouse-fork→ClickHouse-26.3-LTS (0193), tenant-TS→TimescaleDB (0194), and the four-tier OLTP/read-replica/cache/**search=Meilisearch** model (0184). Only ADR-0046 honestly carries its `superseded_by`. **0045/0047/0048 are stale-front-matter drift** — exactly the map §6 hazard. Auditors merging this chunk must trust the 0184/0192/0193 successors over these.

**2. The sharpest single finding — search engine internal contradiction.** ADR-0047 pins **pgroonga-day-1→Tantivy**; ADR-0184 (Accepted) pins **Meilisearch-1.9→Tantivy-Phase-2** and never mentions pgroonga. These are two source-internal Accepted/Proposed ADRs that disagree on both the day-1 engine and the framing of Tantivy. This is the chunk's top consensus question. ADR-0048's KR-morphology rationale is collateral — orphaned onto a search path that's been replaced.

**3. License-claim drift is a recurring micro-pattern (and an OSI-strict-posture risk).** Three "fabricated-clean" license assertions: ADR-0045 "Citus = Apache-2" (0184 says Citus columnar = AGPL3); ADR-0048 "MeCab-ja = BSD-style" (actually GPL/LGPL/BSD tri-license); and the broad LGPL-isolation reliance (pgroonga, mecab-ko, Farasa) that sits uneasily with the map §3 OSI-strict / no-AGPL-GPL-in-product posture. Under a generated-from-ADR masterplan these wrong license facts would propagate — flag for correction.

**4. Retired-vocabulary contamination is concentrated in 0050 and 0051.** "Foundry"/"Foundry-driven" brand (0045 owner, 0046 owner, 0050 throughout, 0050 crate name) and retired wave names ("W-Foundry-Preview" in 0051) — all dead per map §2 (ADR-0335/0347 foundry-retired; M0–M3/Foundry-Preview waves retired). Notably 0050 and 0051 *already* use the correct `oya-governance-*` lane prefix (post-0347), so the corpus is **half-swept**: lane names migrated, brand prose not. This half-migration is itself a signal the sweep was mechanical, not semantic.

**5. The cross-repo fault-line this chunk anchors: assemble-OSS vs own-the-substrate.** ADR-0045 is the cleanest source-side statement of "assemble proven managed OSS" (Postgres+Citus, explicitly rejecting CockroachDB/Spanner) — the direct antithesis of LINUX ADR-0001's "eliminate Postgres, build a from-scratch Rust multi-model engine" (map §1, §5). ADR-0046's own→adopt reversal (in-house-HNSW → Milvus via 0192) actually shows the *source* side itself oscillating on the own-vs-adopt axis, which strengthens the case that the own-vs-adopt *trigger threshold* (not the principle) is the real open question (map §5.5).

**6. Masterplan binding: this chunk is 0/7 bound.** No ADR carries `masterplan_ref`. Two are KEEP-grade backfill candidates regardless of the authored-vs-generated question — **ADR-0049 (residency)** and **ADR-0051 (web-canonical clients)** are decision-dense, regulator/architecture-grounded, and uncontradicted. Under the "generated-from-ADRs" reading, **0045/0047/0048 are dangerous to generate from** (stale + wrong-license-facts); under the "masterplan-as-authority" reading, only their TRUE atoms (Postgres-OLTP, SSPL-forbidden, Tokenizer-trait-principle, residency-immutability) should backfill.

**7. Quality gradient within the chunk (best→weakest):** ADR-0049 (regulator-grounded, near-zero slop, KEEP) ≈ ADR-0051 (Accepted, well-scoped, KEEP) > ADR-0046 (honestly superseded, ARCHIVE) > ADR-0050 (good doctrine, brand-contaminated, AMEND) > ADR-0045 (good OLTP atom, license error + superseded pins, AMEND) > ADR-0048 (sound principle, orphaned + over-scoped, AMEND) > ADR-0047 (core decision contradicted by Accepted successor, AMEND→ARCHIVE the pgroonga pin).

**8. Number-collision watch for merge (map §6.4):** ADR-0051 cites source ADR-0021 (foundry-capability-registry) and ADR-0022; LINUX pilot ADR-0021 is *owned-policy* and ADR-0022 differs too — these are guaranteed collisions on any LINUX→SOURCE merge and must renumber.
