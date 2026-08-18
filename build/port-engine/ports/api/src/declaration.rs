//! What a unit declares, as one uniform recursive node.

use std::collections::{BTreeMap, BTreeSet};

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
