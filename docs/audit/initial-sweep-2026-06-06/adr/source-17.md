# ADR Audit — SOURCE chunk 17 (ADR-0138 … ADR-0144)

- **side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **chunk:** 17
- **slice command:** `ls -1 …/decisions/ADR-*.md | sort | sed -n "113,119p"`
- **range:** ADR-0138 → ADR-0144 (7 ADRs)
- **ADRs actually reviewed:** ADR-0138, ADR-0139, ADR-0140, ADR-0141, ADR-0142, ADR-0143, ADR-0144
- **cross-refs verified on disk:** ADR-0145 (`supersedes: [ADR-0140, ADR-0141]` — confirmed), ADR-0335 (foundry retired→intelligence; `amends: [ADR-0136, ADR-0138, …]`), ADR-0136/0137/0143 (all still `Accepted, superseded_by: []` — stale drift), no Cedar content in ADR-0140 (the 0141/0144 "Cedar substrate" cite is wrong).

> **Cluster headline:** Five of seven ADRs in this slice are foundry-cluster or "M01 micro-service-buildout 2026-05-17/18" artifacts (0138, 0139, 0140, 0141, 0143) authored in a 48-hour burst. Two of them (0140, 0141) are already correctly superseded by ADR-0145; three foundry ones (0138, 0143 + the 0136/0137 they depend on) are downstream-retired by the **foundry brand retirement (ADR-0335)** but carry stale `Accepted` front-matter. ADR-0142 (CRDT trait) and ADR-0144 (EU AI Act tiers) are the two genuinely durable, domain-independent decisions worth backfilling cleanly into the masterplan.

---

### ADR-0138 — Foundry six-path deprecation (Strangler migration)

- **decision_atom:** When consolidating a multi-directory micro-service into one, retire the dead source paths via an atomic move plus a 6-month REPORT-ONLY → BLOCKER CI lane that refuses any new reference to the removed paths (Strangler "atomic-consolidation variant" for zero-current-consumer cutovers).
- **current_status:** Accepted (2026-05-18). `supersedes: [] / superseded_by: []`.
- **disposition:** ARCHIVE (as a *foundry-branded* operational artifact) / the reusable *pattern* should MERGE into a generic deprecation-lane decision.
- **governing:** ADR-0335 (foundry retired → absorbed by intelligence; ADR-0335 `amends: [ADR-0136, ADR-0138]`). The six `microservices/foundry-*` paths this ADR retired are themselves now under a retired brand; the consolidated `microservices/foundry/` target was subsequently retired/renamed to intelligence by ADR-0335 + the foundry→governance lane rename (ADR-0347).
- **truth_flag:** PARTIAL. The Strangler *mechanism* is TRUE and well-formed; the *subject* (foundry six-path topology) is STALE — both the source paths and the consolidation target are retired vocabulary now.
- **in_masterplan:** NO. No `planning_impact`/`masterplan_ref` front-matter; this is an ops-migration ADR, not a planning decision. Only the generic "deprecate structure behind a CI sunset lane" rule is masterplan-worthy, and it is not currently captured as such.
- **tensions:**
  - vs **ADR-0335 / ADR-0347:** the lane name `oya-governance-foundry-six-path-zero-usage` and every `microservices/foundry-*` reference is retired-vocab leakage post-0335/0347 (`oya-foundry-*` → `oya-governance-*`).
  - vs **ADR-0136/0137:** depends on them as "the consolidation"; all three share the stale-`Accepted` drift while 0335 declares the brand retired.
  - Internal: front-matter `related` omits ADR-0134 even though the body names it as the pattern template (minor).
- **hyperscaler_challenge:** Would Google/AWS/Azure make this? **Aligned (on mechanism), questionable (on ceremony).** A CI lane that blocks references to removed paths is normal monorepo hygiene (Google's "no dead path" presubmit). But authoring a full ADR + 6-month soak + ~66 engineer-hours for a directory rename of code that *was never deployed and had zero consumers* is over-process; a hyperscaler would do this as a lint rule, not an ADR. Argues for **archive/merge** into a generic deprecation policy, not standalone preservation.
- **ai_slop:** Fabricated precision — the migration-cost table (`~30 sec each × 493`, "~66 engineer-hours") and a 20-row remap table for a zero-consumer move are ceremony inflation. Mild internal contradiction: claims "zero live consumers" yet justifies a 6-month soak against "future consumer that researches git history."
- **refinement:** Collapse into a single reusable "structural-deprecation sunset-lane" decision (pattern only); drop the foundry-specific remap table to an archive note; rename lane to `oya-governance-*`.
- **consensus_needed:** no (subsumed by the foundry-retirement ruling already made in ADR-0335).

---

### ADR-0139 — Agentic SLO-gated promotion

- **decision_atom:** Every fast-forward of `staging`/`production` is gated on deterministic, multi-window multi-burn-rate SLO evidence (Google SRE Workbook ch.5) computed over an adopted OSS Grafana/LGTM stack, with per-component release pointers (`release/<microservice>/<env>`), an eligibility ledger, event-driven promotion, and automated rollback — the OSS substrate is commodity, the SLO-engine + agentic gate is the owned differentiator.
- **current_status:** Accepted (2026-05-17). `supersedes: [] / superseded_by: []`.
- **disposition:** AMEND. Decision is sound and load-bearing, but carries internal contradictions, retired CI vocabulary, and a since-superseded observability stack reference.
- **governing:** none retires it, but it is *bound* by: ADR-0383 (observability stack is now canonically Loki/Tempo/Mimir/Grafana — this ADR's Grafana/LGTM choice is consistent and arguably the precedent); ADR-0511/0513 (CI orchestration is now Argo Workflows + bespoke oya-ci, not the `.github/workflows/*` GitHub-Actions surface this ADR edits).
- **truth_flag:** PARTIAL → mostly TRUE. The promotion-gating *decision* is TRUE and is a genuine differentiator. STALE/WRONG mechanics: (a) self-contradicts on the ledger — the §Decision body says "**No git-tracked JSONL ledger**… Mimir-native recording rules ARE the ledger," yet the §Consequences file-change table and §Operational both create `registry/promotion-eligibility.jsonl` as an "append-only ledger" and the CI lane "reads `registry/promotion-eligibility.jsonl`"; (b) all GitHub-Actions plumbing (`.github/branch-protection.yaml`, `repository_dispatch`, `promote-*.yml`) is pre-GitHub-Actions-retirement (ADR-0359/0511) and pre-Forgejo (ADR-0363); (c) it edits an inline retracted memory string `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145)` — the Workflow/Ontology integration section is built on a rule ADR-0145 retired the day after.
- **in_masterplan:** PARTIAL. Lists `/specs/masterplan.json` + `/specs/master-plan-sequencing.json` in `related_specs` and `/specs/agentic-slo-gated-promotion.json`, but has no `planning_impact: true` / `masterplan_ref` binding (it predates the planning-ssot-coverage gate; one of the ~91% unbound ADRs).
- **tensions:**
  - **JSONL-vs-Mimir ledger self-contradiction** (above) — the single sharpest internal contradiction in this chunk; an auditor cannot tell which is canonical.
  - vs **ADR-0363/0510 (forge):** the whole branch-protection + promote-workflow surface assumes GitHub Actions / branch refs; the canonical forge is now Forgejo (transitory) → bespoke VCS, and the founder directive is GitHub. The gate's *sink* needs re-pointing.
  - vs **ADR-0511/0513:** Jenkins/GH-Actions CI is transitory; the gate engine should be the `oya` governance overlay over Argo Workflows.
  - vs **ADR-0383:** consistent (same LGTM stack); should cross-link.
- **hyperscaler_challenge:** **Aligned.** Multi-window burn-rate gating on promotion is exactly Google SRE / AWS deployment practice; "adopt OSS observability, own the SLO/gate logic" is the correct build-vs-buy call (rebuilding Prometheus is the rejected 50-person-year detour). The per-component release pointer (Linear/Stripe/Google-per-binary) is industry-leading. Verdict argues for **amend, keep** — fix the ledger contradiction and re-point the CI/forge sink, do not archive.
- **ai_slop:** Hedging/fabricated precision in the competitor-parity preamble ("no competitor enforces SLO-gated promotion at the VCS layer… unique oyatie differentiator") is an unfalsifiable marketing claim. The 17-numbered-component list mixes "decision" and "implementation IP" granularity. The self-contradicting ledger is the real defect, not slop per se.
- **refinement:** (1) Resolve ledger to ONE store (Mimir-native recording rules read most-canonical given the audit-row archive; delete the `registry/*.jsonl` references). (2) Replace GitHub-Actions plumbing with the Argo-Workflows + `oya gate` + Forgejo-commit-status sink (per ADR-0511/0513/0363). (3) Add `planning_impact`/`masterplan_ref`. (4) Cross-link ADR-0383.
- **consensus_needed:** **yes.** "Is the promotion-eligibility ledger the Mimir time-series store (recording-rules-as-ledger) or a git-tracked append-only `registry/promotion-eligibility.jsonl`? They cannot both be canonical." (Secondary: confirm the gate sink re-points from GitHub Actions to Argo Workflows + Forgejo commit-status.)

---

### ADR-0140 — Cross-cutting carriers: adapter-rule exemption

- **decision_atom:** A closed charter of five "cross-cutting carrier" micro-services (drive/mail/messenger/calendar/recordings) may be called directly (gRPC/REST) by any app-tier service for *data-carry* concerns, as a defined exemption to the Workflow+Ontology adapter rule; orchestration/decision flow still routes through Workflow.
- **current_status:** **Superseded** by ADR-0145 (front-matter `status: Superseded, superseded_by: [ADR-0145]` — verified). Note the body §Status still reads "Accepted — 2026-05-18" (stale body vs correct front-matter).
- **disposition:** ARCHIVE (correctly superseded).
- **governing:** **ADR-0145** (Inter-microservice communication: hyperscaler shape with opt-in Workflow+Ontology; `supersedes: [ADR-0140, ADR-0141]`, `retires_feedback_memory: feedback_workflow_objectgraph_adapter_layer`). The whole adapter-rule premise this ADR carves an exemption *to* was itself retired by ADR-0145.
- **truth_flag:** STALE. The carrier-of-record principle survives conceptually, but the framing as "an exemption to the Workflow+Ontology mandatory-adapter rule" is dead because ADR-0145 made Workflow+Ontology *opt-in* (so there is no longer a blanket rule to be exempt from).
- **in_masterplan:** NA (superseded). Whatever survives is folded into ADR-0145's masterplan binding.
- **tensions:**
  - vs **ADR-0145** (governing supersession).
  - vs **ADR-0335:** §Decision references `foundry` as "a substrate, not a carrier" and the not-a-carrier list includes `shorts` (retired→social per ADR-0334) — retired-vocab leakage.
  - Naming-note at the foot flags an ADR-0131 duplicate-filename collision — a real index-poisoning issue worth carrying to synthesis.
- **hyperscaler_challenge:** **Aligned (principle), misaligned (as a standalone exemption ADR).** "Storage/mail/messaging is a substrate every product binds to directly (S3/GCS/Files API)" is correct hyperscaler shape. But a hyperscaler would never frame it as an *exemption*; it would simply not mandate a universal orchestration relay in the first place — which is precisely what ADR-0145 corrected. Argues for archive (already done).
- **ai_slop:** Fabricated precision ("file attach p99 inflates by ~30-80ms per hop", "throughput inflates by ~10×") presented without a benchmark. Otherwise coherent.
- **refinement:** Ensure ADR-0145 captures the surviving carrier-of-record concept (closed-set carriers + ≥3-consumer + carries-data-not-decisions test). Fix the stale "Accepted" body header on archive.
- **consensus_needed:** no (superseded).

---

### ADR-0141 — Workflow+Ontology: read-path direct, write-path orchestrated

- **decision_atom:** Split the Workflow+Ontology adapter rule by operation kind — state-changing writes traverse Workflow (audit/compensation/retry seam); reads go direct cell-bounded gRPC to Ontology with Cedar at ingress — to avoid Workflow becoming the platform-wide read SLO ceiling.
- **current_status:** **Superseded** by ADR-0145 (front-matter verified; body §Status still says "Accepted — 2026-05-18" — stale body).
- **disposition:** ARCHIVE (correctly superseded).
- **governing:** **ADR-0145** (`supersedes: [ADR-0140, ADR-0141]`).
- **truth_flag:** PARTIAL/STALE. The read/write-split *insight* (don't make an orchestrator the read-path SLO ceiling) is TRUE and likely survives inside ADR-0145's "opt-in Workflow+Ontology." The mandatory framing is STALE post-0145. One **WRONG cross-reference:** References §"ADR-0140 — Cedar policy enforcement substrate (referenced by capabilities)" — ADR-0140 is NOT a Cedar substrate ADR (it is cross-cutting carriers); no Cedar content exists in ADR-0140. This is a fabricated/incorrect citation (the real Cedar substrate is ADR-0007/0243/0246).
- **in_masterplan:** NA (superseded).
- **tensions:**
  - vs **ADR-0145** (governing).
  - **Mis-citation of ADR-0140** as "Cedar policy enforcement substrate" — same error appears in ADR-0144 (cluster-wide bad reference; auditors should not trust the 0140 reference text in either).
  - vs **ADR-0243/0246 (Cedar canonical):** the read-path Cedar-at-Ontology-ingress is consistent with Cedar-as-universal-gate, but cites the wrong ADR for it.
- **hyperscaler_challenge:** **Aligned.** "Reads bypass orchestration, writes traverse it" is textbook (AWS Builders' Library, Google Borg/Stubby/Spanner). Sound engineering — the problem is only that it patches a rule that should not have existed (fixed by ADR-0145). Argues for archive with the principle preserved upstream.
- **ai_slop:** Plausible-but-uncheckable precision ("30-150 ms per hop", ">60% of latency budgets", "1-2 ms intra-cell"). Citations to "AWS Builders' Library" / "Google SRE Workbook ch.11" / "Anthropic 2024 architecture overview" are name-drops without specific claims an auditor can verify.
- **refinement:** Confirm ADR-0145 carries the read/write-split principle; correct the ADR-0140 mis-citation wherever it propagated; map the read-path Cedar seam to ADR-0243/0246.
- **consensus_needed:** no (superseded).

---

### ADR-0142 — CRDT portability trait + alternate-adapter compile gate

- **decision_atom:** Own a zero-cost (generic, no boxed dispatch) `CrdtDoc`/`CrdtMap`/`CrdtList`/`CrdtText` trait kernel with Loro 1.x as the deployed primary adapter and Yjs + Automerge as CI-compile-only alternates, so a bus-factor-1 event on Loro is a days-not-weeks composition-root swap.
- **current_status:** Accepted (2026-05-18). `supersedes: [] / superseded_by: []`.
- **disposition:** KEEP (current, correct, non-conflicting, well-formed).
- **governing:** none. Net-new, domain-isolated (collaborative editing); no supersession edge in the map.
- **truth_flag:** TRUE. Well-scoped, technically sound, no retired vocabulary, no internal contradiction.
- **in_masterplan:** PARTIAL. Carries `related_specs: [/specs/products/workflow-studio.json]` but no `planning_impact`/`masterplan_ref`. This is exactly a "true + relevant decision" the founder wants backfilled — the **own-when-proven / portability-trait ratchet** is a masterplan-grade principle (aligns with ADR-0019/0211 "own when proven" ratchet language).
- **tensions:**
  - Mild vs **ADR-0335:** lists workflow-studio/docs/sheets/slides/messenger consumers — none retired, but "foundry providers" indirectly touched elsewhere; no direct conflict.
  - Cross-side resonance (note for synthesis): LINUX's "own-when-proven, vendored-now-owned-later" ratchet (LINUX ADR-0019) is the *same shape* as this CRDT primary-with-CI-compile-alternates pattern. Consistent, not conflicting.
- **hyperscaler_challenge:** **Aligned.** Wrapping a single-maintainer dependency behind an owned trait with exercised alternates is exactly the build-vs-buy hedge Google/AWS apply (storage-backend trait seams, multi-backend dispatch). The zero-cost-generic-over-boxed-dyn choice is the correct hot-path call. No amend/archive pressure.
- **ai_slop:** Minor — the "AWS Outposts wraps EBS/S3/Glacier behind an internal trait" and "Spanner SST is the abstraction seam" precedents are decorative analogies, not load-bearing, but they don't distort the decision. Maintainer-count table is concrete and useful.
- **refinement:** Add `planning_impact: true` + `masterplan_ref`; promote the underlying principle ("no production substrate on a bus-factor-1 dependency without an exercised CI-compiled fallback") to a reusable masterplan invariant, not just a Loro-specific note.
- **consensus_needed:** no (clean keep). Optional founder note: this is a good template for the masterplan's "owned-dependency hedge" invariant.

---

### ADR-0143 — Foundry per-BC release pointer

- **decision_atom:** Each internal bounded context of the consolidated foundry micro-service gets an independent release pointer (`release/foundry-<bc>/<env>`) and its own image tag under one umbrella Helm chart, so a hot-fix to one BC rolling-restarts only that BC's pods (extends ADR-0139's per-component pointers down to sub-service granularity).
- **current_status:** Accepted (2026-05-18). `supersedes: [] / superseded_by: []`.
- **disposition:** ARCHIVE (foundry-branded; subject retired) / the *principle* MERGE into ADR-0139's per-component-pointer decision.
- **governing:** **ADR-0335** (foundry retired→intelligence; `amends: [ADR-0136, ADR-0138]`, foundry brand retired) + **ADR-0139** (the per-component-release-pointer mechanism this ADR merely re-applies). Every `release/foundry-<bc>/*` ref and `oya-foundry-shared-*` crate is retired vocabulary (ADR-0335/0347).
- **truth_flag:** PARTIAL. The "umbrella-chart + per-BC image tags + per-BC rollback" pattern is TRUE and correct; the *foundry-specific instantiation* (six named BCs, `release/foundry-*`) is STALE under the foundry retirement.
- **in_masterplan:** NO. No planning front-matter; it is a deployment-granularity clarification of ADR-0136/0139 for a now-retired service.
- **tensions:**
  - vs **ADR-0335/0347:** foundry brand + `oya-foundry-shared-*` crates retired → these refs are dead-vocab.
  - vs **ADR-0136/0139:** depends on both; inherits the 0136 stale-`Accepted` drift; 0139 itself needs amend (above).
  - Internal: §"Anatomy" cites "per ADR-0143 compatibility with SLSA L3" — a self-referential citation (ADR-0143 citing ADR-0143) that looks like a copy-paste error (likely meant a SLSA ADR).
- **hyperscaler_challenge:** **Aligned (pattern), questionable (as standalone ADR).** Umbrella-chart-with-per-component-image-tags is exactly Bedrock/Vertex AI shape; per-BC rollback granularity matching blast-radius is correct. But it is a thin corollary of ADR-0139 applied to one (now-retired) service — a hyperscaler captures this as deployment policy, not a separate decision record. Argues for **merge into 0139** and archive the foundry instance.
- **ai_slop:** Self-referential citation ("per ADR-0143 compatibility with SLSA L3"). Heavy precedent name-dropping (AWS re:Invent 2024 Bedrock session, GCP blog, Anthropic Q&A, Palantir AIP, Linear) — five "direct precedent" claims for a fairly mechanical Helm-values decision is precedent-inflation.
- **refinement:** Fold the durable rule ("per-component release pointer + per-component image tag under one umbrella chart, rollback granularity = blast radius") into ADR-0139; archive the foundry-specific six-BC instantiation; fix the self-referential SLSA citation.
- **consensus_needed:** no (governed by the foundry-retirement ruling already in ADR-0335; mechanism lives in 0139).

---

### ADR-0144 — EU AI Act graduated risk-tier model

- **decision_atom:** Replace the binary Annex-III yes/no AI-compliance gate with a 5-tier graduated model (Minimal / Limited / General-Purpose-AI / High-Risk / Unacceptable) keyed to EU AI Act 2024/1689, where a capability's effective tier is `max(base_tier_per_archetype, context_tier_per_deployment_context)` and Cedar admission enforces the resolved tier's obligation set.
- **current_status:** Accepted (2026-05-18). `supersedes: [] / superseded_by: []`.
- **disposition:** KEEP (current, correct, well-formed) — with one AMEND-grade reference fix.
- **governing:** none retires it. Bound to the Cedar canonical posture (ADR-0007/0243/0246) for the admission seam.
- **truth_flag:** TRUE (decision) with one WRONG reference. The graduated 5-tier model faithfully mirrors the regulation (Art.5/6/9-15/43/50/52-54/60) — this is the regulation's own taxonomy, not an invented one. The single error: `related: [ADR-0140 (retired per ADR-0145)]` and §References "ADR-0140 — Cedar policy enforcement substrate" — ADR-0140 is cross-cutting carriers, not a Cedar substrate; the Cedar dependency should point at ADR-0007/0243/0246. (The inline `(retired per ADR-0145)` annotation is at least honest that 0140 is dead, but the role attributed to it was always wrong.)
- **in_masterplan:** PARTIAL. Carries two `related_specs` (`eu-ai-act-risk-class-registry.json`, `canonical-tier-schema.json`) but no `planning_impact`/`masterplan_ref`. This is a strong masterplan-backfill candidate — compliance posture is exactly the kind of true+relevant decision the founder wants captured.
- **tensions:**
  - **Mis-citation of ADR-0140** as the Cedar substrate (shared with ADR-0141; cluster-wide bad ref).
  - **Vocabulary collision risk:** uses "tier" / "T2-auto" / "canonical-tier-schema.json" for *risk* tiers. The map flags `tier/tier-system` as RETIRED vocabulary for *tenancy* (ADR-0329 → tenant-class) and notes autonomy tiers T1-T4 are a *different* live axis. EU-AI-Act risk tiers are a **third** distinct "tier" axis — high confusion surface; the schema name `canonical-tier-schema.json` is dangerously ambiguous.
  - Consistent-with **ADR-0144 ↔ ADR-0133** (industry-best-practice conformance) — good.
- **hyperscaler_challenge:** **Aligned.** Encoding the regulator's own 5-tier taxonomy with deployment-context mutation is precisely what AWS Bedrock Guardrails / Google Vertex safety taxonomy / Anthropic RSP (ASL) / NIST AI RMF do. Rejecting both the 3-tier (lossy) and 7-tier (Annex-III-split adds tiers without changing obligations) alternatives is correct reasoning. No amend/archive pressure on the decision itself.
- **ai_slop:** Low. The precedent comparisons (Bedrock 4-tier, Vertex 5-tier, RSP ASL, NIST) are genuinely relevant here (compliance benchmarking is legitimate). Only defect is the wrong ADR-0140 reference and the ambiguous "tier" naming.
- **refinement:** (1) Re-point the Cedar dependency from ADR-0140 to ADR-0007/0243/0246. (2) Disambiguate naming: rename `canonical-tier-schema.json` / `T*-` ids to make clear this is **EU-AI-Act risk class**, not tenant-class (ADR-0329) and not autonomy-tier (T1-T4). (3) Add `planning_impact`/`masterplan_ref`.
- **consensus_needed:** no on the decision; **yes (lightweight)** on naming hygiene: "Confirm the 'tier' vocabulary is namespaced across its three live axes (tenant-class, autonomy T1-T4, EU-AI-Act risk tier) so `canonical-tier-schema.json` is not read as the retired tenancy tier-system."

---

## Chunk notes for synthesis

**Pattern 1 — the "M01 micro-service-buildout 2026-05-17/18" burst.** Five of seven ADRs (0138/0139/0140/0141/0143) were authored within 48 hours by `council-architecture` as part of the per-microservice-flat-layout buildout. They are tightly co-dependent (all cite ADR-0131/0136/0139) and several were *corrected within a day* (0140/0141 superseded by 0145 on 2026-05-18, the same date they were authored). This burst is a cluster of rapidly-iterated drafts; treat them as a *unit* — the durable decisions are 0139 (gating) and 0145 (the reform that replaced 0140/0141), not the intermediate carve-outs.

**Pattern 2 — foundry cluster is downstream-dead.** ADR-0138 and ADR-0143 (and the ADR-0136/0137 they depend on) are foundry-branded. ADR-0335 retired the foundry brand (→ intelligence; CI lanes → governance per ADR-0347), but **none of 0136/0137/0138/0143 had their `status`/`superseded_by` front-matter updated** — they all still read `Accepted, superseded_by: []`. ADR-0335 only `amends` them. This is the *exact stale-front-matter drift* the keystone map calls out for ADR-0136 (§1.3 Foundry dissolution), and it extends to the whole cluster. **Synthesis action:** the foundry cluster's *operational* ADRs (0138 six-path lane, 0143 per-BC pointer) should be archived/merged on the same supersession event as 0136; their reusable patterns (sunset-lane, per-component release pointer) belong to generic decisions (a deprecation-policy ADR, and ADR-0139 respectively), not to foundry.

**Pattern 3 — cluster-wide WRONG cross-reference to ADR-0140.** Both ADR-0141 and ADR-0144 cite "ADR-0140 — Cedar policy enforcement substrate." ADR-0140 is *cross-cutting carriers*, has zero Cedar content, and is itself superseded by ADR-0145. The real Cedar substrate is ADR-0007/0243/0246. This is a fabricated/incorrect citation that propagated across at least two ADRs — auditors and any masterplan generator must NOT trust the ADR-0140 reference text in this neighborhood. (Likely a number-off-by-N authoring error; a Cedar ADR with a nearby number was probably intended.)

**Pattern 4 — fabricated-precision latency/throughput numbers.** 0138 (~66 engineer-hours, ~30s/file), 0140 (~30-80ms/hop, ~10× throughput), 0141 (30-150ms/hop, >60% of budgets, 1-2ms intra-cell), 0142 (20-30ns/op vtable cost). None are tied to a measured benchmark. Individually harmless; as a corpus pattern this is a recurring "quantify to sound rigorous" tell that a masterplan-as-SSOT should strip (keep the decision, drop the invented numbers).

**Pattern 5 — the "tier" overload (load-bearing for synthesis).** ADR-0144 introduces a THIRD distinct meaning of "tier" (EU-AI-Act risk tier) on top of the retired tenancy tier-system (ADR-0329→tenant-class) and the live autonomy tiers T1-T4 (LINUX ADR-0021 policy axis). Its spec `canonical-tier-schema.json` is dangerously named. This is a cross-chunk naming-collision risk that the masterplan must namespace explicitly.

**Cross-chunk tensions to escalate:**
1. **ADR-0139 internal contradiction (JSONL ledger vs Mimir-native ledger)** — the single most consequential defect in this chunk; ADR-0139 is otherwise a keeper and a real differentiator, so this needs a founder/architecture ruling (see consensus question). It also needs its GitHub-Actions/branch-ref plumbing re-pointed to the now-canonical Argo Workflows + `oya gate` + Forgejo commit-status sink (ADR-0511/0513/0363), which collides with the founder's GitHub directive — the forge fault-line (§5.4) lands directly on this ADR.
2. **Foundry stale-`Accepted` drift across 0136/0137/0138/0143** — trust ADR-0335's retirement over these stale headers; archive/merge the operational foundry ADRs at the same event.
3. **ADR-0140 mis-citation cluster** — correct before any masterplan generation; do not let the generator emit "ADR-0140 = Cedar substrate."
4. **Masterplan binding gap** — 0139/0142/0144 are the three true+durable decisions here and ALL lack `planning_impact`/`masterplan_ref` front-matter (the 8.8%-bound-ADR problem). Under *either* open founder reading (authored-as-SSOT or generated-from-ADRs) these three are prime backfill targets; 0142 (owned-dependency hedge) and 0144 (EU-AI-Act 5-tier) are the cleanest, 0139 (SLO gating) needs the ledger ruling first.
