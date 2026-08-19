//! CLOSED serde wire shapes.
//!
//! `deny_unknown_fields` throughout: `type_map_override` for `type_map_overrides` used to parse
//! clean, override nothing, and leave the author reading a green load and the wrong emitted types.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use crate::CONFLICT_REFUSE;
use crate::rule::{DeferredKind, DeriveWireRule, DispositionRule, DocConventionRule, FunctionMappingRule, IdiomWireRule, IntegerArithmeticRule, SelectingFixture, TraitReceiver, UndecidedForm};
use crate::rule_format::{FormatCallsRule, FormatFunctionRule, TargetImportsRule, UnmappableCallsRule, UnmappableTypesRule, BinaryStringRule};
use crate::rule::{ConstantMap, LengthFunctions, ProseTypeNames};

fn default_conflict() -> String {
    CONFLICT_REFUSE.to_owned()
}

/// CLOSED wire shape. An unknown key is a refusal, not a shrug: `type_map_override` for
/// `type_map_overrides` would otherwise parse clean, override nothing, and leave the pack author
/// looking at a green load and the wrong emitted types. `_comment` is declared so prose can live
/// beside the data it explains without punching a hole in the closure.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RulepackDocument {
    #[serde(default, rename = "_comment")]
    pub(crate) _comment: serde_json::Value,
    pub(crate) pair: PairFields,
    #[serde(default)]
    pub(crate) type_map: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) constant_map: Option<ConstantMap>,
    #[serde(default)]
    pub(crate) prose_type_names: Option<ProseTypeNames>,
    #[serde(default)]
    pub(crate) length_functions: Option<LengthFunctions>,
    #[serde(default)]
    pub(crate) type_constructors: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) copy_types: BTreeSet<String>,
    #[serde(default)]
    pub(crate) cast_types: BTreeSet<String>,
    #[serde(default)]
    pub(crate) zero_values: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) trait_object_forms: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) failure_convention: Option<FailureDoc>,
    #[serde(default)]
    pub(crate) function_map: BTreeMap<String, FunctionMappingRule>,
    #[serde(default)]
    pub(crate) format_calls: Option<FormatCallsRule>,
    #[serde(default)]
    pub(crate) target_imports: Option<TargetImportsRule>,
    #[serde(default)]
    pub(crate) unmappable_calls: Option<UnmappableCallsRule>,
    #[serde(default)]
    pub(crate) unmappable_types: Option<UnmappableTypesRule>,
    #[serde(default)]
    pub(crate) binary_string: Option<BinaryStringRule>,
    #[serde(default)]
    pub(crate) integer_arithmetic: Option<IntegerArithmeticRule>,
    #[serde(default)]
    pub(crate) doc_convention: Option<DocConventionRule>,
    #[serde(default)]
    pub(crate) derives: Vec<DeriveWireRule>,
    #[serde(default)]
    pub(crate) idioms: Vec<IdiomWireRule>,
    #[serde(default)]
    pub(crate) literal_constructors: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) type_map_overrides: BTreeMap<String, BTreeMap<String, String>>,
    #[serde(default)]
    pub(crate) deferred_kinds: Vec<DeferredKind>,
    #[serde(default)]
    pub(crate) undecided_forms: Vec<UndecidedForm>,
    #[serde(default)]
    pub(crate) trait_receiver: Option<TraitReceiver>,
    #[serde(default)]
    pub(crate) pointer_dispositions: Vec<DispositionRule>,
    pub(crate) rules: Vec<RuleDocument>,
    pub(crate) applies: BTreeMap<String, Vec<String>>,
}

/// The wire shape of the failure convention.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FailureDoc {
    pub(crate) source_type: String,
    #[serde(default)]
    pub(crate) discards_companion: bool,
    #[serde(default)]
    pub(crate) discard_reason: String,
    /// Why that target type, and what its bounds buy.
    pub(crate) reason: String,
    pub(crate) target_type: String,
    pub(crate) absent: String,
    #[serde(default)]
    pub(crate) constructors: std::collections::BTreeSet<String>,
    #[serde(default)]
    pub(crate) target_type_alternative_reason: String,
    #[serde(default)]
    pub(crate) boxed_alias: String,
    #[serde(default)]
    pub(crate) boxed_alias_reason: String,
    #[serde(default)]
    pub(crate) sentinel_enum: String,
    #[serde(default)]
    pub(crate) sentinel_enum_reason: String,
    #[serde(default)]
    pub(crate) sentinel_enum_exhaustive: bool,
    #[serde(default)]
    pub(crate) sentinel_enum_exhaustive_reason: String,
    #[serde(default)]
    pub(crate) identity_test_grouped: String,
    #[serde(default)]
    pub(crate) identity_test_grouped_reason: String,
    #[serde(default)]
    pub(crate) sentinel_prefix: String,
    #[serde(default)]
    pub(crate) sentinel_prefix_reason: String,
    #[serde(default)]
    pub(crate) param_type: String,
    #[serde(default)]
    pub(crate) param_type_reason: String,
    #[serde(default)]
    pub(crate) identity_test: String,
    #[serde(default)]
    pub(crate) identity_test_reason: String,
    #[serde(default)]
    pub(crate) inferred_construction: String,
    #[serde(default)]
    pub(crate) inferred_construction_reason: String,
    #[serde(default)]
    pub(crate) alias: String,
    #[serde(default)]
    pub(crate) alias_reason: String,
    #[serde(default)]
    pub(crate) constructor_reason: String,
    #[serde(default)]
    pub(crate) sentinel_constructors: std::collections::BTreeSet<String>,
    #[serde(default)]
    pub(crate) sentinel_reason: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PairFields {
    pub(crate) source: String,
    pub(crate) target: String,
}

/// Wire shape for one rule. Selection gating requires `id` / `version` / `selecting_fixtures`;
/// transform apply also requires non-empty `precondition` + `construction`. Closed, for the same
/// reason as [`RulepackDocument`].
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuleDocument {
    #[serde(default, rename = "_comment")]
    pub(crate) _comment: serde_json::Value,
    pub(crate) id: String,
    pub(crate) version: String,
    #[serde(default)]
    pub(crate) precondition: String,
    #[serde(default)]
    pub(crate) captures: Vec<String>,
    #[serde(default)]
    pub(crate) construction: String,
    #[serde(default)]
    pub(crate) precedence: i64,
    // Defaults to the only implemented policy rather than to the empty string. The engine refuses
    // a conflict unconditionally — the kernel has no other code path — so an omitted policy and a
    // stated `refuse` describe the same behaviour, while a stated ANYTHING ELSE describes
    // behaviour that does not exist and is refused below.
    #[serde(default = "default_conflict")]
    pub(crate) conflict: String,
    // Declared, and refused while unimplemented. These two used to be decoded and dropped, which
    // meant a pack author could write a diagnostic requirement or a proof obligation, load green,
    // and get nothing — the field said the engine would do something it had no code for.
    #[serde(default)]
    pub(crate) required_diagnostics: Vec<String>,
    #[serde(default)]
    pub(crate) proof_obligations: Vec<String>,
    #[serde(default)]
    pub(crate) selecting_fixtures: Vec<SelectingFixture>,
}
