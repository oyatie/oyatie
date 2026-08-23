---
purpose: Oyatie — Code Style Standard
doc_status: published
---

# Oyatie — Code Style Standard

> **Owner:** `axis-foundry` (engineering platform).

## 1. Rust (primary)

- `rustfmt` with default settings (project-rooted `rustfmt.toml`)
- `cargo clippy --workspace --all-features --all-targets -- -D warnings` clean (no exceptions in product code; dev-only `#[allow]` requires comment)
- `cargo doc --no-deps` clean (no missing-docs warnings on public items)

### Dev workflow toolchain

Three local-dev tools are first-class — every engineer should use them:

- **`bacon`** (Apache-2 / MIT) — background `cargo check` + `cargo clippy` + `cargo nextest` watcher. Run `bacon` in a side terminal during authoring; receive instant feedback on save. Project ships a `bacon.toml` with curated jobs (`check`, `clippy`, `test`, `nextest`, `doc`, `arch-boundary`).
- **`cargo-machete`** (Apache-2 / MIT) — finds unused Cargo dependencies. Run `cargo machete` periodically (and at every dependency-add PR). Surfaces accidental adoption + supports the in-house preference + license-conscious posture by killing dead deps before they accumulate licenses.
- **`cargo-nextest`** (Apache-2 / MIT) — fast, parallel test runner with sharding + flaky-quarantine integration. Canonical (per project memory + ADR-0024). NEVER use bare `cargo test`. Per [retired `./bin/oya verify`](../TOOLCHAIN.md) bundle.
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
