---
doc_class: Owner-SPEC
owner: policy
status: Active
date: 2026-08-29
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
  - policy/ADR.md
  - policy/PRD.md
---

# Policy decision contract

<trust_boundary>

## Trust zones

- The authorization request, its context attributes, and any caller-supplied
  entity slice are untrusted data. They are validated structurally before
  evaluation and never executed.
- Relationship tuples are tenant-scoped data. A tuple read for one tenant can
  never satisfy a query issued under another.
- Consistency tokens are opaque to callers. Only the tuple store mints them
  and only the tuple store orders them.
- Policy bundle bytes are untrusted until parsed and strict-validated against
  the schema. A bundle that fails to compile does not load, and the
  previously serving bundle keeps serving.

</trust_boundary>

<surfaces>

## Landed

`core/cedar-domain` — pure value types, no I/O:

- Versioned policy records (`PolicyVersion`, `PolicyRule`, `PolicySet`) with
  semver ordering and supersession-chain integrity.
- Evaluation with deny-by-default and explicit-deny-wins over role and
  attribute predicates.
- Authoring-time lint (`lint_policy_version`) classifying conflicts,
  duplicates, and shadowing by blocking severity.
- Version diffing (`diff_policy_versions`) classifying each change and
  reporting whether it widens the allow surface.
- Obligation and advice annotations, surfaced only on Allow decisions.
- ReBAC vocabulary: tenant-scoped `RebacTuple` rendered
  `object#relation@subject`, `RebacSubjectRef` covering concrete objects and
  usersets, `Zookie` and `SnapshotToken` consistency tokens, the
  `RebacTupleStore` port, and the `UsersetRewrite` tree.

## Not landed

- No adapter implements `RebacTupleStore`.
- No namespace configuration binds `(object_type, relation)` to a
  `UsersetRewrite`, and no evaluator walks one.
- No expansion feeds the policy-decision path; the engine still evaluates
  against a caller-assembled entity slice.
- The Cedar engine adapter, bundle store, publish API, and runnable decision
  service remain under `iam/`.

</surfaces>

<invariants>

- A validation failure, an engine fault, a stale consistency pin, or an
  unreachable store yields a refusal that enforcement points treat as deny.
- Annotations are empty on every Deny, explicit or by default.
- A policy identifier is `pol_`-prefixed and non-empty beyond the prefix; a
  version is strict three-part semver with no leading zeroes.
- A tuple's canonical rendering round-trips within a tenant: parsing the
  rendered form of any valid tuple, with its tenant supplied again, yields
  that tuple. The rendering carries `object#relation@subject` and not the
  tenant, so the scope is a parameter of `parse`, never recovered from the
  string.
- The `union` and `intersection` constructors reject an empty child list,
  and `validate()` rejects one anywhere in a tree. This is a constructor
  convention, not a structural guarantee: `UsersetRewrite` has public
  variants and derives `Deserialize` plainly, so a tree that arrived over
  the wire must be `validate()`d before it is trusted.

</invariants>
