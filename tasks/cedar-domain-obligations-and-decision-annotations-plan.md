# cedar-domain-obligations-and-decision-annotations — Plan

## Objective

Add Cedar-style policy obligations and advice annotations to `oya-policy-cedar-domain`.
The slice is entirely contained in a new `src/obligations.rs` mod; it adds annotation
key/value support to `PolicyRule` (carried as `Vec<PolicyAnnotation>`) and surfaces a
collected `Vec<PolicyAnnotation>` on `AuthorizationDecision` for Allow outcomes.
Deny wins unconditionally and suppresses all annotations (PDP/PEP safety invariant).

## Requirements Analysis

### Cedar obligations background
Cedar distinguishes **obligations** (must-execute side effects) from **advice**
(informational hints).  Both are key/value pairs attached to policy rules.  The PDP
collects annotations only from matching *Allow* rules; if any Deny fires the entire
annotation list is discarded — the PEP must not act on advice from a suppressed Allow.

### Acceptance criteria

1. `PolicyAnnotation` is a serde-capable value type with `key: String`, `value: String`,
   and an `AnnotationKind` enum (`Obligation` | `Advice`).
2. `PolicyRule` gains an optional `annotations: Vec<PolicyAnnotation>` field (default
   empty; backward-compat through `#[serde(default)]`).
3. `PolicyRuleInput` mirrors the same `annotations` field.
4. `AuthorizationDecision` gains `annotations: Vec<PolicyAnnotation>` — populated only
   when `allowed = true`; empty when Deny fires (forbid-wins invariant).
5. `PolicySet::authorize` collects annotations from the first matching Allow rule.
6. Full serde round-trip for all new types.
7. Forbid-wins test: an explicit Deny with an annotated Allow present yields
   `AuthorizationDecision { allowed: false, annotations: [] }`.
8. Allow-wins test: matching Allow rule annotations surface on the decision.
9. No-match test: decision has `allowed = false` and empty annotations.
10. Multiple annotation kinds (Obligation + Advice) on one rule all surface together.

### Edge cases
- Empty `annotations` vec on `PolicyRule` is valid (most rules have no annotations).
- `PolicyRuleInput::try_from` preserves annotations unchanged (no validation needed).
- `CedarEvaluationLogEntry` does NOT need updating — obligations are a PEP concern,
  not a log concern; annotations are on `AuthorizationDecision` for the caller.
- `authz_engine::AuthzDecision` is Wave-3 territory — do NOT touch it.

### k8s / cloud-native implications
Annotations are pure value types; they require no network, storage, or k8s API surface.
The downstream PEP (step-up, audit, redaction) consumes annotations out-of-band via the
`AuthorizationDecision` struct returned from `PolicySet::authorize`.

## Ordered Subtasks

- [x] 1. Write plan doc (this file).
- [x] 2. Write spec doc (`docs/specs/task-cedar-domain-obligations-and-decision-annotations.md`).
- [x] 3. Add `src/obligations.rs` with `AnnotationKind` + `PolicyAnnotation` value types + serde.
- [x] 4. Extend `PolicyRule`, `PolicyRuleInput`, `AuthorizationDecision` with `annotations` field.
- [x] 5. Update `PolicySet::authorize` to collect annotations from matching Allow rules.
- [x] 6. Update `TryFrom<PolicyRuleInput> for PolicyRule` to propagate `annotations`.
- [x] 7. Write RED tests (step 3 above causes compile failure before impls land).
- [x] 8. Confirm GREEN: `cargo check --all-targets` + `cargo nextest run`.
- [x] 9. Self-review (correctness, security, cloud-native).
- [x] 10. Simplify.
