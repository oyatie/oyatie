//! What the pack answers about POLICY, as opposed to what a type resolves to.
//!
//! Four questions live here and none of them is a type lookup: whether a construction holds a
//! source string as an owned value, which traits a ported type earns, and which target method
//! carries the source's overflow rule. They are grouped because they share a shape — each is the
//! pack declining to let the engine assume something — and separated from the type tables because
//! a reader chasing "what does `map[string]int` become" should not have to read past them.

use port_engine_api::{Declaration, IdiomRule, TypeRef};

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
                !fields
                    .iter()
                    .any(|field| mentions_kind(field, &rule.blocked_by))
            })
            .map(|rule| rule.name.clone())
            .collect()
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
