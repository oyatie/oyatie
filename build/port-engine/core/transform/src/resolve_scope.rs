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
    /// Per local declaration, the source type refs that decide which traits it EARNS.
    ///
    /// A newtype's is its own underlying type; a struct's are its fields'. Held so the derive rule
    /// can follow a reference to another declaration of this unit instead of assuming it earns
    /// everything. That assumption used to be written down as safe — "every emitted struct gets the
    /// same list" — and it is not: a newtype over a slice earns no total equality, so a struct
    /// holding one and deriving `Eq` does not compile. The corpus proved it.
    pub(crate) derive_inputs: BTreeMap<String, Vec<port_engine_api::TypeRef>>,
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
        Self::with_lengths(
            declarations,
            failure,
            &BTreeSet::new(),
            &BTreeSet::new(),
            &BTreeSet::new(),
            &BTreeMap::new(),
        )
    }

    /// The same, plus the constants that are lengths, which needs the pack's length callees.
    pub fn with_lengths(
        declarations: &[Declaration],
        failure: Option<&port_engine_api::FailureConvention>,
        lengths: &BTreeSet<String>,
        renders: &BTreeSet<String>,
        takes_length: &BTreeSet<String>,
        verbs: &BTreeMap<String, String>,
    ) -> Self {
        let length_constants =
            crate::length_consts::length_constants(declarations, lengths, renders, takes_length);
        // Carried WITH their arguments: a sentinel built by a formatting constructor over constants
        // has a fixed message that is not one literal, and dropping the values would emit a format
        // string with nothing to fill it.
        let carried = crate::sentinel::sentinels_with(declarations, failure, verbs);
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
        let derive_inputs: BTreeMap<String, Vec<port_engine_api::TypeRef>> = declarations
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
                (declaration.name.clone(), inputs)
            })
            .collect();
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
            sentinel_arguments,
            newtypes,
            derive_inputs,
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
