//! Policy version diff — deterministic impact classification between two `PolicyVersion`s.
//!
//! `diff_policy_versions(prev, next)` compares every rule in `prev` against every rule
//! in `next` and classifies each change as one of the [`RuleDelta`] variants.
//! [`ImpactReport::has_widening`] signals whether the change broadens the allow surface
//! or removes a deny guard, enabling pre-publish security gates.
//!
//! Pure value transform: no I/O, no clock, no new dependencies beyond `serde`.

use serde::{Deserialize, Serialize};

use crate::{PolicyEffect, PolicyRuleInput, PolicyVersion};

// ── Key type ─────────────────────────────────────────────────────────────────

/// The identity tuple used to match rules across versions.
///
/// Two rules are considered to address the *same subject* when they share the
/// same `(principal_role, action, resource_prefix, required_attribute)`.  Differences
/// in `effect` or `annotations` are captured as a delta, not a new key.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct RuleKey {
    principal_role: String,
    action: String,
    resource_prefix: String,
    required_attribute: Option<(String, String)>,
}

impl From<&PolicyRuleInput> for RuleKey {
    fn from(r: &PolicyRuleInput) -> Self {
        Self {
            principal_role: r.principal_role.clone(),
            action: r.action.clone(),
            resource_prefix: r.resource_prefix.clone(),
            required_attribute: r.required_attribute.clone(),
        }
    }
}

// ── RuleDelta ─────────────────────────────────────────────────────────────────

/// Classification of a single rule-level change between two policy versions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RuleDelta {
    /// A rule present in `next` that had no matching key in `prev`.
    ///
    /// Includes both Allow and Deny additions; see [`ImpactReport::has_widening`] for
    /// the exact widening predicate (only Allow additions widen the surface).
    RuleAdded(PolicyRuleInput),

    /// A rule present in `prev` that has no matching key in `next`.
    RuleRemoved(PolicyRuleInput),

    /// An Allow→Allow change on the same key where `next` has a *looser* resource
    /// scope: the `next` resource_prefix is a strict prefix of `prev`'s (shorter =
    /// broader match) or a `required_attribute` guard was dropped.
    BroadenedAllow {
        prev_rule: PolicyRuleInput,
        next_rule: PolicyRuleInput,
    },

    /// An Allow→Allow change on the same key where `next` has a *tighter* resource
    /// scope: the `next` resource_prefix is longer than `prev`'s (more specific) or a
    /// `required_attribute` guard was added.
    NarrowedAllow {
        prev_rule: PolicyRuleInput,
        next_rule: PolicyRuleInput,
    },

    /// A Deny rule whose exact key was absent in `prev` and is present in `next`.
    ///
    /// This is a non-widening addition of a deny guard.
    AddedDeny(PolicyRuleInput),

    /// A Deny rule present in `prev` that is absent in `next`.
    ///
    /// Removing a deny guard is always widening.
    RemovedDeny(PolicyRuleInput),

    /// The `effect` of a rule changed between versions (Allow↔Deny).
    EffectFlipped {
        prev_rule: PolicyRuleInput,
        next_rule: PolicyRuleInput,
    },
}

// ── ImpactReport ──────────────────────────────────────────────────────────────

/// Aggregated diff result for a pair of `PolicyVersion`s.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ImpactReport {
    /// Version string of the `prev` input.
    pub prev_version: String,
    /// Version string of the `next` input.
    pub next_version: String,
    /// Ordered list of per-rule deltas (sorted deterministically).
    pub deltas: Vec<RuleDelta>,
}

impl ImpactReport {
    /// Returns `true` if any delta broadens the effective allow surface or removes a
    /// deny guard.
    ///
    /// Widening conditions:
    /// - [`RuleDelta::RuleAdded`] with `effect == Allow`
    /// - [`RuleDelta::RemovedDeny`]
    /// - [`RuleDelta::BroadenedAllow`]
    /// - [`RuleDelta::EffectFlipped`] where `next_rule.effect == Allow` (was Deny)
    pub fn has_widening(&self) -> bool {
        self.deltas.iter().any(|delta| match delta {
            RuleDelta::RuleAdded(r) => r.effect == PolicyEffect::Allow,
            RuleDelta::RemovedDeny(_) => true,
            RuleDelta::BroadenedAllow { .. } => true,
            RuleDelta::EffectFlipped { next_rule, .. } => next_rule.effect == PolicyEffect::Allow,
            _ => false,
        })
    }
}

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

// ── Serde support for PolicyRuleInput ─────────────────────────────────────────

impl Serialize for PolicyRuleInput {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("PolicyRuleInput", 5)?;
        s.serialize_field("effect", &self.effect)?;
        s.serialize_field("principal_role", &self.principal_role)?;
        s.serialize_field("action", &self.action)?;
        s.serialize_field("resource_prefix", &self.resource_prefix)?;
        s.serialize_field("required_attribute", &self.required_attribute)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for PolicyRuleInput {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct PolicyRuleInputVisitor;

        impl<'de> Visitor<'de> for PolicyRuleInputVisitor {
            type Value = PolicyRuleInput;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct PolicyRuleInput")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<PolicyRuleInput, V::Error> {
                let mut effect = None;
                let mut principal_role = None;
                let mut action = None;
                let mut resource_prefix = None;
                let mut required_attribute = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "effect" => {
                            effect = Some(map.next_value()?);
                        }
                        "principal_role" => {
                            principal_role = Some(map.next_value()?);
                        }
                        "action" => {
                            action = Some(map.next_value()?);
                        }
                        "resource_prefix" => {
                            resource_prefix = Some(map.next_value()?);
                        }
                        "required_attribute" => {
                            required_attribute = map.next_value()?;
                        }
                        _ => {
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                Ok(PolicyRuleInput {
                    effect: effect.ok_or_else(|| de::Error::missing_field("effect"))?,
                    principal_role: principal_role
                        .ok_or_else(|| de::Error::missing_field("principal_role"))?,
                    action: action.ok_or_else(|| de::Error::missing_field("action"))?,
                    resource_prefix: resource_prefix
                        .ok_or_else(|| de::Error::missing_field("resource_prefix"))?,
                    required_attribute,
                    annotations: Vec::new(),
                })
            }
        }

        const FIELDS: &[&str] = &[
            "effect",
            "principal_role",
            "action",
            "resource_prefix",
            "required_attribute",
        ];
        deserializer.deserialize_struct("PolicyRuleInput", FIELDS, PolicyRuleInputVisitor)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PolicyEffect, PolicyRuleInput, PolicyScope, PolicyVersion};

    fn allow_rule(role: &str, action: &str, prefix: &str) -> PolicyRuleInput {
        PolicyRuleInput {
            effect: PolicyEffect::Allow,
            principal_role: role.to_string(),
            action: action.to_string(),
            resource_prefix: prefix.to_string(),
            required_attribute: None,
            annotations: Vec::new(),
        }
    }

    fn deny_rule(role: &str, action: &str, prefix: &str) -> PolicyRuleInput {
        PolicyRuleInput {
            effect: PolicyEffect::Deny,
            principal_role: role.to_string(),
            action: action.to_string(),
            resource_prefix: prefix.to_string(),
            required_attribute: None,
            annotations: Vec::new(),
        }
    }

    fn allow_rule_attr(
        role: &str,
        action: &str,
        prefix: &str,
        attr: Option<(&str, &str)>,
    ) -> PolicyRuleInput {
        PolicyRuleInput {
            effect: PolicyEffect::Allow,
            principal_role: role.to_string(),
            action: action.to_string(),
            resource_prefix: prefix.to_string(),
            required_attribute: attr.map(|(k, v)| (k.to_string(), v.to_string())),
            annotations: Vec::new(),
        }
    }

    fn pv(version: &str, rules: Vec<PolicyRuleInput>) -> PolicyVersion {
        PolicyVersion {
            policy_id: "pol_test".to_string(),
            version: version.to_string(),
            scope: PolicyScope::Global,
            supersedes: None,
            rules,
        }
    }

    // ── acceptance: added-allow widens ────────────────────────────────────────

    /// ACCEPTANCE: adding an Allow rule to an empty prev widens the surface.
    #[test]
    fn added_allow_widens() {
        let prev = pv("1.0.0", vec![]);
        let next = pv("1.1.0", vec![allow_rule("editor", "doc.write", "docs:")]);

        let report = diff_policy_versions(&prev, &next);

        assert_eq!(report.prev_version, "1.0.0");
        assert_eq!(report.next_version, "1.1.0");
        assert!(
            report.has_widening(),
            "adding an Allow rule must widen; deltas={:?}",
            report.deltas
        );
        assert!(
            report
                .deltas
                .iter()
                .any(|d| matches!(d, RuleDelta::RuleAdded(r) if r.effect == PolicyEffect::Allow)),
            "expected RuleAdded(Allow) delta"
        );
    }

    // ── acceptance: removed-deny widens ──────────────────────────────────────

    /// ACCEPTANCE: removing a Deny rule widens the effective allow surface.
    #[test]
    fn removed_deny_widens() {
        let deny = deny_rule("admin", "account.delete", "acct:");
        let prev = pv("1.0.0", vec![deny.clone()]);
        let next = pv("1.1.0", vec![]);

        let report = diff_policy_versions(&prev, &next);

        assert!(
            report.has_widening(),
            "removing a Deny rule must widen; deltas={:?}",
            report.deltas
        );
        assert!(
            report
                .deltas
                .iter()
                .any(|d| matches!(d, RuleDelta::RemovedDeny(_))),
            "expected RemovedDeny delta"
        );
    }

    // ── acceptance: narrowed resource prefix is NOT widening ──────────────────

    /// ACCEPTANCE: making an Allow rule more specific (longer prefix) is narrowing,
    /// not widening.
    #[test]
    fn narrowed_resource_prefix_not_widening() {
        // prev: Allow on "docs:" (broader)
        // next: Allow on "docs:private:" (narrower — longer prefix)
        let prev = pv("1.0.0", vec![allow_rule("editor", "doc.write", "docs:")]);
        let next = pv(
            "1.1.0",
            vec![allow_rule("editor", "doc.write", "docs:private:")],
        );

        let report = diff_policy_versions(&prev, &next);

        assert!(
            !report.has_widening(),
            "narrowing the resource_prefix must NOT widen; deltas={:?}",
            report.deltas
        );
        assert!(
            report
                .deltas
                .iter()
                .any(|d| matches!(d, RuleDelta::NarrowedAllow { .. })),
            "expected NarrowedAllow delta; deltas={:?}",
            report.deltas
        );
    }

    // ── acceptance: effect flip Deny→Allow widens ─────────────────────────────

    /// ACCEPTANCE: flipping an existing rule from Deny to Allow widens the surface.
    #[test]
    fn effect_flip_deny_to_allow_widens() {
        let prev_rule = deny_rule("ops", "cluster.drain", "k8s:cluster:");
        let next_rule = allow_rule("ops", "cluster.drain", "k8s:cluster:");

        let prev = pv("1.0.0", vec![prev_rule]);
        let next = pv("1.1.0", vec![next_rule]);

        let report = diff_policy_versions(&prev, &next);

        assert!(
            report.has_widening(),
            "flipping Deny→Allow must widen; deltas={:?}",
            report.deltas
        );
        assert!(
            report.deltas.iter().any(|d| matches!(
                d,
                RuleDelta::EffectFlipped { next_rule, .. } if next_rule.effect == PolicyEffect::Allow
            )),
            "expected EffectFlipped delta with next Allow; deltas={:?}",
            report.deltas
        );
    }

    // ── acceptance: identical versions produce empty report ───────────────────

    /// ACCEPTANCE: diffing a version against itself yields no deltas.
    #[test]
    fn identical_versions_empty_report() {
        let rules = vec![
            allow_rule("reader", "doc.read", "docs:"),
            deny_rule("guest", "admin.write", "admin:"),
        ];
        let v1 = pv("1.0.0", rules.clone());
        let v2 = pv("1.0.0", rules);

        let report = diff_policy_versions(&v1, &v2);

        assert!(
            report.deltas.is_empty(),
            "identical versions must yield empty deltas; got {:?}",
            report.deltas
        );
        assert!(!report.has_widening());
    }

    // ── acceptance: serde round-trip on ImpactReport ──────────────────────────

    /// ACCEPTANCE: ImpactReport (including RuleDelta variants) round-trips through
    /// serde_json without error.
    #[test]
    fn impact_report_serde_round_trip() {
        let prev = pv("1.0.0", vec![]);
        let next = pv(
            "1.1.0",
            vec![
                allow_rule("editor", "doc.write", "docs:"),
                deny_rule("guest", "admin.delete", "admin:"),
            ],
        );

        let report = diff_policy_versions(&prev, &next);
        assert!(report.has_widening());

        let json = serde_json::to_string(&report).expect("ImpactReport serializes");
        let roundtrip: ImpactReport =
            serde_json::from_str(&json).expect("ImpactReport deserializes");

        assert_eq!(report.prev_version, roundtrip.prev_version);
        assert_eq!(report.next_version, roundtrip.next_version);
        assert_eq!(report.deltas.len(), roundtrip.deltas.len());
        assert_eq!(roundtrip.has_widening(), report.has_widening());
    }

    // ── additional: broadened allow widens ───────────────────────────────────

    /// Shortening the resource_prefix (broader match) classifies as BroadenedAllow.
    #[test]
    fn broadened_resource_prefix_widens() {
        // prev: Allow on "docs:private:" (narrower)
        // next: Allow on "docs:" (broader — shorter prefix)
        let prev = pv(
            "1.0.0",
            vec![allow_rule("editor", "doc.write", "docs:private:")],
        );
        let next = pv("1.1.0", vec![allow_rule("editor", "doc.write", "docs:")]);

        let report = diff_policy_versions(&prev, &next);

        assert!(
            report.has_widening(),
            "shortening the resource_prefix must widen; deltas={:?}",
            report.deltas
        );
        assert!(
            report
                .deltas
                .iter()
                .any(|d| matches!(d, RuleDelta::BroadenedAllow { .. })),
            "expected BroadenedAllow delta; deltas={:?}",
            report.deltas
        );
    }

    // ── additional: dropping required_attribute widens ────────────────────────

    /// Removing a required_attribute guard widens the allow surface.
    #[test]
    fn dropping_required_attribute_widens() {
        let prev = pv(
            "1.0.0",
            vec![allow_rule_attr(
                "subscriber",
                "content.read",
                "content:",
                Some(("tier", "premium")),
            )],
        );
        let next = pv(
            "1.1.0",
            vec![allow_rule_attr(
                "subscriber",
                "content.read",
                "content:",
                None,
            )],
        );

        let report = diff_policy_versions(&prev, &next);

        assert!(
            report.has_widening(),
            "dropping required_attribute must widen; deltas={:?}",
            report.deltas
        );
        assert!(
            report
                .deltas
                .iter()
                .any(|d| matches!(d, RuleDelta::BroadenedAllow { .. })),
            "expected BroadenedAllow delta; deltas={:?}",
            report.deltas
        );
    }

    // ── additional: added deny is not widening ────────────────────────────────

    /// Adding a Deny rule is not widening (it restricts the surface).
    #[test]
    fn added_deny_is_not_widening() {
        let prev = pv("1.0.0", vec![]);
        let next = pv("1.1.0", vec![deny_rule("guest", "admin.write", "admin:")]);

        let report = diff_policy_versions(&prev, &next);

        assert!(
            !report.has_widening(),
            "adding a Deny rule must NOT widen; deltas={:?}",
            report.deltas
        );
        assert!(
            report
                .deltas
                .iter()
                .any(|d| matches!(d, RuleDelta::AddedDeny(_))),
            "expected AddedDeny delta"
        );
    }

    // ── additional: effect flip Allow→Deny is not widening ────────────────────

    /// Flipping Allow→Deny narrows the surface; must not trigger has_widening.
    #[test]
    fn effect_flip_allow_to_deny_not_widening() {
        let prev_rule = allow_rule("ops", "cluster.drain", "k8s:cluster:");
        let next_rule = deny_rule("ops", "cluster.drain", "k8s:cluster:");

        let prev = pv("1.0.0", vec![prev_rule]);
        let next = pv("1.1.0", vec![next_rule]);

        let report = diff_policy_versions(&prev, &next);

        assert!(
            !report.has_widening(),
            "flipping Allow→Deny must NOT widen; deltas={:?}",
            report.deltas
        );
        assert!(
            report.deltas.iter().any(|d| matches!(
                d,
                RuleDelta::EffectFlipped { next_rule, .. } if next_rule.effect == PolicyEffect::Deny
            )),
            "expected EffectFlipped delta with next Deny; deltas={:?}",
            report.deltas
        );
    }
}
