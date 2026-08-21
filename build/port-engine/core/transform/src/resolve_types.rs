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
use crate::vocabulary::{TYPE_INTERFACE, TYPE_NAMED_INTERFACE};

impl Resolver<'_> {

    /// Refuse a type this unit declares and is not emitting.
    ///
    /// A declaration that REFUSED is not in the crate, so naming it produces a dangling reference —
    /// the same defect a call to a refused function produces, at the type layer. What is emitted
    /// has to be self-contained, and this is the third and last place a name can enter it.
    ///
    /// # Errors
    /// [`TransformError::UnmappedType`] naming the type and saying it refused.
    fn refuse_unemitted(
        &self,
        type_ref: &TypeRef,
        declaration_name: &str,
    ) -> Result<(), TransformError> {
        if self.emitted.contains(&type_ref.name) {
            return Ok(());
        }
        Err(TransformError::UnmappedType {
            unit: self.unit.0.clone(),
            name: declaration_name.to_owned(),
            type_ref: format!(
                "{} — declared in this unit and not emitted, because it refused. Naming it would \
                 name a type the crate does not contain",
                type_ref.describe()
            ),
        })
    }
    /// The path a named type resolves to, ignoring the question of how a position holds it.
    pub(crate) fn named_path(
        &self,
        type_ref: &TypeRef,
        declaration_name: &str,
    ) -> Result<RustType, TransformError> {
        if self.is_local(type_ref)
            && let Some(local) = self.scope.types.get(&type_ref.name)
        {
            self.refuse_unemitted(type_ref, declaration_name)?;
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
        self.foreign_path(type_ref, declaration_name)
    }

    /// The path another UNIT's type is addressed by, refusing a package that is not one.
    ///
    /// Every site that spells `crate::<module>::<Name>` goes through here, because the spelling is
    /// a CLAIM that the emitted crate has that module — true only for a unit of this model. Two
    /// sites spelled it and only one checked, so `&impl crate::fmt::State` reached the output for a
    /// package the crate has never heard of. One function, so a third caller cannot repeat it.
    ///
    /// # Errors
    /// [`TransformError::UnmappedType`] naming the package and what the pack owes.
    fn foreign_path(
        &self,
        type_ref: &TypeRef,
        declaration_name: &str,
    ) -> Result<RustType, TransformError> {
        if !self.units.contains(&type_ref.package) {
            // THE PACK MAY NAME IT. A library type the target has its own of -- `time.Duration` and
            // `std::time::Duration` -- is answered here, and every entry carries what it claims and
            // what that claim costs where the two do not line up exactly.
            if let Some(mapped) = self
                .foreign_types
                .get(&format!("{}.{}", type_ref.package, type_ref.name))
            {
                return Ok(RustType::path(mapped.form.clone()));
            }
            // The pack may have LOOKED at this type and decided it cannot be mapped, which is a
            // different answer from not having reached it. The target usually has a type of the
            // same name or the same rough purpose; what the reason records is how the shape differs.
            if let Some(reason) = self
                .unmappable_types
                .get(&format!("{}.{}", type_ref.package, type_ref.name))
            {
                return Err(TransformError::UnmappedType {
                    unit: self.unit.0.clone(),
                    name: declaration_name.to_owned(),
                    type_ref: format!(
                        "{} — no faithful target form, and the pack says why rather than leaving \
                         it to be guessed at: {reason}",
                        type_ref.describe()
                    ),
                });
            }
            return Err(TransformError::UnmappedType {
                unit: self.unit.0.clone(),
                name: declaration_name.to_owned(),
                type_ref: format!(
                    "{} — package `{}` is not in this snapshot, so the emitted crate has no \
                     module to reach it through. The pack has to map the type, as it maps the \
                     other types from libraries that do not come along",
                    type_ref.describe(),
                    type_ref.package
                ),
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
        // The source's BARE interface — `interface{}`, spelled `any` since go1.18 — which is not a
        // trait and not a type variable but a value carrying its own type at runtime. The target
        // has no counterpart, and the three things it might become each lose something different:
        // a type parameter fixes ONE type per call where the source admits a different one at every
        // call; `Box<dyn Any>` keeps the dynamism and loses every operation, since the source's
        // callers recover the value by type assertion and the target's must name the type to
        // downcast; and a purpose-built enum invents a closed set where the source has an open one.
        //
        // So this refuses by name rather than picking one. What is missing is a DECISION about what
        // the source's dynamic value becomes, and the decision is not the same for every use: the
        // 11 direct and 4 nested sites in the surveyed corpora are a type assertion helper, a
        // database scan target, and the variadic tail of a formatting call, which want different
        // answers. A single mapping would be wrong for at least two of the three.
        if type_ref.kind == TYPE_INTERFACE {
            return Err(TransformError::UnmappedType {
                unit: self.unit.0.clone(),
                name: declaration_name.to_owned(),
                type_ref: format!(
                    "{} — the source's bare interface is a value carrying its own type at runtime, \
                     and the target has no counterpart. A type parameter fixes one type per call \
                     where the source admits a different one at every call; `Box<dyn Any>` keeps \
                     the dynamism and loses every operation; an enum invents a closed set where the \
                     source has an open one. What is missing is a decision about which, and it is \
                     not the same decision for a type-assertion target as for a formatting argument",
                    type_ref.describe()
                ),
            });
        }

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
            self.refuse_unemitted(type_ref, declaration_name)?;
            return Ok(RustType::path(local.clone()));
        }

        // A named type from ANOTHER unit is addressed through that unit's emitted module. This is
        // the case a flat table could not express at all: the key it would need is qualified, and
        // the answer it would need is a path the pack does not know.
        if type_ref.kind == "named" && !type_ref.package.is_empty() && !self.is_local(type_ref) {
            if let Some(mapped) = self.lookup(&type_ref.qualified()) {
                return Ok(RustType::path(mapped));
            }
            return self.foreign_path(type_ref, declaration_name);
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
