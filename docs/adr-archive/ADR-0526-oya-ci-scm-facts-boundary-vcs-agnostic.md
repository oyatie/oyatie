---
id: ADR-0526
title: "oya-ci scm-facts boundary: VCS-agnostic identifiers (git-facts->scm-facts rename, schema v1 retained) + the ScmFactsSource adapter trait seam (git is transitional impl #1)"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: [ADR-700]
amended_by: [ADR-0552]
depends_on: [ADR-0525]
amends: [ADR-0525, ADR-0515]
related: [ADR-0510, ADR-0515, ADR-0516, ADR-0518, ADR-0520, ADR-0525]
related_specs:
  - /specs/gitops-vcs-replacement.json
  - /specs/masterplan.json
milestone: W1
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0526: oya-ci scm-facts boundary — VCS-agnostic identifiers + the ScmFactsSource adapter seam

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

**Amends ADR-0525** (renames its git-facts boundary to scm-facts) and **amends ADR-0515 D3** (adopts
scm-facts vocabulary). De-risks the ADR-0510 cutover to a single adapter impl-swap. Sequenced AFTER
ADR-0525 (the hermetic boundary must exist before it can be renamed).

## Context

The hermetic oya-ci facts boundary (ADR-0525, refining ADR-0515 D3) names its transitional
implementation — git — throughout the STABLE PORTABLE CONTRACT (the snapshot) and the SWAPPABLE ADAPTER
(the emitter). The founder governing constraint is "cloud native, hyperscale pattern, hermeticity in
mind — GIT IS TRANSITIONAL"; the settled-vision W1 mandate names `scm-facts` explicitly as an interface
to lock (ADR-0520). The ADR-0510 cutover (concretely defined by ADR-0518) makes a bespoke non-git
destination a decided-but-deferred reality the contract must already accommodate.

## Decision

Two coupled, byte-parity-preserving moves, and nothing else:

**(1) RENAME every git-flavored identifier in the boundary to the VCS-agnostic family
`scm-facts` / `scm_facts` / `SCM_FACTS` / `ScmFacts`.** The emitter crate dir + binary
`oya-cloud-ci-git-facts-emitter-app` → `oya-cloud-ci-scm-facts-emitter-app`; the committed snapshot
`git-facts.generated.json` → `scm-facts.generated.json`; the schema id `oya-ci/git-facts/v1` →
`oya-ci/scm-facts/v1` (v1 RETAINED — shape byte-identical, only the namespace word changes); the
producer CLI flag `--git-facts` → `--scm-facts`; symbols `GitFacts`→`ScmFacts`,
`load_git_facts`→`load_scm_facts`; the registry-drift `NON_GATE_CRATES` entry, `GIT_FACTS_FACE`→
`SCM_FACTS_FACE`, `regenerate_git_facts`→`regenerate_scm_facts`, the
`committed_git_facts_equal_regenerated`→`...scm...` check, env `OYA_CI_GIT_FACTS_REGEN`→
`OYA_CI_SCM_FACTS_REGEN`; all gate-test `--git-facts` call-sites; the regen workflow step
name/env/target/diff-path/prose. KEEP the four data JSON keys (head_time_secs, tracked_paths,
last_touch_commit, commit_author_ts_secs) — already VCS-neutral; renaming them would force a producer
parse change and break v1 for zero agnostic-ness gain. KEEP `OYA_CI_PRODUCER_BIN` (producer-naming,
not git-naming). `last_touch_commit`'s git-ish word is deferred to a future v2 (`last_touch_revision`)
to preserve a non-breaking v1.

**(2) REFACTOR the emitter's git-shelling internals** (the three helpers `git_commit_timestamps` /
`git_ls_files` / `git_last_touch` — the ONLY VCS coupling in the entire pipeline) behind a pluggable
trait `ScmFactsSource` exposing exactly three primitives: `tracked_paths()`, `last_touch()`
(path→revision id, generated-class-filtered), `revision_author_timestamps()`. The git CLI becomes the
explicit transitional impl #1 `GitCliScmFactsSource` — the one place permitted to exec git (the
ADR-0515 D3 narrow exception). A bespoke-SCM impl plugs in later as impl #2 by implementing this trait
with ZERO churn to the producer, the gates, or the snapshot shape. No registry/factory and no
`--scm-kind` selection flag now (YAGNI; the seam is the trait); selection is a single hardcoded
`GitCliScmFactsSource` until impl #2 arrives.

## Drivers

- Founder constraint: GIT IS TRANSITIONAL — remove the last leak of the transitional impl name from
  the boundary so the GitHub→bespoke-SCM cutover (ADR-0510) is invisible to the producer + gate corpus.
- The settled-vision W1 "lock the interfaces (...scm-facts...) with infinite-scale baked in"
  (ADR-0520).
- The ADR-0280 transitional-impl-behind-a-stable-interface doctrine (instantiated by the trait seam).

## Alternatives considered

- **`repo-facts`** — rejected: a bespoke hyperscale destination may be a content-addressed
  object/fact service, not a "repo"; `scm` names the capability not the container and matches founder
  vocabulary + the `ScmFactsSource` adapter name.
- **Renaming the four data keys / bumping to v2 now** — rejected (pure churn; breaks non-breaking v1;
  forces a producer parse change).
- **A `--scm-kind` flag / registry now** — rejected (YAGNI, one impl).
- **A back-compat alias for `--git-facts`** — rejected (one caller set, all renamed atomically,
  publish=false, no external consumer).
- **EdenFS/FUSE or building bespoke SCM now** — out of scope (governed by ADR-0510's numeric trigger).

## Consequences

Positive: the stable contract + adapter no longer name git, so the ADR-0510 cutover becomes a one-impl
swap; the single VCS coupling is now an explicit, unit-testable trait seam; the W1 "lock the
interfaces" mandate is satisfied for scm-facts. Cost: a **2-commit settle** is required (commit A =
rename + code + git mv; regenerate the renamed snapshot + the accounting faces; commit B = settle the
regenerated snapshot + faces) because git treats the rename as delete-old + add-new, assigning the
renamed paths a new `last_touch_commit` = commit A — this rides the emitter's existing convergence
mechanism (generated-class files are nulled from last_touch). HARD INVARIANT: identifiers-only change;
the accounting-face COUNTS stay byte-identical (the only legitimate face delta is the renamed path
strings and the schema-id string, which is content-neutral because the producer never reads the
snapshot's schema field). **Amends ADR-0525 and ADR-0515 D3.** door:one-way.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source:
OYA-CI-VCS-AGNOSTIC-SEAM-REFINEMENT-PLAN.md (RATIFY-TO-ADR). Amends ADR-0525 + ADR-0515 D3; de-risks
ADR-0510 cutover. NOTE: the rename touches the canon-id-crosswalk at integration time (registry keys
+ traceability.source_adrs).*
