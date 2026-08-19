//! The source's failure convention.
//!
//! Every real package in the source language returns failure as a trailing result rather than as a
//! distinct type, so an engine that cannot see that convention cannot port anything real. What it
//! needs to see is small: which source type carries the convention, what the target should carry
//! instead, and how the source spells the absent value it compares against.
//!
//! Deliberately NOT here: `Result`, `Ok`, `Err`. Those are target spellings, and the face that
//! renders the target owns them. Putting them in the pack would make a second language pair
//! re-declare the target's own vocabulary, which is the thing the neutral seam exists to prevent.

/// An IDIOM rule: a spelling the target prefers for something the source says another way.
///
/// Distinct from every other rule here because it changes NOTHING about the program — an idiom
/// that alters meaning is not an idiom, it is a bug. What it changes is whether the emitted code
/// reads as written or as translated, which is the bar this engine is held to.
///
/// Seed provenance is REQUIRED where a rule is derived from a seed corpus, per
/// `specs/k8s-port/licensing.json`: "Reject a rust-skills-derived rule without seed_source,
/// seed_license, and seed_commit."
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdiomRule {
    /// Stable identity, so a decision can be cited.
    pub id: String, // data_class: INTERNAL_ONLY
    /// The source shape this recognises. A CLOSED vocabulary: an unrecognised shape refuses
    /// rather than reading as "no idiom", because a rule nobody applies is a rule that is not
    /// there.
    pub shape: String, // data_class: INTERNAL_ONLY
    /// The target method or spelling it becomes.
    pub method: String, // data_class: INTERNAL_ONLY
    /// Why the two are equivalent, and why the target prefers its form.
    pub reason: String, // data_class: INTERNAL_ONLY
    /// Where the rule was derived from.
    pub seed_source: String, // data_class: INTERNAL_ONLY
    /// The seed's licence.
    pub seed_license: String, // data_class: INTERNAL_ONLY
    /// The seed's commit, so the derivation can be re-checked.
    pub seed_commit: String, // data_class: INTERNAL_ONLY
}

/// A derive a ported type EARNS, and the source type kinds that block it.
///
/// Which derives a type earns is a decision about what the source guarantees, so it carries a
/// reason. The BLOCKING set is what makes deriving safe rather than hopeful: the engine emits
/// every corpus struct with the same derives, so a field naming another emitted struct is
/// satisfied by construction, and only kinds the engine emits no type for can block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeriveRule {
    /// The target trait derived.
    pub name: String, // data_class: INTERNAL_ONLY
    /// Source type kinds that make this derive unavailable for the type carrying them.
    pub blocked_by: std::collections::BTreeSet<String>, // data_class: INTERNAL_ONLY
    /// What the source guarantees that makes this derive faithful, and what blocks it.
    pub reason: String, // data_class: INTERNAL_ONLY
}

/// How the source's documentation convention differs from the target's.
///
/// A DECISION about prose, so it carries a reason: rewriting what an author wrote is not something
/// to do silently, and the bound on what is rewritten is the substance of the rule.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct DocConvention {
    /// Whether a leading repetition of the item's own name is dropped.
    pub strip_leading_name: bool,
    /// Words dropped along with the name, so the remainder still reads.
    pub copulas: std::collections::BTreeSet<String>, // data_class: INTERNAL_ONLY
    /// Why the source's form is rewritten, and what is deliberately left alone.
    pub reason: String, // data_class: INTERNAL_ONLY
}

/// How the source's integer arithmetic must be spelled in the target.
///
/// The source defines overflow as WRAPPING and the target panics on it in debug and wraps in
/// release, so the plain operator turns one source program into two target programs. This is the
/// pack saying which spelling carries the source's rule, and why.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct IntegerArithmetic {
    /// Source type names whose arithmetic this governs.
    pub types: std::collections::BTreeSet<String>, // data_class: INTERNAL_ONLY
    /// Source operator to the target method carrying the same rule.
    pub operators: std::collections::BTreeMap<String, String>, // data_class: INTERNAL_ONLY
    /// Why this spelling, and what it costs.
    pub reason: String, // data_class: INTERNAL_ONLY
}

/// How the pack answers for a call the target has no name of its own for.
///
/// A DECISION, so it carries a reason like every other one — an earlier form of this table was a
/// bare spelling map, and three translations sat in it with nobody's name on them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionMapping {
    /// Target template, with `{0}`, `{1}` for the arguments.
    pub form: String, // data_class: INTERNAL_ONLY
    /// The shape the argument must have, when the mapping is CONDITIONAL.
    ///
    /// `panic` is the case this exists for: it is faithful only where the payload is a string
    /// literal, and silently wrong anywhere else. `None` means the mapping holds for any argument.
    pub requires_argument: Option<String>, // data_class: INTERNAL_ONLY
    /// Why this call becomes this form, and what it costs.
    pub reason: String, // data_class: INTERNAL_ONLY
}

/// How the source spells failure, so the engine can recognise it without knowing the language.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailureConvention {
    /// Source type identity of the failure value — the type a trailing result has when the
    /// function can fail.
    pub source_type: String, // data_class: INTERNAL_ONLY
    /// Target type the failure value becomes.
    pub target_type: String, // data_class: INTERNAL_ONLY
    /// Whether a failing return DISCARDS the value carried beside the failure.
    ///
    /// The source's failing return carries both; the target's carries only the failure. The source
    /// documents that the other results are not guaranteed to be meaningful when the error is
    /// non-nil, so discarding is faithful to that convention — and the alternative is refusing most
    /// real fallible code.
    pub discards_companion: bool,
    /// Why the companion may be discarded, and what it costs.
    pub discard_reason: String, // data_class: INTERNAL_ONLY
    /// Why that target type, and what its bounds buy.
    ///
    /// Required, because this is the single most load-bearing type decision the pack makes: it
    /// appears in the signature of every fallible declaration in every ported package.
    pub reason: String, // data_class: INTERNAL_ONLY
    /// How the source spells the ABSENT failure value, which is what a success returns and what a
    /// check compares against.
    pub absent: String, // data_class: INTERNAL_ONLY
    /// Callee identities that PRODUCE a failure, and so never produce the absent value.
    ///
    /// The target's failing return is `Err(..)` and the source's is a value that MAY be absent, so
    /// wrapping an error-typed expression in `Err` is right only when that expression cannot be
    /// nil — and silently wrong when it can, in the direction of reporting failure where the source
    /// reported success.
    pub constructors: std::collections::BTreeSet<String>, // data_class: INTERNAL_ONLY
    /// How a failure is built where the DESTINATION already fixes its type. Empty to use the
    /// general mapping everywhere.
    pub inferred_construction: String, // data_class: INTERNAL_ONLY
    /// Why a second form, and where it applies.
    pub inferred_construction_reason: String, // data_class: INTERNAL_ONLY
    /// The name a unit gives the failure type, so its signatures do not spell it out.
    ///
    /// Empty means every signature carries the full type.
    pub alias: String, // data_class: INTERNAL_ONLY
    /// Why the alias, and what it changes.
    pub alias_reason: String, // data_class: INTERNAL_ONLY
    /// Why those callees, and what admits a new one.
    pub constructor_reason: String, // data_class: INTERNAL_ONLY
    /// Constructors whose SOLE ARGUMENT is the message, so a sentinel built by one is its message.
    ///
    /// A subset of [`Self::constructors`]. `fmt.Errorf` is a constructor and not one of these: its
    /// message is formatted from arguments, which is not a constant expression.
    pub sentinel_constructors: std::collections::BTreeSet<String>, // data_class: INTERNAL_ONLY
    /// What a sentinel becomes, why, and what it costs.
    pub sentinel_reason: String, // data_class: INTERNAL_ONLY
}

impl FailureConvention {
    /// Whether a source type identity is the failure type.
    #[must_use]
    pub fn is_failure(&self, identity: &str) -> bool {
        identity == self.source_type
    }
}
