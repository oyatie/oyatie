---
id: ADR-0552
title: "Stable/volatile SCM-facts split: history-derived facts leave the merged surface"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-12
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-701]
amended_by: []
depends_on: [ADR-0515, ADR-0526, ADR-0539, ADR-0551]
amends: [ADR-0526]
related: [ADR-0111, ADR-0363, ADR-0541, ADR-0544, ADR-0545]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0552: Stable/volatile SCM-facts split — history-derived facts leave the merged surface

## Status

**Proposed - 2026-06-12 (authored for founder sign-off; door: one-way — face schema v2).**

## Context

FRIC-1781234047: once issue #619 wired `oya-ci-required` to run on `push` to `dev`, every
completed post-merge run FAILED in freshness (3/3: d5d8be5d4, 9603514a2, d43d936d5; e.g.
run 27390763020) with `generated_face_stale scm-facts.generated.json` +
`accounting-registry.generated.json` — while the SAME trees were green in their PR-event
runs.

The defect is structural. The committed scm-facts face (v1) embedded HISTORY-derived
volatile facts: per-path `last_touch_commit` revision ids, `commit_author_ts_secs`, and the
`head_time_secs` aging anchor, flowing into committed accounting-registry rows and the
committed gate-baseline's staleness keys. A squash-merge preserves the TREE but mints a new
commit id for every path the PR touched, so the faces that were byte-settled in the lane
recorded revision ids that do not exist in `dev` history: `dev` was un-settled BY
CONSTRUCTION after every content merge. Three independently ledgered symptoms share this
one root cause:

1. FRIC-1781250000 — ANY later commit touching a non-generated-class path (even docs-only)
   un-settled the faces; patched with protocol (the mandated `--verify` last step, PR #696).
2. The "self-referential merge-conflict surface" — concurrent lanes' settle commits
   conflicting in generated JSON (the reason CI stopped byte-comparing at materialization).
3. FRIC-1781234047 — the structural dev-push freshness RED above.

Founder directive (2026-06-12): "flagging, red gating isn't enough. automate everything
that can be automated canonically, universally." Construction beats reaction when costs are
comparable.

## Options considered (all four, on the merits)

**Option 1 — trailing settle-PR automation (REACTIVE; rejected).** A `dev`-push lane that,
when `face-settle --verify` fails, branches, runs the Rust settle automation
(`--settle --commit`, PR #668), and opens a faces-only PR through the gate pipeline.
Precedent: Kubernetes test-infra autobump, Chromium autorollers, autofix.ci. Honest costs:
one extra bot PR per content merge (merge traffic ~doubles); bot-PR churn and races (every
next content merge stales the in-flight settle PR); `GITHUB_TOKEN`-created PRs do not
trigger `pull_request` runs, forcing a fragile `workflow_dispatch` re-dispatch workaround
for the required context; `dev` stays red in the window between merge and settle-PR merge,
so the push lane never becomes a clean signal; and it repairs ONLY symptom 3 — symptoms 1
and 2 (protocol burden, cross-lane settle conflicts, per-PR face diffs polluted with
unrelated last-touch drift) remain. Reaction, not construction.

**Option 2 — merge-queue settle-at-merge (NOT IMPLEMENTABLE NOW; revisit at owned Tide).**
Settling at admission obviates trailing PRs with zero extra traffic. But GitHub's managed
merge queue cannot inject content into the merge it creates — there is no hook to add a
settle commit — and granting a bypass actor direct write to `dev` to do it outside the
queue worsens the trust posture ADR-0515 exists to protect. A bespoke oya-ci Tide
(ADR-0515 direction) could own settle-at-merge, but that is a future substrate, and even
then volatile data would remain IN the merge surface: symptoms 1 and 2 persist. Recorded as
a Tide-era option that option 3 makes unnecessary.

**Option 3 — stable/volatile split (CONSTRUCTIVE; CHOSEN).** History-derived volatile facts
leave the merged surface entirely:

- `scm-facts.generated.json` (committed, schema `oya-ci/scm-facts/v2`) carries ONLY
  tree-derived stable facts: `tracked_paths`.
- A new UNTRACKED, gitignored, CI-rematerialized snapshot
  `ci/facade/scm-facts-snapshot/scm-volatile-facts.generated.json`
  (schema `oya-ci/scm-volatile-facts/v1`) carries `last_touch_commit`,
  `commit_author_ts_secs`, and the deterministic `head_time_secs` aging anchor — the
  ADR-0551 materialized-snapshot pattern, emitted by the same single sanctioned git
  boundary (the scm-facts emitter, ADR-0515 D3/ADR-0526) on every materialization.
- Committed accounting-registry rows drop the `last_touch_commit` column; the committed
  gate-baseline's staleness keys (`stale_over_budget_unreachable`, `untyped_staleness`)
  become deliberately EMPTY: volatile-derived findings are GATE-EVALUATION findings, aged
  from the volatile snapshot at evaluation time by the staleness gate itself (fail-closed
  on a missing snapshot, remediation = the materialize command); their blocking authority
  is the gate lane, not the firewall ratchet. The disposition rows (modes) stay declared,
  so a future flip remains a reviewed DATA edit.

Every committed face becomes a pure function of the committed TREE. A squash-merge
preserves the tree, therefore: a settled PR squash-merged onto `dev` leaves `dev` settled —
the defect class is ELIMINATED by construction, not repaired after the fact. All three
symptoms die together: docs-only/later commits no longer un-settle faces (the
FRIC-1781250000 protocol burden shrinks to real face-relevant changes), concurrent lanes
stop conflicting over last-touch churn and stop absorbing each other's drift into their
settle diffs (attribution becomes exact), and the `dev` push lane becomes a MEANINGFUL
signal — green by construction on honest merges, RED only on a real regression (an
unsettled tree reaching `dev`, a regen-toolchain fault, a bypass push). Precedent (proven
methodology, Rust-native implementation): Bazel splits `volatile-status.txt` from
`stable-status.txt` precisely so stamp data never invalidates hermetic action keys;
reproducible-builds normalizes volatile time out of comparable artifacts (SOURCE_DATE_EPOCH);
SLSA keeps provenance beside, not inside, the subject artifact.

**Option 4 — content-addressed face identity (DESTINATION; ADR-0541).** The Corpus
Liveness Graph replaces path+history identity with content-addressed graph identity and
per-class decay invariants — the durable end-state for staleness semantics. Not shippable
now (ADR-0541 is Proposed substrate research). Option 3 is a strict stepping stone: it
removes history-coupling from merged artifacts, which is prerequisite work for
content-addressed identity, and the volatile snapshot becomes the natural seam where
graph-derived liveness later replaces git-derived aging.

**Construction vs reaction, explicitly:** options 1/2 ADD machinery that runs forever to
repair a defect the data model keeps re-creating; option 3 DELETES the defect's substrate
(and with it the need for that machinery) at comparable one-time cost. Per the founder
directive, construction wins.

## Decision

1. **Stable/volatile split as specified in option 3.** Schema bump `oya-ci/scm-facts/v1`
   → `v2` (this ADR amends the ADR-0526 face shape; the `ScmFactsSource` trait — the
   VCS-agnostic seam — is UNCHANGED: same three primitives, so a future bespoke SCM source
   is unaffected).
2. **Single-owner, canonical (#696 precedent).** No second comparison or settle
   implementation: the freshness gate's own check/regeneration functions and the
   `oya-cloud-ci-face-settle` machinery are reused as-is; the only behavioral change they
   see is that regenerated committed faces are now tree-pure. The emitter remains the single
   out-of-graph git boundary and now writes both artifacts in one pass
   (`--out` + `--volatile-out`).
3. **Universal (R0).** The mechanism is repo-agnostic: any repo using the generated-faces
   pattern gets the same invariant (committed faces = f(tree)); paths are
   flag-overridable (`--volatile-out`), and no oyatie-specific value is baked into the
   split itself.
4. **Fail-closed seams.** The producer hard-errors on a missing/malformed stable face
   (unchanged). The staleness gate hard-fails on a missing/malformed volatile snapshot,
   naming `buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin` (CI rematerializes before
   every gate lane; the snapshot is never silently defaulted). `face-settle --verify`
   stays read-only: its regeneration routes the volatile snapshot to a temp path.
5. **Convergence pin (charter requirement).** Generated-class paths are excluded from
   `last_touch` AT THE EMISSION SEAM for any `ScmFactsSource` implementation (not just the
   git walk), unit-pinned by `generated_class_paths_never_enter_volatile_last_touch`: a
   faces-only settle commit can never grow the volatile snapshot or advance its aging
   anchor, so settle commits stay fixpoints.

## Verification (RED/GREEN)

- Premise RED: 3/3 `dev` push runs failed freshness pre-change (run 27390763020 at
  9603514a2 et al.) while PR-event runs at the same trees were green.
- GREEN by construction: with v2 faces, `git commit --allow-empty` / docs-only commits /
  squash-merges leave `face-settle --verify` green (committed faces are tree-pure); the
  emitter unit tests pin both shapes; the convergence pin test pins the settle fixpoint.
- The full gate suite (`buck2 test //cloud/cloud-ci/...`) is the regression net: freshness,
  registry-drift byte-parity, the firewall ratchet (ADR-0551 merge-base frozen baseline —
  unaffected: gate-baseline keys are paths), and the staleness gate's born-blocking live
  test now aged from the volatile snapshot.

## Consequences

- The committed faces shrink substantially (no per-path revision ids/timestamps); per-PR
  settle diffs now contain ONLY the PR's own face-relevant changes — exact attribution.
- The staleness advisory burn-down (54 keys at the time of writing) moves from the frozen
  baseline to live gate evaluation; the firewall no longer ratchets volatile-derived codes
  (their baseline keys are empty by construction). If a volatile-derived code ever needs
  ratcheting, that requires a frozen-time semantics decision of its own — deliberately out
  of scope here.
- The `dev` push lane's freshness signal becomes meaningful: RED = real regression. No
  trailing bot, no extra merge traffic, no new required context.
- Local protocol relaxes truthfully: only face-relevant tree changes un-settle faces;
  `--verify` before push remains mandated as the canonical backstop.
- Historical faces at old revisions keep the v1 shape; the ADR-0551 merge-base frozen
  baseline reads the gate-baseline face (path keys only) and is shape-compatible across
  the transition.
