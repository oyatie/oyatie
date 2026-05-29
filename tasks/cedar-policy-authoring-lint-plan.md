# Task Plan: cedar-policy-authoring-lint

vertical: foundation
crate: oya-policy-cedar-domain
branch: feat/task-cedar-policy-authoring-lint-2026-05-28

## Objective

Add a pure, deterministic authoring-time static-validation (lint) pass to the
existing Cedar policy domain crate. The lint pass analyses a candidate
`PolicyVersion` **before** it is published (no `PolicySet` involvement), detects
structural defects in the rule set, and returns a structured `PolicyLintReport`.

No network, storage, or Cedar runtime dependency is introduced. All new types are
pure `serde` value types that extend `lib.rs` through the existing `mod`-based
flat-clean-arch layout.

## Subtasks

### [cedar-lint-1] Value types: PolicyLintFinding + PolicyLintReport

**What**

- Add `LintSeverity` enum (`Error | Warning`) with `Serialize/Deserialize`,
  `Clone/Copy/Debug/Eq/PartialEq`.
- Add `PolicyLintFinding { severity: LintSeverity, rule_indices: Vec<usize>,
  reason: String }` with `Serialize/Deserialize`, `Clone/Debug/Eq/PartialEq`.
- Add `PolicyLintReport { findings: Vec<PolicyLintFinding> }` with
  `Serialize/Deserialize`, `Clone/Debug/Eq/PartialEq` and two accessors:
  - `is_clean(&self) -> bool` — true iff `findings` is empty
  - `has_blocking(&self) -> bool` — true iff any finding has severity `Error`

**Acceptance**

- `cargo check -p oya-policy-cedar-domain --all-targets` passes.
- Unit test round-trips a `PolicyLintReport` through `serde_json` and asserts
  `has_blocking()` is true iff any finding is `LintSeverity::Error`.

---

### [cedar-lint-2] Lint function: conflict + duplicate detection

**What**

Add `pub fn lint_policy_version(version: &PolicyVersion) -> PolicyLintReport`
(free function, top-level in `lib.rs`) that:

1. **Duplicate rules**: any two rules with identical `(effect, principal_role,
   action, resource_prefix, required_attribute)` tuples are flagged as
   `LintSeverity::Error` with both rule indices in `rule_indices`.
2. **Conflicting Allow/Deny pairs**: any two rules that share the same
   `(principal_role, action, resource_prefix, required_attribute)` tuple but have
   opposite effects are flagged as `LintSeverity::Error` with both rule indices.

Each detected pair produces exactly one `PolicyLintFinding` (not two).
Input is `&PolicyVersion`; the function is pure and has no side effects.

**Acceptance**

- `cargo nextest run -p oya-policy-cedar-domain` passes.
- A conflicting Allow+Deny pair yields exactly one `Error` finding citing both
  rule indices.
- A duplicate pair yields exactly one `Error` finding.
- A clean policy (no conflicts, no duplicates) yields `is_clean() == true`.

---

### [cedar-lint-3] Lint function: shadow/unreachable detection

**What**

Extend `lint_policy_version` with shadow detection:

- For every pair of same-effect rules `(earlier, later)` where `earlier` appears
  before `later` in `version.rules`:
  - `earlier.resource_prefix` is a **prefix** of `later.resource_prefix` (i.e.
    `later.resource_prefix.starts_with(&earlier.resource_prefix)`) **and**
    `earlier.principal_role == later.principal_role` **and**
    `earlier.action == later.action` **and** the attribute match of `later` is
    equal-or-weaker than `earlier` (defined below).
  - Flag the **later** rule as `LintSeverity::Warning` (unreachable/shadowed).

Attribute weaker-or-equal logic (consistent with `PolicyRule::matches` prefix
semantics):
- If `earlier.required_attribute` is `None`, the earlier rule matches any
  attribute combination, so any `later.required_attribute` is equal-or-weaker.
- If both are `Some(k, v)` and are equal, it is equal.
- If `earlier` has `Some` and `later` has `None`, later is **not** weaker (it is
  broader, so it is NOT shadowed by earlier).

No `panic!`, `unwrap()`, or `expect()` in non-test code.

**Acceptance**

- `cargo nextest run -p oya-policy-cedar-domain` passes.
- A later rule under a broader same-effect prefix is flagged `Warning`.
- Two sibling prefixes (neither is a prefix of the other) produce no shadow finding.
- `cargo clippy -p oya-policy-cedar-domain` (workspace lints) produces no errors.

## Acceptance Summary

| Subtask | Gate |
|---|---|
| cedar-lint-1 | `cargo check --all-targets` green; serde round-trip test passes |
| cedar-lint-2 | `cargo nextest run -p oya-policy-cedar-domain` green; conflict/dup tests |
| cedar-lint-3 | nextest green; shadow/sibling tests; clippy clean |

## Boundaries

- **Only** `crates/oya-policy-cedar-domain/src/lib.rs` and
  `crates/oya-policy-cedar-domain/Cargo.toml` may be modified (no new files,
  no new workspace members, root `Cargo.toml` untouched).
- New logic is a free function + value types added directly to `lib.rs` (no new
  sub-modules unless clearly warranted for size; prefer flat layout per ADR-0509).
- No new runtime dependencies (serde + serde_json already present).
- Pure function: `lint_policy_version` has no I/O, no `async`, no `Arc`/`Mutex`.
