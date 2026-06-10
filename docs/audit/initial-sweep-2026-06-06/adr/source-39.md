# ADR Audit Artifact — source-39

- **side:** SOURCE (`~/Developer/source`, jason931225/oyatie)
- **chunk:** 39 (slice 267–273 of sorted `docs/decisions/ADR-*.md`)
- **range:** ADR-0330 → ADR-0336
- **ADRs reviewed:** 7 (0330, 0331, 0332, 0333, 0334, 0335, 0336)
- **auditor posture:** READ-ONLY; keystone map consulted; trust the superseding ADR over stale front-matter
- **cluster note:** This chunk is the **tenant-class triplet** (0329→0330→0331) plus the **Wave-15 service-boundary retirement series** (0333 cell, 0334 shorts, 0335 foundry) plus a healthcare decomposition (0332) and the **Redis→Valkey substrate swap** (0336). Three of these (0333/0334/0335/0336) are named in the keystone map as governing retirement ADRs.

---

### ADR-0330 — Tenant Class: demo_trial vs paid with Composable Billing Components

- **decision_atom:** `tenant_class` is a closed two-member enum (`demo_trial`|`paid`); the `paid` class carries a composable `billing_components` subset of `{revenue_share, per_seat, per_usage}`; quality/capability/architecture bar is uniform across both classes; cloud-billing is the source of truth and Cedar gates all class-conditional behavior — this is the positive replacement model for the retired Bronze/Silver/Gold/Platinum tier system.
- **domain:** tenancy (cross-cutting: finops-cost)
- **current_status:** Accepted
- **disposition:** KEEP
- **proposed_resolution:** NA (already Accepted)
- **governing:** n/a (this ADR is itself the governing keystone for tenant-class; it supersedes ADR-0316 via the 0329 retirement)
- **truth_flag:** TRUE
- **in_masterplan:** YES (`planning_impact: true`; binds `/specs/tenant-model.json`; enforced by 8 oya-governance lanes)
- **tensions:** (1) References a retired-vocab residue — clause A.7 and B.1.5 still say "Foundry-issued principals / cloud-billing's tenant-model crate" and "oyatie.foundry.*"; per ADR-0335 "foundry" the µservice is retired (the `oyatie.foundry.*` Cedar **principal namespace** survives, so this specific use is legal, but the prose "Foundry" branding is stale). (2) Cites `ADR-0247` for self-modification with an inline rationale; fine. (3) Internal cross-ref hygiene: front-matter `companion_docs` use absolute `/Users/jasonlee/oyatie/...` paths — non-portable but cosmetic.
- **hyperscaler_challenge:** ALIGNED. AWS/GCP/Azure all model commercial posture as principal/account attributes resolved at the request boundary and applied via policy (AWS IAM principal tags + SCPs; Azure AAD app roles), exactly the "read tenant_class from the claim, never inline `if`" pattern here. The two-class + composable-meters model mirrors how Stripe/Snowflake actually bill (usage + seats + rev-share combined, tiers being sales overlays). No argument for amend/archive on the core model.
- **ai_slop:** Low. 2051 lines is long but the length is load-bearing (normative clause numbering that downstream lanes bind to). No fabricated facts.
- **refinement:** AMEND-adjacent only — scrub the residual "Foundry" prose in A.7/B.1.5 to "self-modification principal namespace" per 0335; this is a naming touch, not a disposition change.
- **consensus_needed:** None on the decision. (Founder directive of 2026-05-20 is explicit and quoted; acceptance is final.)

---

### ADR-0331 — Cross-µservice tenant_class Adoption Template

- **decision_atom:** Every active µservice (77 at authoring) MUST plumb tenant_class through twelve canonical adoption surfaces (manifest, PRD §B, ARCHITECTURE, Cedar fragment, capability YAML, OpenSLO, cost-budget, per-context IaC, mobile/SDK, onboarding, tests, observability) via a bespoke per-µservice `IP-tenant-class-adoption.md`, sequenced by ADR-0328 phase order and verified by the `ci-tenant-class-adoption-check` lane.
- **domain:** tenancy (cross-cutting: ci-cd-build)
- **current_status:** Accepted
- **disposition:** KEEP (with a watch-flag, below)
- **proposed_resolution:** NA (Accepted)
- **governing:** n/a
- **truth_flag:** TRUE (PARTIAL on durability — it is an implementation-rollout template, not an architecture invariant; its truth depends on the 77-µservice roster, which the retirement ADRs in this very chunk are actively shrinking)
- **in_masterplan:** PARTIAL. No `planning_impact` flag in front-matter (unlike 0330), but it is enforcement-bearing (5 `oya gate validate` checks). It is plumbing-doctrine rather than a top-level masterplan binding.
- **tensions:** (1) **Roster drift** — it asserts "77/77 µservices" as universal, but ADR-0333 (cell), ADR-0334 (shorts), ADR-0335 (foundry) in this same wave retire µservices, and ADR-0332 adds 8 healthcare µservices. The "77" is already wrong the moment those land; the template's universality claim is sound but the count is stale. (2) A.6 says tenant_class is "derived from `(audience_type, lifecycle_state, payment-contract-state)`" while ADR-0330 B.1.4 treats tenant_class as a mandatory stored field — mild modeling tension (derived-vs-stored) the two ADRs should reconcile. (3) The `Proposed | In-Progress | Done` IP skeleton creates many downstream `Proposed` IPs — those are sub-ADR artifacts, not ADRs, so they do not violate the "no unaccounted proposals" rule.
- **hyperscaler_challenge:** ALIGNED (mechanism), QUESTIONABLE (ceremony). The principal-claim-at-boundary pattern is correct hyperscaler practice. But a 924-surface-touch (77×12), 19,250-line corpus-wide hand-authored rollout with an anti-template/substance-bar prohibition on find-and-replace is the kind of process a hyperscaler would automate via codegen/scaffolding rather than mandate bespoke per service. Argues for **refinement** (allow generated scaffolding + bespoke deltas), not archive.
- **ai_slop:** Low-moderate. The "capacity math" (95 agent-batches, etc.) is process-theater but harmless; core surfaces are concrete and verifiable.
- **refinement:** Re-derive the µservice count from the on-disk roster after the Wave-15 retirements + healthcare adds settle; reconcile the derived-vs-stored tenant_class modeling with ADR-0330 B.1.4.
- **consensus_needed:** "Is per-µservice bespoke authoring of all 12 surfaces the right bar, or should surfaces 1/4/5/6/8/12 (manifest, Cedar, caps-YAML, OpenSLO, IaC, observability labels) be scaffold-generated with only PRD §B / ARCHITECTURE authored bespoke?" — a process question worth a founder ruling.

---

### ADR-0332 — Healthcare Domain Decomposition

- **decision_atom:** Decompose the 14-domain, 215-feature `healthcare-integration` µservice into eight new single-concern domain µservices (`emr`, `diagnostics`, `imaging`, `emergency`, `pharmacy`, `patient-monitoring`, `clinical-decision-support`, `care-management`), each HIPAA-pack-mandatory for paid tenants with its own per-domain compliance shape and industry-leader counterparts, and narrow `healthcare-integration` to the FHIR/HL7v2/DICOM broker substrate only (no `healthcare/` parent folder — ADR-0132 forbids grouping).
- **domain:** product-ux (cross-cutting: compliance-residency)
- **current_status:** Accepted
- **disposition:** AMEND
- **proposed_resolution:** NA (Accepted)
- **governing:** n/a (KEEP-core, AMEND-references)
- **truth_flag:** PARTIAL — the decomposition decision is TRUE and well-grounded (single-concern doctrine + distinct counterpart markets + distinct compliance shapes), but the ADR carries **stale vocabulary** that drifted within the same wave: front-matter lists `ADR-0316-capability-tier-doctrine.md` as a live related anchor and §A.5 Anchor 6 says "**Each new µservice carries its own capability-tier matrix (Bronze/Silver/Gold/Platinum)**" — this is the **retired tier system** (ADR-0316 superseded by ADR-0329/0330, which are siblings dated the *same day* 2026-05-21). This is a direct internal contradiction with the tenant-class triplet in this very chunk.
- **in_masterplan:** PARTIAL (no `planning_impact` flag; `enforcement_status: advisory-until-eight-microservices-scaffold-lands`; it is a Phase-4 long-tail structural decision, not yet a binding masterplan row).
- **tensions:** (1) **Bronze/Silver/Gold/Platinum capability-tier reference (Anchor 6) is retired-vocab** per ADR-0329/0330 — must be rewritten to tenant-class. (2) Phase-4 placement means it competes for sequencing attention while it is "advisory until scaffold lands" — low-severity. (3) Healthcare is a deep B2B-displacement bet whose existence is downstream of the broader "own-everything breadth" question the keystone map flags (§5 fault-line 5).
- **hyperscaler_challenge:** QUESTIONABLE (scope), ALIGNED (decomposition shape). The single-concern, domain-per-service decomposition is exactly how AWS HealthLake / Google Cloud Healthcare API + partners structure the space, and the counterpart analysis is accurate. BUT: a pre-product-market-fit platform authoring eight Epic/Cerner-class clinical EMR/PACS/pharmacy competitors as day-0 structural commitments is the kind of breadth a hyperscaler would gate behind a real customer/market signal. Argues for **amend** (keep the decomposition doctrine; defer/condition the 8-service build on demand), not archive.
- **ai_slop:** Low on facts (the vendor/regulatory citations — Epic ~37% KLAS share, CLIA 42 CFR §493, DEA EPCS 21 CFR §1311, EMTALA 42 USC §1395dd, USP 797/800 — are accurate). Moderate on **ambition-as-substance** (very long counterpart inventories per service).
- **refinement:** AMEND Anchor 6 + the `ADR-0316` front-matter relation to cite ADR-0329/0330 tenant-class (remove Bronze/Silver/Gold/Platinum). This is the single concrete defect.
- **consensus_needed:** "Do we commit day-0 structure for eight clinical-domain hyperscaler-competitors, or hold the decomposition as conditional doctrine that instantiates only on a named healthcare customer/market trigger?"

---

### ADR-0333 — Cell µservice retired; cellular architecture is a pattern, not a service

- **decision_atom:** Retire `microservices/cell/` as a standalone service (RETIRED.md redirect, zero-current-consumer strangler variant) while preserving cellular architecture as a mandatory ADR-0248 pattern: responsibilities re-home to adjacent owners (tenant→cell assignment → tenancy; provisioning/lifecycle/registry → cloud-iac OpenTofu state; health/SLO-burn → observability; audit scoping → audit-chain; routing → api-gateway) and the deterministic selection algorithm becomes the pure `crates/oya-shuffle-sharding` crate.
- **domain:** orchestration-scheduling (cross-cutting: isolation-runtime)
- **current_status:** Accepted (amended 2026-05-21 per ADR-0351, which carves `cell-rebalancer` + `cell-lifecycle` back out)
- **disposition:** KEEP
- **proposed_resolution:** NA (Accepted)
- **governing:** This ADR is itself a governing retirement ADR (keystone map §1.2: supersedes `microservices/cell/{PRD,ARCHITECTURE}.md`; amends ADR-0248/0138/0131). It is the canonical authority for "cell = pattern not service."
- **truth_flag:** TRUE
- **in_masterplan:** YES (`planning_impact: true`; updates `specs/microservices/manifests-index.json` + master-plan-sequencing roster).
- **tensions:** (1) **Self-amendment churn** — the Status block says the absorption "stands" but ADR-0351 immediately re-extracts two bounded contexts (`cell-rebalancer`, `cell-lifecycle`) into *new* µservices. So the clean "absorb into adjacent owners" story is partially reversed within the same day; an auditor reading 0333 alone would miss that two of the absorbed concerns became services again. Flag the 0333↔0351 pair as a churn-couplet. (2) `oya-cell-*` crates declared transition-debt (D-59) — residual-vocab leakage to watch on merge.
- **hyperscaler_challenge:** ALIGNED. The core reasoning ("AWS does not run a central cell service; cellular architecture is a team-topology pattern each service implements in its own boundary") is verbatim correct — this is exactly the AWS cell-based-architecture doctrine (Well-Architected / builder library). Distributing ownership to natural owners and keeping a pure deterministic shuffle-sharding library is the right hyperscaler shape. No argument for amend/archive.
- **ai_slop:** Low. The one-sentence-per-line style (D-1…D-82, M/O/P/R/S/V series) is unusual formatting but each line is a discrete verifiable commitment, not padding.
- **refinement:** Add a forward-pointer in 0333's Status to ADR-0351's carve-out so the absorption map is not read as terminal.
- **consensus_needed:** None on retirement. (Watch the 0333/0351 boundary-thrash as an architecture-stability signal, not a decision contest.)

---

### ADR-0334 — Shorts µservice retired; absorbed into social as short-video flavor

- **decision_atom:** Retire `microservices/shorts/` as a standalone service and fold short-form video into `social` as one media flavor on the shared `Post` aggregate (`media.kind = short_video`), reusing social's follow graph, feed-timeline ranker, moderation pipeline, audio library, copyright-claim/DMCA, DRM (tenant-class gated per ADR-0330), and analytics — no separate `video` or `shorts` service, per ADR-0132 no-grouping.
- **domain:** product-ux
- **current_status:** Accepted
- **disposition:** KEEP
- **proposed_resolution:** NA (Accepted)
- **governing:** This ADR is itself a governing retirement ADR (keystone map §1.2: supersedes `microservices/shorts/{PRD,ARCHITECTURE}.md`; amends ADR-0238/0132).
- **truth_flag:** TRUE
- **in_masterplan:** YES (`planning_impact: true`; removes shorts from `specs/master-plan-sequencing.json` + manifests-index).
- **tensions:** (1) Crate-naming note D-43 routes media kernels under `oya-community-social-*` — couples to the parallel Wave-15K `network→community` rename; mildly confusing that "social" content lands under a "community" crate prefix. Worth a one-line clarification but not a conflict. (2) `oya-shorts-*` crates declared transition-debt (D-42) — residual-vocab leakage to watch. (3) `axis-shorts` ownership team folds into `axis-social` (D-58) — org/decision coherence, fine.
- **hyperscaler_challenge:** ALIGNED. The precedent argument is exactly right — Instagram Reels ⊂ Instagram, YouTube Shorts ⊂ YouTube, X video ⊂ timeline, LinkedIn video ⊂ feed; only TikTok is standalone and TikTok is itself a unified social product, not a multi-service split. Folding short-video into the social product is the decision Meta/Google actually made. No argument for amend/archive.
- **ai_slop:** Low. Concrete absorption map + data-model deltas (`Post.media.short_video.{variant_ladder,audio_track_id,derives_from,drm_policy}`).
- **refinement:** Clarify the `oya-community-social-*` crate-prefix vs "social" product naming so the social/community coupling is unambiguous.
- **consensus_needed:** None.

---

### ADR-0335 — Foundry µservice retired; absorbed by intelligence; Hermes terminology dropped

- **decision_atom:** Retire `microservices/foundry/` as a standalone service and absorb its AI pipeline orchestration, eval, training/RLHF, red-team, guardrails, provider router, and model registry into `intelligence` (the canonical two-layer AI substrate per ADR-0255); drop "Hermes" as a canonical primitive corpus-wide (no replacement brand — "intelligence pipeline" or "oyatie.foundry workflow library"); the `oyatie.foundry.*` **Cedar principal namespace** survives as self-modification authority (ADR-0247) even though the µservice and brand do not.
- **domain:** intelligence-ai (cross-cutting: agentic-platform)
- **current_status:** Accepted
- **disposition:** KEEP
- **proposed_resolution:** NA (Accepted)
- **governing:** This is a top-tier governing retirement ADR (keystone map §1.2 + §2 + Foundry-dissolution chain §1.3). It supersedes the foundry doc-pair + PHASE-01/02; amends ADR-0136/0138/0220/0239/0247/0255. **Brand "foundry" RETIRED → intelligence (consumer AI) + governance (CI/gates).**
- **truth_flag:** TRUE
- **in_masterplan:** YES (`planning_impact: true`; rewrites `specs/microservices/foundry.json` + manifests-index + master-plan-sequencing + root-hub-pointers).
- **tensions:** (1) **Critical merge-hazard flagged by keystone map §1.3:** ADR-0136 still reads `Accepted, superseded_by:[]` on disk while 0335 (and 0247) declare it superseded — auditors must trust 0335 over 0136's stale front-matter. 0335 P-5 honestly records 0136 as "active as historical context," which is the correct framing, but the 0136 file itself is the stale party. (2) **Brand-residue debt is enormous and self-acknowledged:** 122 `oya-foundry-*` crates across 43 dependents kept as transition-debt (D-37/D-38, R-13/R-14) to avoid breaking `cargo check --workspace`; the rename is deferred to a separate wave. Hundreds of "Foundry"/"Hermes" strings persist corpus-wide (this is the keystone map's MFL-0002/0003 brand-residue signal). (3) Distinction between retired µservice/brand and surviving `oyatie.foundry.*` Cedar principal namespace is subtle and a frequent source of "is foundry dead?" confusion — see ADR-0330 A.7 residue in this very chunk.
- **hyperscaler_challenge:** ALIGNED. Consolidating two overlapping AI substrates (consumer-AI `intelligence` + internal-pipeline `foundry`) into one canonical owner, with self-modification as a principal namespace rather than a service, is the right ownership shape — no hyperscaler runs two parallel model-router/eval/guardrail stacks. No argument for amend/archive of the *decision*; the only debt is the deferred crate rename.
- **ai_slop:** Low on the decision; the residue (122 crates, hundreds of strings) is real technical debt the ADR honestly times-out, not slop.
- **refinement:** Schedule the deferred `oya-foundry-* → oya-intelligence-*` crate-rename wave and the corpus-wide Hermes/Foundry string sweep (D-26..D-36, S-12); until then, treat residual `oya-foundry-*` / "Foundry" in *new* work as retired-vocab leakage. Scrub the 0330 A.7/B.1.5 "Foundry" prose noted above.
- **consensus_needed:** None on retirement. (Operational: confirm the crate-rename wave is actually queued, not perpetually deferred.)

---

### ADR-0336 — Valkey is the canonical in-memory KV/cache/pubsub substrate (Redis retired for license drift)

- **decision_atom:** Adopt Valkey (Linux Foundation BSD-3-Clause fork of Redis 7.2.4, mainline 8.x) as the canonical in-memory KV/cache/pubsub/streams substrate and retire Redis 7.4+ (Redis Inc. SSPLv1/RSALv2) from the allow-list on license-drift + hyperscaler-alignment + OSI-strict grounds; RESP3 wire protocol and `redis-rs`/`fred`/`deadpool-redis` client surface are preserved unchanged (wire-compatible by construction), and the corpus-wide vocabulary migration runs as Wave 15-Valkey behind eight new CI lanes.
- **domain:** data-storage (cross-cutting: security-supplychain)
- **current_status:** **Proposed** (2026-05-21; `enforcement_status: advisory-until-corpus-migration-lands`)
- **disposition:** AMEND-then-KEEP (sound decision; status must be resolved)
- **proposed_resolution:** **RATIFY (accept).** Why: the keystone map already treats Redis→Valkey as **canonical-true** (Retired-Vocabulary table row "Redis→Valkey, ADR-0336") and the founder directive (`feedback_valkey_not_redis_2026_05_21`: "License drift on Redis is a hard stop") is explicit and binding; the license facts are correct (Redis Inc. dual SSPL/RSAL relicense 2024-03-20; LF Valkey fork 2024-03-28; AWS/GCP/OCI all shipped managed Valkey within 9 months), it amends the dependency-policy that *already* lists Valkey as the substitution, and there is zero scaffolded `oya-*-redis-*` code to migrate (clean swap). No reason to leave it Proposed. **No unaccounted proposal — RATIFY.**
- **governing:** n/a (governing for the substrate; amends ADR-0013/0045/0211/0212/0328; consistent with keystone-map canonical posture "Valkey not Redis").
- **truth_flag:** TRUE (the only defect is the unaccepted status, which the keystone map and corpus already treat as accepted-in-fact — a status/front-matter drift exactly like the ADR-0005/0136 stale-status pattern).
- **in_masterplan:** YES (`planning_impact: true`; adds sub-wave `15P-Valkey-migration` to `specs/master-plan-sequencing.json`; binds `forbidden-operations.json`).
- **tensions:** (1) **Status drift** — Proposed on disk vs canonical-true in the keystone map's retired-vocab table; ratify to close. (2) **Sub-wave label collision** — front-matter/B2.027 names the migration `15P-Valkey` / `15P-Valkey-migration`, but ADR-0334's completion report already uses `wave: 15O` and the dependency on "Wave 15I landed earlier" implies the 15x letter space is crowded; verify `15P` is unallocated (low severity). (3) Pre-7.4 Redis (BSD-3) retained as non-canonical fallback and Memcached retained for pure-cache — these are correctly scoped, not tensions. (4) DragonflyDB (BSL-1.1) explicitly forbidden and removed from the substitution table — corrects a prior dependency-policy error (good catch, not a conflict).
- **hyperscaler_challenge:** ALIGNED — emphatically. AWS ElastiCache for Valkey, Google Memorystore for Valkey, and OCI Cache with Valkey are all GA; the hyperscaler triad settled on Valkey within 9 months of the relicense. A from-scratch org choosing the in-memory substrate today would pick Valkey for exactly the license + managed-offering + performance (8.x multi-threaded I/O) reasons stated. Argues *for* ratification, not amend/archive.
- **ai_slop:** Low. License/timeline facts are accurate and checkable; the only soft content is the "capacity math" batch estimate. The 12-surface migration mechanic mirrors the 0331 template style (consistent house pattern).
- **refinement:** On ratification, flip `status: Proposed → Accepted`, confirm the `15P` wave label is free, and ensure the GLOSSARY/`glossary.json`/`canonical-primitives.md` Valkey entries + Redis-retired note land (B2.028–B2.031).
- **consensus_needed:** None substantive — the founder directive is explicit. Only the bookkeeping question "ratify now vs after Wave 15-Valkey soak completes?" — recommend ratify now (decision is final; soak governs lane promotion, not acceptance).

---

## Chunk notes

**Disposition tally:** KEEP ×5 (0330, 0331, 0333, 0334, 0335), AMEND ×1 (0332), AMEND-then-RATIFY ×1 (0336). No ARCHIVE, no MERGE, no UNCLEAR. Six of seven are Accepted-and-correct; the seventh (0336) is the only Proposed in the chunk and gets a clear RATIFY.

**This chunk is a high-value, internally-coherent realignment cluster.** Four of the seven are top-tier governing ADRs already cited by the keystone map: 0330 (tenant-class keystone, supersedes 0316), 0333 (cell-as-pattern), 0334 (shorts→social), 0335 (foundry→intelligence + Hermes drop). These are the *causes* of much of the corpus's retired vocabulary, not victims of it — they should feed the masterplan directly and immutably.

**One genuine internal contradiction to surface (same-day drift):** ADR-0332 (healthcare) §A.5 Anchor 6 and its front-matter still invoke the **Bronze/Silver/Gold/Platinum capability-tier** system and cite ADR-0316 as live — while its same-day siblings ADR-0329/0330/0331 *retire exactly that*. This is the sharpest concrete defect in the chunk and a clean AMEND: rewrite 0332's tier references to tenant-class. Secondary residue: ADR-0330 §A.7/B.1.5 carry stale "Foundry" prose that 0335 retires (the `oyatie.foundry.*` Cedar principal namespace survives, so the *meaning* is legal, but the *branding* is dead).

**Two status/front-matter drifts to resolve (trust-the-superseding-ADR pattern):** (a) ADR-0335 supersedes ADR-0136 but 0136 still reads `Accepted, superseded_by:[]` on disk (keystone §1.3) — trust 0335. (b) ADR-0336 is `Proposed` on disk but treated as canonical-true everywhere else (keystone retired-vocab table, dependency-policy, founder directive) — RATIFY.

**Largest deferred-debt item in the chunk:** ADR-0335's 122 `oya-foundry-*` crates + corpus-wide "Foundry"/"Hermes" strings, honestly time-boxed as transition-debt to keep `cargo check --workspace` green. This is real, acknowledged, and the single biggest brand-residue source feeding the keystone map's MFL-0002/0003 leakage signal. Confirm the rename/sweep wave is actually queued.

**One architecture-stability watch (not a decision contest):** ADR-0333 retires `cell` and absorbs its concerns into adjacent owners, but its own Status block records that ADR-0351 *immediately re-extracts* `cell-rebalancer` + `cell-lifecycle` back into new µservices. Same-day absorb-then-re-extract churn — flag the 0333↔0351 couplet as a boundary-thrash signal.

**Hyperscaler verdict across the chunk:** the four retirement/consolidation ADRs (0333/0334/0335/0336) are all strongly ALIGNED — each is precisely the decision AWS/Google/Meta/Snowflake actually made (cellular-as-pattern, short-video-inside-social, one-AI-substrate, Valkey-over-relicensed-Redis). The tenancy pair (0330/0331) is aligned on mechanism; 0331's bespoke-12-surface ceremony is the one process choice a hyperscaler would lean toward codegen. 0332 (healthcare) is the one breadth/scope bet a hyperscaler would gate behind a real market signal rather than commit day-0.

**Masterplan-generation note (the OPEN founder question):** under *either* reading of the unresolved authored-vs-generated masterplan design, these seven are exactly the kind of LIVE, planning-impact-bearing decisions that belong in the generated masterplan — six already carry `planning_impact: true` (0330/0333/0334/0335/0336 + the 0329 sibling), and 0331/0332 are enforcement-bearing plumbing/structure. None is a candidate for archival; the only edits needed are the two vocab AMENDs (0332 tier→tenant-class, 0330 Foundry-prose scrub) and the 0336 RATIFY.
