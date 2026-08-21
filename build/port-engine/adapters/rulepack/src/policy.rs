//! Validating the pack's POLICY: what it defers, and how a trait binds its receiver.
//!
//! Both are DECISIONS rather than facts recovered from the source, so both must carry a reason.
//! A decision without one is a guess wearing a label, and the reason is what travels in the pack
//! digest and therefore into the receipt.

use std::collections::BTreeSet;

use crate::error::RulepackError;
use port_engine_api::{PointerConstruction, PointerDisposition};

use crate::rule::{ConstructionRule, DeferredKind, DispositionRule, LoadedRule, TraitReceiver};

/// Convert and check the ownership rules.
///
/// A reason is required for the same reason it is on a deferral: this is a decision with a cost
/// either way, and the reason is what makes the cost reviewable. An id must be unique, because a
/// decision cites it and two rules answering to one name make a citation ambiguous.
///
/// # Errors
/// [`RulepackError::Schema`] on a missing id, target or reason, and
/// [`RulepackError::Schema`] on a duplicate id.
pub(crate) fn validate_dispositions(
    rules: &[DispositionRule],
) -> Result<Vec<PointerDisposition>, RulepackError> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::with_capacity(rules.len());
    for rule in rules {
        if rule.id.is_empty() {
            return Err(RulepackError::Schema {
                field: "pointer_dispositions[].id",
            });
        }
        if rule.target.trim().is_empty() {
            return Err(RulepackError::Schema {
                field: "pointer_dispositions[].target",
            });
        }
        if rule.reason.trim().is_empty() {
            return Err(RulepackError::Schema {
                field: "pointer_dispositions[].reason",
            });
        }
        // A construction with no reason is the failure mode this pack exists to prevent: a
        // decision that emits something and cannot say why.
        if construction_reason(&rule.construction).trim().is_empty() {
            return Err(RulepackError::Schema {
                field: "pointer_dispositions[].construction.reason",
            });
        }
        if !seen.insert(rule.id.clone()) {
            return Err(RulepackError::Schema {
                field: "pointer_dispositions[].id(duplicate)",
            });
        }
        out.push(PointerDisposition {
            id: rule.id.clone(),
            when_mutated: rule.mutated,
            when_escapes: rule.escapes,
            when_effect_unknown: rule.effect_unknown,
            when_rebound: rule.rebound,
            target: rule.target.clone(),
            receiver: rule.receiver.clone(),
            reference_target: rule.reference_target.clone(),
            reference_owned: rule.reference_owned,
            reference_reason: rule.reference_reason.clone(),
            construction: construction_of(&rule.construction),
            reason: rule.reason.clone(),
        });
    }
    Ok(out)
}

/// Check the declared policy and return the deferred-kind set.
///
/// # Errors
/// [`RulepackError`] on a reasonless deferral, a contradictory one, or an unimplemented receiver
/// mode.
pub(crate) fn validate_policy(
    deferred_kinds: &[DeferredKind],
    trait_receiver: Option<&TraitReceiver>,
    loaded_rules: &[LoadedRule],
) -> Result<BTreeSet<String>, RulepackError> {
    let mut deferred_kind_set = BTreeSet::new();
    for deferred in deferred_kinds {
        if deferred.kind.is_empty() {
            return Err(RulepackError::Schema {
                field: "deferred_kinds[].kind",
            });
        }
        // A deferral without a reason is an omission wearing a label. The reason is what
        // makes it reviewable, and it is what travels in the digest.
        if deferred.reason.trim().is_empty() {
            return Err(RulepackError::Schema {
                field: "deferred_kinds[].reason",
            });
        }
        if let Some(rule) = loaded_rules
            .iter()
            .find(|rule| rule.captures.contains(&deferred.kind))
        {
            return Err(RulepackError::DeferredKindAlsoCaptured {
                kind: deferred.kind.clone(),
                rule: rule.id.0.clone(),
            });
        }
        deferred_kind_set.insert(deferred.kind.clone());
    }

    if let Some(receiver) = trait_receiver {
        if !matches!(receiver.mode.as_str(), "shared" | "exclusive" | "owned") {
            return Err(RulepackError::Schema {
                field: "trait_receiver.mode",
            });
        }
        // A decision without a reason is a guess wearing a label — and this one costs
        // something either way, so the cost is what the reason has to name.
        if receiver.reason.trim().is_empty() {
            return Err(RulepackError::Schema {
                field: "trait_receiver.reason",
            });
        }
    }

    if let Some(receiver) = trait_receiver {
        if !matches!(receiver.mode.as_str(), "shared" | "exclusive" | "owned") {
            return Err(RulepackError::Schema {
                field: "trait_receiver.mode",
            });
        }
        // A decision without a reason is a guess wearing a label — and this one costs something
        // either way, so the cost is what the reason has to name.
        if receiver.reason.trim().is_empty() {
            return Err(RulepackError::Schema {
                field: "trait_receiver.reason",
            });
        }
    }

    Ok(deferred_kind_set)
}

/// The neutral construction a pack rule declares.
fn construction_of(rule: &ConstructionRule) -> PointerConstruction {
    match rule {
        ConstructionRule::Borrow { mutable, reason } => PointerConstruction::Borrow {
            mutable: *mutable,
            reason: reason.clone(),
        },
        ConstructionRule::Wrap { paths, reason } => PointerConstruction::Wrap {
            paths: paths.clone(),
            reason: reason.clone(),
        },
    }
}

/// The reason a construction rule carries, whichever shape it is.
fn construction_reason(rule: &ConstructionRule) -> &str {
    match rule {
        ConstructionRule::Borrow { reason, .. } | ConstructionRule::Wrap { reason, .. } => reason,
    }
}
