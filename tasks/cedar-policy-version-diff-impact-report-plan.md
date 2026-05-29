# Plan: cedar-policy-version-diff-impact-report

## Objective
Add a pure `policy_diff` submodule to `oya-policy-cedar-domain` that compares two
`PolicyVersion`s and produces a deterministic `ImpactReport`.

## Approach
Flat clean-arch mod inside `src/policy_diff.rs`, exposed via `pub mod policy_diff` in
`lib.rs`. No new deps beyond existing `serde`.

## Steps

1. Create `src/policy_diff.rs` with:
   - `RuleDelta` enum (RuleAdded, RuleRemoved, BroadenedAllow, NarrowedAllow, AddedDeny,
     RemovedDeny, EffectFlipped)
   - `ImpactReport` struct (deltas: Vec<RuleDelta>, prev_version: String, next_version: String)
   - `ImpactReport::has_widening()` method
   - `diff_policy_versions(prev, next) -> ImpactReport` function

2. Wire `pub mod policy_diff` in `lib.rs`

3. Write hermetic unit tests covering all acceptance criteria:
   - added-allow widens
   - removed-deny widens
   - narrowed resource prefix is not widening
   - effect-flip detection
   - identical versions yield empty report
   - serde round-trip on ImpactReport

4. Run `cargo check -p oya-policy-cedar-domain --all-targets` (green)
5. Run `cargo nextest run -p oya-policy-cedar-domain` (green)

## Diffing Algorithm

Rules are keyed by `(principal_role, action, resource_prefix, required_attribute)`.
Identical key sets → compare effect; new key in next → Added*; removed key → Removed*.
When same key exists in both versions with same effect → unchanged.
When same key exists but effect flips → EffectFlipped.
BroadenedAllow: an Allow rule in next whose resource_prefix is a prefix of the prev
rule's resource_prefix (next is shorter/looser) and/or dropped required_attribute.
NarrowedAllow: opposite — next resource_prefix is longer (more specific) or added
required_attribute.

## Status
- [x] Plan created
- [ ] Spec created
- [ ] Implementation written
- [ ] Tests pass
