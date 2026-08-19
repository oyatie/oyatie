//! Resolving a source type TREE, which is a different question from looking one up.
//!
//! Split from `resolve_tables.rs` because the two answer different things. That file asks the
//! pack's flat tables what a name means; this one walks a type built out of other types and
//! decides what each layer becomes — a named type in this unit, a named type in another, a
//! composite whose constructor the pack declares, and the interface that is not sized at all.
//!
//! Nothing is guessed at any layer. Passing an unresolved spelling through would produce code
//! that either fails to compile far from its cause or, worse, compiles as an unrelated target
//! type that happens to share a name.

use port_engine_api::TypeRef;
use port_engine_rust_ir::RustType;

use crate::error::TransformError;
use crate::naming::{module_path, to_pascal_case};
use crate::resolve::Resolver;
use crate::resolve_tables::table_key;
use crate::vocabulary::TYPE_NAMED_INTERFACE;

impl Resolver<'_> {
    /// The path a named type resolves to, ignoring the question of how a position holds it.
    pub(crate) fn named_path(
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

    pub(crate) fn resolve_node(
        &self,
        type_ref: &TypeRef,
        declaration_name: &str,
    ) -> Result<RustType, TransformError> {
        // Nested inside a composite, which STORES the value: same answer, same reason.
        if self.is_failure_type(type_ref) {
            return self.failure_target(declaration_name).map(RustType::path);
        }

        // Any OTHER trait reaching here is one in a position the caller did not name — nested
        // inside a slice, a map, a pointer. `Vec<crate::shapes::Named>` names a trait as an element
        // type and does not compile, so this refuses rather than emitting it.
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
            // `{name}` is the type's own non-type datum — an array's LENGTH, a channel's direction.
            // Without it an array could only be rendered as something that forgets how long it is,
            // and forgetting is not a spelling difference: a fixed-size value that copies becomes a
            // heap allocation that moves.
            rendered = rendered.replace("{name}", &type_ref.name);
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

}
