# Rust release optimization exception (Buck2 authority boundary)

Date context: 2026-06-02. Sources are official Rust and Buck2 documentation.

## Direct recommendation

Use Buck2 for all CI, CD, scripts, and build/test lanes. Permit Cargo only for a
narrow production release image/binary optimization experiment where the output is
binary-size, codegen, linker, allocator, or profile evidence. That evidence is not
a branch-protection context and cannot satisfy Phase-0 exit authority.

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
  `panic`, `strip`, debug info, incremental, and codegen-units controls.
- Measure instead of assuming: Rust docs note that higher optimization levels can
  have surprising runtime/size results, including `3` being slower than `2` or size
  modes not always yielding smaller binaries.
- Prefer `codegen-units = 1` and LTO experiments for final binary measurements only
  after Buck2 build/test gates are green, because they trade compile time for output
  quality.
- Consider `panic = "abort"` and `strip` only when the service's observability and
  incident-debug contract still has enough evidence (for example split debug symbols
  retained outside the runtime image).

## Allocator guidance

- Replacing the global allocator is unsafe surface area. The Rust Reference requires
  `#[global_allocator]` to be applied to one static item whose type implements
  `GlobalAlloc`, and only one global allocator is allowed in the crate graph.
- The `GlobalAlloc` contract forbids unwinding from allocator methods and warns that
  optimizers may remove or move allocations, so allocation-count measurements must not
  be treated as semantic proof. Use allocator benchmarks as evidence, not invariants.

## Buck2 handoff

If a release optimization is selected, encode it into the Buck2 toolchain/target graph
or a documented toolchain-cell setting. Do not leave the selected behavior as an
operator-only Cargo invocation.

## Official/upstream sources

- Cargo Book — Profiles: <https://doc.rust-lang.org/stable/cargo/reference/profiles.html>
- rustc Book — Codegen Options: <https://doc.rust-lang.org/stable/rustc/codegen-options/>
- Rust `GlobalAlloc`: <https://doc.rust-lang.org/alloc/alloc/trait.GlobalAlloc.html>
- Rust Reference — `global_allocator`: <https://doc.rust-lang.org/reference/runtime.html#the-global_allocator-attribute>
- Buck2 build command: <https://buck2.build/docs/users/commands/build/>
- Buck2 test command: <https://buck2.build/docs/users/commands/test/>
