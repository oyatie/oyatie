//! What the pack answers about POLICY, as opposed to what a type resolves to.
//!
//! Four questions live here and none of them is a type lookup: whether a construction holds a
//! source string as an owned value, which traits a ported type earns, and which target method
//! carries the source's overflow rule. They are grouped because they share a shape — each is the
//! pack declining to let the engine assume something — and separated from the type tables because
//! a reader chasing "what does `map[string]int` become" should not have to read past them.

use port_engine_api::{Declaration, IdiomRule, TypeRef};

use crate::resolve::Resolver;
use crate::vocabulary::SOURCE_STRING;

impl Resolver<'_> {
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
        self.integer_arithmetic
            .operators
            .get(spelling)
            .map(String::as_str)
    }
}

/// Whether any node of this type tree has one of the given kinds.
///
/// Recursive, because a blocking kind is usually nested: a slice of trait objects blocks every
/// derive its element does, and a check that looked only at the outermost node would call it a
/// plain slice.
fn mentions_kind(type_ref: &TypeRef, kinds: &std::collections::BTreeSet<String>) -> bool {
    kinds.contains(&type_ref.kind)
        || type_ref
            .args
            .iter()
            .any(|arg| mentions_kind(arg, kinds))
}
