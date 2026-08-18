//! Validating the pack's POLICY: what it defers, and how a trait binds its receiver.
//!
//! Both are DECISIONS rather than facts recovered from the source, so both must carry a reason.
//! A decision without one is a guess wearing a label, and the reason is what travels in the pack
//! digest and therefore into the receipt.

use std::collections::BTreeSet;

use crate::error::RulepackError;
use crate::rule::{DeferredKind, LoadedRule, TraitReceiver};

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
