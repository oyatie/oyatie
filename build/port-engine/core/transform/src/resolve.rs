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

use port_engine_api::{Declaration, FailureConvention, TypeRef, UnitId};
use port_engine_rust_ir::RustType;

use crate::error::TransformError;
use crate::naming::{module_path, to_pascal_case};
use crate::ownership::OwnershipContext;
use crate::vocabulary::TYPE_NAMED_INTERFACE;

/// The type names one unit declares, and the target spelling each resolves to.
pub struct LocalScope {
    pub(crate) types: BTreeMap<String, String>,
}

impl LocalScope {
    /// Every named declaration in the unit contributes its emitted name.
    ///
    /// Which kinds are type declarations is not decided here — deciding it would mean naming the
    /// source language's kind vocabulary in the neutral face. Every named declaration is recorded
    /// instead, and a collision is impossible because the front end already refuses two
    /// declarations sharing a name in one namespace.
    pub fn of(declarations: &[Declaration]) -> Self {
        let mut types = BTreeMap::new();
        for declaration in declarations {
            if !declaration.name.is_empty() {
                types.insert(declaration.name.clone(), to_pascal_case(&declaration.name));
            }
        }
        Self { types }
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
    /// Source function identity → a target expression template.
    pub(crate) function_map: &'a BTreeMap<String, String>,
    /// How the source spells failure, when it has a convention for it.
    pub(crate) failure: Option<&'a FailureConvention>,
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
    /// The unit under transform, which decides whether a named type is local.
    pub(crate) unit: &'a UnitId,
}

impl<'a> Resolver<'a> {
    /// The pack's declared trait-receiver mode and its reason.
    pub(crate) fn trait_receiver(&self) -> Option<(&'a str, &'a str)> {
        self.receiver
    }
}
