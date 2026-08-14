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
It is fail-closed-blocked on three mechanically proven defects (B1–B3). `cloud/cloud-kernel` holds
21 crate dirs of the 24-crate `cloud/` frozen-baseline inventory.

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
- The ADR-0611 asterinas crates move `kernel/{core,harness}` → `kernel/asterinas/{core,harness}` in
  the SAME PR (a small intra-kernel move) so both workspaces have clean roots.

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

**Acceptance criteria:** one nested workspace per rung; `cargo metadata` resolves in both; no
cross-workspace path-deps between kuberos and asterinas (they are disjoint ladders).

## Blocker B2 — codemod does not rewrite `[workspace.dependencies]`

**Class:** RESOLVED-ON-DEV (2026-08-14 re-verify).

`rewrite_dep_tables_recursive` now recurses into `[workspace.dependencies]` and its
dev/build siblings (the code comment cites the ADR-0512 carve-out's 5-of-7 edges explicitly).
Landed via PR #1523 (`fa1292c89 fix(reorg-codemod): make the oracle see the workspace it is
migrating`). Re-verified at `origin/dev@4a4f71a14` by reading the source and dry-running the
blocked plan: the workspace-dependency refusal no longer fires.

**Remaining B2 action:** none (the codemod change is on trunk). B2 is NOT a blocker and no
sequencing step dispatches a duplicate enhancement for it.

## Blocker B3 — 33 escaping include!/include_bytes!/include_str! literals

**Class:** SPLIT and partially RESOLVED (2026-08-14 re-verify).

**B3a — the 18 workspace-root ELF embeds: RESOLVED by codemod enhancement (PR #1965).**
The refusal gate now accepts two move-invariant classes: depth-preserving moves with untouched
targets, and targets that co-move with the workspace (the `../../../out/*.elf` shape — the
kuberos workspace relocates as a unit and `out/` rides the move as artifact co-moves). Fixture +
unit tests prove both; fail-closed otherwise.

**B3b — the 6 tests-host `include!` literals naming old sibling crate dirs: REMAINS, kernel-lane
scoped.** They name the OLD directory outright, so no move preserves them; the pre-move fix must
re-home the shared pure layout/signal/timekeep modules so the host harness (its own
`[workspace]` root escaping the kernel's build-std config) can consume them without forking the
single source of truth — a kernel-toolchain-harness refactor that requires the nightly
`build-std` toolchain to verify (host harness + no_std crates must both still compile). Do NOT
attempt blind. Sequencing: this slice runs in the kernel lane with toolchain verification, and
the kuberos move waits on it. Not on the baseline-burn critical path (21 rows vs libs/oya).

## Sequencing

1. Land the B1 workspace split (asterinas re-anchor + kuberos scaffold, own PR, gate-green).
   (The former B2 codemod step is already on trunk via #1523.)
3. Land the B3 pre-move include refactor (own PR, behavior-preserving, gate-green).
4. Re-anchor + execute the move-plan (plan file re-committed under `specs/reorg/` with the
   `kernel/kuberos/` destinations; singleton applies).
5. Burn the 21 `cloud/cloud-kernel/*` frozen-baseline rows in the same PR; `cloud/` then holds
   zero crate dirs.

## Non-goals

- No touching `os/` (separate lane, already dest-verified); the four `cloud/cloud-os/*` crates
  stay under `cloud/` until that lane.
- No behavior change to the framekernel; the move is byte-preserving modulo paths.
- No merge of the two nested workspaces into one (rejected: unresolvable toolchain/edition values).
