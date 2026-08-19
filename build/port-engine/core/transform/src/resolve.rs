//! Source type → target type, by STRUCTURE.
//!
//! Resolution used to be a lookup of a flat spelling in a flat table, which works exactly as long
//! as every type is primitive or has its own row. It fails three ways at once on a real corpus: a
//! composite needs a row per shape rather than per constructor, a type from another package
//! resolves to nothing because the table is keyed by unqualified text, and two packages that each
//! declare a `Point` collide.
//!
//! Now the type is a tree and resolution walks it:
//!
//! 1. a NAMED type in the unit being transformed resolves to that declaration's emitted name;
//! 2. a NAMED type in another unit resolves through that unit's emitted module path;
//! 3. anything else — a primitive, a composite constructor — is answered by the pack;
//! 4. and nothing that reaches the end resolves. It refuses, by name.
//!
//! Nothing is guessed at any step. Passing an unresolved spelling through would produce code that
//! either fails to compile far from its cause or, worse, compiles as an unrelated target type that
//! happens to share a name.

use std::collections::{BTreeMap, BTreeSet};

use port_engine_api::{Declaration, DeriveRule, DocConvention, FailureConvention, FunctionMapping, IdiomRule, IntegerArithmetic, TypeRef, UnitId};
use port_engine_rust_ir::RustType;

use crate::error::TransformError;
pub use crate::resolve_scope::LocalScope;

use crate::signature_table::SignatureTable;
use crate::naming::{module_path, to_pascal_case, to_screaming_snake, to_snake_case};
use crate::ownership::OwnershipContext;
use crate::vocabulary::{CHILD_FIELD, CHILD_METHOD, FLAG_EXPORTED, TYPE_NAMED_INTERFACE};




/// What a resolver needs: the unit's own declarations, the pack's answers, and the unit's identity.
pub(crate) struct Resolver<'a> {
    pub(crate) scope: &'a LocalScope,
    /// Source type identity → target spelling. Keyed by `package.Name` for a named type and by the
    /// bare name for a primitive; the two cannot collide because a qualified key always carries a
    /// separator a bare name may not.
    pub(crate) type_map: &'a BTreeMap<String, String>,
    /// Per-construction overrides of [`Resolver::type_map`].
    ///
    /// One source type does not always take one target type: the same type can need a different
    /// target depending on the item being built — an owned type is right for a field and
    /// impossible for a constant. Which target in which position is a translation decision, so it
    /// is data.
    pub(crate) overrides: Option<&'a BTreeMap<String, String>>,
    /// Target-type templates keyed by type KIND, with `{0}`, `{1}` for the arguments.
    ///
    /// This is what makes a composite resolvable by CONSTRUCTOR rather than by shape: one entry
    /// for `slice` answers every slice, where a flat table needed a row per element type.
    pub(crate) constructors: &'a BTreeMap<String, String>,
    /// Source types whose target counterpart copies; everything else clones on a value read.
    pub(crate) copy_types: &'a BTreeSet<String>,
    /// Source types a conversion reaches by a plain cast.
    pub(crate) cast_types: &'a BTreeSet<String>,
    /// Source function identity → a target expression template.
    pub(crate) function_map: &'a BTreeMap<String, FunctionMapping>,
    /// How the pack answers for a call that FORMATS a template.
    pub(crate) format_calls: &'a port_engine_api::FormatCalls,
    /// Calls the pack refuses to map, and why each one cannot be mapped faithfully.
    pub(crate) unmappable_calls: &'a BTreeMap<String, String>,
    /// How integer arithmetic must be spelled so overflow keeps the source's meaning.
    pub(crate) integer_arithmetic: &'a IntegerArithmetic,
    /// How the source's documentation convention differs from the target's.
    pub(crate) doc_convention: &'a DocConvention,
    /// The derives a ported type earns, and what blocks each.
    pub(crate) derives: &'a [DeriveRule],
    /// Idiom rules: spellings the target prefers, which change nothing about the program.
    pub(crate) idioms: &'a [IdiomRule],
    /// What a SEQUENCE literal becomes, keyed by the type's kind.
    pub(crate) literal_constructors: &'a BTreeMap<String, String>,
    /// How the source spells failure, when it has a convention for it.
    pub(crate) failure: Option<&'a FailureConvention>,
    /// The kinds the pack DEFERS, so a body can refuse to reference one.
    ///
    /// What the engine emits has to be self-contained: a body referring to a declaration the pack
    /// declined to emit produces a crate with a dangling name. Read from the pack rather than
    /// listed here, so a kind that stops being deferred stops causing refusals with no code change.
    pub(crate) deferred: &'a BTreeSet<String>,
    /// Source predeclared constant name → target expression.
    pub(crate) constant_map: &'a BTreeMap<String, String>,
    /// Source type name → target spelling, for the names a DOC COMMENT may use.
    pub(crate) prose_type_names: &'a BTreeMap<String, String>,
    /// Callee identities whose value IS a length, and so is a `usize` in the target.
    pub(crate) length_functions: &'a BTreeSet<String>,
    /// Form id → the pack's recorded reason for not having decided it, so a refusal quotes the
    /// pack rather than restating it in code where the two could drift.
    pub(crate) undecided_forms: &'a BTreeMap<String, String>,
    /// What a call's DESTINATION wants, keyed by callee identity.
    ///
    /// The body translator knows what an expression is and not where it is going. `&x` and a bare
    /// string literal both need the second, and both destinations are signatures the engine has
    /// already translated — see `signatures.rs` for what this cannot answer and why it refuses
    /// rather than approximating.
    pub(crate) signatures: &'a SignatureTable,
    /// The target form a trait takes in each position, keyed by position.
    pub(crate) trait_object_forms: &'a BTreeMap<String, String>,
    /// Source type identity → the target expression for that type's zero value.
    ///
    /// Go fills a struct literal's omitted fields with the zero value; the target has no such rule
    /// and rejects an incomplete literal, so the omitted fields have to be spelled out.
    pub(crate) zero_values: &'a BTreeMap<String, String>,
    /// The declared trait-receiver mode and its reason.
    pub(crate) receiver: Option<(&'a str, &'a str)>,
    /// The pack's ownership rules, and the log every decision is recorded into.
    pub(crate) ownership: &'a OwnershipContext<'a>,
    /// The names of THIS unit that will actually be emitted.
    ///
    /// Self-containment one step further in than [`Self::units`]: a body may name a declaration of
    /// its own unit that itself REFUSED, and the emitted crate then has a call to a function it
    /// does not contain. That is the same defect as naming another package, arrived at from the
    /// inside, and it is what remains of the unresolved-name errors after the package rule.
    ///
    /// Deciding it needs a FIXPOINT — a declaration is emitted only if everything it names is
    /// emitted, which may make more refuse, which may make more refuse. The strict pipeline needs
    /// no iteration because it requires every declaration to translate, so it passes every name.
    pub(crate) emitted: &'a BTreeSet<String>,
    /// Every unit the MODEL has, which is every module the emitted crate will contain.
    ///
    /// What is emitted has to be SELF-CONTAINED. A name from a package outside the model has no
    /// module to be reached through, and emitting `crate::<module>::<name>` for it produces a path
    /// that resolves to nothing — 216 of 226 compile errors across six real ported packages were
    /// exactly that. The engine already refuses a body that names a DEFERRED declaration for this
    /// reason; this is the same rule for the case it did not cover.
    pub(crate) units: &'a BTreeSet<String>,
    /// The unit under transform, which decides whether a named type is local.
    pub(crate) unit: &'a UnitId,
}

impl<'a> Resolver<'a> {
    /// The pack's declared trait-receiver mode and its reason.
    pub(crate) fn trait_receiver(&self) -> Option<(&'a str, &'a str)> {
        self.receiver
    }
}
