# Spec: cedar-domain-obligations-and-decision-annotations

## Objective

Extend `policy-cedar-domain` with Cedar-style policy **obligations** and **advice**
annotations.  Annotations are key/value pairs attached to `PolicyRule` records at
authoring time and collected onto `AuthorizationDecision` at evaluation time — but only
for Allow decisions.  A Deny win unconditionally clears the annotation list (PDP safety
invariant: a PEP must never act on obligations from a suppressed allow).

## Crate boundary

Sole crate: `crates/policy-cedar-domain`.  No new workspace members.  No external
runtime deps added.  All new types live in `src/obligations.rs` and are re-exported via
`lib.rs`.

## Mod layout (flat-clean-arch, ADR-0509)

```
crates/policy-cedar-domain/src/
  lib.rs           ← existing; gains `pub mod obligations;` + re-exports + field additions
  obligations.rs   ← NEW: AnnotationKind, PolicyAnnotation, annotation-collection pass
```

## Contracts

### Wire format (serde)

All new types derive `Serialize` / `Deserialize`.  Wire names follow existing codebase
conventions:

| Type | serde strategy |
|------|---------------|
| `AnnotationKind` | `#[serde(rename_all = "lowercase")]` → `"obligation"` \| `"advice"` |
| `PolicyAnnotation` | struct fields as-is (`key`, `value`, `kind`) |
| `PolicyRule.annotations` | `#[serde(default)]` — absent JSON key → empty `Vec` |
| `PolicyRuleInput.annotations` | `#[serde(default)]` — same |
| `AuthorizationDecision.annotations` | `#[serde(default)]` — empty for Deny |

No OpenAPI/proto surface changes needed: the obligations feature is an in-process
kernel concern surfaced to the adapter layer through the domain struct; the adapter
owns the wire mapping to HTTP/gRPC.

### `AuthorizationDecision` (updated)

```rust
pub struct AuthorizationDecision {
    pub allowed: bool,
    pub reason: String,
    pub matched_policy: Option<String>,
    pub annotations: Vec<PolicyAnnotation>,  // NEW — empty when Deny
}
```

### `PolicyAnnotation`

```rust
pub struct PolicyAnnotation {
    pub kind: AnnotationKind,
    pub key: String,
    pub value: String,
}
```

### `AnnotationKind`

```rust
pub enum AnnotationKind {
    Obligation,
    Advice,
}
```

## Annotation-collection algorithm

`PolicySet::authorize` already does two passes:

1. **Deny pass** — first matching Deny rule returns immediately with `allowed = false`
   and `annotations = vec![]`.
2. **Allow pass** — first matching Allow rule returns with `allowed = true` and
   `annotations = rule.annotations.clone()`.
3. **Default deny** — `annotations = vec![]`.

This matches Cedar's forbid-wins semantics: a Deny win short-circuits and no Allow
annotations are surfaced.

## Testing strategy

All tests live in `src/lib.rs` `#[cfg(test)]` (matching existing pattern).

| Test | Coverage |
|------|---------|
| `annotation_kinds_serde_roundtrip` | `AnnotationKind` + `PolicyAnnotation` serde |
| `allow_decision_surfaces_rule_annotations` | Allow path populates decision.annotations |
| `deny_wins_suppresses_annotations` | Deny path → empty annotations even when Allow annotated |
| `no_match_deny_has_empty_annotations` | Default-deny path |
| `multiple_annotation_kinds_on_one_rule` | Both Obligation + Advice on one rule |
| `authorization_decision_serde_with_annotations` | Full `AuthorizationDecision` round-trip |
| `policy_rule_input_annotations_propagate_through_try_from` | `TryFrom` preserves annotations |

## Observability / SLO

No new SLO surface.  The obligations module is a pure domain kernel; observability
emits are the adapter's responsibility.  Existing `CedarEvaluationLogEntry` is
unchanged.

## Security

- Deny-wins suppression of annotations is a hard invariant enforced in `authorize`.
  A PEP that ignores `allowed = false` to read annotations would be bypassing the PDP;
  this is documented in the struct-level doc comment.
- Annotation `key` / `value` are plain `String`; validation is the policy author's
  responsibility (no injection surface inside the kernel).
