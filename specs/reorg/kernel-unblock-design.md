# Kernel unblock design — `kernel-move-plan.BLOCKED.json` resolution card

**Class:** mixed (refactor → move) · **Capability span:** single (`kernel` meta dir, kuberos framekernel)
**Authority re-queried:** ADR-0701 (live apex), ADR-0562 §8 Fork 2, ADR-0512 kernel carve-out, ADR-0611 (Proposed; kernel/ asterinas workspace landed on dev), `specs/integ-branch-envelopes.json#W-cloud-leaf-cloud-kernel` judgment
**Status:** planning-only / blocked — **NON-DISPATCHING**, explicitly subordinate to `/specs/masterplan.json#masterplan_v2`. The live `planning_entry_contract` is `state: open` with `binding_plan_approval_allowed: false` and `dispatch_allowed: false`, so no execution-wave dispatch may originate from this card until the §Pre-dispatch gates clear. NOT dispatcher-ready.
**Suggested owner profile:** reorg worker with codemod (`tools/oya-reorg-codemod-app`) + nested-workspace expertise; platform-governance reviewer.

## Source context

`specs/reorg/kernel-move-plan.BLOCKED.json` encodes the settled destination shape for the rung-0
kuberos framekernel (de-brand MOVE-11): `hal-kernel` → `kernel/ports/hal`, arch adapters →
`kernel/adapters/{arch-aarch64,arch-x86-64}`, `frame-kernel`/`ksync-kernel`/`user-layout-kernel`/
`app` → `kernel/core/*`, plus 13 nested test/user-program crates riding with their parents.
It is fail-closed-blocked on mechanically proven defects (B1–B3). `cloud/cloud-kernel` holds
**20** crate dirs (7 top-level + 13 riding) of the 24-crate `cloud/` frozen-baseline inventory
(`ci/facade/module-membership/capability-membership-policy.json#legacy_root_freeze.crates` lists
exactly 20 `cloud/cloud-kernel/*` rows + 4 `cloud/cloud-os/*` rows). The apparent twenty-first
`Cargo.toml` is the workspace root, which has no freeze row. `cloud/` is therefore **not** zeroed
by this card: the four `cloud/cloud-os/*` crates remain under `cloud/` until their own `os/` lane.

## Blocker B1 — destination workspace shape conflict

**Class:** design/decision (not a code edit).

**Fresh dry-run evidence (2026-08-14, dev tip):** `oya-reorg-codemod dry-run --plan
specs/reorg/kernel-move-plan.BLOCKED.json` now fails with `move target already exists:
"kernel/Cargo.toml"` — the plan's artifact move of `cloud/cloud-kernel/Cargo.toml` collides
with the ADR-0611 workspace root that landed at `kernel/` (edition 2024, rust-version 1.97.1).
B2's workspace-dependency refusal no longer fires. This confirms the destination-occupation
blocker and motivates the re-anchor below.

`kernel/` is already a nested Cargo workspace from ADR-0611 (asterinas boundary + real-boot):
edition 2024, rust-version 1.97.1, stable, std host crates. The kuberos framekernel needs edition
2021, rust-version 1.85, `nightly-2026-02-28` + `build-std` for `aarch64-unknown-none-softfloat`.
One tree cannot carry two values of `rust-toolchain.toml`, `.cargo/config.toml`, or
`[workspace.package]` edition.

**Design decision (proposed):** give each rung its own nested workspace under the `kernel/` meta
dir — the ADR-0512 carve-out already sanctions nested workspaces for the kernel rung:

```text
kernel/
  kuberos/            # rung 0: the no_std framekernel workspace (Cargo.toml + lock + toolchain + .cargo/config)
    ports/hal/
    adapters/arch-aarch64/  adapters/arch-x86-64/
    core/{frame,ksync,user-layout,app}/
  asterinas/           # ADR-0611 workspace (renamed from the current flat kernel/core + kernel/harness)
```

- `kernel/kuberos/` becomes the move-plan's new destination prefix (all 7 moves + 13 riding crates
  re-anchor from `kernel/` → `kernel/kuberos/`).
- `kernel/` itself stays the meta-dir root (OWNERS + the two nested workspaces), matching the
  registry charter (`kernel/` owns_crates=true, rung 0).
- The ADR-0611 asterinas crates move `kernel/{core,harness}` → `kernel/asterinas/{core,harness}`
  in the SAME PR (a small intra-kernel move) so both workspaces have clean roots.

**Authority-change prerequisite (envelope conflict):** the accepted
`specs/integ-branch-envelopes.json#W-cloud-leaf-cloud-kernel` judgment (`judgment_status: done`,
`land_status: ready_for_integ_kernel`) requires one rung-0 META workspace with the framekernel
absorbed into `kernel/{core,harness}` and names that as the forever shape, including the
`finops_unit_cost` challenge "one nested kernel workspace forever avoids duplicate CI graphs".
This card's two-workspace topology (kuberos + asterinas) therefore **amends that judgment**; the
toolchain/edition conflict above is the mechanical evidence for the amendment. The B1 split PR
must land that envelope authority change (amendment/OVERRULE of `W-cloud-leaf-cloud-kernel` to the
two-nested-workspace shape, with the five-field doctrine record) **before** this card can be
dispatched — see §Pre-dispatch gates. This card neither silently overrules the envelope nor
dispatches against it.

**B1a — cross-workspace transfer mechanism (whole-workspace transfer).** Every re-anchored crate
move goes from the `cloud/cloud-kernel` workspace to the distinct `kernel/kuberos` workspace.
`validate_workspace_ownership` (`tools/oya-reorg-codemod-app/src/plan.rs:544-560`) rejects exactly
this case with `WorkspaceSpan` before any writes. Resolution: the codemod gains a tested
**whole-workspace transfer** mode — when the plan co-moves the old workspace-root artifacts
(`Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/config.toml`) alongside the crates,
`validate_workspace_ownership` admits the move because the destination workspace IS the relocated
source workspace (the workspace root is a co-moving artifact in the same plan). All other
cross-workspace moves stay fail-closed (`WorkspaceSpan` unchanged). Unit + integration tests prove
both the admit and the unchanged refusal.

**B1b — no pre-created move-plan artifact targets.** The B1 split PR creates the kuberos workspace
root by **transferring** `cloud/cloud-kernel`'s root files (`Cargo.toml`, `Cargo.lock`,
`rust-toolchain.toml`, `.cargo/config.toml`) as the scaffold, so the re-anchored move-plan's
artifact list is revised to **drop those four already-transferred entries** and keep only the files
whose destinations are free (`OWNERS`, `BUCK`, `manifest.json`, `.gitignore`, `specs/`, `tests/`,
`out/`). `apply_plan`'s `TargetExists` preflight (`src/plan.rs:103-119`) therefore never fires for
the workspace root, and the split PR's `cargo metadata` gate-green is real (the transferred root
resolves). Do NOT pre-create any artifact that a later move-plan will transfer.

**B1c — rewrite runtime paths during the asterinas re-anchor.** The real-boot binaries retain
default receipt paths (`kernel/harness/asterinas-real-boot/receipts/...` in
`kernel/harness/asterinas-real-boot/src/main.rs:23-24` and `src/boot.rs:48-50`, plus the
`kernel/target/artifacts/*` ISO/log destinations). The reorg codemod rewrites crate identifiers and
BUCK labels but NOT arbitrary Rust string paths, so the re-anchor acceptance must include rewriting
these runtime paths to `kernel/asterinas/harness/asterinas-real-boot/...` and adding tests that
the binaries write into the re-anchored tree — `cargo metadata` alone cannot catch this.

**B1d — rewrite linker-script paths before accepting the moved workspace (P1).** The
content-preserved `cloud/cloud-kernel/.cargo/config.toml` passes
`-Tcrates/oya-cloud-kernel-arch-{aarch64,x86-64}-adapter/linker.ld`, but after the move those
files live at `kernel/kuberos/adapters/arch-{aarch64,x86-64}/linker.ld`. `cargo metadata` does not
invoke the linker, so the acceptance criterion must require (1) the `rustflags` paths in the
transferred `.cargo/config.toml` rewritten to the new adapter face layout, and (2) a **real build**
of both configured targets (`aarch64-unknown-none-softfloat` + `x86_64-unknown-none`) — not just
`cargo metadata` — as move-PR verification.

**B1e — move and retarget the kernel workspace gitignore.** `cloud/cloud-kernel/.gitignore` is the
only tracked top-level workspace file absent from the current artifact list; it would remain behind
and keep `cloud/cloud-kernel` alive. The transfer must include `.gitignore` and rewrite its
workspace-relative patterns (`/target`, `/crates/.../tests-host/target`, `/out/*.elf`,
`/out/*.bin`, `/out/*.log`, `/out/.stage-b-target/`) for the new `kernel/kuberos/` layout
(`/adapters/...`, `/out/*.elf`, ...) so generated ELF/bin/log outputs stay ignored.

**B1f — reconcile the lockfile corpus in the workspace-split PR.** The supply-chain audit gate
(`ci/facade/supply-chain-audit/supply-chain-audit-policy.json#lockfile_corpus`) authoritatively
pins the current corpus (`Cargo.toml`, `cloud/cloud-kernel/Cargo.toml`,
`cloud/cloud-kernel/crates/.../tests-host/Cargo.toml`, `kernel/Cargo.toml`), and its live test
(`ci/facade/supply-chain-audit/tests/supply_chain_audit.rs:163-186`) asserts that exact corpus.
The split PR temporarily adds `kernel/kuberos` while re-anchoring the asterinas pair to
`kernel/asterinas`, so the SCM projection and the configured corpus diverge. The B1 split PR must
therefore update `lockfile_corpus` (+ `min_lockfiles`) AND the committed test expectation in the
same PR; the move PR later re-points the `cloud/cloud-kernel` pair and the `tests-host` pair to
their `kernel/kuberos` destinations. These policy/test transitions are explicit, gate-scoped
changes of their owning PRs — never silent.

**Acceptance criteria:** one nested workspace per rung; `cargo metadata` resolves in both; no
cross-workspace path-deps between kuberos and asterinas (they are disjoint ladders); the B1a
whole-workspace transfer is test-proven; the B1b artifact list contains no pre-created target; the
B1c runtime paths are rewritten and tested; the B1d linker paths are rewritten and both targets
build for real; the B1e `.gitignore` transfers and retargets; the B1f lockfile corpus and test
transition in their owning PRs; the envelope authority change (§B1 prerequisite) lands.

## Blocker B2 — codemod does not rewrite `[workspace.dependencies]`

**Class:** RESOLVED-ON-DEV (2026-08-14 re-verify).

`rewrite_dep_tables_recursive` now recurses into `[workspace.dependencies]` and its
dev/build siblings (the code comment cites the ADR-0512 carve-out's 5-of-7 edges explicitly).
Landed via PR #1523 (`fa1292c89 fix(reorg-codemod): make the oracle see the workspace it is
migrating`). Re-verified at `origin/dev@4a4f71a14` by reading the source and dry-running the
blocked plan: the workspace-dependency refusal no longer fires.

**Remaining B2 action:** none (the codemod change is on trunk). B2 is NOT a blocker and no
sequencing step dispatches a duplicate enhancement for it.

## Blocker B3 — escaping include!/include_bytes!/include_str! literals

**Class:** SPLIT into B3a + B3b (2026-08-14 re-verify). The measured escaping set is **27**
literal sites: **21** workspace-root ELF embeds (`include_bytes!` in the two adapter
`src/user.rs` files — 9 aarch64 + 12 x86-64, matching the 21 frozen
`skip_build_output_path` keys in
`ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-baseline.json`) plus **6**
`tests-host` `include!` literals naming old sibling crate dirs.

**B3a — the 21 workspace-root ELF embeds: sanctioned codemod behavior DESIGNED; implementation
in flight (PR #1965 open, not merged).** The design specifies two move-invariant classes the
refusal gate must accept: depth-preserving moves with untouched targets, and targets that co-move
with the workspace (the `../../../out/*.elf` shape — the kuberos workspace relocates as a unit and
`out/` rides the move as artifact co-moves). Fixture + unit tests prove both; fail-closed
otherwise. The enhancement itself is PR #1965 (`feat(reorg-codemod): accept move-invariant
escaping literals (kernel unblock B3a)`) — **OPEN at 2026-08-14, not on dev** — so B3a clears
only when #1965 lands on dev AND the re-anchored plan dry-runs with zero
`UnrewritablePathLiteral` refusals. Until then the refusal gate on trunk still fires; the card
must not be treated as
dispatcher-ready on B3a alone. **Move-PR companion (baseline relabel):** the move changes all 21
path-qualified keys in `embedded-asset-hermeticity-baseline.json` (`skip_build_output_path`). The
live gate asserts exact set equality (`tests/embedded_asset_hermeticity.rs:110-119`) and the
codemod does not rewrite arbitrary JSON, so the move PR must include a **review-visible path
relabel** of that baseline to the new `kernel/kuberos/` adapter paths WITHOUT raising its ceilings
(`skip_build_output_path` ceiling stays 21).

**B3b — the 6 tests-host `include!` literals naming old sibling crate dirs: REMAINS, kernel-lane
scoped.** They name the OLD directory outright, so no move preserves them; the pre-move fix must
re-home the shared pure layout/signal/timekeep modules so the host harness (its own
`[workspace]` root escaping the kernel's build-std config) can consume them WITHOUT forking the
single source of truth — the host harness must depend on / remain colocated with the production
`user-layout-kernel` module sources, never copy them into in-crate fixtures (a fixture copy would
let production layout/signal/VFS/timekeep changes break the kernel while these host tests stay
green). This is a kernel-toolchain-harness refactor that requires the nightly `build-std`
toolchain to verify (host harness + no_std crates must both still compile). Do NOT attempt blind.
Sequencing: this slice runs in the kernel lane with toolchain verification, and the kuberos move
waits on it. Not on the baseline-burn critical path (20 rows vs libs/oya).

## Sequencing

1. **B1 workspace split + authority change (own PR, gate-green).** Asterinas re-anchor
   (`kernel/{core,harness}` → `kernel/asterinas/{core,harness}`) + kuberos workspace-root transfer
   (B1a codemod whole-workspace-transfer mode, B1b artifact-list revision, B1c runtime-path
   rewrite + tests, B1d linker config rewrite + real build of both targets, B1e `.gitignore`
   transfer/retarget, B1f supply-chain corpus + test transition), PLUS the
   `integ-branch-envelopes.json#W-cloud-leaf-cloud-kernel` amendment to the two-nested-workspace
   shape. (The former B2 codemod step is already on trunk via #1523.)
2. **B3 pre-move include refactor (own PR, behavior-preserving, gate-green).** B3b re-home so the
   host harness consumes the production module sources (no fixture copies); verify on the nightly
   `build-std` toolchain.
3. **Re-anchor + execute the move-plan (own PR).** Plan file re-committed under `specs/reorg/`
   with the `kernel/kuberos/` destinations; artifact list drops the already-transferred workspace
   root (B1b); singleton applies. Includes the B3a baseline path relabel
   (`embedded-asset-hermeticity-baseline.json`, ceilings unchanged) and the manifest/projection
   stage (below).
4. **Manifest + projection retarget in the same move PR.** `manifest.json` and `specs/` are
   non-crate artifacts that `apply_plan` moves content-preserving without rewriting JSON: the
   manifest keeps `metadata_file`/`destination_path`/ownership references under the deleted
   `cloud/cloud-kernel` tree, and `specs/microservice-tier-classification.json` keys the manifest
   by its old path (`cloud/cloud-kernel/manifest.json`). The
   `tier_classification_projection_is_the_governed_manifest_corpus` gate asserts exact equality
   with the live manifest census and will reject the move. The move PR therefore includes an
   explicit artifact rewrite/reprojection stage (manifest internals + tier-classification key) and
   a **stale-source-path check** (no governed artifact may still reference
   `cloud/cloud-kernel/`).
5. **Burn the 20 `cloud/cloud-kernel/*` frozen-baseline rows in the same PR** (regenerate the
   freeze with its declared producer
   `//ci/facade/module-membership:oya-cloud-ci-capability-membership-app-bin -- --emit-legacy-freeze`,
   never hand-edited); `cloud/cloud-kernel` then holds zero crate dirs. The four
   `cloud/cloud-os/*` crates remain under `cloud/` and are NOT part of this burn — they belong to
   the separate `os/` lane (thread-verified: `cloud/` is not zeroed by this card).

## Pre-dispatch gates

This card is planning-only/blocked and NON-DISPATCHING until ALL of:

- `masterplan_v2.planning_entry_contract` transitions from `state: open` to closed
  (`binding_plan_approval_allowed` and `dispatch_allowed` become true);
- the kernel-unblock work item + its dependency edges are registered in
  `/specs/masterplan.json#masterplan_v2.work_items` (+ `dependency_edges`) via the
  coordinator-owned single-writer mutation path — no competing dispatch surface;
- the `integ-branch-envelopes.json#W-cloud-leaf-cloud-kernel` authority change (two-nested-
  workspace shape) is merged;
- B1 (incl. B1a–B1f) and B3b clear; B3a's PR #1965 lands on dev and the re-anchored dry-run is
  zero-refusal;
- the move-plan singleton (`specs/reorg/*-move-plan.json`) is free.

## Non-goals

- No touching `os/` (separate lane, already dest-verified); the four `cloud/cloud-os/*` crates
  stay under `cloud/` until that lane.
- No behavior change to the framekernel; the move is byte-preserving modulo paths (except the
  explicitly rewritten config/linker/gitignore/runtime-path artifacts above).
- No merge of the two nested workspaces into one (rejected: unresolvable toolchain/edition values).
