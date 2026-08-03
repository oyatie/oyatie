# Frozen-reference (gate-baseline) fix — decision memo (task #14)

Status: **DECISION PENDING FOUNDER** (asked 2026-07-09, timed out — user away). Do NOT execute B unilaterally; it reverses two of the founder's own Proposed one-way-door ADRs. Nothing active is blocked by waiting (no capability move in flight).

## Problem

The ci keystone (PR #1216, squash `1801128b1`) relocated the firewall's frozen reference
`ci/facade/artifact-inventory-registry/gate-baseline.generated.json` byte-identically (right path,
stale content — cloud/cloud-ci keys) and deferred content re-keying to a postsubmit materialization
that **isn't wired**. That broke dev's push firewall (89 unjustified regressions + frozen_snapshot
provenance fail). Recovered by direct-push `731fd5669` (materialized correct faces; 104 ci/facade
entries, 0 cloud/cloud-ci). Root cause = the committed-face fragility on a capability move.

## The two options (mutually exclusive; #14 must resolve before the NEXT capability move)

### A — build the ADR-0596-designed controller (keep committed)
- gate-baseline STAYS committed (ADR-0596/0604 compliant; **exact-bytes** trust preserved).
- Finish `main-branch-materialized` + `merge_policy: controller-owned-main-materialization`
  (ADR-0596 §Decision lines 107-110): a controller re-keys + materializes the committed face on
  dev post-merge. This is EXACTLY the "unwired postsubmit materialization" that caused the break —
  A = finish building what the ADR already specifies. **No ADR reversal. Lower risk.**
- Cost: keeps the LAST committed shared merge-surface (ADR-0604 notes gate-baseline + signoff are
  the only residual ones); needs a privileged controller that writes to dev; automates AROUND the
  fragility rather than removing it.

### B — de-commit + regenerate from merge-base SOURCE (reverses 0596/0604)
- STOP committing gate-baseline; regenerate the frozen baseline from the merge-base SOURCE tree
  instead of `git show <merge_base>:<face>`. Within ADR-0596's own "re-point the read first" door.
- Removes the last committed merge-surface (completes the #828/ADR-0595/0604 de-commit north star);
  robust to producer-schema evolution (both sides use HEAD's keying); no controller-writes-to-main.
  This is the "fix the class, don't automate around it" path.
- Cost: trades **exact-bytes → recomputed-bytes** trust (baseline embeds `_provenance.config_digest`;
  a producer/`oya-ci.toml` change across the range flips it). Must **supersede ADR-0596 + ADR-0604**
  with a hardened design + adversarial review.

Recommended: **B** (north-star alignment + fix-the-class doctrine), conditional on invariant
preservation holding under adversarial review. But it reverses deliberate founder ADRs → founder call.

## Evidence (blast-radius agent, commit 731fd5669)

- The **ONLY** consumer of the committed blob is the emitter's `git show <merge_base>:<face_path>`
  (`ci/facade/scm-facts-snapshot/src/main.rs:738,921`). Everything else reads the UNTRACKED
  merge-base snapshot (`gate-baseline.merge-base.generated.json`, `baseline-ratchet/src/lib.rs:117`)
  or the MATERIALIZED on-disk copy (safe iff materialize runs first). Materializer is already
  owned-Rust: `ci/facade/generated-artifact-freshness` (NOT the stale `infra/ci/*.sh` — that path is
  pre-keystone only).
- Producer `ci/facade/artifact-inventory-registry/src/main.rs` takes `--repo-root` and PURELY
  censuses that tree (no ambient git, `main.rs:3-9,171-174,344-379`) → B's regen is mechanically
  feasible: `git worktree` at merge-base + run HEAD's producer. Caveat: also needs a merge-base
  scm-facts face generated at the emitter git boundary.
- The ADR-0596 gate `frozen_reference_artifact_must_stay_committed`
  (`ci/facade/generated-artifact-policy/src/lib.rs:574,634`) RED-blocks a naive de-commit at
  presubmit. B must re-point the emitter read FIRST (ADR-0596 door) or the gate blocks it.

## PR-1 STRANGLER FINDING (2026-07-09) — evidence now favors A

PR-1 (regen-from-source cross-check, additive, face stays committed) was BUILT + its real-binary
cross-check EMPIRICALLY caught a structural gap in B the design missed. Preserved at branch
`feat/frozen-ref-decommit-pr1` (commit cc6b56f6e, unpushed). H3 core verified correct
(`frozen_projection_divergences` compares {mode,frozen_empty,keys} per (gate,code); a mode downgrade
IS caught). The gap: the committed frozen baseline counts keys the producer censuses from
`not-tracked-in-git` CONTROLLER artifacts (`masterplan.generated.json`, `product-graph.html` —
de-committed by ADR-0613; confirmed absent from `git ls-tree HEAD`). B regenerates from a git worktree
(committed files only) → structurally CANNOT reproduce those keys → cross-check RED on real merge-bases.

**Implication:** B's "regenerate from merge-base SOURCE" is really "run the FULL materialize chain
(masterplan gen + arch-graph gen + emitter + producer) at the merge-base worktree" — because the frozen
baseline is a function of the FULLY-MATERIALIZED tree, not committed source alone. Fixable (justified by
ADR-0613's determinism guarantee) but makes B substantially HEAVIER (full-chain merge-base materialize
per CI run).

**Accumulated evidence now points to A** (three findings via diligence): (1) B trades STRUCTURAL frozen
trust → REVIEW-VISIBILITY (adversarial review); (2) B is within ADR-0596's door but reverses 3 ADRs; (3)
B's regen is a full-chain merge-base materialize, not just the producer (this finding). A ("finish the
`controller-owned-main-materialization` the manifest ALREADY declares") avoids ALL THREE: exact-bytes
structural trust (git-show), NO merge-base regen, no ADR reversal. A's cost (last committed surface +
dev-writing controller) is bounded, and the relocation-fragility that caused the keystone break is handled
by A's controller re-keying the face on moves. **REFINED RECOMMENDATION: switch to A.** Reverses the
founder's explicit B choice → founder nod warranted before the heavy B-option-1 build. PR-1's cross-check
tool is reusable under A (validate the controller-materialized face == from-full-materialize regen).

## FULL-CHAIN ATTEMPT FINDING (2026-07-09) — B is the Bazel model; cost objection gone

The full-chain regen fix (run controller generators at the merge-base worktree) was implemented +
real-binary tested → STAYED RED, for a decisive reason: the producer censuses the git-TRACKED path
universe (scm-facts = `git ls-files`), NOT the on-disk filesystem (`artifact-inventory-registry/src/main.rs:3481`
`tracked_paths = scm_facts.tracked_paths`; collect_brand_residue/total-accounting read tracked paths only).
So materializing controller artifacts on disk does nothing — they're UNTRACKED at the merge-base (ADR-0613)
→ producer omits them. AND: the committed `gate-baseline.generated.json` is STALE — it carries keys for
now-untracked files (product-graph/masterplan) + a reclassified tracked workflow (docs-graph-drift.yml),
because it's controller-owned and wasn't re-materialized when #1222 changed the tree. PR-1's
"cross-check regen ≡ committed git-show" validated against a STALE reference → can never be green.

**Refinement (NOT a reversal — CONFIRMS B under the hyperscaler criterion):**
- B = the exact Bazel affected-set model: frozen = producer over merge-base TRACKED tree; candidate =
  same producer over HEAD tracked tree; diff. Same tool both sides, no committed baseline.
- The "full materialize chain" cost concern EVAPORATES — controller generators are irrelevant (untracked
  outputs never censused). Regen = producer-over-merge-base-tracked-tree (git-archive/worktree of tracked
  files + merge-base scm-facts). Cheap.
- Validate via DETERMINISM (regenerate-twice canary, the hyperscaler mechanism) + the firewall's own
  ratchet RED/GREEN fixtures + real-run semantic check — NOT cross-check-against-stale-committed (drop it).
- The staleness is derive-don't-commit made concrete: the committed face went stale BECAUSE it's committed
  high-churn derived state. A patches it with a trunk-writing refresh bot (write-amplification); B removes it.

**Honest dissent:** BOTH independent analyses (architect review + full-chain executor) recommended A.
Overridden on the founder's hyperscaler criterion + the high-churn-baseline distinction (Bazel computes
per-commit base from source, never commits it). REVISED PLAN: emitter regenerates frozen baseline from the
merge-base TRACKED tree (replacing git-show), validated by determinism + firewall fixtures; then de-commit +
invert ADR-0596 gate + provenance. Preserved PR-1 (cc6b56f6e) cross-check tooling repurposed as the
determinism/semantic validator.

## ALL-GENERATED-SETS AUDIT RESULT (2026-07-09, founder-directed) — transition is SURGICAL

Control-plane faces (11): **8 already de-committed** (A1-A6 accounting/crosswalk/enforcement/ttl, A9
masterplan, A10 product-graph — Bazel model PROVEN in production). **3 still committed:**
- **A7 gate-baseline.generated.json** → DE-COMMIT + RECOMPUTE from merge-base TRACKED tree (THE transition).
  CONFIRMED stale (the 2026-07-08 signoff itself documents the stale-face false-positive on 6 pdp-cedar
  files). Safe ONLY because its hand-curation is externalized to gate-baseline.signoff.json (stays committed).
- **A8 board-sync.generated.json** → SUSPECTED stale + NO determinism canary (same setup that let masterplan
  ship stale). FOLLOW-ON: add canary + regen-diff, then de-commit. Not blocking A7.
- **A11 move-manifest.generated.json** → STAY (byte-bound review artifact).

**STAY-COMMITTED — do NOT recompute (the dangerous cases the audit caught):**
- **B1 friction-accounting-baseline.json, B2 embedded-asset-hermeticity-baseline.json, B4 port-placement-baseline.json**
  — explicitly HAND-SHRUNK ("NOT producer-regenerated"); recompute would erase the burn-down ratchet.
- **B3 tier-dependency-acyclicity-baseline.json — THE TRAP:** has an `--emit-baseline` producer (looks
  Bazel-eligible) BUT subset semantics → candidate recompute would LAUNDER new regressions (the PR #670
  hole). Recompute FORBIDDEN; only merge-base-frozen semantics if ever de-committed. Also suspected stale
  (reorg-move-sensitive crate-edge subjects).
- **gate-baseline.signoff.json — CRITICAL:** the one-way door where A7's human intent lives; A7 de-commit is
  safe ONLY because this stays a separate committed input. + ratchet-policy.json (source config),
  warning-baseline.tsv (glossary allowlist), openapi.snapshot.yaml (vendored external), test fixtures/goldens.

**Governance gap:** B1-B4 gate baselines + warning-baseline.tsv are OUTSIDE registry/generated-artifact-control-plane.json
(the SSOT). FOLLOW-ON: declare them (with STAY-COMMITTED intent) so no one naively de-commits them.

**Net: the transition is A7-only now (surgical), NOT fleet-wide.** A7 de-commit cleared. Follow-ons:
A8 canary+de-commit, B3 staleness check, control-plane coverage of B1-B4 + warning-baseline.

## Invariants any solution MUST preserve (ADR-0551 + emitter/firewall headers)

- Frozen ref = merge-base, never working tree (anti-laundering; pre-0551 committed-candidate-face
  was launderable because settle regen makes committed==proposed).
- **Fail-closed**: unresolvable base_ref/merge-base = hard error; missing snapshot = gate fail;
  absent-at-merge-base = DECLARED-empty; empty-without-`missing_at_merge_base` = rejected/tampered.
- **Frozen-policy-wins**: base_ref + face_path read from merge-base via OUT-OF-BAND bootstrap ref,
  never candidate (else `base_ref:HEAD` → self-laundering fixpoint).
- **Frozen-mode-wins**: per-code mode read from FROZEN baseline; candidate must not influence
  which modes/dispositions the frozen baseline carries (B's key risk — regenerate reads
  merge-base source ONLY; producer = HEAD code = review-visible trust tier per ADR-0551).

## Anti-slop review of the keystone code (oh-my-claudecode:code-reviewer, opus)

**APPROVE-WITH-FOLLOWUPS.** Zero CRITICAL/HIGH/MEDIUM. R100⟺byte-identical holds fail-closed under
hostile reading (`is_sanctioned_relocation`, `generated-artifact-policy/src/lib.rs:589`). 3 P3 nits:
- P3-1: add boundary fixtures (`R1000` still-violation, `C100` copy exempted) — cheap ladder hardening.
- P3-2: `FROZEN_MERGE_BASE_FACE_PATH` hand-maintained const (manual-shepherding pattern) — derive from
  ratchet policy at merge-base blob; low value.
- P3-3: redundant `candidate_paths.len()==2` — reviewer says KEEP (panic-safety); do NOT simplify away.
- Follow-up: confirm `ci/adapters/path-resolver` `at_merge_base` `(true,true)`/`(false,false)`
  hard-error arms are pinned by unit tests (new trust root for frozen-ref selection).
