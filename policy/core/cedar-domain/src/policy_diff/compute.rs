//! The diff algorithm: identity-key matching, scope-change merging, and
//! impact classification between two policy versions.

use super::{ImpactReport, RuleDelta, RuleKey};
use crate::policy::{PolicyEffect, PolicyRuleInput, PolicyVersion};

// ── diff_policy_versions ──────────────────────────────────────────────────────

/// Compare two `PolicyVersion`s of the same `policy_id` and return an [`ImpactReport`].
///
/// The comparison is purely structural over the rule lists; it does not validate the
/// policy or interact with any runtime state.  The resulting deltas are deterministic
/// across repeated calls with identical inputs.
///
/// ## Matching strategy
///
/// Rules are first matched by their full identity key
/// `(principal_role, action, resource_prefix, required_attribute)`.  Rules with the
/// same key but different `effect` emit [`RuleDelta::EffectFlipped`].
///
/// For Allow rules that share `(principal_role, action)` but differ only in scope
/// fields (`resource_prefix` / `required_attribute`), the algorithm performs a
/// **scope-change merge**: rather than emitting a spurious `RuleAdded` + `RuleRemoved`
/// pair, it emits [`RuleDelta::BroadenedAllow`] or [`RuleDelta::NarrowedAllow`] and
/// suppresses the raw add/remove for the matched rules.
///
/// # Panics
///
/// Does not panic.
pub fn diff_policy_versions(prev: &PolicyVersion, next: &PolicyVersion) -> ImpactReport {
    use std::collections::{BTreeMap, BTreeSet};

    // Index prev and next rules by their full identity key.
    let prev_map: BTreeMap<RuleKey, &PolicyRuleInput> =
        prev.rules.iter().map(|r| (RuleKey::from(r), r)).collect();

    let next_map: BTreeMap<RuleKey, &PolicyRuleInput> =
        next.rules.iter().map(|r| (RuleKey::from(r), r)).collect();

    let mut deltas: Vec<RuleDelta> = Vec::new();

    // ── Pass 1: scope-change detection for Allow rules ────────────────────────
    //
    // For every prev Allow rule whose exact key is absent from next, try to find a
    // next Allow rule with the same (principal_role, action) but a different scope.
    // If found, emit BroadenedAllow or NarrowedAllow and record both sides as "merged"
    // so they are not double-counted in the add/remove pass.

    let mut prev_merged: BTreeSet<RuleKey> = BTreeSet::new();
    let mut next_merged: BTreeSet<RuleKey> = BTreeSet::new();

    // Collect as owned (key, rule) pairs to avoid double-reference issues.
    let prev_allows: Vec<(RuleKey, PolicyRuleInput)> = prev_map
        .iter()
        .filter(|(_, r)| r.effect == PolicyEffect::Allow)
        .map(|(k, r)| (k.clone(), (*r).clone()))
        .collect();

    let next_allows: Vec<(RuleKey, PolicyRuleInput)> = next_map
        .iter()
        .filter(|(_, r)| r.effect == PolicyEffect::Allow)
        .map(|(k, r)| (k.clone(), (*r).clone()))
        .collect();

    for (prev_key, prev_rule) in &prev_allows {
        // Only consider prev rules whose exact key is gone from next.
        if next_map.contains_key(prev_key) {
            continue;
        }
        if prev_merged.contains(prev_key) {
            continue;
        }

        for (next_key, next_rule) in &next_allows {
            // Skip if this next rule is already merged.
            if next_merged.contains(next_key) {
                continue;
            }

            // Match on same (principal_role, action) with different scope fields.
            if prev_rule.principal_role != next_rule.principal_role
                || prev_rule.action != next_rule.action
            {
                continue;
            }
            if prev_key == next_key {
                continue;
            }

            let prefix_broadened = next_rule.resource_prefix.len()
                < prev_rule.resource_prefix.len()
                && prev_rule
                    .resource_prefix
                    .starts_with(&next_rule.resource_prefix);

            let prefix_narrowed = next_rule.resource_prefix.len() > prev_rule.resource_prefix.len()
                && next_rule
                    .resource_prefix
                    .starts_with(&prev_rule.resource_prefix);

            let attr_dropped =
                prev_rule.required_attribute.is_some() && next_rule.required_attribute.is_none();
            let attr_added =
                prev_rule.required_attribute.is_none() && next_rule.required_attribute.is_some();

            if prefix_broadened || attr_dropped {
                deltas.push(RuleDelta::BroadenedAllow {
                    prev_rule: prev_rule.clone(),
                    next_rule: next_rule.clone(),
                });
                prev_merged.insert(prev_key.clone());
                next_merged.insert(next_key.clone());
                break;
            } else if prefix_narrowed || attr_added {
                deltas.push(RuleDelta::NarrowedAllow {
                    prev_rule: prev_rule.clone(),
                    next_rule: next_rule.clone(),
                });
                prev_merged.insert(prev_key.clone());
                next_merged.insert(next_key.clone());
                break;
            }
        }
    }

    // ── Pass 2: exact-key changes (effect flips, removes) ─────────────────────
    for (key, prev_rule) in &prev_map {
        if prev_merged.contains(key) {
            continue;
        }
        match next_map.get(key) {
            None => {
                // Key gone entirely.
                if prev_rule.effect == PolicyEffect::Deny {
                    deltas.push(RuleDelta::RemovedDeny((*prev_rule).clone()));
                } else {
                    deltas.push(RuleDelta::RuleRemoved((*prev_rule).clone()));
                }
            }
            Some(next_rule) => {
                if prev_rule.effect != next_rule.effect {
                    deltas.push(RuleDelta::EffectFlipped {
                        prev_rule: (*prev_rule).clone(),
                        next_rule: (*next_rule).clone(),
                    });
                }
                // Same key + same effect: unchanged, no delta.
            }
        }
    }

    // ── Pass 3: purely added rules ────────────────────────────────────────────
    for (key, next_rule) in &next_map {
        if next_merged.contains(key) {
            continue;
        }
        if !prev_map.contains_key(key) {
            if next_rule.effect == PolicyEffect::Deny {
                deltas.push(RuleDelta::AddedDeny((*next_rule).clone()));
            } else {
                deltas.push(RuleDelta::RuleAdded((*next_rule).clone()));
            }
        }
    }

    ImpactReport {
        prev_version: prev.version.clone(),
        next_version: next.version.clone(),
        deltas,
    }
}
