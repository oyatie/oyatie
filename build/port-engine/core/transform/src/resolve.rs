//! Source type spelling → target type spelling.
//!
//! Nothing is guessed. A spelling the unit does not declare and the pack does not map REFUSES,
//! because passing it through produces code that either fails to compile far from its cause or
//! compiles as an unrelated type that happens to share a name.

use std::collections::BTreeMap;

use port_engine_api::{Declaration, UnitId};

use crate::error::TransformError;
use crate::naming::to_pascal_case;

/// The type names one unit declares, and the target spelling each resolves to.
pub(crate) struct LocalScope {
    pub(crate) types: BTreeMap<String, String>,
}

impl LocalScope {
    /// A declaration whose kind carries a type name contributes that name to the unit's scope.
    ///
    /// Which kinds those are is not decided here — a declaration is a type declaration exactly
    /// when it has a name and is not one of the value-shaped kinds — because deciding it here
    /// would mean naming the source language's kind vocabulary in the neutral face. What this does
    /// instead is record every named declaration, and let a collision be impossible by the
    /// front end's own single-namespace refusal.
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

pub(crate) struct Resolver<'a> {
    pub(crate) scope: &'a LocalScope,
    pub(crate) type_map: &'a BTreeMap<String, String>,
    pub(crate) overrides: Option<&'a BTreeMap<String, String>>,
    pub(crate) unit: &'a UnitId,
}

impl Resolver<'_> {
    /// Resolve a source type spelling to its target spelling.
    ///
    /// A name the unit itself declares wins over the pack's map. It has to: the map is keyed by
    /// spelling, so a unit declaring a type whose name collides with a mapped one would otherwise
    /// silently emit the mapped type in place of its own, and the emitted code would compile while
    /// meaning something different.
    ///
    /// Nothing is guessed. An unresolvable spelling refuses, because the alternative — passing the
    /// source spelling through and hoping it is also valid target syntax — produces code that
    /// either fails to compile far from the cause or, worse, compiles as some unrelated type that
    /// happens to share a name.
    pub(crate) fn resolve(
        &self,
        type_ref: &str,
        declaration_name: &str,
    ) -> Result<String, TransformError> {
        if type_ref.is_empty() {
            return Err(TransformError::MissingDatum {
                construction: "type resolution".to_owned(),
                name: declaration_name.to_owned(),
                datum: "type",
            });
        }
        if let Some(local) = self.scope.types.get(type_ref) {
            return Ok(local.clone());
        }
        if let Some(mapped) = self.overrides.and_then(|map| map.get(type_ref)) {
            return Ok(mapped.clone());
        }
        if let Some(mapped) = self.type_map.get(type_ref) {
            return Ok(mapped.clone());
        }
        Err(TransformError::UnmappedType {
            unit: self.unit.0.clone(),
            name: declaration_name.to_owned(),
            type_ref: type_ref.to_owned(),
        })
    }
}
