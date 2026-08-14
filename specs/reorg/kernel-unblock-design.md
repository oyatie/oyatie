# Kernel unblock design — `kernel-move-plan.BLOCKED.json` resolution card

**Class:** mixed (refactor → move) · **Capability span:** single (`kernel` meta dir, kuberos framekernel)
**Authority re-queried:** ADR-0701 (live apex), ADR-0562 §8 Fork 2, ADR-0512 kernel carve-out, ADR-0611 (Proposed; kernel/ asterinas workspace landed on dev)
**Status:** dispatcher-ready design; NOT executable until the three blockers below clear and the move-plan singleton is free.
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

**Acceptance criteria:** one nested workspace per rung; `cargo metadata` resolves in both; no
cross-workspace path-deps between kuberos and asterinas (they are disjoint ladders).

## Blocker B2 — codemod does not rewrite `[workspace.dependencies]`

**Class:** RESOLVED-ON-DEV (2026-08-14 re-verify).

`rewrite_dep_tables_recursive` now recurses into `[workspace.dependencies]` and its
dev/build siblings (the code comment cites the ADR-0512 carve-out's 5-of-7 edges explicitly).
Landed via PR #1523 (`fa1292c89 fix(reorg-codemod): make the oracle see the workspace it is
migrating`). Re-verified at `origin/dev@4a4f71a14` by reading the source and dry-running the
blocked plan: the workspace-dependency refusal no longer fires.

**Remaining B2 action:** none (the codemod change is on trunk).

## Blocker B3 — 33 escaping include!/include_bytes!/include_str! literals

**Class:** codemod behavior decision + pre-move refactor.

The literals traverse up out of their own crate, and 6 name old crate directories outright; the
`validate_no_escaping_path_literals` refusal gate (fail-closed, correct) blocks them all.

**Design decision (proposed), precedence from the ci-webhook-gateway lane (2026-08-14):** pre-move
refactor relocates cross-crate include targets INTO their consuming crates (the tests-host
`include!(…)` cases reference sibling crate sources — the included files become in-crate fixtures
or move into the including crate), exactly like the cedar-policy restructure that unblocked the
webhook-gateway move. The 12 freestanding user programs stay embedded via `include_bytes!` against
the moved `out/` — re-pointed in the same PR.

**Acceptance criteria:** zero `EscapingPathLiteral` refusals on dry-run; every remaining literal
resolves inside its crate both pre- and post-move.

## Sequencing

1. Land the B1 workspace split (asterinas re-anchor + kuberos scaffold, own PR, gate-green).
   (The former B2 codemod step is already on trunk via #1523.)
3. Land the B3 pre-move include refactor (own PR, behavior-preserving, gate-green).
4. Re-anchor + execute the move-plan (plan file re-committed under `specs/reorg/` with the
   `kernel/kuberos/` destinations; singleton applies).
5. Burn the 21 `cloud/cloud-kernel/*` frozen-baseline rows in the same PR; `cloud/` then holds
   zero crate dirs.

## Non-goals

- No touching `os/` (separate lane, already dest-verified).
- No behavior change to the framekernel; the move is byte-preserving modulo paths.
- No merge of the two nested workspaces into one (rejected: unresolvable toolchain/edition values).
