//! Plan → `RustIr`, and the proof that nothing in the corpus was silently dropped.

use std::collections::{BTreeMap, BTreeSet};

use port_engine_api::{
    Declaration, PackSemantics, RegionId, RuleId, SourceModel, TransformPlan, UnitId,
};
use port_engine_rust_ir::{RustIr, RustItem};

use crate::error::TransformError;
use crate::signature_table::SignatureTable;
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

    // Regions in the order a READER should meet them: the declaration's own position in its unit
    // first, then the rule's precedence for the several regions one declaration can own. Plan order
    // alone is rule-major, which puts every struct before every constructor and separates a type
    // from the functions that build it; sorting by region id is worse still, because that is
    // alphabetical. A declaration with no position — a unit-level region — sorts first, which is
    // where a prelude belongs.
    let mut region_order: Vec<(isize, usize, String)> = Vec::new();
    let mut items: Vec<(String, RustItem)> = Vec::new();
    // One region may hold several items, because one declaration may emit several — a type
    // and the trait impls its observed satisfactions call for.
    let log = DispositionLog::new();
    let ownership = OwnershipContext::new(semantics.pointer_dispositions(), &log);
    // Every signature the engine can translate, once, so an argument site can ask what its
    // destination wants instead of guessing. A signature that does not translate is omitted, and
    // the declaration owning it refuses on its own terms when its turn comes.
    let signatures = SignatureTable::build(model, semantics, &ownership);
    // unit → the declaration kinds some applied rule captured, for the coverage check below.
    let mut captured_kinds: BTreeMap<UnitId, BTreeSet<String>> = BTreeMap::new();

    for (index, step) in plan.steps.iter().enumerate() {
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
            region_order.push((-1, index, region.clone()));
            items.push((region, item));
            continue;
        }

        let declarations =
            model
                .declarations(&step.unit)
                .ok_or_else(|| TransformError::UnitNotInModel {
                    unit: step.unit.0.clone(),
                })?;
        let scope = LocalScope::with_lengths(
            &declarations,
            semantics.failure_convention(),
            semantics.length_functions(),
            &semantics.format_calls().functions.keys().cloned().collect(),
            semantics.length_argument_callees(),
        );
        // EVERY name, because this pipeline requires every declaration to translate: one that does
        // not fails the whole run, so there is no fixpoint to compute and nothing to exclude.
        let declared_names: BTreeSet<String> = declarations
            .iter()
            .map(|declaration| declaration.name.clone())
            .collect();
        let entry = captured_kinds.entry(step.unit.clone()).or_default();
        for capture in captures {
            entry.insert(capture.clone());
        }

        for (position, declaration) in declarations
            .iter()
            .enumerate()
            .filter(|(_, d)| captures.contains(&d.kind))
        {
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
                    deferred: semantics.deferred_kinds(),
                    constant_map: semantics.constant_map(),
                    prose_type_names: semantics.prose_type_names(),
                    length_functions: semantics.length_functions(),
                    undecided_forms: semantics.undecided_forms(),
                    signatures: &signatures,
                    function_map: semantics.function_map(),
                    format_calls: semantics.format_calls(),
                    unmappable_calls: semantics.unmappable_calls(),
                    unmappable_types: semantics.unmappable_types(),
                    unmappable_facts: semantics.unmappable_facts(),
                    binary_string: semantics.binary_string(),
                    bit_pattern_constants: semantics.bit_pattern_constants(),
                    allocation: semantics.allocation(),
                    sequence_append: semantics.sequence_append(),
                    integer_arithmetic: semantics.integer_arithmetic(),
                    doc_convention: semantics.doc_convention(),
                    derives: semantics.derives(),
                    idioms: semantics.idioms(),
                    literal_constructors: semantics.literal_constructors(),
                    receiver: semantics.trait_receiver(),
                    ownership: &ownership,
                    emitted: &declared_names,
                    units: &model_units,
                    unit: &step.unit,
                },
            )?;
            provenance.insert(RegionId(region.clone()), step.unit.clone());
            region_order.push((
                isize::try_from(position).unwrap_or(isize::MAX),
                index,
                region.clone(),
            ));
            for one in built {
                items.push((region.clone(), one));
            }
        }
    }

    prove_every_declaration_is_accounted_for(plan, semantics, model, &captured_kinds)?;

    // The names each unit gives ITSELF, which no declaration owns: its prelude and its imports.
    // Synthesised after the loop because both are properties of the whole unit, and ordered ahead
    // of every declaration so a reader meets them first.
    crate::prelude::assemble(
        plan,
        semantics,
        model,
        &mut items,
        &mut provenance,
        &mut region_order,
    );

    region_order.sort_by_key(|(position, step, _)| (*position, *step));
    let region_names: Vec<String> = region_order.into_iter().map(|(_, _, name)| name).collect();
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
