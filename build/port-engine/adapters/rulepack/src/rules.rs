//! One rule DOCUMENT into one loaded rule, and every reason a document is rejected.
//!
//! Split from `load.rs` because it is the one part of loading with a single input and a single
//! output: the rule list in, the rule list out, and nothing else of the pack in view. Every check
//! here rejects rather than repairs, and the precedence one is the load-bearing one — declaration
//! order IS the transform order, so a `precedence` that disagrees with it states an order nothing
//! obeys and a reviewer reading the numbers would be reading a fiction.

use std::collections::BTreeSet;

use port_engine_api::RuleId;

use crate::CONFLICT_REFUSE;
use crate::error::RulepackError;
use crate::rule::LoadedRule;
use crate::wire::RuleDocument;

/// Every declared rule, in declaration order, with its identity carried alongside.
///
/// # Errors
/// [`RulepackError`] on an empty list, a reasonless field, an unimplemented semantic, a conflict
/// policy nothing implements, or a precedence that disagrees with declaration order.
pub(crate) fn loaded(
    documents: Vec<RuleDocument>,
) -> Result<(Vec<RuleId>, Vec<LoadedRule>), RulepackError> {
    if documents.is_empty() {
        return Err(RulepackError::Schema { field: "rules" });
    }
    let mut rules = Vec::with_capacity(documents.len());
    let mut loaded_rules = Vec::with_capacity(documents.len());
    let mut seen = BTreeSet::new();
    let mut previous_precedence: Option<i64> = None;
    for rule in documents {
        if rule.id.is_empty() {
            return Err(RulepackError::Schema {
                field: "rules[].id",
            });
        }
        if rule.version.is_empty() {
            return Err(RulepackError::Schema {
                field: "rules[].version",
            });
        }
        if rule.precondition.is_empty() {
            return Err(RulepackError::Schema {
                field: "rules[].precondition",
            });
        }
        if rule.construction.is_empty() {
            return Err(RulepackError::Schema {
                field: "rules[].construction",
            });
        }
        if rule.selecting_fixtures.is_empty() {
            return Err(RulepackError::MissingSelectingFixture { rule: rule.id });
        }
        for fixture in &rule.selecting_fixtures {
            if fixture.id.is_empty() {
                return Err(RulepackError::Schema {
                    field: "rules[].selecting_fixtures[].id",
                });
            }
            if fixture.unit.is_empty() {
                return Err(RulepackError::Schema {
                    field: "rules[].selecting_fixtures[].unit",
                });
            }
        }
        if !seen.insert(rule.id.clone()) {
            return Err(RulepackError::Schema {
                field: "rules(duplicate)",
            });
        }
        // Every field the wire shape carries must either drive behaviour or be refused.
        // These two carry no implementation, so a pack declaring them is told so rather than
        // loading green and receiving nothing.
        if !rule.required_diagnostics.is_empty() {
            return Err(RulepackError::UnimplementedSemantics {
                rule: rule.id,
                field: "required_diagnostics",
            });
        }
        if !rule.proof_obligations.is_empty() {
            return Err(RulepackError::UnimplementedSemantics {
                rule: rule.id,
                field: "proof_obligations",
            });
        }
        if rule.conflict != CONFLICT_REFUSE {
            return Err(RulepackError::UnknownConflictPolicy {
                rule: rule.id,
                policy: rule.conflict,
            });
        }
        // Declaration order IS the transform order — `port_engine_kernel::plan` refuses a
        // unit whose rules arrive out of declared position. `precedence` therefore has to
        // agree with it or the pack states an order that nothing obeys, and a reviewer
        // reading the precedences would be reading a fiction.
        if let Some(previous) = previous_precedence.filter(|p| rule.precedence <= *p) {
            return Err(RulepackError::PrecedenceDisagreesWithOrder {
                rule: rule.id,
                precedence: rule.precedence,
                previous,
            });
        }
        previous_precedence = Some(rule.precedence);

        for capture in &rule.captures {
            if capture.is_empty() {
                return Err(RulepackError::Schema {
                    field: "rules[].captures[]",
                });
            }
        }

        let id = RuleId(rule.id);
        loaded_rules.push(LoadedRule {
            id: id.clone(),
            version: rule.version,
            precondition: rule.precondition,
            construction: rule.construction,
            captures: rule.captures,
            precedence: rule.precedence,
            conflict: rule.conflict,
            selecting_fixtures: rule.selecting_fixtures,
        });
        rules.push(id);
    }
    Ok((rules, loaded_rules))
}
