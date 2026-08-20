//! What ONE UNIT knows about itself: its names, its sentinels, its lengths, its members.
//!
//! Split from `resolve.rs` because the two are different subjects. That file holds the RESOLVER —
//! the pack's tables and the questions asked of them, which are the same for every unit. This holds
//! what is true of one unit only, derived from its own declarations before any of them is built.
//!
//! Everything here is derived ONCE and read many times, and that is the point rather than an
//! optimisation: three sites that each worked out a unit's sentinels would be three chances for
//! them to disagree, and two of them did once.

use std::collections::{BTreeMap, BTreeSet};

use port_engine_api::Declaration;

use crate::naming::{to_pascal_case, to_screaming_snake, to_snake_case};
use crate::vocabulary::{CHILD_FIELD, CHILD_METHOD, FLAG_EXPORTED};

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
    /// The unit's named types that become a target NEWTYPE — a named type whose underlying is not
    /// a struct or an interface.
    ///
    /// The source indexes such a type directly: `type ID [12]byte` admits `id[:]`, because the name
    /// and the array are the same thing there. The target's newtype WRAPS the array, so the same
    /// expression has to reach through the field, and emitting the source's spelling produces
    /// `cannot index into a value of type &Id`. Five packages failed on exactly that.
    ///
    /// Held here rather than asked of the declaration at each use because the body sees only an
    /// identifier: the front end records a type on an expression only where it is needed, and a
    /// receiver carries none. What the body knows is which declaration it is inside, and this maps
    /// that to whether its target shape wraps.
    pub(crate) newtypes: BTreeMap<String, port_engine_api::TypeRef>,
    /// Unexported types this unit's EXPORTED declarations reach.
    ///
    /// Go lets an exported declaration have an unexported type — `var RequestIDKey ctxKeyRequestID`
    /// is idiomatic there, and a consumer can hold the value without being able to name the type.
    /// The target has no such asymmetry: a `pub` item whose type is private is
    /// `private_interfaces`, which `--deny=warnings` makes a build failure.
    ///
    /// So the TYPE is promoted rather than the declaration hidden. Hiding the declaration would
    /// delete an exported name from the ported API, which is the source's contract; widening the
    /// type keeps every consumer able to do exactly what the source let them do.
    ///
    /// Keyed by the promoted name and VALUED by what promoted it, because a declaration that
    /// REFUSES is not in the emitted crate and cannot leak anything. `chi`'s `RouteCtxKey` is an
    /// exported var of type `*contextKey` — and it refuses, as an exported package variable whose
    /// form the pack has not decided — so it widened `contextKey` to `pub` on behalf of an item
    /// that is not there. Six of chi's unexported types were public for that reason.
    pub(crate) publicly_reachable: BTreeMap<String, BTreeSet<String>>,
    /// Per local declaration, the source type refs that decide which traits it EARNS.
    ///
    /// A newtype's is its own underlying type; a struct's are its fields'. Held so the derive rule
    /// can follow a reference to another declaration of this unit instead of assuming it earns
    /// everything. That assumption used to be written down as safe — "every emitted struct gets the
    /// same list" — and it is not: a newtype over a slice earns no total equality, so a struct
    /// holding one and deriving `Eq` does not compile. The corpus proved it.
    pub(crate) derive_inputs: DeriveInputs,
    /// The package this scope IS, so a named type can be told local from foreign.
    ///
    /// `derive_inputs` is keyed by bare name and holds ONE unit, so a reference to another
    /// package's type both misses it and — where the two packages declare the same name — collides
    /// with the local declaration of that name. `TypeRef::package` is what makes the reference
    /// addressable; this is the other half of that comparison.
    pub(crate) package: String,
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
    /// Per sentinel, the CONSTANTS its message interpolates — empty for a plain literal.
    pub(crate) sentinel_arguments: BTreeMap<String, Vec<String>>,
    /// The unit's sentinels in the order the SOURCE declares them, with their raw doc blocks.
    ///
    /// A name-keyed map cannot carry either. The grouped enum needs the order, because a reader
    /// should meet the failures as the package presents them — a reviewer called alphabetical
    /// ordering a tell — and it needs each declaration's documentation, because a variant carries
    /// the source's own words for the failure it is.
    pub(crate) sentinel_order: Vec<(String, String)>,
}

/// Per named declaration, the source type refs that decide which traits it EARNS.
///
/// Keyed by (declaring package, name) because the name alone does not identify a declaration: the
/// fixture corpus alone declares `Counter` in two packages, and a table keyed by bare name answers
/// a reference to one with the other's fields.
pub type DeriveInputs = BTreeMap<(String, String), Vec<port_engine_api::TypeRef>>;

/// The derive inputs every unit of `model` contributes.
///
/// Built across the WHOLE model because a struct field may name another package's type, and whether
/// that type earns a trait is a fact about its declaration rather than about the referring unit.
pub fn model_derive_inputs(model: &dyn port_engine_api::SourceModel) -> DeriveInputs {
    let mut out = DeriveInputs::new();
    for unit in model.units() {
        let Some(declarations) = model.declarations(&unit) else {
            continue;
        };
        out.extend(declaration_inputs(&declarations, &unit.0));
    }
    out
}

/// The same, for one unit's declarations.
fn declaration_inputs(declarations: &[Declaration], package: &str) -> DeriveInputs {
    declarations
        .iter()
        .map(|declaration| {
            let inputs = match declaration.kind.as_str() {
                "named" => vec![declaration.type_ref.clone()],
                _ => declaration
                    .children_of_kind(crate::vocabulary::CHILD_FIELD)
                    .into_iter()
                    .map(|field| field.type_ref.clone())
                    .collect(),
            };
            ((package.to_owned(), declaration.name.clone()), inputs)
        })
        .collect()
}

/// What the PACK contributes to a scope, as one value.
///
/// Grouped rather than passed one by one: these arrive together from the pack, they are threaded
/// unchanged through every construction site, and as separate parameters they pushed the
/// constructor past the argument count clippy allows.
pub struct PackFacts<'a> {
    /// The pack's length functions, for recognising a constant that is a length.
    pub lengths: &'a BTreeSet<String>,
    /// The pack's formatting functions.
    pub renders: &'a BTreeSet<String>,
    /// The callees that take a length argument.
    pub takes_length: &'a BTreeSet<String>,
    /// The pack's format verbs.
    pub verbs: &'a BTreeMap<String, String>,
    /// Every unit's derive inputs, so a field naming another package's type resolves against that
    /// type's own declaration.
    pub derive_inputs: &'a DeriveInputs,
}

static NO_NAMES: BTreeSet<String> = BTreeSet::new();
static NO_VERBS: BTreeMap<String, String> = BTreeMap::new();
static NO_INPUTS: DeriveInputs = BTreeMap::new();

impl Default for PackFacts<'_> {
    fn default() -> Self {
        Self {
            lengths: &NO_NAMES,
            renders: &NO_NAMES,
            takes_length: &NO_NAMES,
            verbs: &NO_VERBS,
            derive_inputs: &NO_INPUTS,
        }
    }
}

impl LocalScope {
    /// Every named declaration in the unit contributes its emitted name.
    ///
    /// Which kinds are type declarations is not decided here — deciding it would mean naming the
    /// source language's kind vocabulary in the neutral face. Every named declaration is recorded
    /// instead, and a collision is impossible because the front end already refuses two
    /// declarations sharing a name in one namespace.
    pub fn of(declarations: &[Declaration], package: &str) -> Self {
        Self::with_facts(declarations, package, None, &PackFacts::default())
    }

    /// The same, plus what the PACK contributes: its failure convention and `PackFacts`.
    pub fn with_facts(
        declarations: &[Declaration],
        package: &str,
        failure: Option<&port_engine_api::FailureConvention>,
        facts: &PackFacts<'_>,
    ) -> Self {
        let length_constants = crate::length_consts::length_constants(
            declarations,
            facts.lengths,
            facts.renders,
            facts.takes_length,
        );
        // Carried WITH their arguments: a sentinel built by a formatting constructor over constants
        // has a fixed message that is not one literal, and dropping the values would emit a format
        // string with nothing to fill it.
        let carried = crate::sentinel::sentinels_with(declarations, failure, facts.verbs);
        let sentinels: BTreeMap<String, String> = carried
            .iter()
            .map(|(name, (message, _))| (name.clone(), message.clone()))
            .collect();
        let sentinel_arguments: BTreeMap<String, Vec<String>> = carried
            .into_iter()
            .map(|(name, (_, args))| (name, args))
            .collect();
        let sentinel_order = declarations
            .iter()
            .filter(|declaration| sentinels.contains_key(&declaration.name))
            .map(|declaration| {
                (
                    declaration.name.clone(),
                    declaration
                        .attr(crate::vocabulary::ATTR_DOC)
                        .unwrap_or_default()
                        .to_owned(),
                )
            })
            .collect();
        // The model-wide table first, then this unit's own declarations over the top. `with_failure`
        // and `of` pass an empty model-wide table, so the local pass is what keeps those paths able
        // to resolve at all; where both hold a key they hold the same declaration.
        let mut derive_inputs: DeriveInputs = facts.derive_inputs.clone();
        derive_inputs.extend(declaration_inputs(declarations, package));
        let newtypes: BTreeMap<String, port_engine_api::TypeRef> = declarations
            .iter()
            .filter(|declaration| declaration.kind == "named")
            .map(|declaration| (declaration.name.clone(), declaration.type_ref.clone()))
            .collect();
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
            publicly_reachable: publicly_reachable(declarations),
            sentinel_arguments,
            newtypes,
            derive_inputs,
            package: package.to_owned(),
            types,
            sentinels,
            sentinel_order,
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

/// The unit's type names that an EXPORTED declaration reaches.
///
/// One level, not a closure over the graph. A type reached by an exported declaration is promoted,
/// and promoting it makes ITS own members exported for the same reason — so the walk is repeated
/// until nothing new appears, because a `pub struct` whose field type is private is the identical
/// diagnostic one level down.
fn publicly_reachable(declarations: &[Declaration]) -> BTreeMap<String, BTreeSet<String>> {
    let mut reachable: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut settled = false;
    while !settled {
        settled = true;
        for declaration in declarations {
            let exported = declaration.flags.iter().any(|flag| flag == "exported")
                || reachable.contains_key(&declaration.name);
            if !exported {
                continue;
            }
            let mut named = Vec::new();
            collect_named_types(declaration, &mut named);
            for name in named {
                if !declarations.iter().any(|other| other.name == name) {
                    continue;
                }
                // The PROMOTER is recorded, so a promotion made on behalf of a declaration that
                // never reaches the crate can be discounted at the point the visibility is decided.
                // A name promoted TRANSITIVELY carries its promoter's own promoters, because the
                // chain is only as real as the item at its head.
                let promoters: BTreeSet<String> = match reachable.get(&declaration.name) {
                    Some(inherited) => inherited.clone(),
                    None => BTreeSet::from([declaration.name.clone()]),
                };
                let entry = reachable.entry(name).or_default();
                let before = entry.len();
                entry.extend(promoters);
                if entry.len() != before {
                    settled = false;
                }
            }
        }
    }
    reachable
}

/// Every LOCAL type name this declaration's types mention, at any depth.
///
/// Reads the type refs rather than the values: what makes a type reachable is being NAMED in a
/// signature, a field or a declared type, and a body that merely constructs one privately does not
/// expose it.
fn collect_named_types(node: &Declaration, into: &mut Vec<String>) {
    collect_from_type(&node.type_ref, into);
    for child in &node.children {
        // A BODY is not an interface. Names it mentions are implementation, and promoting them
        // would make every private helper a type reaches part of the public API.
        if child.kind == "body" {
            continue;
        }
        // NEITHER IS A PRIVATE MEMBER. A private field of a public struct, or an unexported method
        // on an exported type, leaks nothing — the target's own rule is about what a PUBLIC item's
        // signature names, and a private field is not part of one.
        //
        // Walking them promoted nine of memberlist's unexported wire structs — `ping`, `ackResp`,
        // `alive`, `messageType` — to `pub`, which the Go-aware gate correctly called a meaning
        // change rather than a style one: the source says who may name these and the port said
        // something else.
        if matches!(child.kind.as_str(), "field" | "method")
            && !child.flags.iter().any(|flag| flag == "exported")
        {
            continue;
        }
        // A SATISFACTION is not a signature either. It records that this type was seen implementing
        // an interface, which says nothing about who may name the type.
        if child.kind == "implements" {
            continue;
        }
        collect_named_types(child, into);
    }
}

fn collect_from_type(type_ref: &port_engine_api::TypeRef, into: &mut Vec<String>) {
    if !type_ref.name.is_empty() {
        // The LAST path segment, because a local type is recorded by its own name and a foreign one
        // is qualified — and a foreign name matches no local declaration, so it falls out below.
        let local = type_ref.name.rsplit('.').next().unwrap_or(&type_ref.name);
        into.push(local.to_owned());
    }
    for arg in &type_ref.args {
        collect_from_type(arg, into);
    }
}
