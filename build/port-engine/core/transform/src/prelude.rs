//! The names a UNIT gives itself, which no declaration owns.
//!
//! One so far, and it is the longest thing in most emitted signatures: the failure type. Spelled
//! out, `Box<dyn std::error::Error + Send + Sync>` appears in every fallible signature a unit has —
//! eight times in a four-hundred-line module — and reviewers named it twice as the type any author
//! writing it a third time would have aliased.
//!
//! A prelude is not a declaration and has no source of its own, so it is synthesised rather than
//! built from one. What decides it is a property of the WHOLE unit — whether anything in it can
//! fail — which is why it cannot be a per-declaration rule.
//!
//! Emitted only where the unit needs it, so a module that never fails does not gain a name it never
//! uses; and the NAME and the reason are the pack's, so a pack that sets none gets the type spelled
//! out at every site exactly as before.

use port_engine_api::{PackSemantics, SourceModel, TransformPlan, UnitId};
use port_engine_rust_ir::{RustItem, RustType, Visibility};

/// The prelude item each planned unit needs, for the units that need one.
pub(crate) fn preludes(
    plan: &TransformPlan,
    semantics: &dyn PackSemantics,
    model: &dyn SourceModel,
) -> Vec<(UnitId, RustItem)> {
    let Some(convention) = semantics.failure_convention() else {
        return Vec::new();
    };
    if convention.alias.is_empty() {
        return Vec::new();
    }
    let mut units: Vec<&UnitId> = plan.steps.iter().map(|step| &step.unit).collect();
    units.sort_by(|a, b| a.0.cmp(&b.0));
    units.dedup();

    units
        .into_iter()
        .filter(|unit| unit_can_fail(unit, semantics, model))
        .map(|unit| {
            (
                unit.clone(),
                RustItem::TypeAlias {
                    // The alias documents itself: what it names is in the type, and a comment
                    // saying "the crate's result type" adds nothing a reader cannot see.
                    docs: Vec::new(),
                    vis: Visibility::Public,
                    name: convention.alias.clone(),
                    generics: vec!["T".to_owned()],
                    // FULLY QUALIFIED on the right, because the alias shadows the prelude's own
                    // `Result` inside this module and a bare one here would name itself.
                    ty: RustType::path(format!(
                        "std::result::Result<T, {}>",
                        convention.target_type
                    )),
                },
            )
        })
        .collect()
}

/// Whether anything this unit declares can fail.
///
/// Asked of the DECLARATIONS rather than of what was emitted, because the prelude is decided before
/// any of them is built — and a unit whose fallible declarations all refuse still declared them,
/// so a name it does not end up using is a smaller wrong than a name it needs and lacks.
fn unit_can_fail(unit: &UnitId, semantics: &dyn PackSemantics, model: &dyn SourceModel) -> bool {
    model.declarations(unit).is_some_and(|declarations| {
        declarations.iter().any(|declaration| {
            crate::failure::is_fallible(declaration, semantics.failure_convention())
                || declaration.children.iter().any(|member| {
                    crate::failure::is_fallible(member, semantics.failure_convention())
                })
        })
    })
}
