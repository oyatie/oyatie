# oya-faces-merge-driver

Fail-closed local Git merge driver + post-merge settle for the born-accounting generated faces
(task #125 v1, ADR-0572).

## What it is

The accounting faces under `cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/`
(`scm-facts.generated.json`, `accounting-registry.generated.json` and the other producer faces) are
controller outputs, never a contributor merge surface. Any two PRs that touch the tracked-path
universe collide on the multi-MB face files, forcing a serial rebase-and-regenerate per PR.

This driver automates that local rebase/merge flow, fail-closed.

## Why a per-file driver alone cannot regenerate (the spine of the design)

The faces embed git-HISTORY facts, not just working-tree content: every accounting row carries a
`last_touch_commit` SHA plus TTL/aging from commit timestamps (emitted by the ADR-0515-D3
`oya-cloud-ci-scm-facts-emitter-app` git boundary). During an in-progress local merge/rebase the
merge commit SHA does not yet exist, so a mid-merge `%O %A %B` driver cannot authoritatively
regenerate. Therefore the mechanism is two-stage:

1. **Per-file driver (cosmetic).** `oya-faces-merge-driver driver %O %A %B %P` takes *theirs* (`%B`)
   into `%A` atomically so git records the face resolved without conflict markers, then exits 0. The
   value is discarded — it is never authoritative. It declines (exit 1, `%A` untouched) any path that
   is not a control-plane-declared regeneratable face.
2. **Post-merge settle (authoritative).** `oya-faces-merge-driver settle` runs from the
   `.githooks/post-merge` + `.githooks/post-rewrite` hooks AFTER the merge/rebase commit exists. It
   regenerates ALL faces from the committed merged tree (codemod → scm-facts emitter → producer),
   runs a byte-rediff + determinism self-check, and settles them via the
   `oya-cloud-ci-freshness-app::settle_regenerated_faces` engine, so the output satisfies the
   freshness gate (`GENERATED_FACE_PATHS` byte-parity) and registry-drift byte-for-byte.

## Fail-closed contract

On ANY regen failure, non-determinism, drift mismatch, missing producer, dirty non-face tree, or IO
error the driver/settle exits non-zero and leaves the conflict in place. It NEVER writes a guessed or
partial faces file. A half-written face that satisfies nothing would be a false-green vector; the
fail-closed default is "let the human / merge queue see the conflict."

Exit codes: `0` success; `1` the driver declines this merge (git keeps the conflict); `2`
control-plane / regen / drift / determinism / settle / IO / usage failure.

## Universality (policy-as-data)

The regeneratable face set + the buck2 producer/emitter generator targets are read from
`registry/generated-artifact-control-plane.json` (the declared public product contract) — the same
data the `cloud-ci-generated-artifact-control-plane` gate validates. Nothing oyatie-specific is
hardcoded. The control-plane schema is CLOSED (that gate rejects unknown fields), so this crate reads
only existing manifest fields (regeneratable = artifacts whose `merge_policy` is
`never-manual-merge-regenerate-from-source-tree` or `controller-owned-main-materialization`).

## Activation — zero manual git config

Unlike the cargo-lock / friction-ledger merge driver READMEs, activation is NOT a manual
`git config merge.* …` line. The companion `oya-repo-hooks-bootstrap` binary installs the
`merge.oya-faces.{name,driver}` git config, `core.hooksPath .githooks`, and the post-merge/
post-rewrite/post-checkout hook shims idempotently per clone and per CI runner. See
`tools/oya-repo-hooks-bootstrap-app/README.md`.

The `.gitattributes` entries are generated from the control-plane manifest and look like:

```gitattributes
cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/*.generated.json merge=oya-faces
```

## Enforcement layering (honest scope — what v1 does NOT solve)

This driver is the LOCAL automation layer: it only helps actors whose clone the bootstrap configured.
Merge authority stays with the cloud-ci gate apps behind the single required context `oya-ci-required`
(ADR-0515). v1 explicitly does NOT make GitHub server-side squash/merge parallel — local merge
drivers + local hooks do not run on github.com. Two PRs touching the same face still show a conflict
banner there until the v2 server-side merge-queue rebase-and-regenerate lands in `oya/ci-tide`
(deferred follow-up). v1 also does not eliminate the multi-MB git-history churn — registry sharding
by capability (deferred) and the digest-only end-state address that.

Talos-era successor: under the agentic delivery fabric this becomes server-side merge intelligence in
the ADR-0515 cloud-ci/oya-ci Tide admission path. This crate follows the
`tools/oya-cargo-lock-merge-driver-app` / `tools/oya-friction-ledger-merge-driver-app` precedents (no
new doctrine).
