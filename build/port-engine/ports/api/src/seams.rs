//! The seam traits (ADR-0637 D1): what the engine asks of a front end, a rule pack, an IR, and a
//! renderer.
//!
//! All four live on the ports face because all four are implemented OUTSIDE the core, by adapters.
//! An adapter depends on the contract it implements; it must never depend on the engine behind
//! that contract.

use std::collections::{BTreeMap, BTreeSet};

use crate::declaration::Declaration;
use crate::error::PortError;
use crate::failure::{DeriveRule, DocConvention, FailureConvention, FunctionMapping, IdiomRule, IntegerArithmetic};
use crate::identity::{Digest, LanguagePair, RegionId, RuleId, UnitId};
use crate::ownership::PointerDisposition;

/// The canonical semantic model of the source corpus, as produced by a front end.
///
/// The front end owns source-language semantics; this trait owns identity, order, and the
/// declaration tree. `units` is order-significant and MUST be deterministic for a given input —
/// `port-engine-kernel::plan` rejects a duplicate id because that is the shape in which a
/// non-deterministic model reaches the engine.
///
/// The kernel itself still reads only identity and order. [`SourceModel::declarations`] exists for
/// the transform face, which needs to know what a unit declares in order to construct anything at
/// all; a model that answers only with unit ids can produce nothing but empty regions named after
/// its units.
pub trait SourceModel {
    /// Slug of the language this model was read from.
    fn language(&self) -> &str;
    /// Digest of the snapshot this model was derived from (the receipt's `snapshot_digest`).
    fn snapshot_digest(&self) -> Digest;
    /// The translatable units, in deterministic order.
    fn units(&self) -> Vec<UnitId>;
    /// What `unit` declares, in deterministic order.
    ///
    /// `None` means the model does not carry that unit at all; `Some(vec![])` means it carries the
    /// unit and the unit declares nothing. The two are different answers and a caller may act
    /// differently on them, which is why this is not a bare `Vec` — an empty vector standing for
    /// both would let an unknown unit read as an empty one, and an empty one transforms to nothing
    /// without complaint.
    ///
    /// Deliberately NOT defaulted. A default returning "no declarations" would let a front end
    /// that forgot to implement it produce a green, empty translation of a populated corpus, and
    /// the receipt would attribute the emptiness to nothing at all.
    fn declarations(&self, unit: &UnitId) -> Option<Vec<Declaration>>;
}

/// Neutral rule data, addressed by [`LanguagePair`].
///
/// Rule SEMANTICS live in the data, not here. The kernel needs exactly two things: which pair the
/// pack serves, and which of its declared rules apply to a unit — in pack order, because rule
/// order is part of the transform.
pub trait RulePack {
    /// The language pair this pack is authored for.
    fn pair(&self) -> &LanguagePair;
    /// Digest of the pack contents (the receipt's `rulepack_digest`).
    fn digest(&self) -> Digest;
    /// Every rule the pack declares, in pack order.
    fn rules(&self) -> Vec<RuleId>;
    /// The declared rules that apply to `unit`, in pack order. Returning a rule absent from
    /// [`RulePack::rules`] is a pack defect and `port-engine-kernel::plan` refuses it.
    fn rules_for(&self, unit: &UnitId) -> Vec<RuleId>;
}

/// Everything a transform needs from a loaded rule pack.
///
/// A SEAM, and it lives on the ports face for the reason every seam does: the transform consumes
/// it and the rulepack adapter implements it, so defining it in the core face would make an
/// adapter depend on the engine rather than on the contract. Rule-level lookups take a
/// [`RuleId`]; pack-level data — the type map and the deferred-kind set — is asked of the pack as
/// a whole.
///
/// Distinct from [`RulePack`], which answers WHICH rules apply. This answers what a rule MEANS.
pub trait PackSemantics {
    /// Construction id for `rule`, if the pack declares it.
    fn construction(&self, rule: &RuleId) -> Option<&str>;
    /// Precondition id for `rule`, if the pack declares it.
    fn precondition(&self, rule: &RuleId) -> Option<&str>;
    /// Declaration kinds `rule` captures. Empty means the rule is unit-level.
    fn captures(&self, rule: &RuleId) -> Option<&[String]>;
    /// Source type spelling → target type spelling.
    fn type_map(&self) -> &BTreeMap<String, String>;
    /// Target-type templates keyed by source type KIND, with `{0}`, `{1}` for the arguments.
    ///
    /// This is what makes a composite resolvable by CONSTRUCTOR rather than by shape: one entry
    /// for a slice answers every slice, where a table keyed by spelling needed a row per element
    /// type — and could still not express a type from another package.
    fn type_constructors(&self) -> &BTreeMap<String, String>;
    /// Per-construction overrides of [`PackSemantics::type_map`], keyed by construction id.
    ///
    /// One source type does not always map to one target type: the same spelling can need a
    /// different target depending on the item being built — an owned type is right for a field
    /// and impossible for a constant, for instance. Overriding is DATA for the same reason the
    /// base map is: which target a source type takes in which position is a translation decision,
    /// and a decision belongs in the pack rather than in a branch here.
    fn type_map_overrides(&self, construction: &str) -> Option<&BTreeMap<String, String>>;
    /// SOURCE types whose target counterpart copies, so reading one by value needs nothing.
    ///
    /// Keyed by source identity, the same way [`PackSemantics::type_map`] is, so the two tables
    /// answer for the same thing. Everything else MOVES on a plain read, which does not compile
    /// out of a borrow — so the read is cloned, because the source copied and the target would
    /// not.
    fn copy_types(&self) -> &BTreeSet<String>;
    /// SOURCE type identity → the target expression for that type's zero value.
    ///
    /// Keyed like [`PackSemantics::type_map`]. Go fills a struct literal's omitted fields with the
    /// zero value of their type; the target rejects an incomplete literal, so the engine has to
    /// spell the omitted fields out — and what a type's zero LOOKS LIKE in the target is a
    /// translation decision, so the pack owns it rather than the engine assuming `Default`.
    fn zero_values(&self) -> &BTreeMap<String, String>;
    /// The target form a TRAIT takes in each position, with `{0}` for the trait's path.
    ///
    /// Keyed by position — `param`, `result`, `field` — because one form does not answer for all
    /// of them: a borrowed trait object is right for a parameter and impossible for a value the
    /// function returns. A position with no entry REFUSES, because the choice between borrowing,
    /// boxing and sharing is an ownership decision and the engine has no basis to make it.
    fn trait_object_forms(&self) -> &BTreeMap<String, String>;
    /// How the source spells failure, or `None` when it has no such convention.
    ///
    /// A trailing result of the named type is what makes a function FALLIBLE, which the target
    /// expresses as the whole return type. `None` means every result is an ordinary value, and a
    /// source language without the convention needs no rule to say so.
    fn failure_convention(&self) -> Option<&FailureConvention>;
    /// Source function identity → a target expression template, with `{0}`, `{1}` for arguments.
    ///
    /// A call the pack does not answer for emits the source's own name, which the target does not
    /// have. No real package can be ported without this table: every one of them calls its standard
    /// library, and a standard library is exactly the part that does not come along.
    fn function_map(&self) -> &BTreeMap<String, FunctionMapping>;
    /// Callee identities whose value IS a length, and so is a `usize` in the target.
    ///
    /// The source types a length as its own integer, which the type map sends to a signed target
    /// type — right for a value the source typed that way, and wrong for a length.
    fn length_functions(&self) -> &BTreeSet<String>;

    /// How the source's integer arithmetic must be spelled, so overflow keeps its meaning.
    fn integer_arithmetic(&self) -> &IntegerArithmetic;

    /// How the source's documentation convention differs from the target's.
    fn doc_convention(&self) -> &DocConvention;

    /// The derives a ported type earns, and what blocks each.
    fn derives(&self) -> &[DeriveRule];

    /// Idiom rules: spellings the target prefers, which change nothing about the program.
    fn idioms(&self) -> &[IdiomRule];

    /// What a SEQUENCE literal becomes, keyed by the type's kind.
    fn literal_constructors(&self) -> &BTreeMap<String, String>;
    /// SOURCE types a conversion reaches by a plain cast.
    ///
    /// Keyed by source identity like [`PackSemantics::type_map`]. A conversion the pack does not
    /// list is one where the two languages disagree about what conversion MEANS — infallible and
    /// lossy on one side, fallible on the other — and those refuse.
    fn cast_types(&self) -> &BTreeSet<String>;
    /// Ownership rules, in declared order — first match wins.
    ///
    /// Which ownership form a set of observed facts deserves is a translation DECISION with a cost
    /// either way, so it is data with a recorded reason rather than a branch.
    fn pointer_dispositions(&self) -> &[PointerDisposition];
    /// Declaration kinds the pack knowingly does not translate yet.
    fn deferred_kinds(&self) -> &BTreeSet<String>;
    /// Source PREDECLARED constant name → target expression.
    ///
    /// `true` reaches the model as an identifier referring to a universe-scope constant rather than
    /// as a literal, so nothing in the literal path answers for it. A source vocabulary the engine
    /// reads, like the type map — not a decision the engine makes.
    fn constant_map(&self) -> &BTreeMap<String, String>;
    /// Form id → why the pack has not decided that form, in the pack's own words.
    ///
    /// A FORM is a shape within a kind: a package variable something writes and one nothing writes
    /// reach the same rule, and only the first is undecided. A refusal has to say what is MISSING,
    /// and here what is missing is a decision — which the pack records. Reading it back means the
    /// refusal a reader sees and the reason the digest carries are one text, not two that drift.
    fn undecided_forms(&self) -> &BTreeMap<String, String>;
    /// How a trait method binds its receiver, and why the pack chose that.
    ///
    /// `None` is a REFUSAL, not a default. A source interface says nothing about how an
    /// implementation binds its receiver, and the implementations are not all in view, so this
    /// cannot be recovered — it can only be decided. A shared receiver silently forbids the
    /// mutation a mutating method exists to perform, and an exclusive one demands mutability from
    /// implementations that do not need it; both are guesses, and one of them was being made.
    fn trait_receiver(&self) -> Option<(&str, &str)>;
}

/// The neutral intermediate representation handed to a [`Renderer`].
///
/// As with [`SourceModel`], the kernel sees identity and order, never content.
pub trait TargetIr {
    /// Slug of the language this IR will be emitted as.
    fn target_language(&self) -> &str;
    /// The regions this IR emits, in deterministic order.
    fn regions(&self) -> Vec<RegionId>;
}

/// Turns a [`TargetIr`] into emitted bytes, one blob per region.
pub trait Renderer {
    /// Slug of the language this renderer emits.
    fn target_language(&self) -> &str;
    /// Digest of the formatter this renderer applies (the receipt's `formatter_digest`).
    fn formatter_digest(&self) -> Digest;
    /// Render every region of `ir`. The returned key set MUST equal `ir.regions()`;
    /// `port-engine-kernel::emit` enforces that rather than trusting it.
    ///
    /// # Errors
    /// Whatever the implementation refuses with — [`PortError::Render`] exists so that sentence is
    /// true of this closed enum. `port-engine-kernel::emit` adds the region-set proof on top.
    fn render(&self, ir: &dyn TargetIr) -> Result<BTreeMap<RegionId, Vec<u8>>, PortError>;
}
