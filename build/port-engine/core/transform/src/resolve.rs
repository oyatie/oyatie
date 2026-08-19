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
use crate::signature_table::SignatureTable;
use crate::naming::{module_path, to_pascal_case, to_screaming_snake, to_snake_case};
use crate::ownership::OwnershipContext;
use crate::vocabulary::{CHILD_FIELD, CHILD_METHOD, FLAG_EXPORTED, TYPE_NAMED_INTERFACE};

/// The type names one unit declares, and the target spelling each resolves to.
pub struct LocalScope {
    pub(crate) types: BTreeMap<String, String>,
    /// Source name → target name, for every declaration this unit makes.
    ///
    /// A doc comment that NAMES a declaration must name it as the target does. The source's
    /// documentation says `Run` because that is what the method is called there; the emitted method
    /// is `run`, and prose that still says `Run` names nothing — which a reviewer called the
    /// cheapest possible proof that nobody had read the emitted code.
    ///
    /// A name whose target spelling is AMBIGUOUS — two declarations of different kinds sharing one
    /// source name — is absent, because rewriting it would have to pick one and the prose does not
    /// say which.
    pub(crate) renames: BTreeMap<String, String>,
    /// The unit's constants that are LENGTHS, proved from what the unit compares them against.
    ///
    /// Held here because the declaration and every comparison need the same answer: the constant
    /// renders as the target's index type, and the length beside it drops the conversion the call's
    /// mapping adds. Deriving it twice would let a guard compare two different types.
    pub(crate) length_constants: BTreeSet<String>,
    /// Member source name → the declaration that OWNS it.
    ///
    /// Held because a member is emitted exactly when its owner is, and nothing else in the scope
    /// records that. Prose naming a method whose type refused describes an API the crate does not
    /// contain — and asking the top-level emitted set about a member reported EVERY member absent,
    /// including the ones sitting in the output.
    pub(crate) member_owners: BTreeMap<String, String>,
    /// The unit's SENTINEL failures, by source name, with the message each carries.
    ///
    /// Held here because three places need the same answer — what the declaration emits, what a
    /// reference to it renders as, and whether a return of it is provably a failure — and deriving
    /// it three times would let them disagree.
    pub(crate) sentinels: BTreeMap<String, String>,
}

impl LocalScope {
    /// Every named declaration in the unit contributes its emitted name.
    ///
    /// Which kinds are type declarations is not decided here — deciding it would mean naming the
    /// source language's kind vocabulary in the neutral face. Every named declaration is recorded
    /// instead, and a collision is impossible because the front end already refuses two
    /// declarations sharing a name in one namespace.
    pub fn of(declarations: &[Declaration]) -> Self {
        Self::with_failure(declarations, None)
    }

    /// The same, plus the unit's sentinels, which need the pack's failure convention to recognise.
    pub fn with_failure(
        declarations: &[Declaration],
        failure: Option<&port_engine_api::FailureConvention>,
    ) -> Self {
        Self::with_lengths(declarations, failure, &BTreeSet::new(), &BTreeSet::new())
    }

    /// The same, plus the constants that are lengths, which needs the pack's length callees.
    pub fn with_lengths(
        declarations: &[Declaration],
        failure: Option<&port_engine_api::FailureConvention>,
        lengths: &BTreeSet<String>,
        renders: &BTreeSet<String>,
    ) -> Self {
        let length_constants =
            crate::length_consts::length_constants(declarations, lengths, renders);
        let sentinels = crate::sentinel::sentinels(declarations, failure);
        let mut types = BTreeMap::new();
        let mut renames: BTreeMap<String, Option<String>> = BTreeMap::new();
        let mut member_owners: BTreeMap<String, String> = BTreeMap::new();
        for declaration in declarations {
            if declaration.name.is_empty() {
                continue;
            }
            types.insert(declaration.name.clone(), to_pascal_case(&declaration.name));
            // The target's name for a declaration depends on its KIND, and a rename that ignored
            // that renamed a constant into a type's casing: `allowed` came out as `Allowed` in the
            // prose and `ALLOWED` in the code. Naming it wrong is worse than not naming it.
            if let Some(target) = emitted_name(declaration) {
                record_rename(&mut renames, &declaration.name, target);
            }
            // A METHOD and a FIELD are named inside a declaration and are cased like a binding.
            // Both are what documentation refers to most, and neither is in package scope.
            //
            // ONLY those two. A declaration's children are its whole tree — an initialiser, a body,
            // every expression in it — and recording all of them put the initialiser `= true` into
            // the map, so a doc comment saying "when set to true" came out saying `r#true`. A name
            // is a member only if it is declared as one.
            for member in &declaration.children {
                if member.name.is_empty()
                    || !matches!(member.kind.as_str(), CHILD_FIELD | CHILD_METHOD)
                {
                    continue;
                }
                let target = to_snake_case(&member.name);
                record_rename(&mut renames, &member.name, target);
                member_owners.insert(member.name.clone(), declaration.name.clone());
            }
        }
        Self {
            types,
            sentinels,
            length_constants,
            member_owners,
            renames: renames
                .into_iter()
                .filter_map(|(source, target)| Some((source, target?)))
                .collect(),
        }
    }

    /// Whether the unit declares this source name.
    pub(crate) fn contains(&self, name: &str) -> bool {
        self.types.contains_key(name)
    }
}

/// The name the target gives this declaration, by the same rule that emits it.
///
/// `None` for a kind whose emitted name this cannot state — which is not the same as a kind that is
/// not emitted, and is deliberately the conservative reading: a rename it cannot be sure of is one
/// it does not make.
fn emitted_name(declaration: &Declaration) -> Option<String> {
    // EXPORTED only, and this is the rule's bound rather than a convenience. The source capitalises
    // what it exports, so a capitalised word in prose matching an exported name is a reference to
    // it far more often than not — which is the case this rule was built for, `Run` and `Refresh`
    // and `NewVersion`. An UNEXPORTED name is lower-case and indistinguishable from English: a
    // private constant named `allowed` turned the sentence "not allowed in a valid semantic
    // version" into "not ALLOWED", which is a real package's doc comment made worse.
    //
    // What is left is bounded and small: even a false positive changes the CASING of a word and
    // never its meaning, because the rename is always the same word in the target's convention.
    if !declaration.flags.iter().any(|flag| flag == FLAG_EXPORTED) {
        return None;
    }
    match declaration.kind.as_str() {
        // A VALUE, whose name the target shouts.
        "const" | "var" => Some(to_screaming_snake(&declaration.name)),
        "func" => Some(to_snake_case(&declaration.name)),
        "struct" | "named" | "alias" | "interface" => Some(to_pascal_case(&declaration.name)),
        _ => None,
    }
}

/// Record one source name's target spelling, or mark it AMBIGUOUS if it already has a different one.
///
/// Two declarations sharing a source name and casing differently — a type `Value` and a method
/// `Value` — give prose no way to say which it means, so neither is rewritten.
fn record_rename(into: &mut BTreeMap<String, Option<String>>, source: &str, target: String) {
    match into.get(source) {
        Some(Some(existing)) if existing != &target => {
            into.insert(source.to_owned(), None);
        }
        Some(_) => {}
        None => {
            into.insert(source.to_owned(), Some(target));
        }
    }
}

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
