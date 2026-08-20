//! The ports-face seams this adapter implements.
//!
//! [`RulePack`] answers WHICH rules apply; [`PackSemantics`] answers what a rule MEANS. Keeping
//! them apart from the loader keeps "what the pack is" separate from "how the pack is validated".

use std::collections::{BTreeMap, BTreeSet};

use port_engine_api::{
    Allocation, BinaryString, BitPatternConstants, ReadableLiterals, DeriveRule, Digest, DocConvention,
    FailureConvention, FormatCalls, FunctionMapping, IdiomRule, IntegerArithmetic, LanguagePair,
    PackSemantics, PointerDisposition, RuleId, RulePack, SequenceAppend, UnitId,
};

use crate::pack::LoadedRulePack;

impl PackSemantics for LoadedRulePack {
    fn construction(&self, rule: &RuleId) -> Option<&str> {
        self.rule(rule).map(|r| r.construction.as_str())
    }

    fn precondition(&self, rule: &RuleId) -> Option<&str> {
        self.rule(rule).map(|r| r.precondition.as_str())
    }

    fn captures(&self, rule: &RuleId) -> Option<&[String]> {
        self.rule(rule).map(|r| r.captures.as_slice())
    }

    fn type_map(&self) -> &BTreeMap<String, String> {
        &self.type_map
    }

    fn literal_constructors(&self) -> &BTreeMap<String, String> {
        &self.literal_constructors
    }
    fn idioms(&self) -> &[IdiomRule] {
        &self.idioms
    }
    fn derives(&self) -> &[DeriveRule] {
        &self.derives
    }
    fn doc_convention(&self) -> &DocConvention {
        &self.doc_convention
    }
    fn integer_arithmetic(&self) -> &IntegerArithmetic {
        &self.integer_arithmetic
    }
    fn length_argument_callees(&self) -> &BTreeSet<String> {
        &self.length_argument_callees
    }

    fn sequence_append(&self) -> &SequenceAppend {
        &self.sequence_append
    }

    fn allocation(&self) -> &Allocation {
        &self.allocation
    }

    fn binary_string(&self) -> &BinaryString {
        &self.binary_string
    }

    fn bit_pattern_constants(&self) -> &BitPatternConstants {
        &self.bit_pattern_constants
    }

    fn readable_literals(&self) -> &ReadableLiterals {
        &self.readable_literals
    }

    fn unmappable_types(&self) -> &BTreeMap<String, String> {
        &self.unmappable_types
    }

    fn unmappable_facts(&self) -> &BTreeMap<String, String> {
        &self.unmappable_facts
    }

    fn unmappable_calls(&self) -> &BTreeMap<String, String> {
        &self.unmappable_calls
    }

    fn target_imports(&self) -> &BTreeMap<String, String> {
        &self.target_imports
    }

    fn format_calls(&self) -> &FormatCalls {
        &self.format_calls
    }

    fn function_map(&self) -> &BTreeMap<String, FunctionMapping> {
        &self.function_map
    }

    fn failure_convention(&self) -> Option<&FailureConvention> {
        self.failure_convention.as_ref()
    }

    fn trait_object_forms(&self) -> &BTreeMap<String, String> {
        &self.trait_object_forms
    }

    fn zero_values(&self) -> &BTreeMap<String, String> {
        &self.zero_values
    }

    fn cast_types(&self) -> &BTreeSet<String> {
        &self.cast_types
    }

    fn copy_types(&self) -> &BTreeSet<String> {
        &self.copy_types
    }

    fn type_constructors(&self) -> &BTreeMap<String, String> {
        &self.type_constructors
    }

    fn type_map_overrides(&self, construction: &str) -> Option<&BTreeMap<String, String>> {
        self.type_map_overrides.get(construction)
    }

    fn pointer_dispositions(&self) -> &[PointerDisposition] {
        &self.dispositions
    }

    fn deferred_kinds(&self) -> &BTreeSet<String> {
        &self.deferred_kind_set
    }

    fn prose_type_names(&self) -> &BTreeMap<String, String> {
        &self.prose_type_names
    }

    fn length_functions(&self) -> &BTreeSet<String> {
        &self.length_functions
    }

    fn constant_map(&self) -> &BTreeMap<String, String> {
        &self.constant_map
    }

    fn undecided_forms(&self) -> &BTreeMap<String, String> {
        &self.undecided_form_reasons
    }

    fn trait_receiver(&self) -> Option<(&str, &str)> {
        self.trait_receiver
            .as_ref()
            .map(|r| (r.mode.as_str(), r.reason.as_str()))
    }
}

impl RulePack for LoadedRulePack {
    fn pair(&self) -> &LanguagePair {
        &self.pair
    }

    fn digest(&self) -> Digest {
        self.digest.clone()
    }

    fn rules(&self) -> Vec<RuleId> {
        self.rules.clone()
    }

    fn rules_for(&self, unit: &UnitId) -> Vec<RuleId> {
        self.applies.get(unit).cloned().unwrap_or_default()
    }
}
