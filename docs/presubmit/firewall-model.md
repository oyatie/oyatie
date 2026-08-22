# ci firewall / ratchet mental model

The firewall is the single required status check. It is a generic, policy-free engine: two PURE
DATA-over-DATA predicates over the generated faces, with NO per-gate special cases (the per-code
behaviour differences live entirely in the disposition DATA — the `mode` + `frozen_empty` fields).

## The two predicates

### COMPARE-MODE (does this PR add NEW debt?)

For each `(gate, code)` the firewall computes, against the committed baseline:

- `regressions = current_keys \ baseline_keys` — NEW debt this corpus introduced;
- `tolerated = current_keys ∩ baseline_keys` — accepted pre-existing debt (never fails);
- `fixed = baseline_keys \ current_keys` — repaired keys (informational; drives auto-shrink).

A code FAILS iff its `mode` is **`baseline-block-on-new`** AND `regressions` is non-empty. A
**`advisory-until-infra`** code reports its counts (the burn-down dashboard) but NEVER flips the
verdict — until its `infra_prereq` lands and the disposition is flipped to `baseline-block-on-new`
(a DATA edit, not a code change).

### RATCHET-INVARIANT (can debt be laundered into the baseline?)

The baseline may only ever SHRINK on regen. For each `(gate, code)`,
`growth = proposed_keys \ committed_keys` (keys a regen would ADD). Empty growth is an allowed
regen (it auto-shrinks to `committed ∩ proposed`). Non-empty growth is a `ratchet_regression`
FAILURE — UNLESS every grown key is in the founder-signed allowlist. `frozen_empty` codes have a
permanently-empty committed baseline, so ANY proposed key is growth — they can never accumulate a
baseline. Same predicate, no special case.

## The one-way sign-off door

`gate-baseline.signoff.json` is the ONLY human-edited, founder-signed file in the engine. It is NOT
producer-generated and NOT byte-diffed by registry-drift. A key listed under
`_sign_off_additions[gate][code]` is exempted, for ONE regen, from the GROWTH check. Growing
tolerated debt therefore requires an explicit signed decision — never a silent producer re-run.
Keep it tiny and audited; EMPTY = the ratchet is fully closed (baseline can only shrink).

## registry-drift (committed == regenerated)

A separate gate re-runs the producer in a sandbox and byte-diffs every generated face against the
committed copy. A hand-edit to any face — including laundering debt into the baseline — fails this
gate. The faces carry a content digest (FNV-1a) and NO wall-clock, so the diff is deterministic.
The gate-baseline `_provenance` also carries a `config_digest`, so a change to `ci.toml` is
visible in the diff.

## Why a fan-in job, not a reusable workflow

The required-context name is `presubmit` — a single fan-in job that `needs:` every gate lane.
A `gate-registration` meta-test asserts every in-tree gate crate is registered as a lane and wired
into the fan-in's `needs:`, so an in-tree-but-unregistered gate (a silent false-green one level
below the fan-in) fails CI. A reusable `workflow_call` would rename published check-runs to
`<caller> / <job>` and break the required-context name — so adopters use a **composite action**
(does not rename check-runs) + a copy-in matrix template, keeping the fan-in in their own workflow.

## The go-live posture

With the baseline frozen at today, the firewall is GREEN on the current corpus (every current key
is tolerated, zero growth) — yet it still goes RED on any NEW finite violation (proven by the
firewall crate's RED-on-new + ratchet fixtures). That is the whole point: a firewall that blocks
new debt while letting the frozen historical debt age out without churning history.
