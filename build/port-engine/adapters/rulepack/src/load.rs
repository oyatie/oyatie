//! Turning pack BYTES into a loaded pack, which is where every schema rule is enforced.
//!
//! Split from `pack.rs` because the two answer different questions: that file says what a loaded
//! pack IS and what it can be asked, and this one says what a document must satisfy to become one.
//! Every check here rejects a pack rather than repairing it — a pack that loads with a field
//! quietly defaulted is one whose digest promises something its data does not carry.

use std::collections::{BTreeMap, BTreeSet};

use port_engine_api::{
    Allocation, BinaryString, BitPatternConstants, DeriveRule, DocConvention, FailureConvention,
    FormatCalls, FormatFunction, FunctionMapping, IdiomRule, IntegerArithmetic, LanguagePair,
    PointerConstruction, PointerDisposition, RuleId, SequenceAppend, UnitId,
};
use port_engine_hash::digest_bytes;

use crate::CONFLICT_REFUSE;
use crate::error::RulepackError;
use crate::pack::LoadedRulePack;
use crate::policy::{validate_dispositions, validate_policy};
use crate::rule::LoadedRule;
use crate::wire::RulepackDocument;

impl LoadedRulePack {
    /// Load from an in-memory JSON string (test hook / future specs materializer input).
    ///
    /// # Errors
    /// [`RulepackError`] on parse, schema, fixture, selection, undeclared-apply, or pair refusal.
    pub fn load_from_str(json: &str) -> Result<Self, RulepackError> {
        let doc: RulepackDocument =
            serde_json::from_str(json).map_err(|err| RulepackError::Parse {
                detail: err.to_string(),
            })?;
        if doc.pair.source.is_empty() || doc.pair.target.is_empty() {
            return Err(RulepackError::Schema { field: "pair" });
        }
        let pair = LanguagePair {
            source: doc.pair.source,
            target: doc.pair.target,
        };
        // Fail closed on ambiguous pair before we ever plan.
        pair.slug().map_err(RulepackError::Pair)?;

        if doc.rules.is_empty() {
            return Err(RulepackError::Schema { field: "rules" });
        }
        let (rules, loaded_rules) = crate::rules::loaded(doc.rules)?;
        let declared: BTreeSet<&str> = rules.iter().map(|r| r.0.as_str()).collect();

        let mut applies = BTreeMap::new();
        for (unit, rule_ids) in doc.applies {
            if unit.is_empty() {
                return Err(RulepackError::Schema {
                    field: "applies.unit",
                });
            }
            let mut mapped = Vec::with_capacity(rule_ids.len());
            for rule in rule_ids {
                if !declared.contains(rule.as_str()) {
                    return Err(RulepackError::UndeclaredApply {
                        unit: unit.clone(),
                        rule,
                    });
                }
                mapped.push(RuleId(rule));
            }
            applies.insert(UnitId(unit), mapped);
        }

        for rule in &loaded_rules {
            if !rule
                .selecting_fixtures
                .iter()
                .any(|fixture| fixture.selects)
            {
                return Err(RulepackError::NoPositiveFixture {
                    rule: rule.id.0.clone(),
                    fixture_count: rule.selecting_fixtures.len(),
                });
            }
            for fixture in &rule.selecting_fixtures {
                let unit = UnitId(fixture.unit.clone());
                let actual = applies
                    .get(&unit)
                    .is_some_and(|selected| selected.contains(&rule.id));
                if actual != fixture.selects {
                    return Err(RulepackError::FixtureExpectationMismatch {
                        rule: rule.id.0.clone(),
                        fixture: fixture.id.clone(),
                        unit: fixture.unit.clone(),
                        expected: fixture.selects,
                        actual,
                    });
                }
            }
        }

        let deferred_kind_set = validate_policy(
            &doc.deferred_kinds,
            doc.trait_receiver.as_ref(),
            &loaded_rules,
        )?;

        // Digest the embedded bytes exactly — whitespace is part of the identity until a
        // canonicalizer lands with the forever specs/port-rules materializer.
        let digest = digest_bytes(json.as_bytes());
        Ok(Self {
            pair,
            digest,
            rules,
            loaded_rules,
            applies,
            prose_type_names: doc
                .prose_type_names
                .map(|table| table.names)
                .unwrap_or_default(),
            length_argument_callees: doc
                .length_functions
                .as_ref()
                .map(|table| table.argument_callees.clone())
                .unwrap_or_default(),
            length_functions: doc
                .length_functions
                .map(|table| table.names)
                .unwrap_or_default(),
            constant_map: doc
                .constant_map
                .map(|table| table.names)
                .unwrap_or_default(),
            type_map: doc.type_map,
            type_constructors: doc.type_constructors,
            copy_types: doc.copy_types,
            cast_types: doc.cast_types,
            zero_values: doc.zero_values,
            trait_object_forms: doc.trait_object_forms,
            // A mapping with no reason is the failure mode this pack exists to prevent, so an
            // empty one is refused at load rather than emitted with nobody's name on it.
            // Absent means the pack declines to answer, and the transform then refuses integer
            // arithmetic by name rather than emitting an operator whose overflow rule differs.
            // Absent means the pack declines to rewrite documentation at all, which leaves the
            // source's prose exactly as its author wrote it.
            literal_constructors: doc.literal_constructors,
            idioms: doc
                .idioms
                .into_iter()
                .map(|rule| IdiomRule {
                    id: rule.id,
                    shape: rule.shape,
                    method: rule.method,
                    reason: rule.reason,
                    seed_source: rule.seed_source,
                    seed_license: rule.seed_license,
                    seed_commit: rule.seed_commit,
                })
                .collect(),
            derives: doc
                .derives
                .into_iter()
                .map(|rule| DeriveRule {
                    name: rule.name,
                    blocked_by: rule.blocked_by.into_iter().collect(),
                    reason: rule.reason,
                })
                .collect(),
            doc_convention: doc
                .doc_convention
                .map_or_else(DocConvention::default, |rule| DocConvention {
                    strip_leading_name: rule.strip_leading_name,
                    copulas: rule.copulas.into_iter().collect(),
                    source_language_words: rule.source_language_words,
                    source_language_words_reason: rule.source_language_words_reason,
                    passive_openings: rule.passive_openings,
                    passive_openings_reason: rule.passive_openings_reason,
                    reason: rule.reason,
                }),
            integer_arithmetic: doc.integer_arithmetic.map_or_else(
                IntegerArithmetic::default,
                |rule| IntegerArithmetic {
                    types: rule.types.into_iter().collect(),
                    operators: rule.operators,
                    reason: rule.reason,
                },
            ),
            target_imports: doc
                .target_imports
                .map(|rule| rule.paths)
                .unwrap_or_default(),
            unmappable_calls: doc
                .unmappable_calls
                .map(|rule| rule.calls)
                .unwrap_or_default(),
            // CLONED because both maps come from the one rule and each is read on its own. The
            // rule is consumed once; taking two fields out of it needs the take to happen here.
            unmappable_types: doc
                .unmappable_types
                .as_ref()
                .map(|rule| rule.types.clone())
                .unwrap_or_default(),
            unmappable_facts: doc
                .unmappable_types
                .map(|rule| rule.facts)
                .unwrap_or_default(),
            sequence_append: crate::load_values::sequence_append(doc.sequence_append),
            allocation: crate::load_values::allocation(doc.allocation),
            binary_string: crate::load_values::binary_string(doc.binary_string),
            bit_pattern_constants: crate::load_values::bit_pattern_constants(
                doc.bit_pattern_constants,
            ),
            format_calls: crate::load_values::format_calls(doc.format_calls),
            function_map: doc
                .function_map
                .into_iter()
                .map(|(identity, rule)| {
                    (
                        identity,
                        FunctionMapping {
                            form: rule.form,
                            requires_argument: rule.requires_argument,
                            reason: rule.reason,
                        },
                    )
                })
                .collect(),
            failure_convention: doc.failure_convention.map(|failure| FailureConvention {
                source_type: failure.source_type,
                discards_companion: failure.discards_companion,
                discard_reason: failure.discard_reason,
                reason: failure.reason,
                target_type: failure.target_type,
                absent: failure.absent,
                constructors: failure.constructors,
                target_type_alternative_reason: failure.target_type_alternative_reason,
                boxed_alias: failure.boxed_alias,
                boxed_alias_reason: failure.boxed_alias_reason,
                sentinel_enum: failure.sentinel_enum,
                sentinel_enum_reason: failure.sentinel_enum_reason,
                sentinel_enum_exhaustive: failure.sentinel_enum_exhaustive,
                sentinel_enum_exhaustive_reason: failure.sentinel_enum_exhaustive_reason,
                identity_test_grouped: failure.identity_test_grouped,
                identity_test_grouped_reason: failure.identity_test_grouped_reason,
                sentinel_prefix: failure.sentinel_prefix,
                sentinel_prefix_reason: failure.sentinel_prefix_reason,
                param_type: failure.param_type,
                param_type_reason: failure.param_type_reason,
                nullable_type: failure.nullable_type,
                nullable_type_reason: failure.nullable_type_reason,
                nullable_borrowed_type: failure.nullable_borrowed_type,
                nullable_borrowed_type_reason: failure.nullable_borrowed_type_reason,
                satisfaction_reason: failure.satisfaction_reason,
                identity_test: failure.identity_test,
                identity_test_reason: failure.identity_test_reason,
                inferred_construction: failure.inferred_construction,
                inferred_construction_reason: failure.inferred_construction_reason,
                alias: failure.alias,
                alias_reason: failure.alias_reason,
                constructor_reason: failure.constructor_reason,
                sentinel_constructors: failure.sentinel_constructors,
                sentinel_reason: failure.sentinel_reason,
            }),
            type_map_overrides: doc.type_map_overrides,
            undecided_form_reasons: doc
                .undecided_forms
                .iter()
                .map(|entry| (entry.id.clone(), entry.reason.clone()))
                .collect(),
            deferred_kinds: doc.deferred_kinds,
            deferred_kind_set,
            dispositions: validate_dispositions(&doc.pointer_dispositions)?,
            trait_receiver: doc.trait_receiver,
        })
    }
}
