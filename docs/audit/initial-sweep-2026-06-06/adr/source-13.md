# Source ADR Audit — Chunk 13

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** 13
- **Slice requested (lines 85–91 of sorted ADR list):** ADR-0107 … ADR-0113
- **ADRs actually reviewed (7):** ADR-0107, ADR-0108, ADR-0109, ADR-0110, ADR-0111, ADR-0112, ADR-0113
- **Cluster identity:** This chunk is the **agentic-VCS pipeline cluster** (0110–0113: changeset state machine, merge-queue, webhook receiver, `oya vcs done` orchestrator) plus a **naming/lifecycle-tooling cluster** (0107 tools/-suffix, 0108 sunset schema, 0109 lifecycle framework). The VCS cluster is the single most-superseded block in the whole corpus.

---

### ADR-0107 — `tools/` directory canonical-suffix binding (was: implicit `app` layer)

- **decision_atom:** Every crate under `tools/` must end in a canonical 13-value layer suffix (binaries use `-app`); the directory is an organizational hint, not a layer declaration — the original "implicit-app-by-location" exception is removed.
- **current_status:** Superseded (front-matter `status: Superseded`, `superseded_by: ADR-0105`). Self-consistent, machine-readable sunset frontmatter present (`sunset_at 2026-05-15`, `removal_at 2026-08-15`, `sunset_topic`).
- **disposition:** ARCHIVE (well-formed historical record; content fully absorbed by ADR-0105).
- **governing:** ADR-0105 (13-layer enum + check-family patterns; §"Amendment 2026-05-15 — tools/ canonical-suffix binding"). Confirmed `status: Accepted` on disk. Matches keystone map §1.1 (0107→0105).
- **truth_flag:** TRUE (the surviving rule — `tools/*-app` canonical suffix — is true and live in ADR-0105; this file correctly self-describes as carrying no unique content).
- **in_masterplan:** PARTIAL — naming/BNF discipline is plumbing not a masterplan-tier decision; the live rule belongs to ADR-0105's planning surface, not 0107's. No `planning_impact` flag (correct for a superseded shell).
- **tensions:** Retired-vocab leakage — the §body enumerates 8 crates as `oya-governance-*` (e.g. `oya-governance-adr-shape`) yet the original Context block lists them and the convention text still references `tools/` doctrine tied to `oya-foundry-fitness`→`oya-governance` rename (ADR-0347). The crate examples are already on the post-rename `oya-governance-*` prefix, which is internally consistent with the retired-foundry posture — good. Minor: cites ADR-0054 (grit scaffold-claim, itself `deprecated`→ADR-0116) and ADR-0108's sunset schema as live dependencies.
- **hyperscaler_challenge:** ALIGNED. Google/Bazel and Amazon-internal build systems both enforce explicit, mechanically-checkable layer/target naming over directory-implicit conventions. Removing the "directory IS the layer" magic in favor of an explicit suffix enum is exactly what a hyperscaler monorepo would do. Argues for KEEP-the-rule (in 0105) / ARCHIVE-the-shell — no change to substance.
- **ai_slop:** Mild. The triple-amendment structure ("Amendment — no-exception", "Amendment — Superseded", "Original (superseded) decision", "Pre-amendment Status", "Decision (SUPERSEDED)") is heavily redundant — the same supersession is narrated ~5 times. Fabricated-precision smell: cites a verbatim commit hash `1d07b63` for the absorbing amendment. Otherwise honest and well-sourced.
- **refinement:** Collapse to a 6-line tombstone ("Superseded by ADR-0105; rule = tools/ crates end in canonical suffix, binaries `-app`; see ADR-0105 §Amendment 2026-05-15") and let the sunset lane remove it at `removal_at`. The 170-line forensic narration is over-retention for a fully-absorbed rule.
- **consensus_needed:** no.

---

### ADR-0108 — Sunset → deprecation → removal lifecycle automation schema

- **decision_atom:** Every time-bounded sunset clause must carry a machine-readable `SunsetClause` schema (`sunset_at`/`sunset_milestone` + 30-day-deprecation / 90-day-removal defaults + `sunset_topic`) discoverable across ADR front-matter, spec JSON `_sunset`, and Cargo manifest, enforced by a deterministic fitness lane.
- **current_status:** Accepted (2026-05-15). `sunset_topic: adr-0108-self`, no sunset date on itself (correct — it is the schema, not a sunsetting clause).
- **disposition:** KEEP (current, well-formed, non-conflicting; it is the doctrine that makes 0107's own sunset frontmatter meaningful).
- **governing:** n/a (live).
- **truth_flag:** TRUE.
- **in_masterplan:** PARTIAL — no `planning_impact: true` flag despite being a corpus-wide authoring contract (it governs how *every* ADR expresses retirement). Under the keystone's OPEN masterplan question this is a binding-discipline decision; under "masterplan = SSOT" it should at least be referenced. Currently NOT bound to MASTERPLAN.md.
- **tensions:** Composes-with (not conflicts) ADR-0037 (runtime `DeprecationUsed` telemetry — explicitly partitioned: runtime-event side vs static-file side). Dependency on ADR-0109 is mutual-and-clean (0108 = sunset schema, 0109 = generic framework; the "Pattern-B dedicated kernel" carve-out in both is the one coupling to watch). Names `oya-governance-sunset-lifecycle-{kernel,app}` crates — post-foundry-rename prefix, consistent.
- **hyperscaler_challenge:** QUESTIONABLE-to-aligned. The *principle* (machine-readable deprecation with enforced removal deadlines) is exactly hyperscaler discipline (Google's deprecation horizons, AWS API deprecation policy). But "automate the lifecycle of literally every artifact because automation cost ≈ 0" (the 0109 rationale this leans on) is the part Google/AWS would NOT do at this stage — they gate tooling investment on population/ROI. The schema itself is sound; the universalist framing it inherits is the over-reach. Argues for KEEP-schema, AMEND-the-rationale-coupling.
- **ai_slop:** Low. "Live baseline (2026-05-15): 6 violations" with an enumerated table is concrete and falsifiable (good). Some doctrine-quote padding from `feedback_no_exceptions_canonical.md`. The "automation cost ≈ 0 / false positives ≈ 0" claim (inherited theme) is unfalsifiable optimism.
- **refinement:** Add `planning_impact: true` + a `masterplan_ref` so the sunset-discipline binds into the planning SSOT (it currently governs ADRs but is invisible to the masterplan). Add the `#[deprecated]` source-attr recognition (listed as Wave-B follow-up) to close the static/runtime gap.
- **consensus_needed:** no (mechanism is sound) — but see chunk note on the 0109 universal-automation policy, which IS contested.

---

### ADR-0109 — Lifecycle-automation framework (generic kernel + per-lifecycle configs)

- **decision_atom:** One generic, config-driven `oya-governance-lifecycle-kernel` evaluates state-machine lifecycles (ADR-status, crate-status, plan-status, etc.) as data (JSON configs), with a dedicated-kernel "Pattern B" carve-out only for date-arithmetic lifecycles like sunset.
- **current_status:** Accepted (2026-05-15).
- **disposition:** AMEND (sound framework, but the "automate ALL lifecycles because cost ≈ 0" policy and the withdrawn-population-gate need a reconciliation/founder ruling; also the §Decision item 6 vs §"Migration policy" internally contradict — see ai_slop).
- **governing:** n/a (live) — but its own §"Migration policy" partially supersedes its own §Decision item 6.
- **truth_flag:** PARTIAL. The framework + Pattern-A/B taxonomy is TRUE and useful. The "automate-by-default for any state machine, population thresholds do not apply, cost is non-existent" doctrine is an unverified founder-clarification echo (STALE-risk: it is the kind of zero-cost claim the founder elsewhere warns is "plain wrong").
- **in_masterplan:** PARTIAL — no `planning_impact` flag, no masterplan binding, despite proposing 9 new fitness lanes + a governance kernel that touches ADR/crate/plan status (i.e. the lifecycle machinery the masterplan-as-SSOT design itself would consume). Notable: the "adr-status" lifecycle config it ships (`proposed→accepted→superseded→archived` with `requires_supersession_edge`) is *exactly* the mechanism planning-ssot-consolidation.md wants to drive masterplan generation — this ADR is upstream of the open masterplan question and is not cross-linked to it.
- **tensions:**
  - **Self-contradiction:** §Decision item 6 + §Consequences ("Future M-CC phase: convert sunset-lifecycle into a config-driven instance for full DRY") vs §"Migration policy" ("that successor-IP is hereby withdrawn; sunset-lifecycle remains dedicated indefinitely; removing this entry would be a silent regression"). The doc both schedules and forbids the same migration.
  - **0108 coupling:** the Pattern-B registry duplicates 0108's domain description; if either drifts they desync.
  - **Masterplan-design tension:** if `planning-ssot-consolidation.md` wins (ADRs generate masterplan), the `adr-status` lifecycle lane here becomes load-bearing infra for SSOT; if `drift-prevention.md` wins (masterplan is authority, ADRs bind in), this lane is a secondary checker. The ADR doesn't acknowledge it sits on that fault-line.
- **hyperscaler_challenge:** MISALIGNED (on scope), aligned (on shape). The generic-kernel+config shape is good engineering (matches how Google builds reusable check frameworks). But "automate every lifecycle by default, ignore population thresholds, automation cost ≈ 0" is precisely the over-build a hyperscaler would reject: they ruthlessly gate internal-tooling investment on adoption/ROI and would not stand up 9 lifecycle lanes (incl. `feature-flag-status` at 6 occurrences, `capability-status` at 4 crates) speculatively. Argues for AMEND (reinstate a lightweight ROI/population trigger, or explicitly time-box the speculative lanes).
- **ai_slop:** Moderate-to-high. The internal Decision-item-6-vs-Migration-policy contradiction is real slop (two passes left both directions in the file). Fabricated precision: "≈560 LOC", "9+ pure-stage-transition state machines", "scaffolding ≈ 0, maintenance ≈ 0, CI runtime ≈ 0, false positives ≈ 0" — a wall of unfalsifiable zeros. Population numbers in the catalog table (85 ADRs, 283 crates, 336 plan files) are concrete (good).
- **refinement:** (1) Resolve the migration self-contradiction in one direction. (2) Demote the "automate-everything-cost-is-zero" rationale to a non-binding note or attach an ROI trigger. (3) Add `planning_impact: true` + explicitly wire the `adr-status` lifecycle to the masterplan-SSOT design so the open founder question can be decided with this lane in view. (4) Mark the 9-lane catalog as a *candidate* backlog, not a committed wave.
- **consensus_needed:** **yes** — "Do we automate every artifact lifecycle by default (cost-is-zero doctrine), or gate lifecycle-automation on population/ROI like a hyperscaler would?" This is a load-bearing tooling-philosophy decision the founder warned against taking on faith.

---

### ADR-0110 — Changeset state machine

- **decision_atom:** A changeset advances through a closed, monotonic, event-sourced 12-value state enum (9 advancing + 3 terminal-fail) with an Ed25519-signed, dedup-keyed `changeset-event-log` as single source of truth across the dev→staging→production pipeline.
- **current_status:** Superseded (front-matter `superseded_by: [ADR-0363]`). Correct.
- **disposition:** ARCHIVE (superseded; the substrate was never deployed — per ADR-0363, the changeset-state machine had 0–1 dependents and the event-log is frozen as historical evidence).
- **governing:** **ADR-0363** (retire bespoke agentic-VCS → plain git + Forgejo + Prow-shaped cloud-ci; explicitly `supersedes: [ADR-0110, ADR-0112, ADR-0113]`). Verified on disk. Matches keystone map §1.1.
- **truth_flag:** STALE — internally coherent and well-engineered for its moment, but the entire premise (bespoke `oya vcs` changeset substrate over GitHub Actions) is retired. The doctrine survives nowhere; merge/state semantics moved to cloud-ci/Tide (ADR-0511/0513).
- **in_masterplan:** NO — and correctly so now (retired). Note the deep GitHub coupling (event-router on GitHub webhooks, `gh api`, PR-against-`dev`) directly conflicts with the Forgejo-canonical posture that superseded it, and only partially aligns with the founder's GitHub directive (founder wants GitHub-the-host; this ADR wanted GitHub-the-automation-substrate, which ADR-0363 explicitly rejected for plain-git+Forgejo).
- **tensions:**
  - **Forge fault-line (keystone §5):** built entirely on GitHub Actions `workflow_run`/webhooks + `gh api` + `github.com/jason931225/oyatie/pull/N` URLs. ADR-0363 retires this for Forgejo PRs + cloud-ci. The founder's GitHub-host directive does NOT rescue this ADR — its automation model is the thing 0363 killed.
  - **0111 coupling:** 0110 is the contract 0111/0112/0113 build on; archiving 0110 strands 0111 (whose front-matter is still `Proposed`).
  - **CI churn:** assumes GitHub-Actions pr-tests as the CI substrate — itself retired through the 0349/0359/0511 chain (Argo Workflows destination).
- **hyperscaler_challenge:** MISALIGNED. No hyperscaler builds a bespoke per-changeset state machine with Ed25519-signed event rows and a custom monotonicity fitness-lane *on top of* GitHub when the platform (or Prow/Tide, or Google's internal Critique/Piper) already owns merge state. This is reinventing the forge — exactly the "don't reinvent the wheel" the founder cited to retire it. Strongly argues for ARCHIVE (already done).
- **ai_slop:** Low-to-moderate as authored (the engineering is precise and self-aware — closed enums, dedup keys, signatures). The slop is *strategic*, not textual: ~250 lines of rigorous design for a substrate that shipped 0–1 dependents and was never deployed. Fabricated-precision example: a fully-worked example event row with `usd_remaining: 4.73`, `tokens_remaining: 1_842_117` for a system that did not run.
- **refinement:** None for the ADR (archive it). For the corpus: ensure the `changeset-event-log.json` + `event-router.yaml` are physically frozen/marked historical per ADR-0363, and that no live lane still references them.
- **consensus_needed:** no (already governed by ADR-0363) — but it is a load-bearing *example* for the founder's "we don't need bespoke VCS" ruling; worth keeping as the cautionary archived record.

---

### ADR-0111 — Merge queue: projected-merge-state + fix-at-any-stage

- **decision_atom:** The merge queue simulates the projected post-merge state (squash-merge chain + `git merge-tree` conflict check + path-overlap gate + re-test against the projected base) before admitting any PR, and re-validates fix-at-any-stage pushes, running the cheap conflict gate before expensive CI.
- **current_status:** **Proposed** (front-matter `status: Proposed`, `superseded_by: []`) — and this is the chunk's sharpest drift. It carries `planning_impact: true` (the only ADR in this slice that does).
- **disposition:** SUPERSEDE/MERGE (folded into cloud-ci/Tide, NOT standalone). Its front-matter is STALE: it still reads `Proposed` with empty `superseded_by`, but ADR-0363 §3 explicitly states *"ADR-0513 places merge automation in the Prow-shaped cloud-ci/oya-ci Tide component (`oya-ci-tide`), folding ADR-0111 projected-state merge semantics into CI/admission."* ADR-0363 lists ADR-0111 only in `related:`, not `supersedes:` — so it was NOT formally superseded, leaving it dangling.
- **governing:** **ADR-0363 + ADR-0513** (Tide/oya-ci-tide owns merge automation; the *projected-merge-state semantics survive* but are relocated out of the retired `oya vcs` substrate). The valuable algorithm lives on; the host substrate (IP-006 `oya-foundry-vcs-merge-queue-*`) is retired.
- **truth_flag:** PARTIAL — the *algorithm* (projected-merge-state, conflict-before-CI, fix-at-any-stage re-validation) is TRUE and genuinely valuable (this is exactly GitHub/GitLab merge-queue and Prow-Tide batch-testing logic). The *packaging* (a bespoke `oya-foundry-vcs-merge-queue-conflict-kernel` over GitHub) is STALE.
- **in_masterplan:** PARTIAL — uniquely carries `planning_impact: true`, but points at a retired substrate and was never reconciled when 0363/0513 moved its semantics into Tide. So it is "in planning" pointing at the wrong owner — a binding that should be re-targeted to cloud-ci/oya-ci, not dropped.
- **tensions:**
  - **STATUS DRIFT (flag hard):** `Proposed` + `superseded_by:[]` while its three sibling ADRs (0110/0112/0113) are all `Superseded by ADR-0363` and 0363 explicitly absorbs *this* ADR's semantics. This is the keystone §6 "stale front-matter / supersession drift" pattern, in its purest form. Auditors must trust 0363/0513 over 0111's stale front-matter.
  - **Foundry-brand residue:** crate names `oya-foundry-vcs-merge-queue-*` use the RETIRED foundry prefix (should be `oya-vcs-*` per ADR-0363's rename, now folded to cloud-ci/Tide).
  - **GitHub vs Forgejo:** uses `gh api` for ref manipulation; Forgejo "has no native merge queue" (ADR-0363 §3), which is the precise reason Tide owns it now.
- **hyperscaler_challenge:** ALIGNED (algorithm), MISALIGNED (location). Projected-merge-state + batch re-testing + conflict-before-CI is *exactly* what Google's Rapid/TAP, GitHub merge queue, and Prow Tide do — this is the one decision in the VCS cluster a hyperscaler would absolutely make. But they'd put it in the CI/merge-automation layer (Tide), not in a bespoke VCS CLI ratchet. Argues for SUPERSEDE-into-Tide with the *algorithm preserved* (which is what 0513 does) — do not lose the semantics on archive.
- **ai_slop:** Low. Concrete, correct algorithm with honest negative-consequences (quadratic re-validation cost, ref-hygiene GC, MAX_REPOSITION cap). The cost numbers are reasoned, not fabricated.
- **refinement:** (1) **Fix the front-matter NOW:** set `status: Superseded` (or `Merged`) with `superseded_by: [ADR-0363]` / a pointer to ADR-0513-Tide, to clear the drift. (2) Capture the projected-merge-state algorithm as a first-class requirement on the cloud-ci/oya-ci-tide spec so the valuable semantics are not lost in the archive of 0110/0112/0113. (3) Re-target its `planning_impact` binding from the retired VCS substrate to cloud-ci.
- **consensus_needed:** **yes** — "ADR-0111's merge-queue algorithm is valuable and was meant to fold into cloud-ci/Tide (ADR-0513), but its front-matter still says Proposed/un-superseded. Ruling: formally supersede-into-Tide and re-bind its planning_impact to cloud-ci, or re-author the algorithm as a fresh cloud-ci ADR?"

---

### ADR-0112 — Webhook-driven Foundry agent invocation

- **decision_atom:** A bespoke `oya-foundry-webhook-receiver-app` ingests GitHub webhooks (HMAC-verified, `X-GitHub-Delivery`-deduped, append-only delivery log, ≤3 retries) and routes `(event, action)` to Foundry agents to make the pipeline event-driven instead of poll-driven.
- **current_status:** Superseded (`superseded_by: [ADR-0363]`). Correct.
- **disposition:** ARCHIVE (superseded; webhook-receiver was among the 0-deployment dormant crates per ADR-0363).
- **governing:** **ADR-0363** (verified `supersedes: [ADR-0110, ADR-0112, ADR-0113]`).
- **truth_flag:** STALE — sound webhook-security engineering (fail-closed HMAC before dedup, sref-only secrets, idempotency) but the whole "Foundry agent invocation over GitHub webhooks" model is retired (Foundry brand dead → Intelligence; GitHub-webhook automation → Forgejo + cloud-ci event sources).
- **in_masterplan:** NO (retired; correct).
- **tensions:**
  - **Retired-vocab (double):** "Foundry" brand throughout (`oya-foundry-webhook-receiver-*`, "Foundry control plane", "Foundry agent") — RETIRED per ADR-0335/0347/0363 → Intelligence/Governance. Pure brand-residue.
  - **Forge fault-line:** built on GitHub webhooks + `X-Hub-Signature-256` + `gh api` post-back; ADR-0363 moves to Forgejo PRs + cloud-ci. Founder's GitHub-host directive does not rescue the *Foundry-agent-over-webhooks* automation model.
  - **Secret path** `sref://openbao/oya/foundry/github-webhook-secret` hard-codes the retired `foundry` namespace.
- **hyperscaler_challenge:** QUESTIONABLE→MISALIGNED. The webhook-receiver security shape (HMAC fail-closed, delivery dedup, replay-safe log) is correct and hyperscaler-grade *in isolation*. But building a bespoke receiver + agent-router over GitHub, when Prow/cloud-ci event sources or platform-native eventing already exist, is reinventing forge plumbing — the same over-build ADR-0363 retired. Argues for ARCHIVE (done); preserve only the HMAC/dedup security pattern if a cloud-ci event ingress needs it.
- **ai_slop:** Low textual slop; high *strategic* slop (detailed SLOs — "p50 < 500 ms, p99 < 5 s", "1000 events / 24 h fan-in cap" — for a never-deployed receiver). Fabricated precision in latency SLOs for non-running infra.
- **refinement:** None (archive). If cloud-ci ever needs Forgejo/webhook ingress, lift the fail-closed-HMAC + delivery-dedup + bounded-retry pattern (the genuinely reusable kernel) into a non-foundry, non-GitHub-specific spec.
- **consensus_needed:** no (governed by ADR-0363).

---

### ADR-0113 — VCS orchestrator (`oya vcs done`) end-to-end

- **decision_atom:** `oya vcs done` becomes the async-by-default, crash-idempotent kickoff that opens a PR against `dev`, writes the initial changeset-event-log row, returns a `changeset_id` immediately, and lets webhooks drive CI→review→merge→promote, with per-changeset USD/token/invocation cost budgets and an alarmed human `oya vcs override`.
- **current_status:** Superseded (`superseded_by: [ADR-0363]`). Correct.
- **disposition:** ARCHIVE (superseded; the `oya vcs` CLI ratchet is exactly what ADR-0363 retired — "use git as-is, don't even `oya git` wrap").
- **governing:** **ADR-0363** (verified `supersedes: [ADR-0110, ADR-0112, ADR-0113]`; founder basis: "do we even need vcs? retire vcs, we have jenkins + git").
- **truth_flag:** STALE — well-designed (zero-state orchestrator, idempotent on `changeset_id`, cost-budget-first-class) but the `oya vcs` command authority is explicitly retired; `oya` is narrowed to a governance-gate engine with no VCS/CI authority.
- **in_masterplan:** NO (retired; correct). The cost-budget concept (per-changeset USD/token/invocation caps + monthly-team-budget lane) is the one idea here with possible forward life — agentic cost governance — but it is not currently bound anywhere live.
- **tensions:**
  - **`oya vcs` CLI authority** is the single most-directly-retired thing in ADR-0363 ("retire `oya vcs`, `oya git`"). Front-matter correctly reflects supersession.
  - **Foundry residue:** `oya-foundry-vcs-orchestrator-{kernel,app}`.
  - **GitHub coupling:** `gh pr create`, PR-against-`dev`, `github.com/jason931225/oyatie/pull/4` literal URLs → retired for Forgejo/cloud-ci.
  - **Overlaps PR #605 territory:** the agent-orchestration/cost-budget/override-frequency concerns here are thematically adjacent to `agent-execution-controller.md` (decision-pending, treat as canonical not slop) — the cost-budget + override-alarming ideas may want to be salvaged into that lane rather than fully discarded.
- **hyperscaler_challenge:** MISALIGNED (the CLI orchestrator), ALIGNED (the cost-budget primitive). No hyperscaler ships an `oya vcs done` mega-command orchestrating its whole SCM pipeline — they use the platform (Critique/Piper, Prow, GitHub). But *per-unit-of-work cost budgets with monthly team caps and override-frequency alarms* is a genuinely hyperscaler-grade FinOps idea for agentic work. Argues ARCHIVE-the-orchestrator, SALVAGE-the-cost-governance (into agent-execution-controller / cloud-ci).
- **ai_slop:** Low textual; the async/sync output-shape JSON examples are concrete. Strategic slop again (full override-frequency lane, monthly-budget lane, `--wait-timeout-seconds=2h` for a retired command). Fabricated-precision: worked output rows (`total_duration_seconds: 1834`, `usd: 3.17`) for a non-deployed orchestrator.
- **refinement:** None for the ADR (archive). Salvage candidate: extract the per-changeset cost-budget + monthly-team-budget + override-frequency-alarming pattern as a forward-looking agentic-cost-governance note, ideally folded into the PR #605 agent-execution-controller decision rather than lost.
- **consensus_needed:** no for the orchestrator (governed) — but a soft yes on the salvage: "Should the cost-budget / override-frequency governance from retired ADR-0113 be preserved into agent-execution-controller (PR #605) before this ADR is removed?"

---

## Chunk notes for synthesis

**1. This chunk is the agentic-VCS graveyard.** Four of seven ADRs (0110/0111/0112/0113) are the bespoke `oya vcs` pipeline that ADR-0363 retired wholesale ("don't reinvent the wheel; use git as-is; retire vcs"). 0110/0112/0113 carry correct `superseded_by:[ADR-0363]`. **0111 is the lone drift:** still `status: Proposed`, `superseded_by:[]`, yet ADR-0363 §3 + ADR-0513 explicitly *fold its projected-merge-state semantics into cloud-ci/Tide*. ADR-0363 lists 0111 only under `related:`, not `supersedes:` — so 0111 was effectively orphaned (its host retired, its algorithm relocated, its front-matter never updated). This is the keystone §6 stale-front-matter pattern at its sharpest in the whole sweep. **Trust 0363/0513 over 0111's front-matter.**

**2. Strategic slop > textual slop.** The four VCS ADRs are individually well-engineered (closed enums, Ed25519 signing, dedup keys, fail-closed HMAC, idempotency, projected-merge-state). The slop is that ~900 lines of rigorous design shipped 0–1 dependents and were never deployed (per ADR-0363's own audit: of 20 `oya-vcs-*` crates only 2 are wired). Recurrent tell: fully-worked example payloads with fake-precise numbers (`usd_remaining: 4.73`, `tokens_remaining: 1_842_117`, `total_duration_seconds: 1834`, p99 SLOs) for systems that did not run. This is the "fabricated precision" slop class applied at architecture scale.

**3. Two salvage-worthy ideas hide in the graveyard** — flag for the masterplan, do not lose on archive:
   - **ADR-0111 projected-merge-state / conflict-before-CI / fix-at-any-stage** — genuinely hyperscaler-grade (Google TAP, GitHub merge queue, Prow Tide). ADR-0513 already relocates it to `oya-ci-tide`; ensure the spec captures it.
   - **ADR-0113 per-changeset cost budgets (USD/tokens/invocations) + monthly-team-budget + override-frequency alarms** — real agentic-FinOps. Thematically belongs with PR #605 `agent-execution-controller.md` (canonical, decision-pending). Salvage before removal.

**4. Retired-vocabulary leakage is dense here.** Every VCS ADR is saturated with the RETIRED `foundry` brand (`oya-foundry-vcs-*`, `oya-foundry-webhook-receiver-*`, "Foundry control plane", `sref://openbao/oya/foundry/*`). Per ADR-0335/0347/0363 + GLOSSARY (foundry RETIRED; cloud-intelligence/governance are valid), these are all brand-residue. Since the ADRs are superseded this is acceptable-as-history, but any *new* reference to these crate names is retired-vocab leakage (MFL-0002/0003).

**5. Forge fault-line runs straight through this chunk (keystone §5).** All four VCS ADRs are built natively on GitHub (Actions `workflow_run`, webhooks, `gh api`, `github.com/jason931225/oyatie/pull/N`). ADR-0363 retires this for plain-git + self-hosted Forgejo + Prow-shaped cloud-ci; ADR-0510 pushes further to a bespoke hyperscaler monorepo-VCS as destination. The founder's GitHub-host directive (`jason931225/oyatie`) is the *host* layer and does NOT rescue these ADRs — what 0363 killed was the GitHub-*automation-substrate* model (workflow_run-driven agents), independent of where the repo is hosted. Surface, do not resolve.

**6. The 0107/0108/0109 naming-and-lifecycle sub-cluster is the live, healthy part of this chunk** — but under-bound to the masterplan. 0107 is a clean archive-into-0105. 0108 (sunset schema) and 0109 (lifecycle framework) are live, sound mechanisms — yet neither carries `planning_impact: true` or a `masterplan_ref`, despite 0109's `adr-status` lifecycle being *exactly* the supersede/archive machinery that planning-ssot-consolidation.md wants to drive masterplan generation. **This chunk sits directly on the OPEN masterplan question** (authored-SSOT vs generated-from-ADRs): if ADRs generate the masterplan, 0109's adr-status lane is core infra; if masterplan is authority, 0108/0109 should bind into it. Neither ADR acknowledges the fault-line. Flag under both readings.

**7. The one genuine internal contradiction to fix:** ADR-0109 simultaneously schedules (§Decision item 6, §Consequences "Future M-CC phase: convert sunset-lifecycle to config-driven") and forbids (§"Migration policy": "successor-IP hereby withdrawn… removing this entry would be a silent regression") the same sunset-lifecycle→generic-framework migration. A two-pass authoring artifact left both directions in the file. Needs a one-direction resolution.

**8. Cross-chunk hyperscaler verdict for the cluster:** The VCS cluster is the corpus's clearest "build vs reuse" cautionary tale — a hyperscaler would NOT build a bespoke per-changeset state machine + webhook receiver + `oya vcs` orchestrator on top of an existing forge; ADR-0363 reaches the same verdict. The two ideas a hyperscaler WOULD keep (merge-queue projected-state, agentic cost budgets) are precisely the two salvage candidates in note 3. The lifecycle-automation universalism (0109's "automate everything, cost ≈ 0") is the one place a hyperscaler would pull back on ROI grounds — the single contested live decision in this chunk (consensus_needed=yes).
