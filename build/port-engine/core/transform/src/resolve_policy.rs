//! What the pack answers about POLICY, as opposed to what a type resolves to.
//!
//! Four questions live here and none of them is a type lookup: whether a construction holds a
//! source string as an owned value, which traits a ported type earns, and which target method
//! carries the source's overflow rule. They are grouped because they share a shape — each is the
//! pack declining to let the engine assume something — and separated from the type tables because
//! a reader chasing "what does `map[string]int` become" should not have to read past them.

use port_engine_api::{Declaration, IdiomRule, TypeRef};

use crate::naming::to_pascal_case;
use crate::resolve::Resolver;
use crate::resolve_tables::table_key;
use crate::vocabulary::SOURCE_STRING;

impl Resolver<'_> {
    /// Whether this unit DECLARES the named type.
    ///
    /// The target's coherence rule forbids putting an inherent method on a type from elsewhere, so
    /// a constructor for someone else's type stays a free function however it is named.
    pub(crate) fn declares(&self, type_ref: &TypeRef) -> bool {
        !type_ref.name.is_empty() && self.scope.contains(&type_ref.name)
    }

    /// The TYPE NAME a sentinel takes, which is its source name without the convention's prefix.
    ///
    /// The source prefixes a sentinel's name because it has no namespacing inside a package and an
    /// unprefixed one would collide with an ordinary declaration. The target has modules, so
    /// `semver::EmptyString` says everything `semver::ErrEmptyString` does — and the prefix costs
    /// something there that it does not cost in the source, because the target's failure arm is
    /// literally called `Err`. `Err(ErrEmptyString)` stutters at every return, which is what a
    /// reviewer reading a real ported package named.
    ///
    /// Answered HERE because three sites need the same answer — the declaration, the return that
    /// constructs one, and the identity test that downcasts to it — and a rename two of them agreed
    /// on would not compile.
    ///
    /// The prefix stays in three cases, each one a case where dropping it would be a guess or a
    /// loss: the pack declares no prefix, what is left is empty, or another declaration in the unit
    /// already emits that name.
    /// The ENUM a unit's sentinels are grouped into, when the pack groups them and the name is free.
    ///
    /// `None` means each sentinel keeps its own type — because the pack declares no enum, or because
    /// this unit already declares something by that name, and a rename that collides is worse than
    /// the boilerplate it removes.
    pub(crate) fn sentinel_enum_name(&self) -> Option<&str> {
        let convention = self.failure?;
        if convention.sentinel_enum.is_empty() || self.scope.sentinels.is_empty() {
            return None;
        }
        let taken = self
            .scope
            .types
            .values()
            .any(|target| target == &convention.sentinel_enum);
        (!taken).then_some(convention.sentinel_enum.as_str())
    }

    /// How a sentinel is NAMED where a value is wanted: a variant path when grouped, else the type.
    ///
    /// The one answer every site asks for, so the declaration, the return and the identity test
    /// cannot disagree — which they did, once, and it did not compile.
    pub(crate) fn sentinel_path(&self, source: &str) -> String {
        let variant = self.sentinel_type_name(source);
        match self.sentinel_enum_name() {
            Some(group) => format!("{group}::{variant}"),
            None => variant,
        }
    }

    pub(crate) fn sentinel_type_name(&self, source: &str) -> String {
        let full = to_pascal_case(source);
        let Some(convention) = self.failure.filter(|c| !c.sentinel_prefix.is_empty()) else {
            return full;
        };
        let Some(rest) = full.strip_prefix(convention.sentinel_prefix.as_str()) else {
            return full;
        };
        let taken = self
            .scope
            .renames
            .iter()
            .any(|(other, target)| other != source && target == rest);
        match rest.is_empty() || taken {
            true => full,
            false => rest.to_owned(),
        }
    }

    /// Whether converting TO this source type is a plain cast in the target.
    ///
    /// The pack says which, keyed by source identity like every other table. Numeric conversion is
    /// defined to truncate in the source and the target's cast does the same thing; a conversion
    /// the pack does not list is one where the two languages disagree, and those refuse.
    pub(crate) fn converts_by_cast(&self, type_ref: &TypeRef) -> bool {
        self.cast_types.contains(&table_key(type_ref))
    }

    /// Whether a plain read of this source type COPIES in the target.
    pub(crate) fn copies(&self, type_ref: &TypeRef) -> bool {
        self.copy_types.contains(&table_key(type_ref))
    }

    /// The target expression for this source type's zero value, when the pack declares one.
    ///
    /// A COMPOSITE zero is a template, because it cannot be written without the type tree: an
    /// array's zero is its element's zero repeated to its length, and neither the element nor the
    /// length is knowable from the kind alone. `{0}` is the element's own zero and `{name}` the
    /// length the type node carries — the same two substitutions `type_constructors` makes, so a
    /// reader meets one convention rather than two.
    ///
    /// A template whose element has no zero yields `None` rather than a half-substituted string:
    /// an array of something with no zero has no zero either, and saying so is the point.
    pub(crate) fn zero_value(&self, type_ref: &TypeRef) -> Option<String> {
        let template = self.zero_values.get(&table_key(type_ref))?;
        if !template.contains('{') {
            return Some(template.clone());
        }
        let element = type_ref.args.first()?;
        Some(
            template
                .replace("{0}", &self.zero_value(element)?)
                .replace("{name}", &element_count(type_ref)),
        )
    }

    /// Whether the pack answers for a sequence literal of this type kind.
    pub(crate) fn is_sequence_literal(&self, kind: &str) -> bool {
        self.literal_constructors.contains_key(kind)
    }

    /// The target text for a sequence literal of this kind, given its elements.
    pub(crate) fn sequence_form(&self, kind: &str, elements: &[String]) -> Option<String> {
        Some(
            self.literal_constructors
                .get(kind)?
                .replace("{0}", &elements.join(", ")),
        )
    }

    /// The target spelling an idiom rule declares, if the pack declares that idiom.
    ///
    /// `None` means the pack does not carry the rule, and the source form is emitted unchanged —
    /// an idiom is a preference, and a pack that declines to state one has not made an error.
    pub(crate) fn idiom_method(&self, id: &str) -> Option<&str> {
        self.idioms
            .iter()
            .find(|rule| rule.id == id)
            .map(|rule| rule.method.as_str())
    }

    /// Whether this construction holds a source `string` as an OWNED target value.
    ///
    /// Asked of the same table the type resolution uses, so a literal and the position it lands in
    /// cannot disagree about who owns the text. `rust_const` overrides `string` to `&str`, and a
    /// constant's literal therefore stays borrowed.
    pub(crate) fn owns_strings(&self) -> bool {
        let borrowed = self
            .overrides
            .and_then(|overrides| overrides.get(SOURCE_STRING))
            .or_else(|| self.type_map.get(SOURCE_STRING));
        borrowed.is_some_and(|target| !target.starts_with('&'))
    }

    /// What the pack maps a source `string` to in this construction, when it is OWNED.
    ///
    /// `None` when the construction holds a borrowed spelling — `rust_const` overrides `string` to
    /// `&str`, and a constant's literal must stay borrowed.
    pub(crate) fn owned_string_target(&self) -> Option<&str> {
        let target = self
            .overrides
            .and_then(|overrides| overrides.get(SOURCE_STRING))
            .or_else(|| self.type_map.get(SOURCE_STRING))?;
        match target.starts_with('&') {
            true => None,
            false => Some(target),
        }
    }

    /// The traits a type with these field types EARNS.
    ///
    /// A derive is blocked by the KINDS the engine emits no type for — a trait object, a bare
    /// interface, a channel, a function. A field naming another emitted struct cannot block
    /// anything, because every emitted struct gets the same list, so intra-corpus references are
    /// satisfied by construction rather than by ordering the emission.
    ///
    /// Checked through the whole type TREE, not just its root: a `Vec<Box<dyn Error>>` is a slice
    /// whose element blocks, and looking only at `slice` would miss it.
    pub(crate) fn derives_for(&self, fields: &[TypeRef]) -> Vec<String> {
        self.derives
            .iter()
            .filter(|rule| {
                let mut seen = std::collections::BTreeSet::new();
                !fields
                    .iter()
                    .any(|field| self.blocks(field, &rule.blocked_by, &mut seen, &rule.name))
            })
            .map(|rule| rule.name.clone())
            .collect()
    }

    /// Whether this type blocks a derive, FOLLOWING references to this unit's own declarations.
    ///
    /// A named type of this unit earns a trait only if what it holds earns it, so a reference to one
    /// has to be answered by looking at that declaration rather than by assuming. The doc on
    /// `derives_for` used to record the opposite as safe, on the grounds that every emitted struct
    /// gets the same list — which a NEWTYPE breaks: one over a slice earns no total equality, and a
    /// struct holding it derived `Eq` and did not compile.
    ///
    /// `seen` makes a type that reaches itself terminate. A cycle is reached only through a pointer,
    /// and a type is not blocked by its own participation in one — the recursion just stops.
    fn blocks(
        &self,
        field: &TypeRef,
        blocked: &std::collections::BTreeSet<String>,
        seen: &mut std::collections::BTreeSet<String>,
        derive: &str,
    ) -> bool {
        // The FAILURE type is the one interface with a target form, and the pack says which derives
        // that form still earns. Without this the block on interfaces is total, and a type the
        // source satisfied its error interface with cannot meet the bound its own error impl needs.
        //
        // Asked at EVERY node rather than only at the root: the failure sits inside a sequence as
        // often as it stands alone — `[]error` is the shape that found this — and a check that only
        // looked at the outermost type answered for neither.
        let permitted = self
            .failure
            .is_some_and(|convention| convention.field_derives.iter().any(|kept| kept == derive));
        if self.mentions_blocked(field, blocked, permitted) {
            return true;
        }
        // Collected before descending: the visited set is threaded through the recursion, so a
        // borrow held by an iterator across the recursive call would be a second unique borrow.
        let fresh: Vec<String> = named_references(field)
            .into_iter()
            .filter(|name| seen.insert(name.clone()))
            .collect();
        for name in fresh {
            let Some(inputs) = self.scope.derive_inputs.get(&name) else {
                continue;
            };
            if inputs.iter().any(|input| self.blocks(input, blocked, seen, derive)) {
                return true;
            }
        }
        false
    }

    /// The target method carrying the source's overflow rule for this operation, if it has one.
    ///
    /// `None` for a comparison, for float or string arithmetic, and for any operator the pack does
    /// not govern — all of which keep the plain operator, because the rule they carry is the same
    /// in both languages. Only integer arithmetic differs, and only there is the spelling changed.
    pub(crate) fn wrapping_method(&self, node: &Declaration, spelling: &str) -> Option<&str> {
        if !self.integer_arithmetic.types.contains(&node.type_ref.name) {
            return None;
        }
        if divides_without_overflow(node, spelling) {
            return None;
        }
        self.integer_arithmetic
            .operators
            .get(spelling)
            .map(String::as_str)
    }
}

/// Whether any node of this type tree is one of the given blockers.
///
/// A blocker is a KIND — a trait object, a channel, a function — or the NAME of a BASIC type, which
/// is how a float blocks the total-equality derives. The name is consulted only for a basic type,
/// and that restriction is what keeps it safe: a basic type's name is one the language defines, so
/// it cannot collide with a user type that happens to be called `slice`.
///
/// Recursive, because a blocker is usually nested: a slice of trait objects blocks every derive its
/// element does, and a check that looked only at the outermost node would call it a plain slice.
fn mentions_kind(type_ref: &TypeRef, blockers: &std::collections::BTreeSet<String>) -> bool {
    blockers.contains(&type_ref.kind)
        || (type_ref.kind == "basic" && blockers.contains(&type_ref.name))
        || type_ref
            .args
            .iter()
            .any(|arg| mentions_kind(arg, blockers))
}

/// The length an array type carries, which the front end records in the type node's `name`.
///
/// A type node's `name` is where a non-type datum lives: for a named type it is the identity, and
/// for an array it is the length. Empty when the source wrote `[...]T{..}` and let the compiler
/// count, which is a shape the pack has no template for and which yields no zero.
fn element_count(type_ref: &TypeRef) -> String {
    type_ref.name.clone()
}

/// Whether this division or remainder provably cannot overflow, so the plain operator is exact.
///
/// Integer division overflows in exactly ONE case — the minimum value divided by negative one — so
/// a divisor that is a literal other than `-1` cannot reach it. `n.wrapping_div(2)` is then the
/// wrapping form applied where nothing wraps, which a reviewer reading the emitted crate called
/// out as mechanical rather than reasoned: the point of spelling arithmetic that way is to carry a
/// rule the target does not have, and where the target already agrees the spelling says nothing.
///
/// A NEGATIVE literal reaches here as a unary minus rather than as a literal, so it is not one of
/// the shapes matched and keeps the wrapping form — which is the conservative direction.
fn divides_without_overflow(node: &Declaration, spelling: &str) -> bool {
    if spelling != "/" && spelling != "%" {
        return false;
    }
    let Some(divisor) = node.children.get(1) else {
        return false;
    };
    divisor.kind == "literal" && divisor.attr("value").is_some_and(|value| value != "-1")
}

impl Resolver<'_> {
    /// Whether any node of this type tree is blocked, skipping the failure type where the pack says
    /// the derive survives it.
    fn mentions_blocked(
        &self,
        type_ref: &TypeRef,
        blocked: &std::collections::BTreeSet<String>,
        permitted: bool,
    ) -> bool {
        if permitted && self.is_failure_type(type_ref) {
            // Its target form is a boxed trait object: opaque, and already known to carry the
            // derive. Descending into what the source wrote inside it would answer about a type the
            // target does not have.
            return false;
        }
        if blocked.contains(&type_ref.kind)
            || (type_ref.kind == "basic" && blocked.contains(&type_ref.name))
        {
            return true;
        }
        type_ref
            .args
            .iter()
            .any(|arg| self.mentions_blocked(arg, blocked, permitted))
    }
}

/// Every name a type tree refers to, unqualified.
///
/// A reference to another declaration of this unit can sit at any depth — `*stack`, `[]frame`,
/// `map[string]entry` — so the whole tree is walked rather than only its root.
fn named_references(type_ref: &TypeRef) -> Vec<String> {
    let mut out = Vec::new();
    if !type_ref.name.is_empty() {
        out.push(
            type_ref
                .name
                .rsplit('.')
                .next()
                .unwrap_or(&type_ref.name)
                .to_owned(),
        );
    }
    for arg in &type_ref.args {
        out.extend(named_references(arg));
    }
    out
}
