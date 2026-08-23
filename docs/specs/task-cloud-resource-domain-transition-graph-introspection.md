# Spec: cloud-resource-domain-transition-graph-introspection

## Summary

Purely additive introspection surface for `ResourceState` in `cloud-resource-domain`.
Exposes the implicit transition graph as a queryable, const-fn API — no mutation, no new
dependencies, no async.

## New API (all on `impl ResourceState`)

### `as_str(self) -> &'static str`
Returns a canonical lowercase string label for each variant:
- `Pending`   → `"pending"`
- `Running`   → `"running"`
- `Stopped`   → `"stopped"`
- `Terminated`→ `"terminated"`
- `Error`     → `"error"`

### `parse(s: &str) -> Option<Self>`
Inverse of `as_str`. Returns `None` for any unrecognised input (fail-closed).

### `is_active(self) -> bool`
Returns `true` iff state is `Running`. Reflects "resource is actively consuming compute".

### `is_quiescent(self) -> bool`
Returns `true` iff state is `Stopped`. Reflects "resource is idle but not destroyed".

### `allowed_next(self) -> &'static [ResourceState]`
Returns the ordered slice of legal successor states reachable from `self` in a single
transition, as defined by the existing `state_transition_allowed` predicate.

Transition table (source → allowed targets):
| Source     | Allowed next (incl. self-loop) |
|------------|-------------------------------|
| Pending    | Pending, Running, Error, Terminated |
| Running    | Running, Stopped, Error, Terminated |
| Stopped    | Stopped, Running, Error, Terminated |
| Error      | Error, Terminated |
| Terminated | Terminated (self-loop only — terminal state) |

Note: `state_transition_allowed` permits `current == next` (self-loop) for all states.
`allowed_next` must include the self-loop to agree with that predicate.

### `can_transition_to(self, next: Self) -> bool`
Public wrapper over the private `state_transition_allowed` free function. Allows callers
to pre-check a transition without holding a `Resource` reference.

## Constraints

- All methods are `pub`; `as_str`, `parse`, `is_active`, `is_quiescent`, `allowed_next`
  are `const fn` where the Rust edition/language allows (edition 2021; const match is
  stable).
- `parse` cannot be `const fn` in stable Rust 2021 due to `str` comparison in const
  context; it is a regular `pub fn`.
- No new crate dependencies.
- `DataClass` / `Classified` posture unchanged — `ResourceState` carries no PII.
- `#![cfg_attr(test, allow(clippy::unwrap_used, ...))]` exemption already present.

## Tests

1. **round_trip**: `parse(s.as_str()) == Some(s)` for all 5 variants.
2. **parse_unknown**: `parse("bogus") == None`, `parse("") == None`.
3. **classifiers**: `is_active` true only for Running; `is_quiescent` true only for Stopped.
4. **terminated_has_no_outgoing_transitions** (except self): `allowed_next(Terminated)` contains only `Terminated`.
5. **transition_graph_agrees_with_predicate** (table-driven, 5×5 = 25 pairs): for every
   `(from, to)` pair, `can_transition_to(from, to)` == `allowed_next(from).contains(&to)`.
