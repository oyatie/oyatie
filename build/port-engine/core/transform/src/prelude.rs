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

use std::collections::{BTreeMap, BTreeSet};

use port_engine_api::{PackSemantics, RegionId, SourceModel, TransformPlan, UnitId};
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

    let _ = convention;
    units
        .into_iter()
        .flat_map(|unit| {
            prelude_items(unit, semantics, model)
                .into_iter()
                .map(move |item| (unit.clone(), item))
        })
        .collect()
}

/// The prelude ONE unit needs, or none.
///
/// Per-unit because two callers ask it and neither can use the other's shape: the plan-driven
/// assembly walks planned steps, and the survey walks the model directly with no plan at all. They
/// asked different questions once, and the survey's emitted packages silently lacked the alias
/// every fallible signature in them refers to.
pub(crate) fn prelude_items(
    unit: &UnitId,
    semantics: &dyn PackSemantics,
    model: &dyn SourceModel,
) -> Vec<RustItem> {
    let Some(convention) = semantics.failure_convention() else {
        return Vec::new();
    };
    if convention.alias.is_empty() || !unit_can_fail(unit, semantics, model) {
        return Vec::new();
    }
    // NOT UNDER A NAME THIS UNIT ALREADY DECLARES. `tidwall/gjson` declares `type Result struct`,
    // and the pack's alias is also `Result` — two public items of one name in one module, which is
    // a redefinition error if both are emitted and a silent shadow of whichever loses if they are
    // not. The unit's own type is the SOURCE'S CONTRACT and the alias is this engine's convenience,
    // so the alias is the one that yields.
    //
    // A unit that both declares the name and needs the alias refuses at the point it needs it,
    // rather than being given an alias under an invented second name — which would put a spelling
    // in the emitted API that no reader of the source could predict.
    let declares_alias = |name: &str| {
        model
            .declarations(unit)
            .iter()
            .flatten()
            .any(|declaration| crate::naming::to_pascal_case(&declaration.name) == name)
    };
    if declares_alias(&convention.alias) {
        return Vec::new();
    }
    // The failure type gets a NAME of its own when the pack gives it one, and the result alias then
    // defaults to that name rather than to the spelling. Both aliases fit on a line; the one that
    // carried the spelling in its default did not, and broke across four.
    let failure = match convention.boxed_alias.is_empty() {
        true => convention.target_type.clone(),
        false => convention.boxed_alias.clone(),
    };
    let mut items = Vec::new();
    if !convention.boxed_alias.is_empty() {
        items.push(RustItem::TypeAlias {
            docs: Vec::new(),
            vis: Visibility::Public,
            name: convention.boxed_alias.clone(),
            generics: Vec::new(),
            ty: RustType::path(convention.target_type.clone()),
        });
    }
    items.push(RustItem::TypeAlias {
        // The alias documents itself: what it names is in the type, and a comment saying "the
        // crate's result type" adds nothing a reader cannot see.
        docs: Vec::new(),
        vis: Visibility::Public,
        name: convention.alias.clone(),
        // The failure slot is a PARAMETER with a default, not a fixed type. Both spellings mean
        // the same thing at every call site the engine emits — `Result<T>` resolves the default —
        // and only this one can also be written with a different failure where a caller has one.
        // Without it the alias is a fixed shape wearing a type parameter, which a reviewer read as
        // an alias that could not be used for anything but what generated it.
        generics: vec!["T".to_owned(), format!("E = {failure}")],
        // FULLY QUALIFIED on the right, because the alias shadows the prelude's own `Result`
        // inside this module and a bare one here would name itself.
        ty: RustType::path("std::result::Result<T, E>".to_owned()),
    });
    items
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

/// The imports each unit needs, read from the items it actually EMITTED.
///
/// Derived from the output rather than from the declarations, unlike the prelude above, and for a
/// reason the compile proof enforces: an unused type alias is dead code and allowed, where an
/// unused import is a warning and denied. A unit whose sentinels all refused must not gain an
/// import for them.
///
/// One entry so far. The sentinel form spells `fmt::Display`, `fmt::Formatter` and `fmt::Result`,
/// three names per sentinel, and a unit with seven of them names one std module twenty-one times.
/// The short form and this import are one decision — the lowering cannot emit one without the
/// other — so they are derived from the same fact and cannot drift apart.
pub(crate) fn imports(
    items: &[(String, RustItem)],
    provenance: &BTreeMap<RegionId, UnitId>,
    declared: &BTreeMap<String, String>,
) -> Vec<(UnitId, RustItem)> {
    let mut by_unit: BTreeMap<UnitId, Vec<RustItem>> = BTreeMap::new();
    for (region, item) in items {
        if let Some(unit) = provenance.get(&RegionId(region.clone())) {
            by_unit.entry(unit.clone()).or_default().push(item.clone());
        }
    }
    by_unit
        .into_iter()
        .flat_map(|(unit, unit_items)| {
            crate::emitted_names::import_items(&unit_items, declared)
                .into_iter()
                .map(move |item| (unit.clone(), item))
        })
        .collect()
}

/// Place each unit's prelude and imports into the assembly, ordered ahead of its declarations.
///
/// Here rather than in the assembly itself because the ORDER between them is part of this
/// decision: imports at -2 and the prelude at -1, so a reader meets `use` lines before the alias,
/// which is where a person writing the module by hand would have put them.
pub(crate) fn assemble(
    plan: &TransformPlan,
    semantics: &dyn PackSemantics,
    model: &dyn SourceModel,
    items: &mut Vec<(String, RustItem)>,
    provenance: &mut BTreeMap<RegionId, UnitId>,
    order: &mut Vec<(isize, usize, String)>,
) {
    // Imports LAST, and that is an ORDER rather than a preference: they are read from what the unit
    // emitted, and the prelude is one of the things it emits. Built one after the other rather than
    // from a list of both, because a list evaluates its elements before the first is placed — which
    // had the import scan looking at a unit that did not have its aliases yet.
    let prelude = preludes(plan, semantics, model);
    for (what, position, produced) in [("prelude", -1, prelude)] {
        for (unit, item) in produced {
            let region = crate::naming::region_id_for_unit(&unit, what);
            // ONCE per region, however many items land in it. A unit's prelude is two aliases now,
            // and registering the region per item put it in the order twice — which renders the
            // whole region twice and defines every name in it twice.
            if provenance
                .insert(RegionId(region.clone()), unit)
                .is_none()
            {
                order.push((position, 0, region.clone()));
            }
            items.push((region, item));
        }
    }

    let declared = semantics.target_imports();
    for (unit, item) in imports(items, provenance, declared) {
        let region = crate::naming::region_id_for_unit(&unit, "imports");
        if provenance
            .insert(RegionId(region.clone()), unit)
            .is_none()
        {
            order.push((-2, 0, region.clone()));
        }
        items.push((region, item));
    }
}
