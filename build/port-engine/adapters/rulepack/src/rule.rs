//! The loaded shapes: a rule, its selecting fixtures, and a declared deferral.

use std::collections::{BTreeMap, BTreeSet};

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
    /// Required `rebound` value; omitted means "do not care".
    #[serde(default)]
    pub rebound: Option<bool>,
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
    /// Target form for a REFERENCE-typed parameter — a map or a slice — with `{0}` for the
    /// resolved type itself. Absent means this disposition declines the reference position.
    #[serde(default)]
    pub reference_target: Option<String>,
    #[serde(default)]
    pub reference_owned: bool,
    /// Why a reference takes that form, and what it costs.
    #[serde(default)]
    pub reference_reason: Option<String>,
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

/// The source's PREDECLARED constants and their target spellings.
///
/// A source vocabulary the engine READS, like [`type_map`](crate::LoadedRulePack::type_map) — not a
/// decision the engine makes. `true` reaches the model as an identifier referring to a
/// universe-scope constant rather than as a literal, so nothing in the literal path answers for it.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstantMap {
    /// Source constant name → target expression.
    pub names: BTreeMap<String, String>,
    /// Why the table exists and what it deliberately omits.
    pub reason: String,
}

/// Callees whose value IS a length, so the target types it `usize`.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LengthFunctions {
    /// Callee identities.
    pub names: BTreeSet<String>,
    /// Callees that TAKE a length, rather than yielding one.
    #[serde(default)]
    pub argument_callees: BTreeSet<String>,
    /// Why a read in such a position is not evidence against a constant being a length.
    #[serde(default)]
    pub argument_callees_reason: String,
    /// Why a length is not the source's own integer in the target.
    pub reason: String,
}

/// Source TYPE names a doc comment may name, and the target's spelling for each.
///
/// Deliberately small, and the bound is the substance: every name in it is unambiguous in English,
/// so a word matching one is naming a type. An ordinary English word that is also a type name is
/// absent, because rewriting prose to fix a type name it never meant makes the prose worse.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProseTypeNames {
    /// Source type name → target spelling.
    pub names: BTreeMap<String, String>,
    /// Why this set and not the full type map.
    pub reason: String,
}

/// A FORM the pack has not decided, keyed by an id the engine names when it declines.
///
/// Distinct from [`DeferredKind`], which is per declaration KIND and mutually exclusive with a
/// rule that captures that kind. A form is a SHAPE within a kind: a package variable something
/// writes is translated by the same rule as one nothing writes, and only the first is undecided.
/// The front end cannot tell them apart without deciding, which is not its job — it records the
/// fact and the engine names the form.
///
/// The reason is REQUIRED and travels in the pack digest, for the same reason a deferral's does:
/// what the engine declines has to say what is missing, and here what is missing is a decision.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UndecidedForm {
    /// The id the engine quotes this reason by.
    pub id: String,
    /// Why the form is undecided, and where the analysis lives.
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
    #[serde(default)]
    pub source_language_words: Vec<String>,
    #[serde(default)]
    pub source_language_words_reason: String,
    #[serde(default)]
    pub passive_openings: Vec<String>,
    #[serde(default)]
    pub passive_openings_reason: String,
    /// Why the source's form is rewritten, and what is deliberately left alone.
    pub reason: String,
}

/// A derive a ported type earns, and the source type kinds that block it.
///
/// The reason is REQUIRED and travels in the pack digest: which derives a type earns is a claim
/// about what the source guarantees, and a derive nobody justified is a capability invented rather
/// than carried across.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeriveWireRule {
    /// The target trait derived.
    pub name: String,
    /// Source type kinds that make this derive unavailable.
    pub blocked_by: Vec<String>,
    /// What the source guarantees that makes it faithful, and what blocks it.
    pub reason: String,
}

/// An idiom rule, with the seed provenance the licensing policy requires.
///
/// `specs/k8s-port/licensing.json` rejects a rust-skills-derived rule without `seed_source`,
/// `seed_license` and `seed_commit`, so all three are REQUIRED here rather than optional: a rule
/// whose derivation cannot be re-checked is a rule nobody can audit.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdiomWireRule {
    /// Stable identity, so a decision can be cited.
    pub id: String,
    /// The source shape this recognises, from a closed vocabulary.
    pub shape: String,
    /// The target method or spelling it becomes.
    pub method: String,
    /// Why the two are equivalent, and why the target prefers its form.
    pub reason: String,
    /// Where the rule was derived from.
    pub seed_source: String,
    /// The seed's licence.
    pub seed_license: String,
    /// The seed's commit, so the derivation can be re-checked.
    pub seed_commit: String,
}
