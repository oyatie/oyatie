//! The loaded pack: validation on the way in, and the seam impls on the way out.

use std::collections::{BTreeMap, BTreeSet};

use port_engine_api::{FormatCalls, DeriveRule, Digest, DocConvention, FailureConvention, FunctionMapping, IdiomRule, IntegerArithmetic, LanguagePair, PackSemantics, PointerDisposition, RuleId, RulePack, UnitId};
use port_engine_hash::digest_bytes;

use crate::error::RulepackError;
use crate::policy::{validate_dispositions, validate_policy};
use crate::rule::{DeferredKind, LoadedRule, TraitReceiver};
use crate::wire::RulepackDocument;
use crate::{CONFLICT_REFUSE, RULEPACK_GO_RUST_V1_JSON, RULEPACK_V0_JSON};

/// Loaded neutral rule pack implementing [`RulePack`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedRulePack {
    pub(crate) pair: LanguagePair,
    pub(crate) digest: Digest,
    pub(crate) rules: Vec<RuleId>,
    pub(crate) loaded_rules: Vec<LoadedRule>,
    pub(crate) applies: BTreeMap<UnitId, Vec<RuleId>>,
    pub(crate) type_map: BTreeMap<String, String>,
    /// Source predeclared constant name → target expression.
    pub(crate) constant_map: BTreeMap<String, String>,
    /// Callee identities whose value is a length.
    pub(crate) length_functions: BTreeSet<String>,
    /// Source type name → target spelling, for the names a doc comment may use.
    pub(crate) prose_type_names: BTreeMap<String, String>,
    pub(crate) type_constructors: BTreeMap<String, String>,
    pub(crate) copy_types: BTreeSet<String>,
    pub(crate) cast_types: BTreeSet<String>,
    pub(crate) zero_values: BTreeMap<String, String>,
    pub(crate) trait_object_forms: BTreeMap<String, String>,
    pub(crate) failure_convention: Option<FailureConvention>,
    pub(crate) function_map: BTreeMap<String, FunctionMapping>,
    pub(crate) format_calls: FormatCalls,
    pub(crate) target_imports: BTreeMap<String, String>,
    pub(crate) unmappable_calls: BTreeMap<String, String>,
    pub(crate) length_argument_callees: BTreeSet<String>,
    pub(crate) integer_arithmetic: IntegerArithmetic,
    pub(crate) doc_convention: DocConvention,
    pub(crate) derives: Vec<DeriveRule>,
    pub(crate) idioms: Vec<IdiomRule>,
    pub(crate) literal_constructors: BTreeMap<String, String>,
    pub(crate) type_map_overrides: BTreeMap<String, BTreeMap<String, String>>,
    pub(crate) deferred_kinds: Vec<DeferredKind>,
    pub(crate) deferred_kind_set: BTreeSet<String>,
    /// Form id → the pack's recorded reason for not having decided it.
    pub(crate) undecided_form_reasons: BTreeMap<String, String>,
    pub(crate) trait_receiver: Option<TraitReceiver>,
    pub(crate) dispositions: Vec<PointerDisposition>,
}

impl LoadedRulePack {
    /// Load and validate the embedded v0 rulepack mirror (fixture-gated).
    ///
    /// # Errors
    /// [`RulepackError`] on parse, schema, fixture, selection, undeclared-apply, or pair refusal.
    pub fn load_embedded() -> Result<Self, RulepackError> {
        Self::load_from_str(RULEPACK_V0_JSON)
    }


    /// Load and validate the embedded go→rust v1 pack.
    ///
    /// # Errors
    /// [`RulepackError`] on any of the same refusals as [`Self::load_embedded`].
    pub fn load_embedded_go_rust() -> Result<Self, RulepackError> {
        Self::load_from_str(RULEPACK_GO_RUST_V1_JSON)
    }

    /// The kinds this pack knowingly leaves untranslated, with their recorded reasons.
    #[must_use]
    pub fn deferred(&self) -> &[DeferredKind] {
        &self.deferred_kinds
    }

    /// Borrow the language pair.
    #[must_use]
    pub fn language_pair(&self) -> &LanguagePair {
        &self.pair
    }

    /// Loaded rule records in pack order (each has a validated positive fixture).
    #[must_use]
    pub fn loaded_rules(&self) -> &[LoadedRule] {
        &self.loaded_rules
    }

    /// Total positive selecting fixtures across every loaded rule.
    #[must_use]
    pub fn selecting_fixture_count(&self) -> usize {
        self.loaded_rules
            .iter()
            .map(|r| {
                r.selecting_fixtures
                    .iter()
                    .filter(|fixture| fixture.selects)
                    .count()
            })
            .sum()
    }

    /// Look up a loaded rule by id.
    #[must_use]
    pub fn rule(&self, id: &RuleId) -> Option<&LoadedRule> {
        self.loaded_rules.iter().find(|r| &r.id == id)
    }
}
