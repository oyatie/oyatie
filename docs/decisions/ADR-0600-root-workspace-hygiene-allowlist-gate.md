---
id: ADR-0600
title: "Root-workspace-hygiene allowlist gate — make committed repo-root scratch structurally impossible"
status: Accepted
planning_impact: false
deciders: founder
date: 2026-06-24
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: []
depends_on: [ADR-0515, ADR-0539, ADR-0551, ADR-0552, ADR-0555]
related: [ADR-0523, ADR-0548, ADR-0562]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0600: Root-workspace-hygiene allowlist gate

## Status

**Proposed - 2026-06-24 (authored for founder sign-off; door: two-way — the allowlist is DATA
and the gate can be tuned or retired by editing/removing policy without a structural commitment).**

## Context

The repository root accumulated committed process scratch: the Jun-10 G011 slice-6 burndown logs
(`slice06-*.log`), retest/target scratch (`retest-targets.txt`, `backfill-targets.txt`,
`branch-wired-members.txt`, `final-targets.txt`), a dead root shell-runner (`run-slice.sh`, which
also violates the no-shell doctrine), and the protocol-induced `premise.txt` / `review-verdict.txt`
lane pollution (FRIC-1781132000 — the documented lane protocol once mandated committing those at
the repo root). The founder flagged this as committed garbage in the repo root and asked that the
recurrence be made structurally impossible ("friction = process failure → productize the gate").

A partial layer already exists: `cloud-ci-total-accounting` emits the `scratch_artifact`
frozen_empty code, driven by the producer's `unit-class-policy.json` DENYLIST of KNOWN scratch
shapes (`root_suffix .log`, `run-slice.sh`, `retest-targets.txt`, …). That denylist is
zero-tolerance on the shapes it names — but it is structurally blind to a NEW root-scratch shape
nobody has named yet. A `git add -f sandbox-notes.txt` at root sails through, because no rule
matches it. The denylist cannot close the class; only an ALLOWLIST can.

The `.gitignore` already carries a root-scratch prevention layer (`/*.log`, `/run-*.sh`,
`/slice*-*.txt`, …). But `.gitignore` is prevention-only: it does not catch `git add -f`, and it
does nothing about files that are ALREADY tracked. The gate must be the real enforcement.

## Decision

Ship `oya-cloud-ci-root-workspace-hygiene-app` — a born-blocking, UNIVERSAL, HERMETIC gate with a
default-DENY posture and the legitimate root surface expressed as DATA.

**Allowlist-as-DATA (universal).** `root-workspace-hygiene-policy.json` declares
`allowed_root_files` (a rule table of `exact`/`suffix`/`prefix` basename matchers) and
`allowed_root_dirs` (the permitted top-level capability/meta homes). The engine reads both as
DATA, so it runs on ANY repo; oyatie's allowlist is just the pack. Oyatie's `allowed_root_files`
admits exactly the legitimate root surface (`Cargo.toml`, `Cargo.lock`, `.buckconfig*`,
`.buckroot`, `rust-toolchain.toml`, `rustfmt.toml`, `reindeer.toml`, `deny.toml`, `README*`,
`LICENSE*`, `CLAUDE.md`, `AGENTS.md`, `HANDOFF.md`,
`oya-ci.toml`, `Makefile`, `Dockerfile.distroless`, `.gitignore`, `.gitattributes`,
`.editorconfig`); `allowed_root_dirs` enumerates the ~47 real top-level directories.
The removed scratch shapes are NOT allowlisted.

**Pure evaluator (hermetic).** The gate is a pure function over `(policy DATA, observed
tracked-path inventory)`. The inventory is the producer's git-ls-files snapshot
(`scm-facts.generated.json`) — a declared input — so the evaluator does no shell, network, clock,
rand, or filesystem access beyond its declared inputs, is deterministic, and is buck2-built.

**Default-DENY (born-blocking).** Any TRACKED file at the repo ROOT (a path carrying no `/`) whose
basename matches no allowlist rule fires `root_workspace_unallowlisted_file`. Any tracked path
whose first segment is not a permitted top-level directory fires `root_workspace_unallowlisted_dir`.
Both are born-blocking — there is no frozen baseline of accepted root scratch, because the clean
tree has zero offenders (this PR removes the three that existed).

**Auto-fix, not flag-only.** Each finding carries a concrete per-path remediation: `git rm` the
scratch (relying on the `.gitignore` backstop) or relocate it under `.omc/` (the gitignored scratch
home); a genuinely-legitimate NEW root file is admitted by adding a reviewed allowlist rule (a DATA
edit, never a scanner change).

**`.gitignore` backstop (prevention layer).** Extended with `/*.tmp`, `/*.out`, `/*-targets.txt`,
`/premise.txt`, `/review-verdict.txt`, `/branch-wired-members.txt`. The GATE remains the real
enforcement; `.gitignore` cannot catch `git add -f` or already-tracked files.

This PR also removes the three confirmed-dead TRACKED root scratch files
(`backfill-targets.txt`, `branch-wired-members.txt`, `final-targets.txt`) — verified
unreferenced by any live tooling/CI/registry/specs — and regenerates `scm-facts.generated.json`
+ the frozen `gate-baseline` via the materialize tooling. The other named scratch
(`slice06-*.log`, `retest-targets.txt`, `run-slice.sh`, root `premise.txt`/`review-verdict.txt`)
was already swept off `dev` and is `.gitignore`-covered; the root `premise.txt`/`review-verdict.txt`
intended home (`.omc/ultragoal/`) is un-ignored and already carries the live `premise.txt`.

## Consequences

- Committed repo-root scratch becomes structurally impossible: the allowlist born-blocks ANY
  unrecognized root file, closing the class the shape-based denylist leaves open. The two layers
  compose — denylist = zero-tolerance on known shapes, allowlist = default-DENY on everything else.
- The gate is universal: another repo adopts it by replacing the allowlist/allowed-dir DATA; zero
  engine change. It is hermetic (pure evaluator over declared inputs, buck2-built) and auto-fixing
  (each finding emits a concrete relocate/`git rm` remediation).
- A NEW legitimate root file (e.g. a future top-level config) requires a one-line reviewed DATA
  edit to `allowed_root_files`. This is the intended friction: every new root surface is a
  reviewed decision, not an accident.
- Risk surface: an allowlist that accidentally FORBIDS a legitimate root file would break every
  PR. Mitigated by the live-corpus self-test (`live_tracked_root_tree_is_allowlist_clean_green`),
  which asserts the gate is GREEN over today's full tracked tree — so a too-narrow allowlist is
  caught in this PR, not in a future contributor's PR.

## Alternatives considered

- **Extend the existing `scratch_artifact` denylist only.** Rejected: a denylist enumerates known
  scratch shapes and is permanently blind to unnamed ones; it cannot make the class impossible.
  The allowlist is kept as a complementary layer, not a replacement.
- **`.gitignore` only.** Rejected: prevention-only; does not catch `git add -f` or already-tracked
  files. Kept as the backstop layer beneath the gate.
- **A producer-face gate (mirror `manifest-hygiene`).** Unnecessary: the gate needs only the
  tracked-path inventory (already emitted as `scm-facts`) plus its own allowlist DATA. Mirroring
  the self-contained `rust-first-automation-hygiene` policy-as-data shape avoids touching the
  producer / `oya-ci-config` gate-face enum / accounting registry shape.

## Governed surfaces

This ADR OWNS and JUSTIFIES the gate crate; its verbatim tracked paths (the canonical, byte-stable
enumeration the born-accounting producer credits — the same set `register_crate` would emit) are:

```
ci/facade/repo-root-hygiene/BUCK
ci/facade/repo-root-hygiene/Cargo.toml
ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json
ci/facade/repo-root-hygiene/src/lib.rs
ci/facade/repo-root-hygiene/tests/root_workspace_hygiene.rs
.claude/BUCK
.claude/OWNERS
.codex/BUCK
.codex/OWNERS
tools/hooks/BUCK
tools/hooks/OWNERS
```

The `.claude/BUCK`, `.claude/OWNERS`, `.codex/BUCK`, `.codex/OWNERS`,
`tools/hooks/BUCK`, and `tools/hooks/OWNERS` markers are the reviewed DATA/ownership
surface that lets enforcement-liveness declare the existing hook/config corpus as Buck inputs
without broad root-state drift or generated-face edits.

## Born-accounting

Adds one gate crate (`ci/facade/repo-root-hygiene`) and one
decision node (ADR-0600). The crate is OWNERS-covered by the breadth-unlimited
`ci/OWNERS` (`cloud-ci-platform`, ADR-0555); its Cargo.toml is swept by the
`cloud/cloud-ci/gates/*` workspace glob; its files are auto-accounted via the regenerated
`scm-facts`. It is registered as a homogeneous matrix lane in
`.github/workflows/oya-ci-required.yml` (the gate-registration meta-test requires this). Gate apps
carry no catalog/SLO records (they are CI machinery, not runtime services). No
`oya-ci-config`/`gate-disposition` change is needed (this is a self-contained policy-as-data gate,
not a producer-face gate).

## Supersedes / feeds

- Complements (does NOT supersede) the `cloud-ci-total-accounting` `scratch_artifact` denylist and
  ADR-0555's husk-block-on-new.
- Productizes the recurrence-prevention for FRIC-1781132000 (lane-scratch root pollution) at the
  gate layer, per "friction = process failure → productize the gate".

## Addendum 2026-07-02 — GitHub community-health surfaces placement

The GitHub community-health surfaces (code of conduct, contribution guidelines,
issue forms, pull-request template) are tool-mandated GitHub special files (the
instructions-store markdown allow-list class). Under this decision's default-DENY
root allowlist they do NOT get root allowlist rules; they live under the
`.github/` permitted meta home, where GitHub's community profile resolves them
equally. The registered surface is:

- `.github/CODE_OF_CONDUCT.md` — Contributor Covenant 2.1
- `.github/CONTRIBUTING.md` — contributor on-ramp deferring to `docs/AGENTS.md`
- `.github/PULL_REQUEST_TEMPLATE.md` — GitHub prefill mirroring `templates/pull-request-template.md`
- `.github/ISSUE_TEMPLATE/config.yml` — blank issues off; security reports routed to private advisories
- `.github/ISSUE_TEMPLATE/bug-report.yml` — defect form requiring a `MISTAKES-LEDGER` row check
- `.github/ISSUE_TEMPLATE/feature-request.yml` — feature form requiring acceptance criteria + verification path
- `.github/ISSUE_TEMPLATE/blocker-resolution-card.yml` — dispatcher-ready blocker card per the root-hub `blocker_policy`
- `.github/OWNERS` — ADR-0555 ownership seed for the `.github/` community-health tree (`council-architecture`); `.github/workflows/` retains its narrower `cloud-ci-platform` marker
