//! The deterministic, ordered transform to execute. Data only: holding it does not run it.

use crate::identity::{LanguagePair, RuleId, UnitId};

/// One step of a [`TransformPlan`]: apply `rule` to `unit`.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PlanStep {
    /// The unit the rule applies to.
    pub unit: UnitId, // data_class: INTERNAL_ONLY
    /// The rule to apply.
    pub rule: RuleId, // data_class: INTERNAL_ONLY
}

/// The deterministic, ordered transform to execute. Data only: holding it does not run it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransformPlan {
    /// The pair this plan translates.
    pub pair: LanguagePair, // data_class: INTERNAL_ONLY
    /// The steps, in execution order (model unit order, then pack rule order).
    pub steps: Vec<PlanStep>, // data_class: INTERNAL_ONLY
}
