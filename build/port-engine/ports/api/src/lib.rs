//! # port-engine-api — neutral seam types for the owned deterministic port engine (W0-B Slice 2).
//!
//! ADR-0637 (archived; live via apex ADR-0704) D1 assigns the ports face: `SourceModel`,
//! `RulePack`, `TransformPlan`, `TargetIr`, `Renderer`, and six-axis `Receipt`. Slice 2 lands
//! those types here; `port-engine-kernel` owns neutrality enforcement and the `plan` / `emit` /
//! `verify` entry points.
//!
//! Zero dependencies by design: seam types name no corpus type and carry no adapter machinery.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Fail-closed readiness gate. `true` once Slice 2 seam types are present.
pub const fn w0_ready() -> bool {
    true
}

// ---------------------------------------------------------------------------------------------
// Seams (ADR-0637 D1).
// ---------------------------------------------------------------------------------------------

/// The source→target language pair a [`RulePack`] is authored for.
///
/// This is DATA, not a type parameter: the rule namespace is `specs/port-rules/lang/<pair>/**`,
/// so a second pair is a second directory of rule data over the same engine, never a second
/// engine. Both fields are opaque slugs — the kernel compares them and never interprets them.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct LanguagePair {
    /// Slug of the language being read (matches [`SourceModel::language`]).
    pub source: String, // data_class: INTERNAL_ONLY
    /// Slug of the language being emitted (matches [`TargetIr::target_language`]).
    pub target: String, // data_class: INTERNAL_ONLY
}

/// True for the bytes a [`LanguagePair`] slug may contain: ASCII lowercase alphanumeric, `_`, and
/// `+` (so `c++` stays spellable).
const fn is_slug_byte(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'+'
}

impl LanguagePair {
    /// The `<pair>` path segment of the rule namespace, e.g. `source-target`.
    ///
    /// FAIL-CLOSED, and the reason is an addressing collision rather than tidiness. The segment is
    /// the two slugs joined by `-`, so if a slug may itself contain `-` the join is not injective:
    /// `("a-b", "c")` and `("a", "b-c")` both render `a-b-c`, and once this value addresses
    /// `specs/port-rules/lang/<pair>` the two pairs select the SAME rule namespace. One of them is
    /// then reading or overwriting the other's rules with no error anywhere. Refusing the
    /// ambiguity here is the only place it is cheap: after the join the information is gone.
    ///
    /// The rule is that neither slug may be empty or carry a byte outside [`is_slug_byte`], the
    /// grammar of ONE portable path component. That is what the ADR fixes the segment to be —
    /// a single component of the form `<source>-<target>` — and the grammar is derived from that
    /// USE rather than from the separator collision alone. A slug of `a/b` renders `a/b-c`, which
    /// is two components, not one; a slug of `..` or a leading `/` is worse than a wrong name,
    /// because `Path::join` documents that an absolute operand REPLACES the receiver, so the
    /// namespace root would be discarded rather than descended from. Refusing the whole class here
    /// costs one predicate; enumerating the hostile bytes costs a review round each.
    ///
    /// # Errors
    /// [`PortError::AmbiguousLanguagePair`] when either slug is empty or carries a byte the
    /// component grammar does not admit (the separator among them).
    pub fn slug(&self) -> Result<String, PortError> {
        for slug in [&self.source, &self.target] {
            if slug.is_empty() || !slug.bytes().all(is_slug_byte) {
                return Err(PortError::AmbiguousLanguagePair {
                    source: self.source.clone(),
                    target: self.target.clone(),
                });
            }
        }
        Ok(format!("{}{PAIR_SEPARATOR}{}", self.source, self.target))
    }
}

/// The byte joining the two slugs of a [`LanguagePair::slug`], and therefore the byte neither slug
/// may contain.
pub const PAIR_SEPARATOR: char = '-';

/// The join is injective only while the separator sits OUTSIDE the slug grammar. Asserted at
/// compile time rather than argued in prose, so widening [`is_slug_byte`] to admit `-` fails the
/// build instead of silently making two pairs address one rule namespace.
const _: () = assert!(
    !is_slug_byte(PAIR_SEPARATOR as u8),
    "the separator must sit outside the slug grammar or the join stops being injective"
);

/// A stable identity for one translatable unit of the source model.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UnitId(pub String); // data_class: INTERNAL_ONLY

/// A stable identity for one rule in a [`RulePack`].
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RuleId(pub String); // data_class: INTERNAL_ONLY

/// A stable identity for one emitted region (the ADR-0597 registered regenerable region).
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RegionId(pub String); // data_class: INTERNAL_ONLY

/// An opaque content digest. The kernel COMPARES digests and never computes one — hashing is an
/// adapter concern, and keeping it out of here is what lets the receipt seam stay pure.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Digest(pub String); // data_class: INTERNAL_ONLY

/// One node of a unit's declaration tree: what the unit declares, as the front end saw it.
///
/// UNIFORM BY DESIGN. A constant, a struct field, a function parameter and an interface method are
/// all this one shape, and what tells them apart is [`Declaration::kind`] — a value, not a field
/// name and not an enum variant. The alternative shape, with `fields` / `methods` / `params` /
/// `results` as distinct fields, would have pushed one source language's declaration taxonomy into
/// a seam that [`LanguagePair`] deliberately keeps as data. A second language pair is a second
/// directory of rule data over the same engine; it must not be a second seam.
///
/// Every string here is opaque. The engine compares `kind`, `type_ref` and `flags` and never
/// interprets them — `int` is not a number to the engine and `func` is not a function. Meaning is
/// assigned by the rule pack, which selects on these values and says what to construct from them.
#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Declaration {
    /// What this node is, as an opaque slug the rule pack selects on. // data_class: INTERNAL_ONLY
    pub kind: String,
    /// The declared identifier. Empty is legal — an unnamed result is a real declaration.
    pub name: String, // data_class: INTERNAL_ONLY
    /// The declared type, as an opaque slug. Empty when the node declares no type.
    pub type_ref: String, // data_class: INTERNAL_ONLY
    /// Boolean facts, as a set of opaque slugs rather than named booleans, so a front end can
    /// record a new one without widening this seam.
    pub flags: BTreeSet<String>, // data_class: INTERNAL_ONLY
    /// Key→value facts that do not fit a set: a constant's value, and whatever a later front end
    /// needs to record. Separate from [`Declaration::flags`] because the two answer different
    /// questions — membership versus value — and folding a flag in as `"exported" => "1"` would
    /// lose the difference between an absent key and an empty one.
    pub attrs: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
    /// Nested declarations in significant order. Order is the front end's to decide and the
    /// engine's to preserve: it is semantic for a parameter list and for struct fields, and a
    /// front end that sorts what must stay positional has produced a defective model.
    pub children: Vec<Declaration>, // data_class: INTERNAL_ONLY
}

impl Declaration {
    /// True when `flag` is set on this node.
    #[must_use]
    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.contains(flag)
    }

    /// Children whose `kind` is exactly `kind`, in declared order.
    #[must_use]
    pub fn children_of_kind(&self, kind: &str) -> Vec<&Self> {
        self.children.iter().filter(|c| c.kind == kind).collect()
    }

    /// Value recorded under `key`, if the front end recorded one.
    #[must_use]
    pub fn attr(&self, key: &str) -> Option<&str> {
        self.attrs.get(key).map(String::as_str)
    }
}

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
    /// Per-construction overrides of [`PackSemantics::type_map`], keyed by construction id.
    ///
    /// One source type does not always map to one target type: the same spelling can need a
    /// different target depending on the item being built — an owned type is right for a field
    /// and impossible for a constant, for instance. Overriding is DATA for the same reason the
    /// base map is: which target a source type takes in which position is a translation decision,
    /// and a decision belongs in the pack rather than in a branch here.
    fn type_map_overrides(&self, construction: &str) -> Option<&BTreeMap<String, String>>;
    /// Declaration kinds the pack knowingly does not translate yet.
    fn deferred_kinds(&self) -> &BTreeSet<String>;
}

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

/// The six receipt axes ADR-0637 fixes. Every emitted-byte change must be attributable to at least
/// one of them; see `port-engine-kernel::verify`.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReceiptAxis {
    /// The upstream pin the source snapshot was taken at.
    Pin,
    /// The digest of the source snapshot.
    Snapshot,
    /// The digest of the engine binary.
    Engine,
    /// The digest of the rule pack.
    RulePack,
    /// The digest of the toolchain.
    Toolchain,
    /// The digest of the formatter.
    Formatter,
}

/// Every axis, in declaration order. Registered as a constant so a seventh axis cannot be added
/// without the comparison in [`Receipt::differing_axes`] being updated alongside it.
pub const RECEIPT_AXES: [ReceiptAxis; 6] = [
    ReceiptAxis::Pin,
    ReceiptAxis::Snapshot,
    ReceiptAxis::Engine,
    ReceiptAxis::RulePack,
    ReceiptAxis::Toolchain,
    ReceiptAxis::Formatter,
];

/// The six-axis provenance of one emission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    /// The upstream pin (an opaque revision identifier).
    pub pin: String, // data_class: INTERNAL_ONLY
    /// Digest of the source snapshot.
    pub snapshot_digest: Digest, // data_class: INTERNAL_ONLY
    /// Digest of the engine that emitted.
    pub engine_digest: Digest, // data_class: INTERNAL_ONLY
    /// Digest of the rule pack in force.
    pub rulepack_digest: Digest, // data_class: INTERNAL_ONLY
    /// Digest of the toolchain in force.
    pub toolchain_digest: Digest, // data_class: INTERNAL_ONLY
    /// Digest of the formatter in force.
    pub formatter_digest: Digest, // data_class: INTERNAL_ONLY
}

impl Receipt {
    /// The axes on which `self` and `other` disagree.
    #[must_use]
    pub fn differing_axes(&self, other: &Self) -> BTreeSet<ReceiptAxis> {
        let mut differing = BTreeSet::new();
        for axis in RECEIPT_AXES {
            let differs = match axis {
                ReceiptAxis::Pin => self.pin != other.pin,
                ReceiptAxis::Snapshot => self.snapshot_digest != other.snapshot_digest,
                ReceiptAxis::Engine => self.engine_digest != other.engine_digest,
                ReceiptAxis::RulePack => self.rulepack_digest != other.rulepack_digest,
                ReceiptAxis::Toolchain => self.toolchain_digest != other.toolchain_digest,
                ReceiptAxis::Formatter => self.formatter_digest != other.formatter_digest,
            };
            if differs {
                differing.insert(axis);
            }
        }
        differing
    }

    /// The axes that say NOTHING — an empty pin or an empty digest.
    ///
    /// [`Receipt::differing_axes`] answers "did this axis move", which is only a usable answer
    /// when the axis carries a value on both sides. An unfilled axis makes an apparent difference
    /// absence of information rather than evidence of a cause, and `port-engine-kernel::verify`
    /// must not spend it as an explanation. Walks [`RECEIPT_AXES`] for the same reason
    /// `differing_axes` does: a seventh axis cannot be added without this answer being updated
    /// alongside it.
    #[must_use]
    pub fn incomplete_axes(&self) -> BTreeSet<ReceiptAxis> {
        let mut incomplete = BTreeSet::new();
        for axis in RECEIPT_AXES {
            let empty = match axis {
                ReceiptAxis::Pin => self.pin.is_empty(),
                ReceiptAxis::Snapshot => self.snapshot_digest.0.is_empty(),
                ReceiptAxis::Engine => self.engine_digest.0.is_empty(),
                ReceiptAxis::RulePack => self.rulepack_digest.0.is_empty(),
                ReceiptAxis::Toolchain => self.toolchain_digest.0.is_empty(),
                ReceiptAxis::Formatter => self.formatter_digest.0.is_empty(),
            };
            if empty {
                incomplete.insert(axis);
            }
        }
        incomplete
    }
}

/// A typed, fail-closed refusal. Every variant carries enough to act on without re-deriving.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PortError {
    /// A language slug did not match the one it was paired against.
    LanguageMismatch {
        /// What the consumer required.
        expected: String, // data_class: INTERNAL_ONLY
        /// What it was handed.
        actual: String, // data_class: INTERNAL_ONLY
    },
    /// The source model emitted the same unit id twice, so step order is ambiguous.
    DuplicateUnit {
        /// The repeated id.
        unit: UnitId, // data_class: INTERNAL_ONLY
    },
    /// `rules_for` returned a rule the pack does not declare.
    UndeclaredRule {
        /// The unit it was returned for.
        unit: UnitId, // data_class: INTERNAL_ONLY
        /// The undeclared rule.
        rule: RuleId, // data_class: INTERNAL_ONLY
    },
    /// `rules_for` returned pack-declared rules in an order that is not the pack's own.
    RuleOrderViolation {
        /// The unit `rules_for` was asked about.
        unit: UnitId, // data_class: INTERNAL_ONLY
        /// The rule that arrived out of pack order (or a second time).
        rule: RuleId, // data_class: INTERNAL_ONLY
    },
    /// A renderer's emitted region set was not exactly the IR's region set.
    RegionSetMismatch {
        /// Regions the IR declared that the renderer did not emit.
        missing: BTreeSet<RegionId>, // data_class: INTERNAL_ONLY
        /// Regions the renderer emitted that the IR did not declare.
        unexpected: BTreeSet<RegionId>, // data_class: INTERNAL_ONLY
    },
    /// A [`TargetIr`] declared the same region identity twice.
    DuplicateRegion {
        /// The repeated region identity.
        region: RegionId, // data_class: INTERNAL_ONLY
    },
    /// A [`RulePack`] declared the same rule identity twice.
    DuplicateRule {
        /// The repeated rule identity.
        rule: RuleId, // data_class: INTERNAL_ONLY
    },
    /// A renderer refused for a reason of its own.
    Render {
        /// The renderer's own description of its refusal.
        detail: String, // data_class: INTERNAL_ONLY
    },
    /// A [`LanguagePair`] cannot address a rule namespace unambiguously.
    AmbiguousLanguagePair {
        /// The source slug as supplied.
        source: String, // data_class: INTERNAL_ONLY
        /// The target slug as supplied.
        target: String, // data_class: INTERNAL_ONLY
    },
}

impl fmt::Display for PortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LanguageMismatch { expected, actual } => {
                write!(
                    f,
                    "language mismatch: expected `{expected}`, got `{actual}`"
                )
            }
            Self::DuplicateUnit { unit } => {
                write!(
                    f,
                    "duplicate source unit `{}`: plan order is ambiguous",
                    unit.0
                )
            }
            Self::UndeclaredRule { unit, rule } => write!(
                f,
                "rule `{}` applied to unit `{}` is not declared by the pack",
                rule.0, unit.0
            ),
            Self::RuleOrderViolation { unit, rule } => write!(
                f,
                "rule `{}` arrived out of pack order for unit `{}`: rules_for must answer in the \
                 order rules() declares",
                rule.0, unit.0
            ),
            Self::RegionSetMismatch {
                missing,
                unexpected,
            } => write!(
                f,
                "renderer region set mismatch: {} missing, {} unexpected",
                missing.len(),
                unexpected.len()
            ),
            Self::DuplicateRegion { region } => write!(
                f,
                "duplicate declared region `{}`: region identity is ambiguous",
                region.0
            ),
            Self::DuplicateRule { rule } => write!(
                f,
                "duplicate declared rule `{}`: rule order is ambiguous",
                rule.0
            ),
            Self::Render { detail } => write!(f, "renderer refused: {detail}"),
            Self::AmbiguousLanguagePair { source, target } => write!(
                f,
                "language pair (`{source}`, `{target}`) cannot address a rule namespace \
                 unambiguously: neither slug may be empty or carry a byte outside the path \
                 component grammar (`{PAIR_SEPARATOR}` among them), because the joined value is \
                 ONE path component"
            ),
        }
    }
}

impl std::error::Error for PortError {}
