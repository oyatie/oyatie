//! The resolver's TABLE lookups, and the one question that depends on POSITION.
//!
//! Kept apart from the type walk because they answer different questions. The walk asks what a type
//! IS in the target; these ask what the pack says about it — whether it copies, what its zero looks
//! like, what a trait becomes where it stands, and what a failure value becomes. Every one of them
//! is keyed the same way, and that shared key rule is the whole reason they are together: a table
//! that keyed a type differently from the others is how `copy_types` came to be checked against
//! target spellings while `type_map` was checked against source ones.

use port_engine_api::{Declaration, TypeRef};
use port_engine_rust_ir::RustType;

use crate::error::TransformError;
use crate::naming::{module_path, to_pascal_case, to_snake_case};
use crate::resolve::Resolver;
use crate::vocabulary::{SOURCE_STRING, TYPE_NAMED_INTERFACE};

impl Resolver<'_> {
    /// The target type a failure value becomes.
    ///
    /// # Errors
    /// [`TransformError::Unsupported`] when the pack declares no convention, which is how a source
    /// with a failure convention and a pack without one refuses instead of emitting an infallible
    /// signature for a function that can fail.
    pub(crate) fn failure_target(&self, declaration_name: &str) -> Result<String, TransformError> {
        self.failure
            .map(|convention| convention.target_type.clone())
            .ok_or_else(|| TransformError::Unsupported {
                name: declaration_name.to_owned(),
                detail: "the pack declares no failure convention, so a fallible signature has no \
                         target error type"
                    .to_owned(),
            })
    }

    /// The target path a free function's IDENTITY resolves to.
    ///
    /// A local function keeps its bare name; one from another unit is reached through that unit's
    /// emitted module, exactly as a type from another unit is — the same `module_path` both sides
    /// use, so a call and a type reference to the same unit cannot disagree.
    ///
    /// # Errors
    /// [`TransformError::Unsupported`] when the front end recorded no identity. That happens for a
    /// call to a value of function type, a conversion, or a method value — each a real shape with
    /// no path form, and each better refused by name than emitted as the source's own spelling,
    /// which would name nothing in the target.
    pub(crate) fn function_path(
        &self,
        identity: Option<&str>,
        declaration_name: &str,
    ) -> Result<String, TransformError> {
        let Some(identity) = identity.filter(|value| !value.is_empty()) else {
            return Err(TransformError::Unsupported {
                name: declaration_name.to_owned(),
                detail: "a call whose callee the front end could not identify — a value of \
                         function type, a conversion, or a method value — has no path form, and \
                         emitting the source's spelling would name nothing"
                    .to_owned(),
            });
        };

        // The pack answers first, for anything the target has no name of its own for.
        if let Some(mapped) = self.function_map.get(identity) {
            return Ok(mapped.form.clone());
        }

        let Some((package, name)) = identity.rsplit_once('.') else {
            // A builtin: no package, and the pack did not answer for it.
            return Err(TransformError::Unsupported {
                name: declaration_name.to_owned(),
                detail: format!(
                    "`{identity}` is a source builtin the pack does not map, and the target has no \
                     function of that name"
                ),
            });
        };

        if package == self.unit.0 {
            return Ok(to_snake_case(name));
        }
        Ok(format!("{}::{}", module_path(package), to_snake_case(name)))
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
