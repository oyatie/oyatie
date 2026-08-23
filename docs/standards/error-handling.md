---
purpose: Canonical error-handling discipline for the oyatie Rust workspace.
doc_status: published
---

---
doc_class: Standard
shape: ~
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Canonical error-handling discipline for the oyatie Rust workspace. Mandates
  `thiserror` for library error enums, `anyhow`/`eyre` only at top-level binary
  edges, `Result<T, E>` at every API boundary, prohibition of `.unwrap()` /
  `.expect()` in production code outside tests, and silent-failure prevention
  per the `silent-failure-hunter` reviewer-agent role. Resolves the
  `standards/error-handling.md` forward-reference sentinel in
  `docs/AGENTS.md` §During-change discipline.
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
planned_enforcement_ref: governance-error-boundary
enforcement_status:
  governance-error-boundary: existing
  governance-no-unwrap-prod: existing
  governance-silent-failure: F-PENDING-SILENT-FAILURE (crate missing; tracked in registry/stub-audit/2026-05-17/missing-fitness-crates.json)
  governance-audit-emission: existing
  F-FITNESS-ERROR-HANDLING-LANES: meta-lane per OP-11 audit
  F-FITNESS-ASPIRATIONAL-ENFORCEMENT-DETECTION: meta-lane per OP-11
meta_policy: ADR-0133 (chained-enforcement planning contract, pending)
companion_docs:
  - docs/standards/code-style-rust.md
  - docs/standards/testing.md
  - docs/standards/observability.md
  - docs/AGENTS.md
related_adrs:
  - ADR-0053
  - ADR-0052
  - ADR-0054
  - ADR-0083
---

# Error Handling

## Doctrinal authority — [decision-principles.json](../../specs/decision-principles.json) + [forbidden-operations.json](../../specs/forbidden-operations.json)

Per [`forbidden-operations.json`](../../specs/forbidden-operations.json) FO-04 ("No untyped
values at API boundaries") and [`AGENTS.md`](../AGENTS.md) §During-change
discipline, every Rust API boundary in `oyatie-*` crates MUST use typed errors.
This standard names the libraries, the boundary rule, and the silent-failure
prevention pattern.

## 1. The boundary rule

Hyperscaler consensus (AWS, Microsoft, Google) on Rust error handling:

- **Library crates** (`oyatie-*-kernel`, `oyatie-*-domain`, `oyatie-*-app`,
  `oyatie-*-adapter-*`): expose matchable error **enums** via
  [`thiserror`](https://docs.rs/thiserror). Callers can branch on variants;
  variants form part of the public API and are SemVer-governed.
- **Binary crates** (`oyatie-*-runtime-*`, CLI tools, integration test
  drivers): MAY use [`anyhow`](https://docs.rs/anyhow) or
  [`eyre`](https://github.com/eyre-rs/eyre) for type-erased error
  propagation at the top level. The crate MUST NOT re-export `anyhow::Error`
  from any `pub fn` consumed by another crate.

Lane: `governance-error-boundary` refuses library crates that
declare `anyhow` as a non-`dev-dependencies` entry, and refuses binary
crates that expose `anyhow::Error` in a `pub fn` signature.

Sources: [oneuptime — thiserror + anyhow](https://oneuptime.com/blog/post/2026-01-25-error-types-thiserror-anyhow-rust/view),
[Luca Palmieri — Error Handling in Rust](https://www.lpalmieri.com/posts/error-handling-rust/),
[Markaicode — Rust Error Handling 2025](https://markaicode.com/rust-error-handling-2025-guide/).

## 2. `thiserror` discipline

Every library error enum MUST:

1. Derive `thiserror::Error`, `Debug`, and (for kernel/domain types) be
   `#[non_exhaustive]` so adding variants is not a breaking change.
2. Carry a `#[error("...")]` message per variant — present-tense, declarative,
   includes the dynamic context (`{0}`, `{path:?}`).
3. Use `#[from]` only for transparent wrapper conversions; do NOT collapse
   multiple distinct conditions into one `From` impl.
4. Implement `std::error::Error::source()` so callers can walk the chain.
5. Be the **only** error type returned by the crate's public `Result`
   aliases.

Example shape:

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FoundryAdapterError {
    #[error("provider {provider} returned status {status}")]
    UpstreamStatus { provider: String, status: u16 },

    #[error("response body did not match schema {schema}")]
    SchemaMismatch { schema: &'static str, #[source] source: serde_json::Error },

    #[error("autonomy ceiling T{tier} forbids this invocation")]
    AutonomyDenied { tier: u8 },
}

pub type Result<T> = std::result::Result<T, FoundryAdapterError>;
```

## 3. `anyhow` / `eyre` discipline

Binary crates use `anyhow::Result<T>` / `eyre::Result<T>` for top-level
propagation. Rules:

1. `main()` returns `anyhow::Result<()>` (or `eyre::Result<()>`).
2. Use `.context("...")` / `.with_context(|| ...)` at every layer transition
   to attach diagnostic context. Treat unattached error returns as a defect.
3. Do **not** sprinkle `anyhow` deeper than the binary's `main.rs` and its
   adjacent `runtime::*` modules.
4. Configure `eyre` with `color-eyre` or `tracing-error` for backtrace
   capture only in development/staging builds; production strips backtraces
   (cost) but emits structured error events per
   [`observability.md`](observability.md).

## 4. No `unwrap()` / `expect()` outside tests

Lane `governance-no-unwrap-prod` (clippy `unwrap_used = "deny"`)
refuses:

- `.unwrap()`
- `.expect("...")`
- `Option::unwrap_or` / `Result::unwrap_or` are allowed only when the
  fallback is a legitimate default, not a panic-disguise.
- `panic!`, `todo!`, `unimplemented!`, `unreachable!` (`unreachable!` is
  allowed in genuinely-unreachable arms with a `// SAFETY: ...` comment).

Allow-list:

- Test code (`#[cfg(test)]`, `tests/`, `benches/`): `.unwrap()` / `.expect()`
  are fine; failing the test is the intent.
- `build.rs`: `.expect()` with a diagnostic message is allowed.
- `main()` of CLI tools: `.expect()` is allowed only on top-level
  error-display fallback (rare); prefer `?` + `anyhow::Result`.

If a piece of production code is **provably** infallible (e.g., a regex
compiled from a literal), document it with an inline comment AND prefer
`once_cell::Lazy` / `LazyLock` + `.unwrap_or_else(|| unreachable!(...))`.

## 5. `Result<T, E>` at API boundaries

Every public function that can fail returns `Result<T, E>` where `E` is the
crate's `thiserror` enum (libraries) OR `anyhow::Error` (binary internals).
Forbidden patterns:

1. Returning `Option<T>` to encode failure — `None` does not carry the
   reason. Use `Result<T, E>` and let the caller convert if appropriate.
2. Returning a magic-value sentinel (e.g., `-1`, `""`, `Default::default()`)
   on failure — refused at code review.
3. Logging an error and returning `Ok` — see §6.

Sub-rule (canonical): pure parsing or validation helpers MAY return
`Option<T>` when the "absence" is semantically meaningful (e.g.,
`find_first(...)`); this is a canonical sub-rule of the Result-returner
contract, not an exception.

## 6. Silent-failure prevention

The `silent-failure-hunter` reviewer agent (per
[`AGENTS.md`](../AGENTS.md) §Per-change-class reviewer agents) inspects every
PR that touches error paths and refuses:

1. **Log-and-swallow** — `tracing::error!(...)` followed by `Ok(())` /
   `return;` without propagation. Either propagate the error OR document
   the recovery path in code + ADR.
2. **`let _ =` on a fallible call** — assignment-to-underscore on `Result`
   is presumptive swallowing. Allow only when paired with an inline comment
   `// intentional: ...`.
3. **Empty match arms on `Err(_)`** — same rule; require comment.
4. **Drop guard pattern without error capture** — `Drop` impls that perform
   I/O MUST capture the result into a panic-safe channel or audit-chain
   emission per [`observability.md`](observability.md).
5. **Retry loops without max-attempts and backoff** — refused.
6. **Error returned but not surfaced to the audit chain** at cross-pillar
   boundaries — see §7.

Lane: `governance-silent-failure` (F-PENDING-SILENT-FAILURE; clippy-driven;
full gate is the reviewer-agent verdict; crate creation tracked in
registry/stub-audit/2026-05-17/missing-fitness-crates.json; enforced at PR review
until crate lands; meta-policy ADR-0133 chained-enforcement planning contract).

## 7. Audit-chain integration

Every error crossing a pillar boundary (kernel→domain, app→adapter,
cross-axis contract surface) MUST emit an `EVT-ERROR-*` audit event with:

- `evt_id`: ULID.
- `pillar_from`, `pillar_to`.
- `error_class`: the `thiserror` variant name (machine-extractable).
- `data_class`: the data classification at the boundary, per
  [`data-class.md`](data-class.md).
- `tenant_id`, `actor_id`, `trace_id` (W3C trace context).

The lane `governance-audit-emission` validates that every
cross-pillar error path has an emission point. Source:
[`observability.md`](observability.md) §3.

## 8. Conversion patterns

Use the `?` operator with `#[from]` impls to chain errors across layers.
For non-transparent conversions (e.g., adapter error → domain error), write
a manual `From` impl with a TODO-free body and a unit test that proves the
mapping is total.

```rust
impl From<adapter::Error> for domain::Error {
    fn from(e: adapter::Error) -> Self {
        match e {
            adapter::Error::Network(_)   => domain::Error::Transient,
            adapter::Error::Auth(_)      => domain::Error::Forbidden,
            adapter::Error::SchemaMismatch { .. } => domain::Error::ContractDrift,
        }
    }
}
```

## 9. Diagnostic reporting

For user-facing CLI / API errors, use [`miette`](https://docs.rs/miette) or
`tracing-error` to render structured diagnostics with span context. For
internal-only paths, `tracing::error!` with structured fields is sufficient.

Source: [Markaicode — Rust Error Handling 2025](https://markaicode.com/rust-error-handling-2025-guide/),
[eyre](https://github.com/eyre-rs/eyre).

## 10. Anti-patterns

1. **`Box<dyn std::error::Error>` in a library public API.** Type-erased
   errors leak implementation details and break SemVer reasoning.
2. **`anyhow::anyhow!("string")` in libraries.** Use `thiserror` with a
   named variant.
3. **Catching `panic!` via `catch_unwind`.** Reserved for FFI boundaries
   and the runtime supervisor; production-code panics indicate bugs.
4. **`io::Error` returned from a domain function.** Map at the adapter
   boundary to a domain variant.
5. **Skipping the audit-chain emission** on cross-pillar errors. The lane
   refuses; do not paper over.

## 11. Sources scanned

- [oneuptime — thiserror + anyhow](https://oneuptime.com/blog/post/2026-01-25-error-types-thiserror-anyhow-rust/view).
- [Luca Palmieri — Error Handling in Rust](https://www.lpalmieri.com/posts/error-handling-rust/).
- [Markaicode — Rust Error Handling 2025](https://markaicode.com/rust-error-handling-2025-guide/).
- [eyre](https://github.com/eyre-rs/eyre).
- [Momori — thiserror, anyhow](https://momori.dev/posts/rust-error-handling-thiserror-anyhow/).
- [`.omc/scratch/hyperscaler-best-practices-2026-05-12.md`](../../.omc/scratch/hyperscaler-best-practices-2026-05-12.md)
  Domain 3 "Error handling".
- [`docs/AGENTS.md`](../AGENTS.md) §Per-change-class reviewer agents
  (`silent-failure-hunter`).
