---
doc_class: DecisionRecord
shape: ~
length_cap: 250
authority_tier: 2
status: Superseded
doc_status: published
date: 2026-05-15
purpose: |
  Formalise the thiserror (library crates) / anyhow (binary crates) tier
  decision + `.unwrap()` / `.expect()` prohibition in non-test library code
  + silent-failure prevention via the `silent-failure-hunter` reviewer
  agent. Resolves the missing-ADR finding from TG3 sprawl audit pair-7
  (docs/standards/error-handling.md was the sole authority for these
  rules; this ADR provides the decision-log entry for traceability).
canonical_authority: docs/CONSTITUTION.md
supersedes: ~
superseded_by: [ADR-700]
sunset_topic: adr-0083-infallible-audit-signature
sunset_milestone: adr-0083-merge-historical-2026-05-15
related_adrs:
  - ADR-0015
  - ADR-0017
  - ADR-0037
  - ADR-0056
  - ADR-0069
companion_docs:
  - docs/standards/error-handling.md
  - docs/standards/code-style-rust.md
  - docs/standards/clean-architecture.md
  - docs/standards/testing.md
---

# ADR-0083: Rust Error-Handling Tier Decision — thiserror at libraries / anyhow at binaries / no panics in library code

> **Status:** Accepted — 2026-05-15
> **Date:** 2026-05-15
> **Owner:** `council-architecture`
> **Supersedes:** none
> **Related:** ADR-0015, ADR-0017, ADR-0037, ADR-0056, ADR-0069

---

## Context

Every `oya-*` Rust crate exposes a public surface that callers — internal
agents, downstream products, generated SDKs, integration tests — bind
against. Without a structured error-handling tier, three failure modes
recurred:

1. **Untyped errors at API boundaries.** `String` and `Box<dyn Error>`
   leaked across crate boundaries; matchable variants disappeared; SDK
   generators could not produce typed branching for clients.
2. **`.unwrap()` / `.expect()` in library code.** Pre-merge hooks blocked
   commits in some lanes but not others; the rule lived only in `docs/
   standards/error-handling.md` with no ADR-level decision record.
3. **Silent drops on the `Err` path** of `Result` returners — a check
   lane whose job was to surface conditions instead reported clean when
   I/O failed. CONV-2 of the TG2 11-facet debate flagged this pattern
   across 4 sub-checks; the structural fix is a typed-error policy with
   matchable variants the lane CANNOT discard silently.

The TG3 standards-sprawl audit (evidence/audits/standards-sprawl-audit-
1778812600.json pair-7) identified that `docs/standards/error-handling.md`
was the sole authority for the thiserror/anyhow tier rule but had no
corresponding ADR. Without an ADR, the rule had no version-bump trigger,
no sunset clause, no formal supersession path — it was a standard
masquerading as a decision.

## Decision

We adopt a **three-tier** error-handling policy applied uniformly across
every `oya-*` Rust crate. Per-tier rules below are normative
(RFC-2119 keywords as defined in docs/standards/error-handling.md§1).

### Tier 1 — Library crates (kernel / domain / app / adapter / api / worker / infrastructure / service / rest / cli / bindings)

- Public errors **MUST** be matchable enums exported via
  [`thiserror`](https://docs.rs/thiserror). Variants form part of the
  public API and are SemVer-governed per ADR-0037.
- `.unwrap()` and `.expect("…")` are **forbidden** outside `#[cfg(test)]`
  modules and outside doc-tests. Use `?` propagation or pattern-match
  the `Err` arm with an explicit finding emission.
- `anyhow::Error` and `eyre::Error` **MUST NOT** appear in public-API
  return types. Type-erased errors hide variant information from callers.
- Every fallible function **MUST** return `Result<T, ConcreteEnum>` where
  `ConcreteEnum` lives in the same crate and is exhaustively
  pattern-matchable.

### Tier 2 — Binary crates (runtime / standalone CLI tools / integration test drivers)

- **MAY** use [`anyhow`](https://docs.rs/anyhow) or
  [`eyre`](https://github.com/eyre-rs/eyre) for type-erased propagation
  at the top-level `main()` and immediate call frames.
- The binary's `main()` **MUST NOT** re-export `anyhow::Error` from any
  Rust API surface (no `pub fn` returning `anyhow::Result<T>`); type-
  erasure stops at the binary boundary.
- `.unwrap()` is permitted ONLY when the failure mode is documented as
  "cannot occur given the invariants enforced at the previous tier" and
  the rationale is captured in a one-line comment.

### Tier 3 — Test code

- `.unwrap()` and `.expect()` are unrestricted in `#[cfg(test)]` modules
  and in integration tests under `tests/`. Tests are the safety net for
  the invariants Tier 1 + Tier 2 enforce; panicking on broken invariants
  IS the failure signal.

### Silent-failure prevention

- The `silent-failure-hunter` reviewer-agent role (per
  docs/standards/error-handling.md§4) **MUST** sign `## Code Review` on
  every PR that adds an error-handling change. Patterns flagged:
  `let _ = fallible(...);` (Err discard); `if let Ok(...) = ...` without
  an `else` arm that pushes a finding (the CONV-2 anti-pattern);
  `filter_map(Result::ok)` on read_dir / read_to_string iterators.

## Consequences

### Positive

- **Variant-stable SDK generation.** ADR-0037 stability tiers can be
  enforced per variant because variants are explicit. Cargo `cargo-public-
  api` and `cargo-semver-check` diff matchable enums precisely.
- **No silent fail-open.** The CONV-2 pattern that 3 lenses of the TG2
  debate flagged becomes mechanically diff-able by silent-failure-hunter.
- **Traceability.** Future drift triggers an ADR-supersession requirement
  per the standard's evolution policy — this decision is no longer
  buried in a standard alone.

### Negative

- **Boilerplate.** Adding a `thiserror`-derived enum per crate has a
  syntactic cost. Mitigation: kernel crates already permit `thiserror` as
  the sole non-`core::*` / `std::*` dep per docs/standards/clean-
  architecture.md§2.1; the boilerplate is already permitted.
- **Migration cost.** Existing crates that returned `String` errors or
  `Box<dyn Error>` need conversion. Per ADR-0037 tier matrix, libraries
  at the `preview` tier can break callers without deprecation timelines;
  `stable` and `GA` libraries require a major version bump and a 6-12-
  month deprecation window per their declared tier.

### Neutral

- This ADR formalises rules that already lived in
  `docs/standards/error-handling.md` — no behavioral change for code
  that already followed the standard.

## Lane enforcement

- `oya-governance-error-boundary` (declared in error-handling.md
  frontmatter) enforces this ADR on every PR.
- `silent-failure-hunter` reviewer-agent signature required on
  error-handling changes per the per-change-class reviewer matrix in
  docs/AGENTS.md§Per-change-class-reviewer-agents.

## Sources scanned

- `docs/standards/error-handling.md` (Accepted, 2026-05-12) — the
  policy text this ADR formalises as a decision record.
- TG3 standards-sprawl audit findings at
  `evidence/audits/standards-sprawl-audit-1778812600.json#pair-7-error-
  handling` (verdict: NO-CANONICAL-PARTNER-FILE-FIXUPTASK) — surfaced
  the missing-ADR gap.
- ADR-0037 (public-api-stability-tiers-and-deprecation) — pairs with
  this ADR for the variant-stability matrix.
- ADR-0056 (rust-clean-architecture-bnf) — defines the 12-layer enum
  that Tier 1 / Tier 2 classifications map to.
- TG2 11-facet debate synthesis CONV-2 (fail-open IO) — empirical
  evidence that silent-failure prevention is load-bearing.

## Decision log

- 2026-05-15 — Authored. Closes
  `F-AUTHOR-ADR-ERROR-HANDLING-TIER-DECISION` filed in TG3 sprawl audit.
- 2026-05-15 — Amendment: authorize `append_classifications` Result return
  (see Amendment 2026-05-15 below). Closes Phase E1 DENY-promotion final
  blocker on Site 3 (`oya-audit-chain-domain` private fn `.expect()`).

## Amendment 2026-05-15 — `append_classifications` Tier 1 Compliance

### Context

Phase E1 DENY promotion of the workspace `[lints.clippy]` block from
`warn` → `deny` for `unwrap_used` / `expect_used` / `panic` stalled on a
single residual production `.expect()` site (Site 3) at
`crates/oya-audit-chain-domain/src/lib.rs:343` — the private helper
`append_legacy_data_classes` that hides a `.expect(...)` on the result of
the fallible `try_append_legacy_data_classes`. That `.expect(...)` masks
two real failure modes the inner function explicitly returns:
`AuditChainError::EmptyTenantId` and `AuditChainError::TenantShardMismatch`.

The compatibility-seam `pub fn append_classifications<C>(...)` calls into
that helper, so the `.expect(...)` propagates to the public API. Per Tier
1 of this ADR, infallible returns cannot stand atop fallible internals
without erasing matchable failure information; per
[`feedback_no_silent_regression`](../../) the two `AuditChainError`
variants are public-contract failures that callers MUST be able to match.

### Decision

Authorize the breaking change on `oya-audit-chain-domain` from `0.1.0`
to `0.2.0`. The new public signature is canonical Tier 1:

```rust
pub fn append_classifications<C>(
    &mut self,
    tenant_id: impl Into<String>,
    surface: impl Into<String>,
    plane: Plane,
    purpose: Purpose,
    data_classifications: impl IntoIterator<Item = C>,
    decision: impl Into<String>,
) -> Result<&AuditEvent, AuditChainError>
where
    C: Into<DataClassification>;
```

The previously-returned infallible reference `-> &AuditEvent` is
immediately superseded — no grace window. Rationale: Tier 1 forbids
production `.expect()`; the new variant set (`EmptyTenantId`,
`TenantShardMismatch`) was always reachable on the existing
`.expect(...)` path, so callers that depended on the infallible signature
were depending on a Tier-1-violating shape that the workspace lint
promotion now mechanically rejects. This is the canonical sub-rule
(per `feedback_no_exceptions_canonical`): a one-line breaking change
restoring canonical Tier 1 conformance is the canonical path, not a
sub-clause sub-rule preserving the older shape.

### Version bump

`oya-audit-chain-domain` 0.1.0 → 0.2.0 (override
`[package].version = "0.2.0"` on this crate; remaining crates continue
to inherit `[workspace.package].version`).

### Sunset

The previous infallible signature is superseded with no grace window.
All in-tree callers (~28 production sites in `oya-application-app`, 1
in `oya-audit-chain-usecase`, 4 in `oya-cloud-observability-domain`
test fixtures, 6 in `oya-audit-chain-domain` / `-file-adapter`
integration tests, 1 in this crate's internal tests) are propagated to
`?` on the same commit as the signature change.

### Lane interaction

- `lean-a4-semver` (cargo-semver-checks) WILL flag the
  `append_classifications` return-type change as a major-version break.
  The crate's version bump to `0.2.0` satisfies the lane.
- `lean-a10-regression` (per `feedback_no_silent_regression`) is
  satisfied by (a) this ADR amendment, (b) the version bump, (c) the
  Tier 1 conformance restoration that eliminates the silent-failure
  pattern Site 3 contained.

### Consequences

Positive: workspace-wide `[lints.clippy]` reaches DENY. Tier 1 enforced
mechanically across every `oya-*` crate going forward. The
`AuditChainError::{EmptyTenantId, TenantShardMismatch}` variants become
typed branches every caller MUST handle, eliminating the silent-failure
pattern that Site 3 hid.

Negative: 35+ call sites updated this commit. Mitigated by the uniform
`.map_err(...)?` pattern across `oya-application-app` and `?`
propagation in `oya-audit-chain-usecase`.
