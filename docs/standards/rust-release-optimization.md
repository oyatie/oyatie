# Rust release optimization exception (Buck2 authority boundary)

Date context: 2026-06-02. Sources are official Rust and Buck2 documentation.

## Direct recommendation

Use Buck2 for all CI, CD, scripts, and build/test lanes. Permit Cargo only for a
narrow production release image/binary optimization experiment where the output is
binary-size, codegen, linker, allocator, PGO, or profile evidence. That evidence is
not a branch-protection context and cannot satisfy Phase-0 exit authority.

A production release optimization experiment is a measurement workflow, not a new
execution authority:

1. Prove the ordinary Buck2 build/test/gate lane first.
2. Run only the documented Cargo exception: release/custom-release profile builds
   or release-only rustc flag experiments using the `build`/`rustc` subcommands
   enumerated in `specs/buck2-authority-policy.json`.
3. Record the target triple, CPU compatibility policy, profile/codegen settings,
   allocator setting, representative workload, and before/after measurements.
4. If the experiment wins, encode the selected behavior into the Buck2 toolchain,
   Buck2 target graph, or Buck2-built OCI image path before it becomes standard.

## Required evidence for a Cargo release optimization run

- Commit SHA and service binary name.
- Target triple and release/custom profile name.
- Profile/codegen settings under test (`opt-level`, `lto`, `codegen-units`, `panic`,
  `strip`, debug/split-debuginfo, target CPU/features where applicable).
- Allocator under test, if any, plus proof that the `#[global_allocator]` singleton
  is owned by the intended crate graph.
- Binary size before/after and runtime memory/latency sample where relevant.
- Non-claim label: "release optimization evidence only; not CI merge authority".
- Follow-up Buck2 target or toolchain-cell change if the optimization becomes the
  selected path.

## Rust profile/codegen guidance

- Start from Cargo's release/custom profiles only for the exception. The Cargo Book
  defines release as the optimized artifact profile and exposes `opt-level`, `lto`,
  `panic`, `strip`, debug info, incremental, and codegen-units controls. The default
  release profile is already optimized (`opt-level = 3`) but does not enable LTO and
  uses 16 codegen units.
- Measure instead of assuming: Rust docs note that higher optimization levels can
  have surprising runtime/size results, including `3` being slower than `2` or size
  modes not always yielding smaller binaries. Treat `opt-level = 2`, `3`, `"s"`, and
  `"z"` as workload-specific candidates, not universal winners.
- Prefer ThinLTO before fat LTO for routine experiments because Cargo documents ThinLTO
  as substantially less expensive while still aiming for similar gains. Fat LTO and
  `codegen-units = 1` are final-binary measurement candidates after Buck2 gates are
  green, because they trade compile time for output quality.
- Keep release incremental compilation disabled. The rustc book says incremental
  compilation inhibits some optimizations and is not recommended for release builds.
- Consider `panic = "abort"` only when the service does not rely on unwinding,
  `catch_unwind`, FFI unwind behavior, or panic-as-recovery contracts. Tests and build
  scripts do not validate the same panic setting, so this needs release-artifact
  evidence.
- Prefer `strip = "debuginfo"` for default production service artifacts unless an
  evidence packet proves `strip = "symbols"` still preserves the crash-reporting,
  profiling, and incident-debug contract. Rustc warns that stripping symbols can make
  traces incomprehensible on some platforms and is not a security/obfuscation control.
- Avoid `target-cpu = "native"` for shared production images. Rustc defines `native`
  as the build host CPU, which is correct only for CPU-pinned artifacts; shared images
  need an explicit minimum CPU baseline or a per-architecture artifact contract.

## PGO guidance

- Profile-guided optimization is a release-exception candidate when a service has a
  stable, representative workload. The rustc book documents the four-step flow:
  build with profile generation, run the instrumented binary, merge `.profraw` into
  `.profdata`, and rebuild with `profile-use`.
- PGO profiles are only as good as the training workload. Evidence must label the
  workload as representative or non-representative, include the profile-generation
  commit SHA, and retain enough data to reproduce or reject the result.
- Do not make the `cargo-pgo` convenience subcommand or its installer step an
  authority path. The rustc book lists `cargo-pgo` as a community-maintained
  convenience, but Oyatie
  CI/CD/build authority remains Buck2; direct rustc flags or Buck2 toolchain encoding
  are the preferred handoff when PGO is selected.

## Allocator guidance

- Replacing the global allocator is unsafe surface area. The Rust Reference requires
  `#[global_allocator]` to be applied to one static item whose type implements
  `GlobalAlloc`, and only one global allocator is allowed in the crate graph.
- The `GlobalAlloc` contract forbids unwinding from allocator methods and warns that
  optimizers may remove or move allocations, so allocation-count measurements must not
  be treated as semantic proof. Use allocator benchmarks as evidence, not invariants.
- Allocator experiments must report at least RSS/working-set, allocation-heavy latency,
  fragmentation/release-back behavior where measurable, platform/target triple, and a
  rollback plan. Do not select an allocator from a blog post or crate popularity alone.

## Anti-patterns

- Reintroducing Cargo check/clippy/test/nextest/bench/install style commands in
  CI/CD/scripts/build/test lanes under the label of release optimization.
- Treating convenience subcommands such as cargo-chef, cargo-leptos, cargo-pgo,
  cargo-llvm-cov, cargo-fuzz, or cargo-mutants as active CI/CD/build authority;
  wrappers must be Buck2 targets or deletion-tagged bridges with retirement paths.
- Treating smaller binaries as automatically better when the change removes backtrace,
  profiling, or debug-symbol evidence needed by operations.
- Using `target-cpu = "native"` in a shared image built on one CPU class and deployed on
  another.
- Claiming production readiness, performance, memory efficiency, or security from
  profile settings alone without service measurements and target/non-claim labeling.

## Buck2 handoff

If a release optimization is selected, encode it into the Buck2 toolchain/target graph
or a documented toolchain-cell setting. Do not leave the selected behavior as an
operator-only Cargo invocation.

## Official/upstream sources

- Cargo Book — Profiles: <https://doc.rust-lang.org/stable/cargo/reference/profiles.html>
- rustc Book — Codegen Options: <https://doc.rust-lang.org/stable/rustc/codegen-options/>
- rustc Book — Profile-guided Optimization: <https://doc.rust-lang.org/stable/rustc/profile-guided-optimization.html>
- Rust `GlobalAlloc`: <https://doc.rust-lang.org/alloc/alloc/trait.GlobalAlloc.html>
- Rust Reference — `global_allocator`: <https://doc.rust-lang.org/reference/runtime.html#the-global_allocator-attribute>
- Buck2 build command: <https://buck2.build/docs/users/commands/build/>
- Buck2 test command: <https://buck2.build/docs/users/commands/test/>
