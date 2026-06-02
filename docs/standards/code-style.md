---
purpose: Oyatie — Code Style Standard
doc_status: published
---

# Oyatie — Code Style Standard

> **Owner:** `axis-foundry` (engineering platform).

## 1. Rust (primary)

Buck2 is the active CI/CD/build/test authority for Rust. Cargo-based lint, test,
doc, or watcher commands are historical/local-advisory references only unless they
fit the documented production release image/binary optimization exception in
[`rust-release-optimization.md`](rust-release-optimization.md) or the metadata-only
Buck2 graph-generation exception in [`specs/buck2-authority-policy.json`](../../specs/buck2-authority-policy.json).

- `rustfmt` with default settings (project-rooted `rustfmt.toml`); CI/CD lanes run
  formatting checks through Buck2-owned targets or generated policy gates.
- Lint hygiene is clean under the Buck2 lint/gate target set; product-code
  exceptions still require comments and reviewer approval.
- Rustdoc/API documentation hygiene is a Buck2-owned documentation target or
  generated policy gate, not an independent Cargo merge authority.

### Dev workflow toolchain

Local editor feedback tools may exist, but they are not merge, CI, CD, build, or
Phase-0 exit authority. The canonical workflow is:

- Run Buck2 affected build/test/gate targets for the touched graph.
- Treat legacy Cargo-based local tools as non-authority hints unless the run is a
  documented production release artifact optimization measurement.
- Encode any selected release optimization back into the Buck2 toolchain, target
  graph, or Buck2-built OCI path before it becomes standard.
- Naming: `snake_case` for functions/vars/modules; `UpperCamelCase` for types/traits; `SCREAMING_SNAKE_CASE` for consts
- Error type: `thiserror`-derived per crate; never panic at API boundaries; `Result<T, E>` always; `?` propagation preferred over `match`
- Async: `tokio` + structured concurrency; `JoinHandle`s explicitly awaited or detached with cancellation; never spawn-and-forget without supervision
- Tracing: `#[tracing::instrument]` on every async public fn; structured fields per [logging-tracing.md](logging-tracing.md)
- Documentation: every public item has `///` rustdoc; per-crate README; per-module `//!` comment
- Tests: per-module `#[cfg(test)] mod tests`; integration tests in `tests/`; fixtures shared via `dev-dependencies`
- Lifetimes: explicit when non-elided; prefer owned types in API boundaries; borrowed in hot loops only
- Panics: only in unrecoverable paths; document with `# Panics` rustdoc
- Unsafe: requires comment explaining invariant + `// SAFETY:` rustdoc; reviewer must approve

### Forbidden in Rust product code

- `unwrap()` / `expect()` outside test code (use `?` or explicit error handling)
- `panic!()` outside unrecoverable paths
- `println!` / `eprintln!` (use tracing)
- Long match arms — extract to functions
- Builder pattern without `Default` impl (use `Default + with_*` or stay-with-fn-args)

## 2. TypeScript / JavaScript (auxiliary; per-frontend / per-Workspace UI)

- `tsc` + `prettier` + `eslint` strict
- Naming: `camelCase` for functions/vars; `PascalCase` for types/classes; `SCREAMING_SNAKE_CASE` for consts
- No `any`; use `unknown` + narrowing
- Strict null checks; no `!` non-null assertion outside test code

## 3. Korean comments + identifiers

- Code identifiers in English (per industry convention)
- Code comments in English (canonical)
- KR-locale UI strings in Korean (via i18n table)
- KR statute references may include Korean term + English transliteration (e.g. `// 통상임금 (ordinary wage)`)

## 4. File organization

- One Rust crate per flat-crates target per ADR-0015
- Per-crate `lib.rs` + per-module file
- Per-public-API trait in `<crate>::ports`; impl in `<crate>::adapters`
- Per-domain entity in `<crate>::entities`; use case in `<crate>::usecases`

## 5. Imports

- Group: std → external crates → workspace crates → super → crate
- Glob imports forbidden in product code (only in `prelude.rs`)
- Per-import sorting alphabetical within group

## 6. Sources
`.rustfmt.toml`, `.clippy.toml`, project Cargo workspace, ADR-0015, [TOOLCHAIN.md](../TOOLCHAIN.md).
