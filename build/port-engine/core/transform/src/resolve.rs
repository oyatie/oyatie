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

use port_engine_api::{Declaration, TypeRef, UnitId};
use port_engine_rust_ir::RustType;

use crate::error::TransformError;
use crate::naming::{module_path, to_pascal_case};
use crate::ownership::OwnershipContext;
use crate::vocabulary::TYPE_NAMED_INTERFACE;

/// The type names one unit declares, and the target spelling each resolves to.
pub(crate) struct LocalScope {
    types: BTreeMap<String, String>,
}

impl LocalScope {
    /// Every named declaration in the unit contributes its emitted name.
    ///
    /// Which kinds are type declarations is not decided here — deciding it would mean naming the
    /// source language's kind vocabulary in the neutral face. Every named declaration is recorded
    /// instead, and a collision is impossible because the front end already refuses two
    /// declarations sharing a name in one namespace.
    pub(crate) fn of(declarations: &[Declaration]) -> Self {
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

impl Resolver<'_> {
    /// Whether a plain read of this source type COPIES in the target.
    pub(crate) fn copies(&self, type_ref: &TypeRef) -> bool {
        self.copy_types.contains(&table_key(type_ref))
    }

    /// The target expression for this source type's zero value, when the pack declares one.
    pub(crate) fn zero_value(&self, type_ref: &TypeRef) -> Option<String> {
        self.zero_values.get(&table_key(type_ref)).cloned()
    }
}

/// The identity a pack table is keyed by.
///
/// `package.Name` for a named type, the bare name for a primitive, and the KIND for a composite
/// that has no name at all — the same three cases [`Resolver::resolve_node`] keys on, extracted so
/// that every table agrees on what a type is called.
fn table_key(type_ref: &TypeRef) -> String {
    let qualified = type_ref.qualified();
    if qualified.is_empty() {
        type_ref.kind.clone()
    } else {
        qualified
    }
}

impl Resolver<'_> {
    /// Resolve a source type to a target type.
    ///
    /// # Errors
    /// [`TransformError::UnmappedType`] when nothing answers for the type, and
    /// [`TransformError::MissingDatum`] when a declaration that needs a type carries none.
    pub(crate) fn resolve(
        &self,
        type_ref: &TypeRef,
        declaration_name: &str,
    ) -> Result<RustType, TransformError> {
        if type_ref.is_empty() {
            return Err(TransformError::MissingDatum {
                construction: "type resolution".to_owned(),
                name: declaration_name.to_owned(),
                datum: "type",
            });
        }
        self.resolve_node(type_ref, declaration_name)
    }

    /// Resolve a source type appearing in a named POSITION — a parameter, a result, a field.
    ///
    /// The position matters for exactly one kind. A trait has no size in the target, so it reaches
    /// a position as a reference, a box or a shared pointer, and those differ in who owns the value
    /// and how long it lives. The pack declares a form per position; a position it has no form for
    /// refuses, because choosing between them is an ownership decision and not a spelling.
    ///
    /// # Errors
    /// [`TransformError::UnmappedType`] when nothing answers for the type or for its position.
    pub(crate) fn resolve_in(
        &self,
        type_ref: &TypeRef,
        declaration_name: &str,
        position: &str,
    ) -> Result<RustType, TransformError> {
        if type_ref.kind != TYPE_NAMED_INTERFACE {
            return self.resolve(type_ref, declaration_name);
        }
        let Some(template) = self.trait_object_forms.get(position) else {
            return Err(TransformError::UnmappedType {
                unit: self.unit.0.clone(),
                name: declaration_name.to_owned(),
                type_ref: format!(
                    "{} in `{position}` position — a trait has no size in the target, and the pack \
                     declares no form for it there. Borrowing, boxing and sharing are different \
                     decisions about who owns the value",
                    type_ref.describe()
                ),
            });
        };
        let path = self.named_path(type_ref, declaration_name)?;
        Ok(RustType::path(template.replace("{0}", &path.spelling())))
    }

    /// The path a named type resolves to, ignoring the question of how a position holds it.
    fn named_path(
        &self,
        type_ref: &TypeRef,
        declaration_name: &str,
    ) -> Result<RustType, TransformError> {
        if self.is_local(type_ref)
            && let Some(local) = self.scope.types.get(&type_ref.name)
        {
            return Ok(RustType::path(local.clone()));
        }
        if let Some(mapped) = self.lookup(&table_key(type_ref)) {
            return Ok(RustType::path(mapped));
        }
        if type_ref.package.is_empty() {
            return Err(TransformError::UnmappedType {
                unit: self.unit.0.clone(),
                name: declaration_name.to_owned(),
                type_ref: type_ref.describe(),
            });
        }
        Ok(RustType::path(format!(
            "{}::{}",
            module_path(&type_ref.package),
            to_pascal_case(&type_ref.name)
        )))
    }

    fn resolve_node(
        &self,
        type_ref: &TypeRef,
        declaration_name: &str,
    ) -> Result<RustType, TransformError> {
        // A trait reaching here is a trait in a position the caller did not name — nested inside a
        // slice, a map, a pointer. `Vec<crate::shapes::Named>` names a trait as an element type and
        // does not compile, so this refuses rather than emitting it.
        if type_ref.kind == TYPE_NAMED_INTERFACE {
            return Err(TransformError::UnmappedType {
                unit: self.unit.0.clone(),
                name: declaration_name.to_owned(),
                type_ref: format!(
                    "{} nested inside another type — a trait has no size in the target, and a \
                     composite holding one needs its own rule",
                    type_ref.describe()
                ),
            });
        }

        // A name the unit itself declares wins over everything. It has to: a unit declaring a type
        // whose name collides with a mapped one would otherwise emit the mapped type in place of
        // its own, and the result compiles while meaning something else.
        if self.is_local(type_ref)
            && let Some(local) = self.scope.types.get(&type_ref.name)
        {
            return Ok(RustType::path(local.clone()));
        }

        // A named type from ANOTHER unit is addressed through that unit's emitted module. This is
        // the case a flat table could not express at all: the key it would need is qualified, and
        // the answer it would need is a path the pack does not know.
        if type_ref.kind == "named" && !type_ref.package.is_empty() && !self.is_local(type_ref) {
            if let Some(mapped) = self.lookup(&type_ref.qualified()) {
                return Ok(RustType::path(mapped));
            }
            return Ok(RustType::path(format!(
                "{}::{}",
                module_path(&type_ref.package),
                to_pascal_case(&type_ref.name)
            )));
        }

        // A primitive, or a named type the pack maps by identity.
        if let Some(mapped) = self.lookup(&table_key(type_ref)) {
            return Ok(RustType::path(mapped));
        }
        if let Some(mapped) = self.lookup(&type_ref.name) {
            return Ok(RustType::path(mapped));
        }

        // A composite: the pack answers for the CONSTRUCTOR and the arguments resolve recursively.
        if let Some(template) = self.constructors.get(&type_ref.kind) {
            let mut rendered = template.clone();
            for (index, arg) in type_ref.args.iter().enumerate() {
                let resolved = self.resolve_node(arg, declaration_name)?;
                rendered = rendered.replace(&format!("{{{index}}}"), &resolved.spelling());
            }
            if rendered.contains('{') {
                return Err(TransformError::UnmappedType {
                    unit: self.unit.0.clone(),
                    name: declaration_name.to_owned(),
                    type_ref: format!(
                        "{} — the pack's `{}` template expects more arguments than the type has",
                        type_ref.describe(),
                        type_ref.kind
                    ),
                });
            }
            return Ok(RustType::path(rendered));
        }

        Err(TransformError::UnmappedType {
            unit: self.unit.0.clone(),
            name: declaration_name.to_owned(),
            type_ref: type_ref.describe(),
        })
    }

    /// A named type is local when it has no package, or when its package IS this unit.
    fn is_local(&self, type_ref: &TypeRef) -> bool {
        type_ref.package.is_empty() || type_ref.package == self.unit.0
    }

    fn lookup(&self, key: &str) -> Option<String> {
        if key.is_empty() {
            return None;
        }
        self.overrides
            .and_then(|map| map.get(key))
            .or_else(|| self.type_map.get(key))
            .cloned()
    }
}
