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

use std::collections::BTreeMap;

use port_engine_api::{Declaration, TypeRef, UnitId};
use port_engine_rust_ir::RustType;

use crate::error::TransformError;
use crate::naming::{module_path, to_pascal_case};

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
    /// The declared trait-receiver mode and its reason.
    pub(crate) receiver: Option<(&'a str, &'a str)>,
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

    fn resolve_node(
        &self,
        type_ref: &TypeRef,
        declaration_name: &str,
    ) -> Result<RustType, TransformError> {
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
        let key = if type_ref.qualified().is_empty() {
            type_ref.kind.clone()
        } else {
            type_ref.qualified()
        };
        if let Some(mapped) = self.lookup(&key) {
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
