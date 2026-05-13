---
doc_class: Standard
shape: ~
length_cap: 250
authority_tier: 2
status: pending approval
purpose: |
  Canonical Rust code style for the oyatie workspace. Defines clippy-pedantic with
  cherry-picked allow-list, `#![deny(unsafe_code)]` policy, the `[workspace.lints]`
  inheritance table, the `oya-<context>-<role>[-<capability>]` naming convention, and
  the kernel ← domain ← app ← {api, worker, adapter} ← runtime module organization
  per ADR-0015.
lift_target: oyatie/docs/standards/code-style-rust.md
canonical_authority: docs/CONSTITUTION.md
enforced_by: oya-foundry-fitness-clippy-pedantic
companion_docs:
  - docs/standards/error-handling.md
  - docs/standards/dependency-policy.md
  - docs/standards/testing.md
  - docs/decisions/ADR-0015-flat-crates.md
---

# Code Style — Rust

## Constitutional authority — [CONSTITUTION.md](../CONSTITUTION.md)

The workspace ships in Rust. This standard governs every `oya-*` crate.
[`error-handling.md`](error-handling.md) governs error types;
[`testing.md`](testing.md) governs evidence runs;
[`dependency-policy.md`](dependency-policy.md) governs crate selection.

## 1. Toolchain pin

Per [`.omc/specs/lts-versions-verified-2026-05-12.md`](../specs/lts-versions-verified-2026-05-12.md):

- The workspace MUST pin `rust-toolchain.toml` to the current stable channel
  rounded down to the latest minor live for ≥ 30 days (currently **1.95.0**).
- `Cargo.toml [workspace.package] rust-version` MUST equal the toolchain pin.
- Edition: **2024** for new crates; legacy crates MAY remain on 2021 until
  ADR-tracked migration lands.
- `cargo-deny` MUST be pinned to a version whose MSRV ≤ the workspace
  `rust-version`.

Source: [Cargo Book — rust-toolchain.toml](https://rust-lang.github.io/rustup/overrides.html),
[RFC 3537 — MSRV resolver](https://rust-lang.github.io/rfcs/3537-msrv-resolver.html).

## 2. Workspace lint inheritance

Single `[workspace.lints]` table at the root `Cargo.toml`. Every member
crate declares `[lints] workspace = true`.

Pitfall: `workspace.lints` is **not** implicitly inherited; each member
MUST opt in. Override via `#![allow(...)]` in `lib.rs` / `main.rs`, **not**
in `Cargo.toml`.

Sources: [Cargo Book — Lints](https://doc.rust-lang.org/cargo/reference/lints.html),
[Mainmatter — cargo-autoinherit](https://mainmatter.com/blog/2024/03/18/cargo-autoinherit/).

### 2.1 The canonical `[workspace.lints]` table

```toml
[workspace.lints.rust]
unsafe_code              = "deny"   # see §4 for exceptions
unused_must_use          = "deny"
unreachable_pub          = "warn"
missing_docs             = "warn"   # warn in libraries; deny on public API of stable crates
rust_2018_idioms         = "warn"
rust_2024_compatibility  = "warn"

[workspace.lints.clippy]
pedantic                 = { level = "warn", priority = -1 }
nursery                  = { level = "warn", priority = -1 }
cargo                    = { level = "warn", priority = -1 }

# Cherry-picked DENY set (escalations from pedantic warn).
unwrap_used              = "deny"   # see error-handling.md §3
panic                    = "deny"
expect_used              = "warn"
dbg_macro                = "deny"
todo                     = "deny"
unimplemented            = "deny"
print_stdout             = "warn"
print_stderr             = "warn"
mod_module_files         = "deny"   # prefer `<dir>.rs`, not `<dir>/mod.rs`

# Pragmatic ALLOW set (pedantic false positives).
module_name_repetitions  = "allow"
must_use_candidate       = "allow"
missing_errors_doc       = "allow"
missing_panics_doc       = "allow"
multiple_crate_versions  = "allow"  # cargo-deny enforces this with audit trail
```

CI runs `cargo clippy --workspace --all-features --all-targets -- -D warnings`
per AGENTS.md D10. Source:
[Clippy Lints](https://rust-lang.github.io/rust-clippy/master/index.html),
[Effective Rust — Item 29: Listen to Clippy](https://effective-rust.com/clippy.html).

## 3. Formatting

- `cargo fmt --all` MUST pass at commit time.
- `rustfmt.toml` MUST be checked in at workspace root.
- Recommended settings: `edition = "2024"`, `max_width = 100`,
  `imports_granularity = "Crate"`, `group_imports = "StdExternalCrate"`,
  `reorder_imports = true`, `use_field_init_shorthand = true`.

## 4. `unsafe` policy

Default policy: every crate MUST include `#![forbid(unsafe_code)]` OR
`#![deny(unsafe_code)]` at crate root.

Exceptions (FFI, perf-critical primitives, kernel allocators):

1. The crate MUST allow-list the lint at the file level with
   `#![allow(unsafe_code)]` plus an ADR cite in the file header.
2. Every `unsafe` block MUST carry a `// SAFETY:` comment documenting the
   invariants the caller must uphold (AWS Firecracker convention).
3. Every unsafe surface MUST be covered by a `cargo-fuzz` harness per
   [`testing.md`](testing.md) §5 AND, where feasible, a Kani harness per
   ADR-RST-003 (pending).
4. CI lane `oya-foundry-fitness-unsafe-kani` enforces (2) + (3).

Sources: [AWS — How Kani is used](https://aws.amazon.com/blogs/opensource/how-open-source-projects-are-using-kani-to-write-better-software-in-rust/),
[Firecracker](https://firecracker-microvm.github.io/),
[AWS — Sustainability with Rust](https://aws.amazon.com/blogs/opensource/sustainability-with-rust/).

## 5. Naming conventions

Per [`docs/AGENTS.md`](../AGENTS.md) §Repository topology and
ADR-0015 (flat crates), every Rust crate path under `crates/` matches:

```
oya-<context>-<role>[-<capability>]
```

Rules:

- `<context>` ∈ axis or platform layer: `foundry`, `cloud`, `saas`, `search`,
  `workspace`, `ads`, `platform`, `tenancy`, `audit`, `tooling`.
- `<role>` ∈ architectural layer: `kernel` (pure-domain types, no I/O),
  `domain` (business logic), `app` (use-cases), `api` / `worker` / `adapter`
  (process boundary), `runtime` (binary).
- `<capability>` ∈ optional capability slug (lowercase, hyphen-separated).
- Package name MUST equal the directory name. Lane: `cargo-prefix`
  ([DOC-CATALOG.md](../DOC-CATALOG.md) §4).

Inside a crate:

- Modules: `snake_case`, file-per-module (no `mod.rs`); enforced by
  `clippy::mod_module_files`.
- Types: `PascalCase`. Traits: noun phrases (`Provider`, `EventEmitter`),
  not `-able` adjectives.
- Functions / methods: `snake_case`, verb-first.
- Constants / statics: `SCREAMING_SNAKE_CASE`.
- Generic params: single capital letter (`T`, `E`, `R`) or descriptive
  PascalCase when ≥ 2 generics in scope.

## 6. Module organization — kernel ← domain ← app ← {api, worker, adapter} ← runtime

Per ADR-0015 layered architecture, dependencies flow **downward** only:

```
runtime  ──► { api │ worker │ adapter } ──► app ──► domain ──► kernel
```

| Layer | Role | MAY import | MUST NOT import |
|---|---|---|---|
| `kernel` | Pure domain types; no I/O, no async, no provider deps | (nothing project-internal) | anything from `domain`/`app`/`adapter`/`api`/`worker`/`runtime` |
| `domain` | Business invariants; pure functions on kernel types | `kernel` | `app`, `adapter`, `api`, `worker`, `runtime` |
| `app` | Use-cases; orchestrates domain via traits | `kernel`, `domain` | concrete adapters; only trait abstractions |
| `api`, `worker` | Process-boundary inputs (HTTP, gRPC, queue, scheduler) | `kernel`, `domain`, `app` | other `api`/`worker` peers (use traits) |
| `adapter` | Provider implementations (cloud SDKs, DBs, message brokers) | `kernel`, `domain`, `app` | other adapters; cross-axis adapters |
| `runtime` | Binaries that compose api/worker/adapter via DI | every lower layer | (none) |

Lanes enforcing this:

- `oya-foundry-fitness-flat-crates` validates the path / name pair.
- `oya-foundry-fitness-layering` (NEW; pending) refuses upward imports.
- `oya-foundry-fitness-provider-coupling` per Directive 4 refuses provider
  imports outside `oya-*-adapter-<provider>-*` crates.

## 7. Async / runtime

- Tokio is the workspace monoculture. `async-std` and `smol` are forbidden.
- Multi-thread Tokio scheduler is default; `tokio::task::spawn_blocking` for
  any operation ≥ 10–100 µs of CPU.
- `async fn in traits` is the default for new traits (edition 2024); use
  `async-trait` only for object-safe trait objects where needed.

Source: [corrode — State of Async Rust](https://corrode.dev/blog/async/),
[Tokio docs](https://docs.rs/tokio).

## 8. Public API discipline

- `cargo public-api` diffs the public surface on every PR; breaking diffs
  require an ADR cite and a SemVer major bump (or a deprecation cycle per
  ADR-0037).
- `cargo semver-checks` runs in CI as the de-facto linter.
- Every `pub fn` on a stable crate MUST carry rustdoc (`missing_docs = warn`
  → `deny` at stable promotion).

Sources: [cargo-semver-checks](https://crates.io/crates/cargo-semver-checks),
[cargo-public-api](https://github.com/cargo-public-api/cargo-public-api).

## 9. Anti-patterns

1. **Per-crate clippy lint sets** that diverge from the workspace table —
   either upstream the lint or document the divergence as a file-level
   `#![allow]` with ADR cite.
2. **Mixing `mod.rs` and `module.rs` styles** in one crate — pick the
   file-style; CI enforces.
3. **Provider SDKs imported into `app` or `domain`** — must live in
   `adapter-<provider>-*` crates.
4. **`unsafe` without `SAFETY:` comment + fuzz harness** — refused by the
   unsafe-kani lane.
5. **`unwrap()` / `expect()` in production code** — see
   [`error-handling.md`](error-handling.md) §3.

## 10. Sources scanned

- [Cargo Book — Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html).
- [Cargo Book — Lints](https://doc.rust-lang.org/cargo/reference/lints.html).
- [RFC 3389 — manifest-lint](https://rust-lang.github.io/rfcs/3389-manifest-lint.html).
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/index.html).
- [Effective Rust — Item 29: Listen to Clippy](https://effective-rust.com/clippy.html).
- [AWS — Sustainability with Rust](https://aws.amazon.com/blogs/opensource/sustainability-with-rust/).
- [Microsoft — Hyperlight](https://opensource.microsoft.com/blog/2024/11/07/introducing-hyperlight-virtual-machine-based-security-for-functions-at-scale/).
- [Azure SDK Rust Guidelines](https://azure.github.io/azure-sdk/rust_introduction.html).
- ADR-0015 (flat crates), ADR-0017 (`oya-` prefix), ADR-0037 (deprecation).
