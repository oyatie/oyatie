# Pre-lane 0.6 — no_std kernel-subtree EXCLUDE set (the 12-entry exclude list)

**Lane:** pre-lane 0.5 / gates pre-lane 0.6 · **Authority:** D-CONFORM (`UNIFIED-EXECUTION-PLAN.md` §6) · **Mode:** READ-ONLY (no edits, no builds)

## Decision context (D-CONFORM)

The migration-fit decision for nested workspaces is **collapse-2-STD / exclude-the-12-kernel-subtree / exclude-vendored**. The two std workspaces (`stack/operating-system`, `stack/kubernetes`) collapse into ONE consolidated STD root. The no_std framekernel subtree at `stack/kernel` must NOT be merged into that STD root: it pins its own nightly toolchain + custom bare-metal targets + `-Z build-std`, which are fundamentally incompatible with the STD root's stable/precompiled-`core` build. Therefore every `[workspace]`-declaring manifest under `stack/kernel` must be named in the consolidated STD root's `[workspace] exclude` key so cargo never tries to absorb it.

> "nested-workspaces = collapse-2-STD / **exclude-the-12-kernel-subtree** / exclude-vendored (pre-lane 0.6 must prove the full 12-entry kernel-exclude inert)." — UNIFIED-EXECUTION-PLAN.md:110

## The 12 exclude entries (paths relative to repo root `stack/`)

Each is a self-contained `[workspace]` root (the framekernel workspace + 11 detached child manifests, each carrying an empty `[workspace]` table to detach from the parent). All 12 build on the **kernel's own** `nightly-2026-02-28` toolchain (`rust-toolchain.toml`) targeting the custom bare-metal triples `aarch64-unknown-none-softfloat` / `x86_64-unknown-none` with `-Z build-std` — never the STD root's stable toolchain.

| # | Exclude path | Workspace kind | name | Build target / toolchain |
|---|---|---|---|---|
| 1 | `kernel` | **framekernel** root workspace (7 members: kernel, hal, arch-aarch64, arch-x86_64, frame, ksync, user_layout) | (virtual) | nightly-2026-02-28 · `build-std=[core,alloc,compiler_builtins]` · default `aarch64-unknown-none-softfloat`, `--target x86_64-unknown-none` for x86 (root `.cargo/config.toml`) |
| 2 | `kernel/crates/arch-x86_64/user-src` | detached `[workspace]` (ELF user target) | user-hello-x86_64 | `x86_64-unknown-none` · `build-std=[core,compiler_builtins]` · user.ld @ 0x40_0000, static, no-pie |
| 3 | `kernel/crates/arch-x86_64/user-spawn-src` | detached `[workspace]` (ELF user target) | user-spawn-x86_64 | `x86_64-unknown-none` · `build-std=[core,compiler_builtins]` · user.ld @ 0x40_0000 |
| 4 | `kernel/crates/arch-x86_64/user-exec-src` | detached `[workspace]` (ELF user target) | user-exec-x86_64 | `x86_64-unknown-none` · `build-std=[core,compiler_builtins]` · user.ld @ 0x40_0000 |
| 5 | `kernel/crates/arch-x86_64/user-signal-src` | detached `[workspace]` (ELF user target) | user-signal-x86_64 | `x86_64-unknown-none` · `build-std=[core,compiler_builtins]` · user.ld @ 0x40_0000 |
| 6 | `kernel/crates/arch-x86_64/user-clock-src` | detached `[workspace]` (ELF user target) | user-clock-x86_64 | `x86_64-unknown-none` · `build-std=[core,compiler_builtins]` · user.ld @ 0x40_0000 |
| 7 | `kernel/crates/arch-x86_64/user-smpdemo-src` | detached `[workspace]` (ELF user target) | user-smpdemo-x86_64 | `x86_64-unknown-none` · `build-std=[core,compiler_builtins]` · user.ld @ 0x40_0000 |
| 8 | `kernel/crates/arch-x86_64/user-fsbase-src` | detached `[workspace]` (ELF user target) | user-fsbase-x86_64 | `x86_64-unknown-none` · `build-std=[core,compiler_builtins]` · user.ld @ 0x40_0000 |
| 9 | `kernel/crates/arch-x86_64/user-init-src` | detached `[workspace]` (ELF user target) | user-init-x86_64 | `x86_64-unknown-none` · `build-std=[core,compiler_builtins]` · user.ld @ 0x40_0000 |
| 10 | `kernel/crates/arch-aarch64/user-smpdemo-src` | detached `[workspace]` (ELF user target) | user-smpdemo | `aarch64-unknown-none-softfloat` · `build-std=[core,compiler_builtins]` · user.ld, cortex-a72 |
| 11 | `kernel/crates/arch-x86_64/fsbase-worker-src` | detached `[workspace]` (std/musl worker) | fsbase-worker-x86_64 | **NOT bare-metal**: fully-static `x86_64` musl ELF built INSIDE Docker (`rust:alpine`, `--platform linux/amd64`) via `build.sh`; no `.cargo/config.toml`. Still outside the STD root (own toolchain inside Docker). |
| 12 | `kernel/crates/arch-aarch64/tests-host` | detached `[workspace]` (host std test harness) | arch-aarch64-layout-tests | **NOT bare-metal**: host `std` libtest crate; its own `.cargo/config.toml` clears `build-std` + sets `target=aarch64-apple-darwin`. `include!`s `user_layout/src/layout.rs` to host-test the pure EL0 math. |

### The 9 `user-*-src` ELF test targets (rows 2–10)

user-src · user-spawn-src · user-exec-src · user-signal-src · user-clock-src · user-smpdemo-src (x86_64) · user-fsbase-src · user-init-src · user-smpdemo-src (aarch64) = **9**. Each is a freestanding ring-3 ELF embedded into the kernel image via `include_bytes!` (some feature-gated: `signal-demo`, `clock-demo`, `smp-sched-demo`), linked at user base 0x40_0000 with its own `user.ld`, issuing raw `syscall`/`svc` — outside the kernel TCB.

## Build-isolation confirmation (per row)

- **Framekernel root (#1):** `stack/kernel/Cargo.toml` is a virtual workspace; `rust-toolchain.toml` pins `channel = "nightly-2026-02-28"` + components `rust-src, rustfmt, clippy, llvm-tools` + bare-metal targets; root `.cargo/config.toml` sets `[unstable] build-std = ["core","alloc","compiler_builtins"]` and injects the kernel PVH/virt linker scripts. None of this is STD-root-compatible. NOTE: the root's *current* `[workspace] exclude` already self-excludes rows 2–4, 6, 9, 10 internally (8 entries) — this is intra-kernel hygiene; the consolidated STD root must list ALL 12 (rows 1–12), i.e. the whole `stack/kernel` subtree, because from the STD root only `kernel` (and its nested manifests) need excluding.
- **Rows 2–9 (x86_64 user ELFs):** each ships its own `.cargo/config.toml` with `build-std=[core,compiler_builtins]`, `target = "x86_64-unknown-none"`, `user.ld` @ 0x40_0000, `relocation-model=static`, `-no-pie`, `-static`. Each carries an empty `[workspace]` table to detach. Inherits the kernel nightly via the kernel `rust-toolchain.toml`. Built via per-crate `build.sh` (copies outside repo first to avoid parent-config merge).
- **Row 10 (aarch64 user-smpdemo):** own `.cargo/config.toml` with `build-std=[core,compiler_builtins]`, `target = "aarch64-unknown-none-softfloat"`, `user.ld`, `target-cpu=cortex-a72`, static, no-pie. Detached `[workspace]`.
- **Row 11 (fsbase-worker):** std/musl, built in Docker `rust:alpine --platform linux/amd64` (9 docker/musl refs in `build.sh`); NO `.cargo/config.toml`; detached `[workspace]`. Its own (Docker-internal) toolchain, so it never touches the STD root.
- **Row 12 (tests-host):** host `std` libtest; own `.cargo/config.toml` explicitly clears `build-std`/`build-std-features` and sets `target = "aarch64-apple-darwin"`; detached `[workspace]`. Run with `cargo test --manifest-path crates/arch-aarch64/tests-host/Cargo.toml`.

## Verdict

CONFIRMED — the 12 paths above are the complete no_std (+ 2 non-bare-metal-but-own-toolchain) exclude set for the consolidated STD root's `[workspace] exclude` key. Every entry is a self-contained `[workspace]` root that builds on the kernel's own `nightly-2026-02-28` + custom bare-metal targets (or its own Docker/host toolchain for rows 11–12), and none can be absorbed by the STD root. Count reconciles exactly to the D-CONFORM "12-entry kernel-exclude": framekernel (1) + 9 user-*-src + fsbase-worker-src (1) + tests-host (1) = 12.

### Exclude key for the consolidated STD root Cargo.toml
```toml
exclude = [
    "kernel",
    "kernel/crates/arch-x86_64/user-src",
    "kernel/crates/arch-x86_64/user-spawn-src",
    "kernel/crates/arch-x86_64/user-exec-src",
    "kernel/crates/arch-x86_64/user-signal-src",
    "kernel/crates/arch-x86_64/user-clock-src",
    "kernel/crates/arch-x86_64/user-smpdemo-src",
    "kernel/crates/arch-x86_64/user-fsbase-src",
    "kernel/crates/arch-x86_64/user-init-src",
    "kernel/crates/arch-aarch64/user-smpdemo-src",
    "kernel/crates/arch-x86_64/fsbase-worker-src",
    "kernel/crates/arch-aarch64/tests-host",
]
```
> Excluding `kernel` (the framekernel workspace root) already keeps cargo out of the whole subtree; the 11 nested paths are listed explicitly for documentation/inert-proof completeness and to match the kernel root's own intra-workspace exclude discipline.
