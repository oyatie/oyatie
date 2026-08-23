# Spec: cedar-policy-authoring-lint

vertical: foundation
crate: policy-cedar-domain
task-slug: cedar-policy-authoring-lint

## Objective

Extend `policy-cedar-domain` with a static authoring-time lint pass that
detects structural defects in a `PolicyVersion` **before** it is published to a
`PolicySet`. The lint pass is pure (no I/O, no Cedar runtime, no network) and
returns a structured `PolicyLintReport` that callers can gate on before invoking
`PolicySet::publish`.

This fills an authoring-quality gap: the existing `publish` validation only
rejects structurally invalid rules (empty fields, bad semver) but cannot detect
semantic conflicts between rules within the same version.

## Vertical and Crate

- **Vertical**: foundation
- **Crate**: `policy-cedar-domain` (`crates/policy-cedar-domain/`)
- **Flat-clean-arch layout**: new types and function land directly in `src/lib.rs`
  as a contiguous `// ── lint ──` section, matching the existing single-file
  convention for this crate. No new `mod`, no new crate.

## New Public Surface

```rust
// ── LintSeverity ──────────────────────────────────────────────────────────────
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum LintSeverity { Error, Warning }

// ── PolicyLintFinding ─────────────────────────────────────────────────────────
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyLintFinding {
    pub severity: LintSeverity,
    pub rule_indices: Vec<usize>,
    pub reason: String,
}

// ── PolicyLintReport ──────────────────────────────────────────────────────────
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyLintReport {
    pub findings: Vec<PolicyLintFinding>,
}

impl PolicyLintReport {
    pub fn is_clean(&self) -> bool { ... }
    pub fn has_blocking(&self) -> bool { ... }
}

// ── entry point ───────────────────────────────────────────────────────────────
pub fn lint_policy_version(version: &PolicyVersion) -> PolicyLintReport { ... }
```

No changes to any existing type signatures or behavior.

## Detection Rules

### DR-1 Duplicate rules (Error)

Two rules at indices `i` and `j` (`i < j`) with identical `(effect,
principal_role, action, resource_prefix, required_attribute)` are an exact
duplicate. Emits one `PolicyLintFinding` with `severity = Error` and
`rule_indices = [i, j]`.

### DR-2 Conflicting Allow/Deny pair (Error)

Two rules at indices `i` and `j` (`i < j`) sharing `(principal_role, action,
resource_prefix, required_attribute)` but with opposite effects. Emits one
`PolicyLintFinding` with `severity = Error` and `rule_indices = [i, j]`.

### DR-3 Shadowed/unreachable rule (Warning)

For a pair of same-effect rules `(earlier at index i, later at index j)`,
`j > i`, the later rule is shadowed if all of:
1. `later.resource_prefix.starts_with(&earlier.resource_prefix)` — earlier
   prefix subsumes later's resources.
2. `earlier.principal_role == later.principal_role`
3. `earlier.action == later.action`
4. Attribute is equal-or-weaker: `earlier.required_attribute` is `None`
   (matches anything), OR both are the same `Some(k, v)`.

Emits one `PolicyLintFinding` with `severity = Warning`, `rule_indices = [i, j]`,
reason identifies the shadow relationship.

Note: if `earlier.required_attribute` is `Some` and `later.required_attribute`
is `None`, the later rule is actually **broader** than the earlier one, so it is
NOT shadowed — no finding.

## Prefix Semantics Consistency

`DR-3` uses `str::starts_with` which is identical to the `PolicyRule::matches`
method already in `lib.rs`:
```rust
query.resource.starts_with(&self.resource_prefix)
```
This ensures lint shadow detection is consistent with runtime evaluation
semantics.

## Contracts

### No OpenAPI / proto3 surface

This is a pure domain library crate with no HTTP/gRPC adapter. The lint types
will be exposed via the REST adapter of any service that wraps this crate (e.g.,
an authz management service) when that adapter is built. This spec is
intentionally adapter-free.

### Serde contract

All new types derive `serde::Serialize` and `serde::Deserialize`.
`LintSeverity` serializes as `"Error"` and `"Warning"` (default PascalCase
from `serde` without `rename_all`).

Example JSON for a `PolicyLintReport`:
```json
{
  "findings": [
    {
      "severity": "Error",
      "rule_indices": [0, 2],
      "reason": "rules 0 and 2 conflict: Allow and Deny on identical (principal_role, action, resource_prefix, required_attribute)"
    }
  ]
}
```

## Mod Layout (flat-clean-arch per ADR-0509)

`src/lib.rs` sections (existing + new):

```
// crate-level doc + cfg_attr(test, allow(...))
// use declarations
// PolicyScope, PolicyEffect, PolicyRuleInput, PolicyRule, PolicyVersion, ...  [existing]
// impl PolicySet                                                               [existing]
// impl TryFrom<PolicyRuleInput> for PolicyRule                                [existing]
// impl PolicyRule::matches                                                     [existing]
// validate_policy_id, parse_semver                                            [existing]
// pub mod authz_engine { ... }                                                [existing]
// BackboneWriteOperation, CedarRuntime*                                       [existing]
// ── lint ───────────────────────────────────────────────────────────────────
// LintSeverity                                                                [new]
// PolicyLintFinding                                                           [new]
// PolicyLintReport + impl                                                     [new]
// pub fn lint_policy_version                                                  [new]
// ── lint helpers (private) ─────────────────────────────────────────────────
// fn detect_duplicate_and_conflict_findings                                   [new]
// fn detect_shadow_findings                                                   [new]
// fn attribute_is_equal_or_weaker                                             [new]
// #[cfg(test)] mod tests { ... }                                              [existing + new tests]
```

## Testing Strategy

All tests reside in `#[cfg(test)] mod tests` inside `lib.rs` (existing pattern).

| Test | Verifies |
|------|----------|
| `lint_report_serde_roundtrip` | `PolicyLintReport` serializes and deserializes via `serde_json`; `has_blocking` true iff any `Error` finding |
| `lint_detects_conflict_allow_deny_pair` | conflicting Allow+Deny on same tuple → one `Error` finding, both indices |
| `lint_detects_duplicate_rules` | two identical rules → one `Error` finding |
| `lint_clean_policy_is_clean` | policy with no conflicts/duplicates/shadows → `is_clean() == true` |
| `lint_detects_shadowed_rule` | later rule under broader same-effect prefix → one `Warning` finding |
| `lint_sibling_prefixes_not_shadowed` | two rules with sibling (non-prefix) resource prefixes → no shadow finding |
| `lint_broader_attr_not_shadowed` | later rule has `None` attr, earlier has `Some` → not shadowed |

Tests use `.unwrap()` / `.expect()` freely under the existing
`#[cfg_attr(test, allow(clippy::unwrap_used, ...))]` exemption.

## Dependencies

No new dependencies. `serde` and `serde_json` are already in `[dependencies]`.

## Boundaries

- Root `Cargo.toml` — **not touched**.
- Other crates — **not touched**.
- Existing public types and functions — **signatures unchanged**.
- `PolicySet::publish` behavior — **unchanged** (lint is pre-publish, not a gate
  inside publish).
