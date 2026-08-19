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

/// Wire shape of one ownership rule.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispositionRule {
    /// Stable identity, so a decision can be cited.
    pub id: String,
    /// Required `mutated` value; omitted means "do not care".
    #[serde(default)]
    pub mutated: Option<bool>,
    /// Required `escapes` value; omitted means "do not care".
    #[serde(default)]
    pub escapes: Option<bool>,
    /// Required `effect_unknown` value; omitted means "do not care".
    #[serde(default)]
    pub effect_unknown: Option<bool>,
    /// Target type template for a parameter, with `{0}` for the pointee.
    pub target: String,
    /// Target form for a receiver. Absent means this disposition declines the receiver position,
    /// which is a refusal rather than a fallback.
    #[serde(default)]
    pub receiver: Option<String>,
    /// How an ARGUMENT reaches a parameter holding this form.
    ///
    /// The same decision seen from the other end: the target says what `*T` becomes in a
    /// parameter, and this says what `&x` becomes when handed to one.
    pub construction: ConstructionRule,
    /// Why these facts deserve this form, and what it costs.
    pub reason: String,
}

/// How an argument reaches a parameter, as STRUCTURE.
///
/// Tagged by `kind`, because the two shapes take different data: a borrow needs only its
/// exclusivity, and a wrap needs the paths it passes the value through.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ConstructionRule {
    /// The argument lends.
    Borrow {
        /// Whether the borrow is exclusive.
        mutable: bool,
        /// Why, and what it costs the caller.
        reason: String,
    },
    /// The argument is wrapped by each path in turn, innermost first.
    Wrap {
        /// The wrapping paths, innermost first.
        paths: Vec<String>,
        /// Why, and what it costs the caller.
        reason: String,
    },
}

/// How a trait method binds its receiver, and why.
///
/// The reason is REQUIRED and travels in the pack digest, because this is a decision rather than
/// a fact: the source interface does not carry the answer, so somebody chose it and the record of
/// who and why is the only thing that makes the choice reviewable.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraitReceiver {
    /// `shared`, `exclusive`, or `owned`.
    pub mode: String,
    /// Why this mode, and what it costs.
    pub reason: String,
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

/// How the pack answers for a call the target has no name of its own for.
///
/// The reason is REQUIRED and travels in the pack digest. An earlier form of this table was a bare
/// spelling map, so `errors.New` becoming a boxed trait object and `len` gaining a cast were
/// decisions with nobody's name on them.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionMappingRule {
    /// Target template, with `{0}`, `{1}` for the arguments.
    pub form: String,
    /// The shape the argument must have, when the mapping is CONDITIONAL.
    #[serde(default)]
    pub requires_argument: Option<String>,
    /// Why this call becomes this form, and what it costs.
    pub reason: String,
}

/// How the source's integer arithmetic must be spelled in the target.
///
/// The reason is REQUIRED and travels in the pack digest, because this is a decision with a cost
/// on both sides: the plain operator is shorter and means something different on overflow, and the
/// wrapping form is exact and spelled that way everywhere.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntegerArithmeticRule {
    /// Source type names whose arithmetic this governs.
    pub types: Vec<String>,
    /// Source operator to the target method carrying the same rule.
    pub operators: std::collections::BTreeMap<String, String>,
    /// Why this spelling, and what it costs.
    pub reason: String,
}

/// How the source's documentation convention differs from the target's.
///
/// The reason is REQUIRED and travels in the pack digest, because rewriting prose somebody wrote
/// is not something to do silently — and the BOUND on what gets rewritten is the substance of it.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocConventionRule {
    /// Whether a leading repetition of the item's own name is dropped.
    pub strip_leading_name: bool,
    /// Words dropped along with the name, so the remainder still reads.
    pub copulas: Vec<String>,
    /// Why the source's form is rewritten, and what is deliberately left alone.
    pub reason: String,
}
