//! Which of a unit's declarations will actually be EMITTED.
//!
//! Split from `survey.rs` because the two answer different questions. That file measures what the
//! engine can translate; this decides which of those translations survive the fact that a body
//! naming a declaration that REFUSED is not self-contained.
//!
//! It is a fixpoint, and the direction is the whole of it. Starting from "everything is emittable"
//! and SHRINKING converges — the set only ever loses members — and gets mutually recursive
//! functions right: both translate, so neither is removed. Starting from empty and growing would
//! also converge and would refuse them both, because on the first round neither can see the other.

use std::collections::{BTreeMap, BTreeSet};

use port_engine_api::{PackSemantics, RulePack, RuleId, SourceModel};

use crate::resolve::LocalScope;
use crate::signature_table::SignatureTable;
use crate::survey::{Site, survey_declaration};
use crate::survey_report::SurveyReport;

/// Every name every unit declares, which is where the fixpoint starts.
pub(crate) fn emittable_names(model: &dyn SourceModel) -> BTreeMap<String, BTreeSet<String>> {
    model
        .units()
        .into_iter()
        .filter_map(|unit| {
            let declarations = model.declarations(&unit)?;
            Some((
                unit.0,
                declarations
                    .iter()
                    .map(|declaration| declaration.name.clone())
                    .collect(),
            ))
        })
        .collect()
}

/// One round of the fixpoint: the names that still translate given what is currently believed.
pub(crate) fn shrink<P>(
    model: &dyn SourceModel,
    pack: &P,
    rules: &[port_engine_api::RuleId],
    signatures: &SignatureTable,
    units: &BTreeSet<String>,
    believed: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeMap<String, BTreeSet<String>>
where
    P: RulePack + PackSemantics,
{
    let every = BTreeSet::new();
    let mut next = BTreeMap::new();
    for unit in model.units() {
        let Some(declarations) = model.declarations(&unit) else {
            continue;
        };
        let scope = LocalScope::with_failure(&declarations, pack.failure_convention());
        let mut kept = BTreeSet::new();
        for (position, declaration) in declarations.iter().enumerate() {
            let mut round = SurveyReport {
                translated: Vec::new(),
                refused: Vec::new(),
                uncaptured: Vec::new(),
                deferred: Vec::new(),
                ported: Vec::new(),
            };
            survey_declaration(
                &Site {
                    unit: &unit,
                    units,
                    emitted: believed.get(&unit.0).unwrap_or(&every),
                    position,
                    scope: &scope,
                },
                declaration,
                rules,
                pack,
                signatures,
                &mut round,
            );
            // Only a REFUSAL removes a name. A declaration nothing captures is deferred or
            // uncaptured, and neither means the name is absent for a caller's purposes — the
            // caller's own refusal for naming it is raised where the reference is.
            // A DROPPED METHOD is a refusal of the method, not of the type. It arrives in the same
            // list because it is reported the same way, and reading it as the type's own refusal
            // would rebuild the cascade this change exists to break — through a harder-to-see door,
            // since the type would be emitted and simultaneously unnameable.
            //
            // So the type keeps its name and each SURVIVING method earns its own qualified one.
            // That is what lets a body calling a dropped method refuse: it names something the
            // emitted crate does not contain, which is the rule that already governs every other
            // reference. The set still only shrinks, so the fixpoint's termination is untouched.
            let dropped: BTreeSet<&str> = round
                .refused
                .iter()
                .filter(|entry| entry.kind == crate::survey::KIND_METHOD_ENTRY)
                .filter_map(|entry| entry.name.split("::").nth(1))
                .collect();
            if round.refused.len() == dropped.len() {
                kept.insert(declaration.name.clone());
                for method in declaration.children_of_kind(crate::vocabulary::CHILD_METHOD) {
                    if !dropped.contains(method.name.as_str()) {
                        kept.insert(format!("{}::{}", declaration.name, method.name));
                    }
                }
            }
        }
        next.insert(unit.0, kept);
    }
    next
}
