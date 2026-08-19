//! Turning pack BYTES into a loaded pack, which is where every schema rule is enforced.
//!
//! Split from `pack.rs` because the two answer different questions: that file says what a loaded
//! pack IS and what it can be asked, and this one says what a document must satisfy to become one.
//! Every check here rejects a pack rather than repairing it — a pack that loads with a field
//! quietly defaulted is one whose digest promises something its data does not carry.

use std::collections::{BTreeMap, BTreeSet};

use port_engine_api::{
    DeriveRule, DocConvention, FailureConvention, FunctionMapping, IdiomRule, IntegerArithmetic,
    LanguagePair, PointerConstruction, PointerDisposition, RuleId, UnitId,
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
            doc_convention: doc.doc_convention.map_or_else(DocConvention::default, |rule| {
                DocConvention {
                    strip_leading_name: rule.strip_leading_name,
                    copulas: rule.copulas.into_iter().collect(),
                    reason: rule.reason,
                }
            }),
            integer_arithmetic: doc.integer_arithmetic.map_or_else(
                IntegerArithmetic::default,
                |rule| IntegerArithmetic {
                    types: rule.types.into_iter().collect(),
                    operators: rule.operators,
                    reason: rule.reason,
                },
            ),
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
