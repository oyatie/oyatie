---
id: ADR-0572
title: "born-accounting faces merge driver + hermetic git-hooks bootstrap (task #125 v1)"
status: Proposed
planning_impact: true
deciders: founder
date: 2026-06-22
door: two-way
owner: cloud-ci-platform
supersedes: []
superseded_by: []
amends: []
depends_on: [ADR-0515, ADR-0539, ADR-0555]
related: [ADR-0523, ADR-0546, ADR-0548, ADR-0551, ADR-0554, ADR-0562, ADR-0563, ADR-0568]
related_specs:
  - /specs/capability-registry.json
  - /registry/generated-artifact-control-plane.json
milestone: W1
---

# ADR-0572: born-accounting faces merge driver + hermetic git-hooks bootstrap

## Status

**Proposed - 2026-06-22 (authored for founder sign-off; door: two-way — additive local glue,
generates no faces of its own, and is fully removable by deleting the two crates + the
`.gitattributes` glob block + the per-clone git config, without unwinding any SSOT; the producer
remains the sole face generator and the cloud-ci gates remain the sole merge authority).**

## Context

The born-accounting generated faces under
`cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/` (`scm-facts.generated.json`,
`accounting-registry.generated.json`, and the other producer faces) enumerate the whole tracked-path
universe and embed git-history facts: every accounting row carries a `last_touch_commit` SHA plus
TTL/aging derived from commit timestamps emitted by the ADR-0515-D3 `oya-cloud-ci-scm-facts-emitter`
git boundary. Any change to the tracked-path universe rewrites large regions of the multi-MB faces,
so any two open PRs collide on them; each must rebase + re-materialize + re-verify a full CI cycle.
The cost is O(open-PRs) serial cycles — the anti-scalable pattern the founder pipeline bar exists to
kill (task #125; the five serial rebases of the wave-1 train were the trigger).

Because the faces embed git-history facts and the merge commit SHA does not exist mid-merge, a
mid-merge `%O %A %B` driver CANNOT authoritatively regenerate them. The authoritative regeneration
must run AFTER the merge/rebase commit exists.

## Decision

Commission two additive, fail-closed local-glue crates (task #125 v1):

### D1 — `tools/oya-faces-merge-driver-app` (the merge driver + post-merge settle)

A two-stage mechanism:

- **Per-file driver (cosmetic).** `oya-faces-merge-driver driver %O %A %B %P` takes *theirs* (`%B`)
  into `%A` atomically so git records the declared face resolved without conflict markers, then
  exits 0. The value is discarded — never authoritative. It DECLINES (exit 1, `%A` untouched) any
  path that is not a control-plane-declared regeneratable face, so git keeps the conflict.
- **Post-merge settle (authoritative).** `oya-faces-merge-driver settle`, run from the
  `.githooks/{post-merge,post-rewrite}` hooks AFTER the commit exists, regenerates ALL faces from the
  committed merged tree (codemod → scm-facts emitter → producer), runs a byte-rediff + determinism
  self-check, and settles them through the doctrine-blessed
  `oya-cloud-ci-freshness-app::settle_regenerated_faces` engine, so the output satisfies the ADR-0539
  freshness gate (`GENERATED_FACE_PATHS` byte-parity) and registry-drift byte-for-byte.

### D2 — Fail-closed contract (a wrong face is a false-green vector)

On ANY regen failure, non-determinism, drift mismatch, missing producer, dirty non-face tree, or IO
error the driver/settle exits non-zero and leaves the conflict in place. It NEVER writes a guessed or
partial faces file. The determinism self-check runs BEFORE the settle stages anything; the per-file
`%A` write is atomic (temp + rename) so a crash leaves `%A` byte-untouched. This is the ADR-0548-D7
fixer-self-validation discipline applied to the faces.

### D3 — `tools/oya-repo-hooks-bootstrap-app` (hermetic zero-manual-step activation)

`.gitattributes merge=oya-faces` NAMES a driver; it does NOT DEFINE one — activation needs
`merge.oya-faces.driver` in each clone's git config, per-clone local state the repo cannot carry
(the same gap leaves the cargo-lock / friction-ledger drivers un-activated, their READMEs documenting
a manual `git config` line). The founder bar is AUTOMATED, not flag-only. The bootstrap binary
idempotently writes the `merge.oya-faces.{name,driver}` git config, `core.hooksPath .githooks`, the
`.githooks/{post-merge,post-rewrite,post-checkout}` shims, and the `.gitattributes` generated glob
block (derived from the control-plane so it cannot drift from policy).

### D4 — Universality (policy-as-data, closed-schema-respecting)

The regeneratable face set + the buck2 producer/emitter generator targets are read from
`registry/generated-artifact-control-plane.json` (regeneratable = artifacts whose `merge_policy` is
`never-manual-merge-regenerate-from-source-tree` or `controller-owned-main-materialization` — exactly
the `cloud-ci-generated-artifact-control-plane` gate's `diff_policy_allowed_generated_edit_paths`
predicate). The control-plane schema is CLOSED (that gate rejects unknown fields), so these crates add
NO `merge_driver`/`shard_key` policy fields — they read only existing fields. Nothing oyatie-specific
is hardcoded.

### D5 — Irreducible-glue ledger (ADR-0515 D3 / ADR-0523)

The settle subprocesses the BUILT face-generation binaries (codemod → emitter → producer) exactly as
`infra/ci/materialize-cloud-ci-generated-faces.sh`, the freshness gate's `regenerate_faces_with_buck2`,
and register-crate's `Buck2RegenAdapter` do — the scm-facts emitter is the single sanctioned git
boundary. The bootstrap's `git config --local` writes (git is the config store) and the 2-line
`.githooks` shims (git mandates hooks be executables it spawns) are the minimal git-integration glue.
No other shell.

## Scope (v1) and what this does NOT solve

v1 is the LOCAL agent/rebase automation. It explicitly does NOT make GitHub server-side
squash/merge parallel — local merge drivers + local hooks do not run on github.com; merge authority
stays with the cloud-ci gates behind `oya-ci-required` (ADR-0515). Deferred follow-ups:

- **v2 — server-side merge-queue rebase-and-regenerate** in `oya/ci-tide` (the merge-queue projected
  state controller regenerates faces from the rebased candidate before admission). This is where the
  server-path false-green window actually closes; it is an amendment to ADR-0515/the projected-state
  lifecycle layer the control-plane already declares.
- **Registry sharding by capability** (split `accounting-registry.generated.json` into per-capability
  shards) to shrink the conflict class + history churn; this fights the freshness gate's currently
  hardcoded `GENERATED_FACE_PATHS` list and is sequenced as its own slice.
- **Digest-only end-state** (commit a content digest, CI computes the full face ephemerally) to
  eliminate multi-MB git-history churn — a deeper freshness/registry-drift gate-contract change.

## Governed surfaces

The following repo paths are governed by this ADR. The accounting gate validates that each is
justified (this ADR is the justification reference):

```
tools/oya-faces-merge-driver-app/BUCK
tools/oya-faces-merge-driver-app/Cargo.toml
tools/oya-faces-merge-driver-app/OWNERS
tools/oya-faces-merge-driver-app/README.md
tools/oya-faces-merge-driver-app/src/lib.rs
tools/oya-faces-merge-driver-app/src/main.rs
tools/oya-faces-merge-driver-app/tests/cli_fixtures.rs
tools/oya-faces-merge-driver-app/tests/settle.rs
tools/oya-repo-hooks-bootstrap-app/BUCK
tools/oya-repo-hooks-bootstrap-app/Cargo.toml
tools/oya-repo-hooks-bootstrap-app/OWNERS
tools/oya-repo-hooks-bootstrap-app/README.md
tools/oya-repo-hooks-bootstrap-app/src/lib.rs
tools/oya-repo-hooks-bootstrap-app/src/main.rs
tools/oya-repo-hooks-bootstrap-app/tests/idempotent.rs
```

## Consequences

- The local agent rebase-and-regenerate loop becomes one fail-closed command instead of a manual
  `rebase → checkout --theirs faces → materialize → diff-check` dance.
- The merge-driver activation bootstrap problem is solved hermetically with zero manual git config,
  retroactively unblocking the cargo-lock / friction-ledger driver activation pattern too.
- A wrong/partial faces file can never be emitted: every settle path fails closed before staging.
- The server-path scalability win is explicitly deferred to v2 (ci-tide); v1 is the honest bridge.
