//! The loaded shapes: a rule, its selecting fixtures, and a declared deferral.

use port_engine_api::RuleId;
use serde::Deserialize;

/// One selecting fixture bound to a rule (W0-B plan §5.3 minimum shape).
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectingFixture {
    /// Stable fixture identity.
    pub id: String,
    /// Unit the fixture exercises (deterministic ordering key with `id`).
    pub unit: String,
    /// Whether the rule is expected to select for `unit`.
    pub selects: bool,
}

/// One loaded rule record (identity + fixture gate + construction data).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedRule {
    /// Rule id.
    pub id: RuleId,
    /// Rule version string.
    pub version: String,
    /// Precondition id the transform evaluates.
    pub precondition: String,
    /// Construction id the transform applies into `RustIr`.
    pub construction: String,
    /// Declaration kinds this rule captures. Empty means the rule is unit-level.
    pub captures: Vec<String>,
    /// Declared precedence. Load-bearing: the loader refuses a pack whose precedence disagrees
    /// with declaration order, so this can never be a second, silently-ignored ordering.
    pub precedence: i64,
    /// Declared conflict policy. Only [`CONFLICT_REFUSE`] is implemented.
    pub conflict: String,
    /// Selection fixtures, including at least one validated positive fixture.
    pub selecting_fixtures: Vec<SelectingFixture>,
}

/// A declaration kind the pack knowingly does not translate, and why.
///
/// The reason is REQUIRED and travels in the pack bytes, therefore in the pack digest, therefore in
/// the receipt. That is the whole difference between a deferral and an omission: both emit nothing,
/// but one of them is a decision somebody made and can be found again.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeferredKind {
    /// The declaration kind left untranslated.
    pub kind: String,
    /// Why it is deferred, and where the analysis lives.
    pub reason: String,
}
