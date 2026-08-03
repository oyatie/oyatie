# Delete `cloud/cloud-kernel`

## Context

Keep the adopted Asterinas substrate in `kernel/`; remove the bespoke 20-crate framekernel in
`cloud/cloud-kernel`.

Not for lack of quality — it is 17,000+ lines with zero `todo!()`, two symmetric ISA backends, and a
loom-model-checked `ksync`. It goes because it has no owner and **cannot be worked on**:

- **It cannot be edited.** Touching any of its 50 graph-invisible `.rs` files trips `RefuseUnowned`,
  which returns before anything else is evaluated → required context red. A *deletion* routes to FULL
  and merges. The gate already forbids developing it and permits removing it.
- **It cannot commit its own evidence.** `out/conformance/` is `unit_class: scratch`, budget 0,
  guarded by `scratch_artifact_cannot_be_laundered_by_registration`.
- **One commit has ever touched its source** — the import `072a66f37`, 2026-06-10.

Carrying it costs 12 stale `nightly-2026-02-28` pins exempted from drift checking, 21 hermeticity
anchors, 20 catalog rows, 20 membership rows, ~831 KiB of committed ELFs, and 50 files that fail the
"everything in the build graph" goal.

**Reversibility already secured.** Tag `kernel-snapshot-2026-06-08` → `26173992778a`, pushed. That
commit was *not* an ancestor of `dev`, was untagged, and hung off one branch ref; it holds the
bring-up harness (QEMU runners both ISAs, `assert-talos-boot`, `diff-oracle`, `check-tcb`) that `dev`
never had. `072a66f37` remains an ancestor, so `git checkout 072a66f37 -- cloud/cloud-kernel`
restores the dev-shaped copy.

## Three corrections that shape this plan

1. **Rung 0 must NOT change — changing it is the RED.** `REQUIRED_OWNED_STACK_LAYERS` /
   `REQUIRED_OWNED_STACK_LADDER_RUNGS` in `ci/facade/cross-artifact-agreement/src/lib.rs` hold
   masterplan **layer names**, not paths (two of six — `durability-plane`,
   `governance-iam-console` — aren't directories). They validate `specs/masterplan.json`, never the
   filesystem. Deleting the `cloud-kernel` rung shifts every later index → cascading
   `@ladder-order` + `@rung-index`. **Leave both arrays and the masterplan rung; edit only anchor
   strings** (`source_anchors` is validated as non-empty, with no path-existence check).

2. **`crate-catalog-coverage` is 20 rows, not 7 — VERIFIED 20 of 20.** The gate is *name*-keyed and
   13 crates have bare names (`arch-aarch64-layout-tests`, `fsbase-worker-x86_64`,
   `user-{clock,exec,fsbase,hello,init,procinfo,signal,smpdemo,spawn}-x86_64`, `user-procinfo`,
   `user-smpdemo`). No path or `oya-` grep will ever surface them. This is the single most likely
   cause of a wasted CI round.

3. **`git rm -r` leaves the directory behind.** Local `.gitignore`s cover `out/conformance/`,
   `out/*.{log,bin,normalized}`, `target/`, `.stage-b-target/`. On any machine that has run the
   conformance or Stage-B build, the tree survives as untracked files and re-trips path-based gates.
   An explicit `rm -rf cloud/cloud-kernel` must follow the `git rm`.

## Must land in the same commit — a gate REDs otherwise

| File | Action | Count |
|---|---|---|
| `ci/facade/module-membership/capability-membership-policy.json` | **Regenerate, never hand-edit** | `legacy_root_freeze.crates` **350 → 330** |
| `ci/facade/crate-catalog-coverage/crate-catalog-coverage-policy.json` | Delete **20** rows (see correction 2) | `uncatalogued` **197 → 177** |
| `ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-baseline.json` | Delete all **21** rows | ceiling **21 → 0**; leave `sites_floor: 72` (124 survive) |
| `ci/facade/affected-target-set/affected-set-policy.json` | Delete 2 exact `linker.ld` paths from `require_owner_patterns` | — |
| `ci/facade/affected-target-set/tests/affected_set.rs` | **Atomic with the above** — `shipped_pack_parses_and_matches_the_engine` (~L709) reads the live policy off disk | — |
| `ci/facade/artifact-inventory-registry/src/unit-class-policy.json` **+** `libs/oya-ci-config/src/bundled/unit-class-policy.json` | Delete **2 rules each** (`scratch-kernel-conformance`, `scratch-kernel-stress`). Byte-identical mirrors, **no sync test** | — |
| `specs/microservice-tier-classification.json` | Delete the `cloud/cloud-kernel/manifest.json` row (born-blocking tier coverage) | — |

Regeneration (prints to stdout, never writes; paste the block back). No `--allow-new` — the refusal
fires on growth, and 350 → 330 is a shrink:

```
buck2 run //ci/facade/module-membership:oya-cloud-ci-capability-membership-app-bin -- \
    --repo-root . --emit-legacy-freeze
```

## Mechanical, same PR

`Cargo.toml` exclude entry (verified **inert** — no member glob matches, and the tree declares its
own `[workspace]`; fix the "Only THREE exclusions remain" comment) · `specs/capability-registry.json`
drop `cloud/cloud-kernel` from `meta_directory_absorbs[0].current_dirs`, keep `kernel/` ·
`oya-deps.toml:25` → `exclusions = []` (**key must stay**) · `.gitignore` 8 lines (keep `:198`
`kernel/harness/*/receipts/`) · `specs/reorg/kernel-move-plan.BLOCKED.json` delete file ·
`registry/fixuptasks.jsonl:420` narrow the row (it has a merge driver — don't hand-delete a line
blind) · `cloud/cloud-os/manifest.json:225` `does_not_own` → `kernel/` (**leave `:318`**, it quotes a
real GitHub issue title) · `.omc/ultragoal/friction-ledger.jsonl:121` close the row ·
**`ci/facade/artifact-inventory-registry/src/lib.rs:2426`** — a Rust literal
`["OWNERS", "os/OWNERS", "cloud/cloud-kernel/OWNERS"]`; verify whether a missing OWNERS there is
fail-closed.

ADR text edits: **0537** (ladder rung 0 anchor) · **0538** (quotes the root exclude array verbatim —
must match) · **0547** (kernel-crate census 4 → 0) · **0554** (delete 2 disposition table rows).

## Deliberately NOT touched

- `cross-artifact-agreement` arrays and the masterplan rung — correction 1.
- **A4 coupled pair**: `rust_toolchain_drift.rs` `EXCLUDED_PREFIXES` (index 9 of 12) + its paired
  `gate-self-conformance-policy.json` exemption. An *unused* exception fires nothing, so leaving both
  is green; removing the exemption alone is RED. Shorter diff, zero risk.
- **ADR-0562** (Accepted) — editing an Accepted ADR implies an Amended flip + propagation. Let the
  new ADR supersede Fork 2 instead.
- **`evidence/*` (2 files)** — hash-chained audit records (`current_row_git_sha`). Do not edit or
  delete. Discharge by **adding** a new multispectrum record for this change, the repo's own pattern.
- `core-dependency-isolation/src/lib.rs:2048` (`kuberos_is_not_denied_by_kube_prefix`) — live
  regression proof for the `kube`/`kuberos` prefix trap.
- Dated audit snapshots under `docs/audit/initial-sweep-2026-06-06/` (append-only by charter) and
  ~9 in-memory `json!`/tempdir fixtures.

## Coverage that must survive

`affected_set.rs` has 4 kernel-named tests. Three are redundant or re-pointable, but
**`bare_metal_kernel_linker_script_without_buck2_platform_refuses_until_wired` is the only proof that
an exact-literal (non-glob) `require_owner_patterns` entry yields `RefuseUnowned`.** Removing the two
`linker.ld` rows leaves `["**/*.rs"]`, so that class loses its input. **Keep the test, substitute a
synthetic exact path** (e.g. `fixtures/exact-owner-required.ld`) in the inline `policy()` fixture.
Preserve the `!bytes.contains("out_of_graph_roots")` assertion — path-independent anti-regression.

## Two things needing a decision, surfaced not assumed

- **ADR-0524 is Accepted and one-way**, and its entire subject is this tree — but it names it by the
  *pre-move* path `stack/kernel`, so no `cloud/cloud-kernel` grep finds it. Its decision was "bring
  the kernel port into the one buck2 graph" **and** "retire the git-tracked `out/*.elf` carrier
  blobs". Deleting satisfies the second and abandons the first. **This is a supersession event and
  argues the PR should carry its own short ADR** rather than a bare `git rm`.
- **The 620 KiB `user-musl.elf` is unlicensed third-party** (static musl, no build script, absent
  from `deny.toml`, `oss-stewardship-registry.json`, `dependency-rationales.json`). `git rm` removes
  it from the tree, **not from history**. The sanctioned purge path —
  `registry/history-only-retirement/control-plane.json` — is `HOLD(Planning)`,
  `dispatch_authorized: false`. If history purge is wanted, that control plane must be unblocked
  first; it is not available today.

## Execution order

1. Re-confirm `data/core/cloud-kernel` and `marketplace/core/cloud-kernel` have no `out/conformance/`
   (checked clean: 0 tracked files match the substring) — the `contains` rule matches them too.
2. `git rm -r cloud/cloud-kernel` **then** `rm -rf cloud/cloud-kernel` (correction 3).
3. Apply the same-commit table; regenerate the membership freeze.
4. Sweep for stragglers by **both** path *and* the 13 bare crate names.
5. Commit body records the tag, `072a66f37`, and the reason: **no owner, not no quality.** Carry the
   one portable idea forward as prose: *syscall status is a trichotomy, not a boolean — a syscall in
   dispatch returning a fixed value is not implemented, and an ENOSYS return is the demand signal
   that prioritises the next one.*

## Verification

Establish a pristine `origin/dev` control first: a bare `buck2 test //ci/...` fails ~18 targets that
pass in CI, because the required workflow materializes generated faces in an out-of-graph pre-step a
local run skips.

```
buck2 test //ci/facade/module-membership/... //ci/facade/crate-catalog-coverage/... \
           //ci/facade/embedded-asset-hermeticity/... //ci/facade/affected-target-set/... \
           //ci/facade/artifact-inventory-registry/... //ci/facade/cross-artifact-agreement/... \
           //ci/facade/service-tier-metadata/... //libs/oya-ci-config/...
buck2 test //ci/...     # diff the failure set against the control
```

Quote `Cache hits: %` on every run — buck2 test can serve a stale green.

Acceptance numbers:

| metric | before | after |
|---|---:|---:|
| tracked package manifests | 891 | 871 |
| `include_*!` sites | 145 | 124 |
| `skip_build_output_path` ceiling | 21 | 0 |
| `uncatalogued` | 197 | 177 |
| `legacy_root_freeze.crates` | 350 | 330 |

Measured `oya-ci-required` durations are **17–73 min** (failures cluster 53–64), so budget
**~60–75 min per FULL-tier round** plus queue — effective runner capacity is 2 of 3 and trunk runs
now compete for it. Realistic: **2–3 rounds**, an afternoon. The week-long estimate assumed serial
discovery through CI, which this enumeration exists to avoid.

## Follow-ups, not this PR

- Keep `kernel_side_adapters: []` **empty** — the zero-maintenance property lives there, not in
  Asterinas. The first adapter starts an MPL-2.0 upgrade treadmill.
- **Nothing on `dev` proves Asterinas boots**: 28 files name it, **0 in `.github/`**; the QEMU targets
  are `rust_binary` so `buck2 test` never runs them; receipts are gitignored. One lane should boot it
  on x86_64 and commit the serial log. Until then the kept substrate has weaker in-repo evidence than
  the deleted one.
- The pin declares `covered_file_source_pointer_required_before_distribution: true`; nothing enforces
  it. One file.
