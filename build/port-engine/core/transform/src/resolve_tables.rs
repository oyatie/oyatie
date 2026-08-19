//! The resolver's TABLE lookups, and the one question that depends on POSITION.
//!
//! Kept apart from the type walk because they answer different questions. The walk asks what a type
//! IS in the target; these ask what the pack says about it — whether it copies, what its zero looks
//! like, what a trait becomes where it stands, and what a failure value becomes. Every one of them
//! is keyed the same way, and that shared key rule is the whole reason they are together: a table
//! that keyed a type differently from the others is how `copy_types` came to be checked against
//! target spellings while `type_map` was checked against source ones.

use port_engine_api::{Declaration, DeriveRule, TypeRef};
use port_engine_rust_ir::RustType;

use crate::error::TransformError;
use crate::naming::{module_path, to_pascal_case, to_snake_case};
use crate::resolve::Resolver;
use crate::vocabulary::{
    POSITION_PARAM, SOURCE_STRING, TYPE_ARRAY, TYPE_NAMED_INTERFACE,
};

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
    /// Whether the pack declares the idiom that an index-only name is the target's index type.
    ///
    /// Asked here so the parameter rule and the loop-counter rule are gated by the SAME declaration:
    /// a pack that drops it gets the conversions back in both places or in neither.
    pub(crate) fn idiom_index_counter(&self) -> Option<&str> {
        self.idiom_method(crate::vocabulary::IDIOM_INDEX_COUNTER)
    }

    /// The name this unit gives the failure type, when the pack declares one.
    ///
    /// `None` where it does not, and every signature then spells the type out — which is what the
    /// engine did before, and is still what a pack that sets no alias gets.
    pub(crate) fn failure_alias(&self) -> Option<&str> {
        self.failure
            .map(|convention| convention.alias.as_str())
            .filter(|alias| !alias.is_empty())
    }

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
        // The FAILURE type is the one interface the pack has already answered for, and its reason
        // covers every position that STORES the value: it chose an owned boxed form because a
        // failure outlives the call that produced it, so a reference would need a lifetime the
        // caller cannot supply. A field, a result and a composite element all have that problem.
        //
        // A PARAMETER does not, and is left refusing. The source's error is an interface value the
        // caller keeps after passing it, so owning it in the target would CONSUME a value the
        // source never consumed — the same reason a source string parameter borrows rather than
        // taking a `String`. What a borrowed failure parameter should be is a decision of its own.
        if self.is_failure_type(type_ref) && position != POSITION_PARAM {
            return self.failure_target(declaration_name).map(RustType::path);
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

    /// Whether this is the source type the pack's failure convention is about.
    ///
    /// By NAME rather than by kind, because the pack names the source type its convention governs
    /// and a pack that governs a different one must not be silently overruled here.
    pub(crate) fn is_failure_type(&self, type_ref: &TypeRef) -> bool {
        type_ref.kind == TYPE_NAMED_INTERFACE
            && self
                .failure
                .is_some_and(|convention| convention.source_type == type_ref.name)
    }

    /// A named type is local when it has no package, or when its package IS this unit.
    pub(crate) fn is_local(&self, type_ref: &TypeRef) -> bool {
        type_ref.package.is_empty() || type_ref.package == self.unit.0
    }

    pub(crate) fn lookup(&self, key: &str) -> Option<String> {
        if key.is_empty() {
            return None;
        }
        self.overrides
            .and_then(|map| map.get(key))
            .or_else(|| self.type_map.get(key))
            .cloned()
    }
}


/// The identity a pack table is keyed by.
///
/// `package.Name` for a named type, the bare name for a primitive, and the KIND for a composite
/// that has no name at all — the same three cases [`Resolver::resolve_node`] keys on, extracted so
/// that every table agrees on what a type is called.
///
/// An ARRAY is keyed by its kind DESPITE having a name, because that name is its LENGTH. A type
/// node's `name` carries whatever non-type datum the kind needs, and for every other kind that is
/// an identity; for an array it is a number. Falling through to it made `[4]int64` look up the key
/// `4`, so every table missed — silently, since a miss is indistinguishable from a type the pack
/// declines to answer for.
pub(crate) fn table_key(type_ref: &TypeRef) -> String {
    if type_ref.kind == TYPE_ARRAY {
        return type_ref.kind.clone();
    }
    let qualified = type_ref.qualified();
    if qualified.is_empty() {
        type_ref.kind.clone()
    } else {
        qualified
    }
}
