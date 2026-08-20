//! What this engine can and cannot do with a source tree it has never seen.
//!
//! [`crate::apply`] is fail-closed: the first construct it cannot translate ends the run. That is
//! the right behaviour for PRODUCING a port — a partial port that compiles is worse than no port,
//! because nothing downstream can tell which half is real.
//!
//! It is the wrong behaviour for MEASURING one. Pointed at a real third-party package, `apply`
//! reports the first refusal and says nothing about the other nine hundred declarations, so the
//! engine's own maturity can only be discovered one construct at a time. A survey attempts every
//! declaration independently and reports all of it, which turns "what is missing" from a guess into
//! a ranked list.
//!
//! Two things this deliberately does NOT do, because both would make the number flattering:
//!
//! - it does not consult the pack's `applies` map. That map is policy — which units this programme
//!   has decided to port — and a survey asks a capability question instead: what COULD the engine
//!   do with this source. A survey restricted to units somebody already listed would measure the
//!   list.
//! - it does not count a refusal as a partial success. A declaration is translated or it is not,
//!   and the reason is carried verbatim so the ranking is by real cause rather than by category.

use std::collections::BTreeMap;

use std::collections::BTreeSet;

use port_engine_api::{Declaration, PackSemantics, RulePack, SourceModel, UnitId};

use crate::error::TransformError;
use crate::signature_table::SignatureTable;
use crate::items::build_item;
use crate::ownership::{DispositionLog, OwnershipContext};
use crate::resolve::{LocalScope, Resolver};
use crate::survey_report::{PortedRegion, SurveyEntry, SurveyReport};

/// Attempt every declaration in `model` independently and report what happened to each.
#[must_use]
pub fn survey<P>(model: &dyn SourceModel, pack: &P) -> SurveyReport
where
    P: RulePack + PackSemantics,
{
    let mut report = SurveyReport {
        translated: Vec::new(),
        refused: Vec::new(),
        uncaptured: Vec::new(),
        deferred: Vec::new(),
        ported: Vec::new(),
    };

    let rules = pack.rules();
    // One translation of every signature in the model, so an argument site can ask what its
    // destination wants. Built here rather than per declaration: it is a property of the whole
    // model, and rebuilding it each time would make the survey quadratic in the corpus.
    let signature_log = DispositionLog::new();
    let signature_ownership = OwnershipContext {
        rules: pack.pointer_dispositions(),
        log: &signature_log,
    };
    let signatures = SignatureTable::build(model, pack, &signature_ownership);
    // Every module the emitted crate will have. A name from outside them has nothing to be reached
    // through, and emitting a path for it produces a crate that does not build.
    let units: BTreeSet<String> = model.units().into_iter().map(|unit| unit.0).collect();

    // THE FIXPOINT. A declaration is emitted only if everything it names is emitted, and refusing
    // one may make another refuse — a chain of calls falls together. Starting from "everything is
    // emittable" and SHRINKING is what makes it converge: the set only ever loses members, so it
    // terminates in at most one round per declaration and in practice in two or three.
    //
    // Starting from empty and growing would also converge and would be wrong: it would refuse a
    // pair of mutually recursive functions that both translate perfectly well.
    let every = BTreeSet::new();
    let mut emittable = crate::reachable::emittable_names(model);
    loop {
        let shrunk =
            crate::reachable::shrink(model, pack, &rules, &signatures, &units, &emittable);
        if shrunk == emittable {
            break;
        }
        emittable = shrunk;
    }

    for unit in model.units() {
        let Some(declarations) = model.declarations(&unit) else {
            continue;
        };
        let scope = LocalScope::with_lengths(
            &declarations,
            pack.failure_convention(),
            pack.length_functions(),
            &pack.format_calls().functions.keys().cloned().collect(),
            pack.length_argument_callees(),
        );
        for (position, declaration) in declarations.iter().enumerate() {
            survey_declaration(
                &Site {
                    units: &units,
                    unit: &unit,
                    emitted: emittable.get(&unit.0).unwrap_or(&every),
                    position,
                    scope: &scope,
                },
                declaration,
                &rules,
                pack,
                &signatures,
                &mut report,
            );
        }
    }

    // The unit's PRELUDE and its IMPORTS, which belong to no declaration. The survey emits real
    // packages, so it needs both exactly as the assembly path does — and it had neither, which is
    // why every package `port` emitted spelled the failure type out in full and named `std::fmt`
    // three times per sentinel.
    for unit in model.units() {
        // Against what this unit ACTUALLY emitted, not against what its source could do. A unit
        // whose every fallible function refused needs no failure alias, and gaining one anyway is
        // an engine introduction that nothing asked for.
        let already: Vec<port_engine_rust_ir::RustItem> = report
            .ported
            .iter()
            .filter(|region| region.unit == unit)
            .flat_map(|region| region.items.clone())
            .collect();
        let prelude =
            crate::emitted_names::retain_used(crate::prelude::prelude_items(&unit, pack, model), &already);
        if !prelude.is_empty() {
            report.ported.push(PortedRegion {
                region: crate::naming::region_id_for_unit(&unit, "prelude"),
                unit: unit.clone(),
                position: -1,
                items: prelude,
            });
        }
        // Read from what this unit ACTUALLY emitted, which is the only evidence that cannot produce
        // an import nothing uses.
        let emitted: Vec<port_engine_rust_ir::RustItem> = report
            .ported
            .iter()
            .filter(|region| region.unit == unit)
            .flat_map(|region| region.items.clone())
            .collect();
        let imports = crate::emitted_names::import_items(&emitted, pack.target_imports());
        if !imports.is_empty() {
            report.ported.push(PortedRegion {
                region: crate::naming::region_id_for_unit(&unit, "imports"),
                unit,
                position: -2,
                items: imports,
            });
        }
    }
    report
}

/// Where one declaration SITS: which unit, which position in it, and what names that unit has.
///
/// One value rather than three parameters, because they travel together and always will: a
/// declaration is only ever surveyed in the context of the unit it belongs to.
pub(crate) struct Site<'a> {
    pub(crate) unit: &'a UnitId,
    pub(crate) units: &'a BTreeSet<String>,
    /// The names of this unit still believed to be emittable, this round.
    pub(crate) emitted: &'a BTreeSet<String>,
    pub(crate) position: usize,
    pub(crate) scope: &'a LocalScope,
}

/// The kind a dropped method is reported under, and the kind the fixpoint reads back to tell a
/// method's refusal apart from its type's. One spelling, so the two cannot disagree.
pub(crate) const KIND_METHOD_ENTRY: &str = "method";

pub(crate) fn survey_declaration<P>(
    site: &Site<'_>,
    declaration: &Declaration,
    rules: &[port_engine_api::RuleId],
    pack: &P,
    // Built once for the whole model, not per declaration: it is a translation of every signature
    // in the snapshot, and rebuilding it for each one would make the survey quadratic in a corpus.
    signatures: &SignatureTable,
    report: &mut SurveyReport,
) where
    P: RulePack + PackSemantics,
{
    let entry = |reason: Option<String>| SurveyEntry {
        unit: site.unit.0.clone(),
        name: declaration.name.clone(),
        kind: declaration.kind.clone(),
        reason,
    };

    // The LAST rule in pack order that captures this kind. Pack order is precedence order, so the
    // later rule is the more specific one — `rust_struct_body` over `rust_struct`, which is the
    // difference between measuring what the engine can do and what it could do before bodies.
    let Some((rule, construction)) = rules
        .iter()
        .filter(|rule| {
            pack.captures(rule)
                .is_some_and(|kinds| kinds.iter().any(|kind| kind == &declaration.kind))
        })
        .filter_map(|rule| Some((rule, pack.construction(rule)?)))
        .next_back()
    else {
        if pack.deferred_kinds().contains(&declaration.kind) {
            report.deferred.push(entry(Some(format!(
                "deferred by policy: `{}`",
                declaration.kind
            ))));
        } else {
            report.uncaptured.push(entry(None));
        }
        return;
    };

    let log = DispositionLog::new();
    let ownership = OwnershipContext {
        rules: pack.pointer_dispositions(),
        log: &log,
    };
    // The REPORTING log: what lands here becomes a refusal entry against the method's own name.
    let drops = crate::dropped::DropLog::new();
    let resolver = Resolver {
        scope: site.scope,
        type_map: pack.type_map(),
        overrides: pack.type_map_overrides(construction),
        constructors: pack.type_constructors(),
        copy_types: pack.copy_types(),
        cast_types: pack.cast_types(),
        zero_values: pack.zero_values(),
        trait_object_forms: pack.trait_object_forms(),
        failure: pack.failure_convention(),
        deferred: pack.deferred_kinds(),
        constant_map: pack.constant_map(),
        prose_type_names: pack.prose_type_names(),
        length_functions: pack.length_functions(),
        undecided_forms: pack.undecided_forms(),
        signatures,
        function_map: pack.function_map(),
        format_calls: pack.format_calls(),
        unmappable_calls: pack.unmappable_calls(),
        unmappable_types: pack.unmappable_types(),
        unmappable_facts: pack.unmappable_facts(),
        binary_string: pack.binary_string(),
        bit_pattern_constants: pack.bit_pattern_constants(),
        allocation: pack.allocation(),
        sequence_append: pack.sequence_append(),
        integer_arithmetic: pack.integer_arithmetic(),
        doc_convention: pack.doc_convention(),
        derives: pack.derives(),
        idioms: pack.idioms(),
        literal_constructors: pack.literal_constructors(),
        receiver: pack.trait_receiver(),
        ownership: &ownership,
        drops: &drops,
        emitted: site.emitted,
        units: site.units,
        unit: site.unit,
    };

    match build_item(construction, declaration, &resolver)
        .and_then(|items| refuse_unrenderable(items, declaration))
    {
        Ok(items) => {
            // KEPT, not counted and discarded. A survey that only counts can say a package is 70%
            // translated and show nobody what the 70% looks like — and what it looks like is the
            // bar this engine is actually held to.
            report.ported.push(PortedRegion {
                unit: site.unit.clone(),
                region: crate::naming::region_id_for_declaration(site.unit, rule, &declaration.name),
                position: isize::try_from(site.position).unwrap_or(isize::MAX),
                items,
            });
            report.translated.push(entry(None));
        }
        Err(error) => report.refused.push(entry(Some(crate::survey_cause::refusal_of(&error)))),
    }
    // A method dropped from a type that WAS emitted still refuses, under its own name. The type
    // survives because its shape does not depend on any body; the method is reported because a
    // reader looking at the emitted type has no other way to learn it is not there. Trading a loud
    // cascade for a silent hole would be the worse failure of the two.
    for dropped in drops.records() {
        report.refused.push(SurveyEntry {
            unit: site.unit.0.clone(),
            name: format!("{}::{}", dropped.owner, dropped.name),
            kind: KIND_METHOD_ENTRY.to_owned(),
            reason: Some(dropped.reason),
        });
    }
}

/// Refuse items that BUILD but do not RENDER.
///
/// Emittability has to include renderability, and it did not. The renderer parses what the transform
/// produced — the emitted tree is text that has to be valid target syntax — and a region it refuses
/// is dropped from the output. That happens AFTER the fixpoint has decided who is emittable, so the
/// callers of a dropped declaration are emitted anyway and name something that is no longer there.
///
/// One mapping template rendering wrong took `rol31` out of `xxhash` and left eight functions
/// calling it, which is exactly the dangling reference the fixpoint exists to prevent — arriving one
/// layer below where the fixpoint could see it.
///
/// So the check moves INTO the fixpoint: a declaration that will not render is not emittable, and
/// everything that names it refuses with it.
///
/// # Errors
/// [`TransformError::Ir`] carrying what the renderer said it could not parse.
fn refuse_unrenderable(
    items: Vec<port_engine_rust_ir::RustItem>,
    declaration: &Declaration,
) -> Result<Vec<port_engine_rust_ir::RustItem>, TransformError> {
    match port_engine_rust_ir::lower_file(&items) {
        Ok(_) => Ok(items),
        Err(error) => {
            let _ = declaration;
            Err(TransformError::Ir(error))
        }
    }
}
