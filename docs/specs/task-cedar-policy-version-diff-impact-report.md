# Spec: cedar-policy-version-diff-impact-report

**Crate**: `policy-cedar-domain`  
**Module**: `policy_diff`  
**Priority**: high | **Effort**: M | **Lane**: foundation

## Purpose

Fill the gap between `lint_policy_version` (authoring-time static analysis) and
`CedarRuntimeEvaluator` (runtime evaluation) by providing a deterministic diff over two
`PolicyVersion` values that classifies each rule delta's security impact.

## Public Surface

```rust
pub mod policy_diff {
    /// Classification of a single rule-level change between two policy versions.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub enum RuleDelta { ... }

    /// Aggregated diff result for a pair of PolicyVersions.
    #[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
    pub struct ImpactReport {
        pub prev_version: String,
        pub next_version: String,
        pub deltas: Vec<RuleDelta>,
    }

    impl ImpactReport {
        /// True if any delta broadens the allow surface or removes a deny.
        pub fn has_widening(&self) -> bool;
    }

    /// Compare two PolicyVersions of the same policy_id.
    pub fn diff_policy_versions(prev: &PolicyVersion, next: &PolicyVersion) -> ImpactReport;
}
```

## RuleDelta Variants

| Variant | Condition | Widening? |
|---|---|---|
| `RuleAdded(PolicyRuleInput)` | Rule key absent in prev, present in next | If Allow: yes |
| `RuleRemoved(PolicyRuleInput)` | Rule key present in prev, absent in next | If Deny: yes |
| `BroadenedAllow { prev_rule, next_rule }` | Allow→Allow, same key but next resource_prefix is prefix of prev's (shorter) OR dropped required_attribute | yes |
| `NarrowedAllow { prev_rule, next_rule }` | Allow→Allow, same key but next resource_prefix is longer OR added required_attribute | no |
| `AddedDeny(PolicyRuleInput)` | Rule key absent in prev, present in next with Deny effect | no |
| `RemovedDeny(PolicyRuleInput)` | Rule key present in prev with Deny, absent in next | yes |
| `EffectFlipped { prev_rule, next_rule }` | Same `(role, action, resource_prefix, required_attribute)` key, effect changed | Widening if Allow→Deny is narrowing; Deny→Allow is widening |

Note: `RuleAdded` covers non-Deny additions. `AddedDeny` is a special case of RuleAdded
for Deny-effect rules to enable separate widening logic. `RemovedDeny` is a special case
of `RuleRemoved`.

## Widening Logic

`has_widening()` returns true if any of:
- `RuleAdded(r)` where `r.effect == Allow`
- `RemovedDeny(_)`
- `BroadenedAllow { .. }`
- `EffectFlipped { next_rule, .. }` where `next_rule.effect == Allow` (was Deny)

## Acceptance Tests

1. `added_allow_widens` — prev has no rules, next adds an Allow rule → `has_widening() == true`
2. `removed_deny_widens` — prev has a Deny rule, next removes it → `has_widening() == true`
3. `narrowed_resource_prefix_not_widening` — Allow rule, next has longer resource_prefix →
   `NarrowedAllow` delta, `has_widening() == false`
4. `effect_flip_deny_to_allow_widens` — same key, prev Deny, next Allow → `EffectFlipped`,
   `has_widening() == true`
5. `identical_versions_empty_report` — same rules → empty deltas, `has_widening() == false`
6. `impact_report_serde_round_trip` — serialize/deserialize `ImpactReport` via serde_json

## Constraints

- Pure: no I/O, no clock, no new workspace members, no new deps beyond existing `serde`
- Does not call `lint_policy_version`
- Does not modify `authz_engine`, `obligations`, or `CedarRuntimeEvaluator`
