# ADR Audit Artifact — source-29

- **side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **chunk:** source-29
- **range (ls slice):** lines 197–203 of `ls -1 docs/decisions/ADR-*.md | sort`
- **ADRs reviewed (7):** ADR-0222, ADR-0223, ADR-0234, ADR-0235, ADR-0236, ADR-0237, ADR-0238
- **auditor:** ADR AUDITOR (coverage backfill)
- **date:** 2026-06-06
- **baseline:** keystone map `_map/canonical-posture-and-supersession-map.md`

---

### ADR-0222 — Saga + compensating-transaction portfolio policy

- **decision_atom:** Every cross-microservice write MUST be a saga (forward_action + compensation_action + idempotency_key) coordinated solely by the workflow engine, with both forward and compensating actions recorded in the audit chain; XA/2PC stays banned.
- **domain:** orchestration-scheduling (cross-cut: api-contracts — it constrains the inter-µsvc write contract)
- **current_status:** Accepted (2026-05-18); enforcement advisory-until-cross-flow-recatalogued.
- **disposition:** KEEP (with a minor AMEND-watch on retired vocab — see truth_flag).
- **proposed_resolution:** NA (Accepted).
- **governing:** n/a.
- **truth_flag:** PARTIAL — the saga shape itself is TRUE and current, but the doc carries retired-vocabulary leakage: "foundry" axis in Alt-1 (RETIRED → intelligence/governance per ADR-0335/0347) and Istio mesh enforcement in D-2 (`ADR-0148` Istio; SOURCE orchestration canon is now Talos+CAPI+ArgoCD per ADR-0375, and the keystone map does not list Istio as live mesh — the `oya-saga-coordinator-token` mesh-rejection mechanism is bound to a possibly-stale mesh choice). The core decision is sound; the bindings are stale.
- **in_masterplan:** PARTIAL — portfolio-wide architecture invariant (carries `enforced_by: oya gate validate saga-shape`, a planning/gate binding) but no explicit `planning_impact`/`masterplan_ref` front-matter.
- **tensions:** Depends on ADR-0035 (workflow engine) and reaffirms ADR-0145 (2PC ban) — consistent. Tension with eventing canon: D-2/Alt-3 mandate orchestration over choreography while SOURCE eventing is Pulsar+Oxia (ADR-0377) — not a contradiction (saga orchestration can sit atop Pulsar), but the "choreography rejected" stance should be reconciled with the transactional-outbox pattern that survives ADR-0005→0377. Istio (ADR-0148) reference is the live tension vs Talos-substrate canon.
- **hyperscaler_challenge:** ALIGNED. AWS (Step Functions / DistributedSagas), Temporal, Azure Saga guidance all make exactly this decision — sagas+compensation as the canonical cross-service write shape with a central orchestrator. Verdict argues KEEP, not archive. The only hyperscaler nuance: big shops also permit choreography for high-throughput edges; SOURCE's "orchestration-only" is slightly stricter than hyperscaler practice but defensible for audit-chain observability.
- **ai_slop:** Low. Citations (AWS re:Invent 2017, Temporal, Cadence, Stripe migrations) are real and on-point. Rust struct is illustrative, not fabricated enforcement.
- **refinement:** AMEND to (a) drop/relabel the retired "foundry" axis in Alt-1, (b) re-bind D-2's mesh-enforcement clause to the current mesh/admission posture (Talos+CAPI; confirm whether Istio is still live or replaced).
- **consensus_needed:** "Is portfolio-mandatory saga orchestration (no choreography) compatible with the Pulsar+Oxia transactional-outbox pattern we kept from ADR-0005→0377, or do we permit choreography on high-throughput event edges?"

---

### ADR-0223 — oya git drop-in surface with explicit policy verbs

- **decision_atom:** `oya git <subcommand>` is a thin drop-in git wrapper that preserves git's observable behavior and emits a local audit-ledger event, while policy lifecycle stays in explicit `oya vcs <claim|work|verify|done|…>` verbs (git ops never infer policy state).
- **domain:** forge-vcs (cross-cut: ci-cd-build — governance ledger/verbs).
- **current_status:** Accepted (2026-05-18).
- **disposition:** AMEND — sound principle, but stranded on top of a heavily-superseded VCS stack and obsolescent vocabulary; the drop-in/explicit-verb split survives, the surrounding `oya vcs` policy-ratchet surface largely does not.
- **proposed_resolution:** NA (Accepted).
- **governing (partial):** ADR-0363 (retire agentic-VCS foundry → plain git + Forgejo PRs; supersedes the changeset-SM/merge-queue/`oya vcs` orchestration that ADR-0223 leans on); ADR-0510 (Forgejo transitory; bespoke monorepo-VCS is the declared destination); ADR-0116 (retire external coord tooling). ADR-0223's `oya vcs <claim|work|verify|done|promote|queue|watch>` surface is exactly the bespoke-VCS orchestration ADR-0363 retired — those verbs are casualties even though the git-drop-in idea is not.
- **truth_flag:** PARTIAL/STALE — the `oya git` drop-in design is TRUE and harmless; the `oya vcs` policy-verb surface it preserves is STALE (post-0363 the lifecycle is plain git + Forgejo PRs + cloud-CI gates, not a bespoke `oya vcs` claim/queue/merge state machine). `axis-foundry` decider is RETIRED naming.
- **in_masterplan:** PARTIAL — governance/forge tooling decision; no explicit masterplan binding front-matter; relevance depends on which VCS-end-state survives (see forge fault-line).
- **tensions:** Forge three-way fault-line (§5.4 of keystone map): founder directive = GitHub `jason931225/oyatie`; ADR-0363 = Forgejo PRs; ADR-0510 = bespoke monorepo-VCS destination. ADR-0223 references `git push`/PR/GitHub auth (`oya submit`) which aligns with the GitHub/plain-git reading but contradicts the bespoke-own-the-VCS long-horizon canon. ADR-0111 merge-queue (related) is itself in the retired changeset/merge-queue family.
- **hyperscaler_challenge:** QUESTIONABLE. Google (Piper/CitC), Microsoft (1ES) wrap git/VCS with audit + policy, so a thin audited git wrapper is aligned in spirit. BUT the explicit-bespoke-verb policy ratchet (`oya vcs claim/queue/promote`) is the opposite of hyperscaler direction, which favors standard git + server-side policy (branch protection, required checks) over a bespoke client-side lifecycle CLI. Argues for AMEND (keep `oya git` audit wrapper; retire the bespoke `oya vcs` lifecycle verbs in favor of Forgejo/GitHub server-side gates).
- **ai_slop:** Low. Hyrum's Law invocation is apt; rejected-alternatives are concrete and reasonable.
- **refinement:** AMEND — re-scope to the surviving piece (audited `oya git` drop-in + ledger) and explicitly mark the `oya vcs` lifecycle-verb surface as retired per ADR-0363; drop `axis-foundry` decider naming.
- **consensus_needed:** "Post-ADR-0363/0510, does the `oya git` audited drop-in survive as a real tool, or is plain git + Forgejo/GitHub server-side gating sufficient — i.e. is there any client-side `oya`-prefixed git surface in the destination architecture at all?"

---

### ADR-0234 — Social/Connect Expansion Planning Contract (PR #130)

- **decision_atom:** Accept the Connect social-expansion PRDs (community-social, connect-shorts, connect-network[retired into community], connect-anonymous) as a non-binding PLANNING CONTRACT — catalog/planning surfaces only, all maturity/crypto/hyperscaler claims advisory until real crates+validators land, with `connect-anonymous` blocked from GA pending a structured threat model.
- **domain:** product-ux (cross-cut: docs-ssot-masterplan — it is explicitly a planning-contract artifact, not production scope).
- **current_status:** Accepted (2026-05-17).
- **disposition:** AMEND — the "planning contract, claims advisory until validators land" governance is sound and worth KEEPing, but the body uses retired vocabulary and a now-superseded sub-product topology (`connect-network` retired into `community`; `connect-shorts` naming) per ADR-0238/0334.
- **proposed_resolution:** NA (Accepted).
- **governing (partial-supersession):** ADR-0238 (super-app dissolution; `network` retired, `anonymous` folded into community as a posting-mode) and ADR-0334 (shorts merged into social) jointly overtake the sub-product topology ADR-0234 enumerates. ADR-0234 is not wholly archived because its governance posture (advisory-until-validator) is reusable, but its product map is stale.
- **truth_flag:** STALE — sub-product list (`connect-network`, `connect-anonymous`, `connect-shorts`) is superseded; `network` is RETIRED (ADR-0238 verification shows `microservices/network/RETIRED.md`), `anonymous` folder deleted 2026-05-21 (anonymity = posting-mode in community), `shorts` per ADR-0334 merged into social. The masterplan-anchor decision ("cite ADR-0234 not ADR-0126") is TRUE.
- **in_masterplan:** YES — explicitly binds masterplan/platform references to cite `ADR-0234` (purpose + Consequences), and gates PR #130 merge. It is a planning-contract artifact by design.
- **tensions:** Topology conflict with ADR-0238 (the authoritative dissolution body) and ADR-0334. Anonymity-as-microservice vs anonymity-as-posting-mode is the sharpest drift. No conflict on governance philosophy (advisory-until-validator matches ADR-0236/0235 sibling pattern).
- **hyperscaler_challenge:** ALIGNED (process), QUESTIONABLE (scope). The "planning contract distinct from production scope; claims advisory until validators exist" discipline is exactly how hyperscalers separate design-partner PRDs from GA commitments — aligned, argues KEEP. But Google/AWS/Azure would question whether a single org should simultaneously stand up consumer social + shorts + anonymous-workplace + professional-network as first-class surfaces (breadth/focus) — that is a product-strategy challenge, not an archive trigger for this planning ADR.
- **ai_slop:** Low-moderate. Heavy "hyperscaler_bar / industry_patterns_adopted / anti_patterns_avoided" framing is exactly the aspirational-enforcement drift that sibling ADR-0236 was written to police — slightly self-aware (it marks them advisory) but the framing density is borderline slop.
- **refinement:** AMEND — update the sub-product topology to the post-0238/0334 reality (community absorbs network + anonymity-mode; shorts→social), keep the advisory-until-validator and anonymous-GA-block governance.
- **consensus_needed:** "Connect social-expansion (consumer social + shorts + anonymous-workplace + LinkedIn-class network, now all folded into `community`/`social`) — is this breadth still in the masterplan, or is it speculative product surface to archive given the founder's 'if it's not needed, it's not needed' directive?"

---

### ADR-0235 — Connect Core Public Contracts (PR #131)

- **decision_atom:** Accept six Connect core public contracts (mail.alias, perimeter-auth-result, calendar.ical_feed, calendar.video_call_link, messenger.presence, messenger.reaction — all v1) as planning-stage public contracts, advisory until schemas+validators land, all routed through Workflow/Ontology mediation with immutable personal/work `context_kind`+`ownership_pillar` boundaries.
- **domain:** api-contracts (cross-cut: tenancy — dual-context/pillar isolation).
- **current_status:** Accepted (2026-05-17).
- **disposition:** KEEP (lightly AMEND-watch). The contract names are concrete, narrow, and standards-grounded (iCal, presence, alias) and the dual-context isolation rules are core; less topology-rot than ADR-0234.
- **proposed_resolution:** NA (Accepted).
- **governing:** n/a (not superseded). Sits under the ADR-0238 dissolution but its contracts are per-µservice (mail/calendar/messenger) which all survive as first-class µservices — so the topology is still valid.
- **truth_flag:** TRUE — the six contracts attach to mail/messenger/calendar, all of which remain first-class post-0238; the advisory-until-validator and pillar-isolation constraints are current canon. Minor: spec paths under `/specs/products/connect/*` reflect the pre-flatten layout that ADR-0238 §Numbering note says was flattened to `/specs/microservices/*.json` — a path-drift to verify.
- **in_masterplan:** PARTIAL — binds core PRDs to cite this ADR for contract naming; planning-stage, not yet a shipped API commitment. Carries reviewer-blocking authority over future contract-widening PRs.
- **tensions:** Dual-context/pillar isolation (ADR-0235) aligns with SOURCE tenancy canon (ADR-0244 tenant primitive; ADR-0162/0163 audit slicing). Mediation-via-Workflow/Ontology aligns with ADR-0222 (saga) and the no-direct-cross-µsvc-call rule. Possible path-drift vs ADR-0238 spec-flatten (products/connect → microservices flat).
- **hyperscaler_challenge:** ALIGNED. Treating cross-surface fields as addressable, versioned public contracts (vs implicit PRD prose) and gating presence/alias/calendar-availability behind explicit privacy-isolation is exactly Google/Microsoft/Apple practice (e.g. iCal feeds, presence APIs, plus-aliasing with no cross-account join). Verdict argues KEEP. No archive pressure.
- **ai_slop:** Low. Contracts are specific and real-world-grounded; not aspirational maturity claims.
- **refinement:** AMEND only the spec paths if the `/specs/products/connect/*` → `/specs/microservices/*` flatten (per ADR-0238) has landed; otherwise KEEP as-is.
- **consensus_needed:** None material. (Minor: confirm spec-path canonical location post-flatten.)

---

### ADR-0236 — OP-11 Corpus Remediation Planning Contract

- **decision_atom:** Record the OP-11 aspirational-enforcement-drift remediation as a PROPOSED planning contract — the masterplan summary + `registry/fixuptasks.jsonl` are the source of truth (not a missing audit corpus), and no aspirational-enforcement detector may be declared a required CI gate until it has a real CLI, fixture-tree+negative integration tests, and an actual branch-protection row.
- **domain:** governance-process (cross-cut: ci-cd-build — detector/gate machinery; docs-ssot-masterplan — anti-aspirational-enforcement doctrine).
- **current_status:** Proposed (2026-05-17) — intentionally not accepted.
- **disposition:** AMEND→RATIFY (accept the honest doctrine) OR fold into a standing anti-aspirational-enforcement standard. The CONTENT (don't claim a gate is required before it exists) is exactly the audit doctrine the founder's "ADRs feed a generated masterplan; no unaccounted proposals" goal wants enforced.
- **proposed_resolution:** RATIFY — accept it. Why: it is a meta-governance guardrail that directly serves the SSOT-integrity goal (every cited lane/gate must actually resolve on-branch); leaving it Proposed indefinitely is itself an unaccounted-proposal smell. If the OP-11-specific corpus has since been superseded by the broader `planning-ssot-coverage` gate work, then RATIFY-and-generalize rather than DROP.
- **governing:** none (it is itself a governing/doctrine ADR). Conceptually adjacent to ADR-0116 (related) and the keystone map's "ADR-number-keyed gate names FORBIDDEN" planning-ssot doctrine.
- **truth_flag:** TRUE — the doctrine is correct and self-consistent; it explicitly refuses to fabricate enforcement, which is the opposite of slop. Only staleness risk: it references a one-off "OP-11 audit" moment (2026-05-17); the durable rule deserves a non-dated home.
- **in_masterplan:** YES (binding intent) — directly references `/specs/masterplan.json`, `/specs/master-plan-sequencing.json`, `registry/fixuptasks.jsonl` and constrains what may be claimed as a required gate in the planning system. This is a masterplan-integrity ADR.
- **tensions:** Strongly REINFORCES the keystone map's masterplan-drift-prevention thread and the "8.8% ADR binding" finding (planning-ssot-drift-prevention.md). No conflict; if anything it is the doctrine the audit itself runs on. Tension only with any ADR that claims a not-yet-existing required gate (e.g. ADR-0222 advisory-lane, ADR-0237 NEW lanes) — but those are honestly marked advisory, so they comply.
- **hyperscaler_challenge:** ALIGNED. Google/AWS/Azure release engineering forbids declaring a check "required" before it runs against real artifacts in branch protection (presubmit reality). This is textbook-correct release hygiene — argues for RATIFY, not archive.
- **ai_slop:** None — it is anti-slop tooling by intent. The single risk is the ADR being so meta it never gets ratified and rots in Proposed.
- **refinement:** RATIFY and lift the durable rule into a standing standard (anti-aspirational-enforcement / required-check-reality) decoupled from the one-shot OP-11 corpus event.
- **consensus_needed:** "Ratify ADR-0236's anti-aspirational-enforcement rule as standing doctrine (a required check must resolve against real branch artifacts before any ADR may call it required), or has `planning-ssot-coverage` already absorbed it — in which case archive OP-11 as completed?"

---

### ADR-0237 — Connect dissolution: Strangler-pattern migration

- **decision_atom:** Migrate the legacy `oya-connect-{mail,messenger,calendar}` crates to the 8 flat µservices via a 6-phase Strangler Pattern (parallel-ship → adapter shim → feature-flagged canary traffic-shift → zero-active-usage proof → code-removal sweep → umbrella retirement), each phase gated by a concrete verification command, with no big-bang cutover (incompatible with ADR-0139 SLO-gated promotion).
- **domain:** governance-process (cross-cut: ci-cd-build — migration lanes/gates; product-ux — the Connect surfaces being migrated).
- **current_status:** Accepted (2026-05-17). Operational companion to ADR-0238.
- **disposition:** KEEP the migration methodology; AMEND the vocabulary/tooling bindings. The Strangler+canary+zero-usage-proof+removal discipline is excellent and durable; several tooling references are retired.
- **proposed_resolution:** NA (Accepted).
- **governing (partial, on tooling refs only):** ADR-0363 (retire `oya vcs claim/verify/done/promote` — Phase 1/5 lean on `oya vcs` verbs and `oya vcs --help` deprecation hints, now stale) and ADR-0145 (which RETIRES `feedback_workflow_objectgraph_adapter_layer` — ADR-0237 §related already annotates this inline). The migration body is not superseded; only its tool/vocab citations are.
- **truth_flag:** PARTIAL — methodology TRUE and current; STALE bindings: `oya vcs` lifecycle verbs (ADR-0363), `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145)`, `.github/branch-protection.yaml` (GitHub-Actions-shaped CI is retired per ADR-0359/0511 → Argo Workflows; gate sink is now Forgejo Commit Status), and the renamed lanes already half-updated to `oya-governance-connect-*` (good — matches ADR-0347 prefix) while §body still also uses bare `connect-legacy-symbol-zero-usage`. Network/anonymous topology in Phase 6's HG-list is stale per ADR-0238's own retirement of `network`/`anonymous`.
- **in_masterplan:** PARTIAL — operational migration plan with concrete gates; carries CI-lane registrations (`oya-governance-connect-legacy-symbol-zero-usage`, `…-umbrella-retirement-readiness`). No explicit masterplan_ref but it is execution-binding for the dissolution.
- **tensions:** Internally consistent with ADR-0238 (its target topology) and ADR-0114/0139 (canary + SLO-gated promotion). External tension: CI substrate references (`.github/branch-protection.yaml`, GitHub Actions implicit) vs current Argo-Workflows+Forgejo canon; `oya vcs` verbs vs ADR-0363. Phase-6 HG-NETWORK/HG-ANONYMOUS gates contradict ADR-0238's verification block which RETIRES network (`RETIRED.md`) and deletes anonymous (posting-mode in community).
- **hyperscaler_challenge:** ALIGNED. Strangler-fig migration with feature-flagged percentage canary, zero-usage verification via telemetry+dep-graph before deletion, and mandatory terminal removal is exactly Google/AWS/Microsoft large-scale migration practice (e.g. Google's "no zombie code", AWS gradual traffic shifting). Strongly argues KEEP. The only over-engineering note: a 3-month adapter soak + 6-week canary for an internal crate migration is heavyweight, but defensible given Hyrum's-Law external consumers.
- **ai_slop:** Low — but heavy SKILL.md self-citation ("per skill §Step 3/§Step 4…") reads as scaffold padding; the substance (commands, gates, cost table) is concrete and real.
- **refinement:** AMEND — (1) replace `oya vcs` verbs/hints with the post-0363 plain-git+Forgejo surface; (2) replace `.github/branch-protection.yaml` with Argo-Workflows/Forgejo-Commit-Status gate sink; (3) reconcile Phase-6 HG-list with ADR-0238's network-retired/anonymous-as-posting-mode reality; (4) keep the renamed `oya-governance-connect-*` lanes consistently in §body.
- **consensus_needed:** "Is the full 6-phase / 6–12-month Strangler (3-mo adapter soak + 6-wk canary) warranted for an internal crate-family rename, or is a lighter adapter-shim + zero-usage-gate + delete cycle sufficient now that the external-consumer surface is governed by Forgejo/GitHub server-side gates?"

---

### ADR-0238 — Super-app (Connect) expansion into 8 flat µservices

- **decision_atom:** Dissolve the inherited `Connect` super-app into first-class flat single-concern µservices (mail, messenger, calendar, community[absorbs network + anonymity-mode], social, shorts) per ADR-0131/0132 — each with its own ChangeSet lane, OpenSLO, IaC/HPA, and HG-maturity gate, no direct cross-µservice imports (Workflow/Ontology only), umbrella retiring when all HG-<MS> gates hold 30d at p99.
- **domain:** product-ux (cross-cut: orchestration-scheduling — per-µservice SLO/IaC/HPA topology).
- **current_status:** Accepted (2026-05-17); renumbered from ADR-0126 on 2026-05-18.
- **disposition:** KEEP (AMEND-watch for internal topology inconsistency). The flat-per-concern decomposition is the terminal-state proof of ADR-0132 no-grouping and is canonical; the doc has internal drift between its decision table and its own verification block.
- **proposed_resolution:** NA (Accepted).
- **governing:** none (it is governed-by ADR-0132/0131 which it instantiates). It is itself the authority for the Connect topology; ADR-0237 operationalizes it; ADR-0234 plans into it.
- **truth_flag:** PARTIAL — core decision TRUE; internal INCONSISTENCY is the live defect: the §Decision table still lists `network` as a concern absorbed-into-community AND lists `social` + an `anonymity-mode` row, but §Verification + §Negative simultaneously state `microservices/network/RETIRED.md`, `microservices/anonymous/` deleted 2026-05-21, and the §Decision row for `shorts` survives while ADR-0334 (keystone map) says shorts merged into social. So "8 flat µservices" is no longer literally 8 (network retired, anonymous demoted to posting-mode, shorts contested vs social) — the headline count is STALE even though the dissolution principle is TRUE. `oya vcs claim/verify/done/promote` (structural commitment #1) is retired-vocab per ADR-0363. `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145)` cited inline. `M03 phase plan` is retired Wave/Milestone vocab (keystone §2, M0–M3 RETIRED → Wave names).
- **in_masterplan:** YES — `related_specs: /specs/per-microservice-flat-layout.json` + per-µservice spec pointers; it is the binding topology decision the masterplan's product graph derives from (product-graph.md/.html node strip in ADR-0237 Phase 6).
- **tensions:** Internal: "8 µservices" headline vs network-retired/anonymous-deleted/shorts-merged reality (self-contradictory between Decision table and Verification block). External: shorts-as-µservice (ADR-0238) vs shorts-merged-into-social (ADR-0334 per keystone map §1.2) — direct topology conflict to resolve. `oya vcs` verbs vs ADR-0363. M03 milestone vocab vs retired M0–M3. Connect-as-super-app product breadth feeds the same focus tension as ADR-0234.
- **hyperscaler_challenge:** ALIGNED on architecture, QUESTIONABLE on product breadth. Per-surface microservices (not per-suite bundles) with independent SLO/HPA/compliance overlays is exactly AWS/Google/Microsoft/Stripe practice — the decomposition is correct and argues KEEP. BUT a hyperscaler would challenge whether one platform should own mail + messenger + calendar + forum + social-feed + short-video + professional-network + anonymous-workplace concurrently — that breadth (a Google-Workspace + LinkedIn + TikTok + Teamblind superset) is the deeper "own everything" focus question, arguing the product SCOPE (not the decomposition pattern) deserves a founder gut-check.
- **ai_slop:** Moderate. Very long with dense industry-precedent and cost tables; the file-count claims ("community 126 files, social 96, shorts 97, network 100, anonymous 102 populated") are precise scaffold-population numbers that read as fabricated-precision (and several of those folders are now retired/deleted per the same doc), which is a slop/staleness tell.
- **refinement:** AMEND — reconcile the §Decision table with the §Verification reality (drop network as a live concern, fold anonymity to community posting-mode, resolve shorts-vs-social against ADR-0334), restate the headline as the surviving N concerns, and strip retired `oya vcs`/`M03`/objectgraph-adapter vocab.
- **consensus_needed:** "What is the canonical post-retirement Connect topology — exactly which flat µservices survive (network retired, anonymous→community posting-mode, shorts vs ADR-0334's merge-into-social), and is the full Workspace+LinkedIn+TikTok+Teamblind product breadth in the masterplan or trimmed?"

---

## Chunk notes

**Two clusters in this slice.** (1) Two portfolio/tooling architecture ADRs — ADR-0222 (saga) and ADR-0223 (oya git) — that are doctrinally sound but stranded on superseded substrate references (Istio mesh, `oya vcs`/agentic-VCS retired by ADR-0363). (2) A tight five-ADR "Connect dissolution" family (0234/0235/0236/0237/0238) authored in a single 2026-05-17 session, all bearing the same renumber-to-avoid-Bominal-ADR-0126 provenance.

**Strongest KEEPs (true + current + non-conflicting in substance):** ADR-0222 saga shape, ADR-0235 core public contracts, ADR-0238's decomposition PRINCIPLE, ADR-0237's Strangler METHODOLOGY, and ADR-0236's anti-aspirational-enforcement DOCTRINE. The last is the most undervalued item in the slice: ADR-0236 is the only Proposed ADR here and its content is literally the governance the founder's "generated-masterplan, no unaccounted proposals" goal depends on — I flag it RATIFY.

**The one Proposed ADR — ADR-0236 — must resolve to RATIFY** (accept as standing anti-aspirational-enforcement doctrine), not DROP. Leaving it Proposed is itself the unaccounted-proposal smell the audit is meant to eliminate.

**Sharpest TRUTH defect:** ADR-0238 is internally self-contradictory — its §Decision still headlines "8 flat µservices" while its own §Verification/§Negative blocks retire `network` (`RETIRED.md`), delete `anonymous` (2026-05-21, demoted to a community posting-mode), and the keystone map records `shorts` merged into `social` (ADR-0334). The "8" is stale; the surviving count and the shorts-vs-social conflict need a founder ruling. ADR-0234 inherits the same stale sub-product topology.

**Retired-vocabulary leakage across the slice (auditors should sweep):** `oya vcs` lifecycle verbs and the bespoke agentic-VCS lifecycle (ADR-0223/0237/0238) — retired by ADR-0363; `axis-foundry` decider (ADR-0223) — foundry brand RETIRED (ADR-0335/0347); `feedback_workflow_objectgraph_adapter_layer` (ADR-0237/0238) — retired by ADR-0145 (already annotated inline in two places, a good pattern); `M03 phase plan` (ADR-0238) — M0–M3 milestone vocab RETIRED → Wave names; `.github/branch-protection.yaml` + implicit GitHub-Actions CI (ADR-0237) — CI is now Argo Workflows (ADR-0511) with Forgejo Commit Status as gate sink; Istio mesh enforcement (ADR-0222 via ADR-0148) — should be reconciled against the Talos+CAPI substrate canon.

**Cross-cutting founder question (product strategy, not vocabulary):** the Connect family commits the platform to a Google-Workspace + LinkedIn + TikTok + Teamblind product superset. The decomposition pattern is hyperscaler-aligned and correct; the product BREADTH is the real "is this in the masterplan / is this needed" question and recurs in ADR-0234 and ADR-0238. Surface, do not resolve.

**No GARBAGE, no hard ARCHIVE in this slice.** Everything is either current-true (KEEP) or sound-but-stale (AMEND); the Connect family is too foundational to archive, but its product map and tooling refs need a refresh pass and the headline "8 µservices" needs correcting to the post-retirement reality.
