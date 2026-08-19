//! Plan → `RustIr`, and the proof that nothing in the corpus was silently dropped.

use std::collections::{BTreeMap, BTreeSet};

use port_engine_api::{
    Declaration, PackSemantics, RegionId, RuleId, SourceModel, TransformPlan, UnitId,
};
use port_engine_rust_ir::{RustIr, RustItem};

use crate::error::TransformError;
use crate::items::{build_item, build_unit_item};
use crate::naming::{region_id_for, region_id_for_declaration};
use crate::ownership::{DispositionLog, DispositionRecord, OwnershipContext};
use crate::resolve::{LocalScope, Resolver};
use crate::vocabulary::{
    CONSTRUCTION_EMPTY_CANARY, CONSTRUCTION_PASS_THROUGH, PRECONDITION_UNIT_PRESENT,
};

/// Apply `plan` constructions against `model` using pack semantics → deterministic [`RustIr`].
///
/// Step order is plan order; within a declaration-level step, declaration order. Each emitted item
/// becomes one IR region with a syn AST.
///
/// Before returning, every declaration of every planned unit is checked to be either captured by a
/// rule or deferred by pack policy. That check is the difference between a translator and a filter:
/// without it a declaration no rule happens to select is dropped in silence, the emit is green, the
/// receipt is reproducible, and the only evidence that something was lost is that it is not there.
///
/// # Errors
/// [`TransformError`] on missing semantics, failed precondition, unknown construction, unresolvable
/// type, an uncaptured declaration, an unsupported construct, or an IR refusal.
pub fn apply(
    plan: &TransformPlan,
    semantics: &dyn PackSemantics,
    model: &dyn SourceModel,
) -> Result<RustIr, TransformError> {
    apply_with_provenance(plan, semantics, model).map(|output| output.ir)
}

/// Everything one transform run produced.
///
/// The dispositions travel WITH the IR rather than being recoverable from it. An ownership
/// decision is an inference over facts a reader cannot see from the emitted code — `&mut self`
/// looks the same whether it was proven or assumed — so dropping the reasoning would leave a
/// reviewer with the conclusion and no way to ask why.
#[derive(Debug)]
pub struct TransformOutput {
    /// The emitted IR.
    pub ir: RustIr,
    /// Which unit each region came from.
    pub region_units: BTreeMap<RegionId, UnitId>,
    /// Every ownership decision, in the order it was made.
    pub dispositions: Vec<DispositionRecord>,
}

/// [`apply`], plus which unit each emitted region came from.
///
/// The provenance is not derivable from a region id by parsing it. Region ids are built from
/// SANITIZED segments, and sanitization is lossy — two different unit ids can sanitize to the same
/// text, and a unit id containing adjacent non-alphanumerics produces a segment separator of its
/// own. Anything downstream that needs to group regions by unit (a module layout, a per-unit
/// output tree) must be told, not left to re-derive it from a string that no longer distinguishes.
///
/// # Errors
/// The same [`TransformError`] set as [`apply`].
pub fn apply_with_provenance(
    plan: &TransformPlan,
    semantics: &dyn PackSemantics,
    model: &dyn SourceModel,
) -> Result<TransformOutput, TransformError> {
    let model_units: BTreeSet<String> = model.units().into_iter().map(|u| u.0).collect();
    let mut provenance: BTreeMap<RegionId, UnitId> = BTreeMap::new();

    let mut region_names: Vec<String> = Vec::new();
    let mut items: Vec<(String, RustItem)> = Vec::new();
    // One region may hold several items, because one declaration may emit several — a type
    // and the trait impls its observed satisfactions call for.
    let log = DispositionLog::new();
    let ownership = OwnershipContext::new(semantics.pointer_dispositions(), &log);
    // unit → the declaration kinds some applied rule captured, for the coverage check below.
    let mut captured_kinds: BTreeMap<UnitId, BTreeSet<String>> = BTreeMap::new();

    for step in &plan.steps {
        let construction =
            semantics
                .construction(&step.rule)
                .ok_or_else(|| TransformError::MissingSemantics {
                    rule: step.rule.0.clone(),
                    field: "construction",
                })?;
        let precondition =
            semantics
                .precondition(&step.rule)
                .ok_or_else(|| TransformError::MissingSemantics {
                    rule: step.rule.0.clone(),
                    field: "precondition",
                })?;
        let captures =
            semantics
                .captures(&step.rule)
                .ok_or_else(|| TransformError::MissingSemantics {
                    rule: step.rule.0.clone(),
                    field: "captures",
                })?;

        check_precondition(precondition, &step.unit, &step.rule, &model_units)?;

        if captures.is_empty() {
            let region = region_id_for(&step.unit, &step.rule);
            let item = build_unit_item(construction, &region).ok_or_else(|| {
                TransformError::UnknownConstruction {
                    rule: step.rule.0.clone(),
                    construction: construction.to_owned(),
                }
            })?;
            provenance.insert(RegionId(region.clone()), step.unit.clone());
            region_names.push(region.clone());
            items.push((region, item));
            continue;
        }

        let declarations =
            model
                .declarations(&step.unit)
                .ok_or_else(|| TransformError::UnitNotInModel {
                    unit: step.unit.0.clone(),
                })?;
        let scope = LocalScope::of(&declarations);
        let entry = captured_kinds.entry(step.unit.clone()).or_default();
        for capture in captures {
            entry.insert(capture.clone());
        }

        for declaration in declarations.iter().filter(|d| captures.contains(&d.kind)) {
            let region = region_id_for_declaration(&step.unit, &step.rule, &declaration.name);
            let built = build_item(
                construction,
                declaration,
                &Resolver {
                    scope: &scope,
                    type_map: semantics.type_map(),
                    overrides: semantics.type_map_overrides(construction),
                    constructors: semantics.type_constructors(),
                    copy_types: semantics.copy_types(),
                    cast_types: semantics.cast_types(),
                    zero_values: semantics.zero_values(),
                    trait_object_forms: semantics.trait_object_forms(),
                    failure: semantics.failure_convention(),
                    function_map: semantics.function_map(),
                    receiver: semantics.trait_receiver(),
                    ownership: &ownership,
                    unit: &step.unit,
                },
            )?;
            provenance.insert(RegionId(region.clone()), step.unit.clone());
            region_names.push(region.clone());
            for one in built {
                items.push((region.clone(), one));
            }
        }
    }

    prove_every_declaration_is_accounted_for(plan, semantics, model, &captured_kinds)?;

    let refs: Vec<&str> = region_names.iter().map(String::as_str).collect();
    let mut ir = RustIr::new(&refs);
    // Grouping by region, because a declaration emits a LIST: a type that satisfies an interface
    // emits the type and an impl per satisfaction, and they belong to the one region the
    // declaration owns.
    let mut grouped: BTreeMap<String, Vec<RustItem>> = BTreeMap::new();
    for (region, item) in items {
        grouped.entry(region).or_default().push(item);
    }
    for (region, region_items) in grouped {
        ir.set_items(&region, region_items)
            .map_err(TransformError::Ir)?;
    }
    Ok(TransformOutput {
        ir,
        region_units: provenance,
        dispositions: log.records(),
    })
}

/// Every declaration of every planned unit must be captured by a rule or deferred by policy.
///
/// Deferral is DECLARED, not inferred. A kind the pack lists in `deferred_kinds` is one someone
/// wrote down as knowingly untranslated, with the reason travelling in the pack and therefore in
/// the pack digest and therefore in the receipt. A kind that is merely unselected is indisputably
/// lost work, and it must not look like a decision.
fn prove_every_declaration_is_accounted_for(
    plan: &TransformPlan,
    semantics: &dyn PackSemantics,
    model: &dyn SourceModel,
    captured_kinds: &BTreeMap<UnitId, BTreeSet<String>>,
) -> Result<(), TransformError> {
    let deferred = semantics.deferred_kinds();
    let planned_units: BTreeSet<&UnitId> = plan.steps.iter().map(|step| &step.unit).collect();

    for unit in planned_units {
        let Some(declarations) = model.declarations(unit) else {
            continue;
        };
        let empty = BTreeSet::new();
        let captured = captured_kinds.get(unit).unwrap_or(&empty);
        for declaration in &declarations {
            if captured.contains(&declaration.kind) || deferred.contains(&declaration.kind) {
                continue;
            }
            return Err(TransformError::UncapturedDeclaration {
                unit: unit.0.clone(),
                name: declaration.name.clone(),
                kind: declaration.kind.clone(),
            });
        }
    }
    Ok(())
}

fn check_precondition(
    precondition: &str,
    unit: &UnitId,
    rule: &RuleId,
    model_units: &BTreeSet<String>,
) -> Result<(), TransformError> {
    match precondition {
        PRECONDITION_UNIT_PRESENT => {
            if model_units.contains(&unit.0) {
                Ok(())
            } else {
                Err(TransformError::Precondition {
                    rule: rule.0.clone(),
                    unit: unit.0.clone(),
                    precondition: precondition.to_owned(),
                })
            }
        }
        other => Err(TransformError::Precondition {
            rule: rule.0.clone(),
            unit: unit.0.clone(),
            precondition: other.to_owned(),
        }),
    }
}
