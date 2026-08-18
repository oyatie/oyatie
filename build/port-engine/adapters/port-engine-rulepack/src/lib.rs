//! # port-engine-rulepack — fixture-gated neutral RulePack loader (W0-B Slice 10).
//!
//! ADR-0637 D1 / W0-B plan §Slice 5: rule SEMANTICS live in data under forever home
//! `specs/port-rules/**` (integ/specs). This adapter embeds a package-local v0 mirror and
//! implements [`RulePack`]. **Every loaded rule MUST carry ≥1 positive selecting fixture**, and
//! every positive or negative fixture MUST agree with the selection derived from `applies`.
//! Missing, empty, or false fixtures cannot manufacture coverage. Digest is SHA-256 of the
//! embedded JSON bytes via `port-engine-hash`. Neutral only — no corpus vocabulary.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use port_engine_api::{Digest, LanguagePair, RuleId, RulePack, UnitId};
use port_engine_hash::digest_bytes;
use port_engine_transform::PackSemantics;
use serde::Deserialize;

/// Embedded v0 mirror of forever `specs/port-rules/**` (integ/specs owns the live tree).
const RULEPACK_V0_JSON: &str = include_str!("rulepack-v0.json");

/// Embedded go→rust pack v1: the declaration-level rules, type map, and deferral policy that
/// translate the hermetic Go corpus. Same forever home as v0.
const RULEPACK_GO_RUST_V1_JSON: &str = include_str!("rulepack-go-rust-v1.json");

/// The only conflict policy the engine implements. A pack may not declare another: the kernel
/// refuses a duplicate rule or region outright, and there is no code path that would do anything
/// else with a different value here.
pub const CONFLICT_REFUSE: &str = "refuse";

/// Fail-closed readiness gate. `true` once Slice 10 fixture-gated load is present.
pub const fn w0_ready() -> bool {
    true
}

/// One selecting fixture bound to a rule (W0-B plan §5.3 minimum shape).
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectingFixture {
    /// Stable fixture identity.
    pub id: String,
    /// Unit the fixture exercises (deterministic ordering key with `id`).
    pub unit: String,
    /// Whether the rule is expected to select for `unit`.
    pub selects: bool,
}

/// One loaded rule record (identity + fixture gate + construction data).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedRule {
    /// Rule id.
    pub id: RuleId,
    /// Rule version string.
    pub version: String,
    /// Precondition id the transform evaluates.
    pub precondition: String,
    /// Construction id the transform applies into `RustIr`.
    pub construction: String,
    /// Declaration kinds this rule captures. Empty means the rule is unit-level.
    pub captures: Vec<String>,
    /// Declared precedence. Load-bearing: the loader refuses a pack whose precedence disagrees
    /// with declaration order, so this can never be a second, silently-ignored ordering.
    pub precedence: i64,
    /// Declared conflict policy. Only [`CONFLICT_REFUSE`] is implemented.
    pub conflict: String,
    /// Selection fixtures, including at least one validated positive fixture.
    pub selecting_fixtures: Vec<SelectingFixture>,
}

/// A declaration kind the pack knowingly does not translate, and why.
///
/// The reason is REQUIRED and travels in the pack bytes, therefore in the pack digest, therefore in
/// the receipt. That is the whole difference between a deferral and an omission: both emit nothing,
/// but one of them is a decision somebody made and can be found again.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeferredKind {
    /// The declaration kind left untranslated.
    pub kind: String,
    /// Why it is deferred, and where the analysis lives.
    pub reason: String,
}

/// Typed refusal from rulepack decode / validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RulepackError {
    /// JSON could not be parsed.
    Parse {
        /// Parser detail.
        detail: String,
    },
    /// Required field missing / empty / invalid.
    Schema {
        /// Which field failed.
        field: &'static str,
    },
    /// A declared rule has zero selecting fixtures (W0-B hard stop).
    MissingSelectingFixture {
        /// Rule id that failed the fixture gate.
        rule: String,
    },
    /// A rule declares fixtures but none is a positive selection example.
    NoPositiveFixture {
        /// Rule id that failed the positive-fixture gate.
        rule: String,
        /// Number of declared negative fixtures.
        fixture_count: usize,
    },
    /// A fixture's expected selection disagrees with the loaded `applies` policy.
    FixtureExpectationMismatch {
        /// Rule whose fixture failed.
        rule: String,
        /// Stable fixture identity.
        fixture: String,
        /// Unit exercised by the fixture.
        unit: String,
        /// Selection recorded by the fixture.
        expected: bool,
        /// Selection derived from `applies`.
        actual: bool,
    },
    /// `applies` referenced a rule absent from `rules`.
    UndeclaredApply {
        /// Unit that referenced the rule.
        unit: String,
        /// Undeclared rule id.
        rule: String,
    },
    /// The pack declares semantics the engine does not implement.
    UnimplementedSemantics {
        /// Rule that declares it.
        rule: String,
        /// Which declared field has no implementation behind it.
        field: &'static str,
    },
    /// Declared precedence disagrees with declaration order.
    PrecedenceDisagreesWithOrder {
        /// Rule whose precedence is out of order.
        rule: String,
        /// Precedence it declares.
        precedence: i64,
        /// Precedence of the rule declared before it.
        previous: i64,
    },
    /// A conflict policy the engine has no implementation for.
    UnknownConflictPolicy {
        /// Rule that declares it.
        rule: String,
        /// The policy string found.
        policy: String,
    },
    /// A deferred kind is also captured by a rule — the pack says both things at once.
    DeferredKindAlsoCaptured {
        /// The contradictory kind.
        kind: String,
        /// A rule that captures it.
        rule: String,
    },
    /// Language pair cannot address a rule namespace.
    Pair(port_engine_api::PortError),
}

impl fmt::Display for RulepackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse { detail } => write!(f, "rulepack JSON parse failed: {detail}"),
            Self::Schema { field } => write!(f, "rulepack schema missing or invalid: {field}"),
            Self::MissingSelectingFixture { rule } => write!(
                f,
                "rulepack rule `{rule}` cannot load without ≥1 selecting fixture"
            ),
            Self::NoPositiveFixture {
                rule,
                fixture_count,
            } => write!(
                f,
                "rulepack rule `{rule}` declares {fixture_count} fixture(s) but none selects it"
            ),
            Self::FixtureExpectationMismatch {
                rule,
                fixture,
                unit,
                expected,
                actual,
            } => write!(
                f,
                "rulepack fixture `{fixture}` for rule `{rule}` and unit `{unit}` expected \
                 selects={expected}, derived selects={actual}"
            ),
            Self::UndeclaredApply { unit, rule } => write!(
                f,
                "rulepack applies rule `{rule}` to unit `{unit}` but rules[] does not declare it"
            ),
            Self::UnimplementedSemantics { rule, field } => write!(
                f,
                "rulepack rule `{rule}` declares `{field}`, which the engine does not implement —                  a pack may not declare semantics that are silently dropped"
            ),
            Self::PrecedenceDisagreesWithOrder {
                rule,
                precedence,
                previous,
            } => write!(
                f,
                "rulepack rule `{rule}` declares precedence {precedence} after {previous}:                  declaration order is the transform order, so a precedence that disagrees with it                  is a second ordering nothing obeys"
            ),
            Self::UnknownConflictPolicy { rule, policy } => write!(
                f,
                "rulepack rule `{rule}` declares conflict policy `{policy}`; only                  `{CONFLICT_REFUSE}` is implemented"
            ),
            Self::DeferredKindAlsoCaptured { kind, rule } => write!(
                f,
                "rulepack defers kind `{kind}` and also captures it in rule `{rule}`: the pack                  cannot both translate it and record it as untranslated"
            ),
            Self::Pair(err) => write!(f, "rulepack language pair refused: {err}"),
        }
    }
}

impl std::error::Error for RulepackError {}

fn default_conflict() -> String {
    CONFLICT_REFUSE.to_owned()
}

/// CLOSED wire shape. An unknown key is a refusal, not a shrug: `type_map_override` for
/// `type_map_overrides` would otherwise parse clean, override nothing, and leave the pack author
/// looking at a green load and the wrong emitted types. `_comment` is declared so prose can live
/// beside the data it explains without punching a hole in the closure.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RulepackDocument {
    #[serde(default, rename = "_comment")]
    _comment: serde_json::Value,
    pair: PairFields,
    #[serde(default)]
    type_map: BTreeMap<String, String>,
    #[serde(default)]
    type_map_overrides: BTreeMap<String, BTreeMap<String, String>>,
    #[serde(default)]
    deferred_kinds: Vec<DeferredKind>,
    rules: Vec<RuleDocument>,
    applies: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PairFields {
    source: String,
    target: String,
}

/// Wire shape for one rule. Selection gating requires `id` / `version` / `selecting_fixtures`;
/// transform apply also requires non-empty `precondition` + `construction`. Closed, for the same
/// reason as [`RulepackDocument`].
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleDocument {
    #[serde(default, rename = "_comment")]
    _comment: serde_json::Value,
    id: String,
    version: String,
    #[serde(default)]
    precondition: String,
    #[serde(default)]
    captures: Vec<String>,
    #[serde(default)]
    construction: String,
    #[serde(default)]
    precedence: i64,
    // Defaults to the only implemented policy rather than to the empty string. The engine refuses
    // a conflict unconditionally — the kernel has no other code path — so an omitted policy and a
    // stated `refuse` describe the same behaviour, while a stated ANYTHING ELSE describes
    // behaviour that does not exist and is refused below.
    #[serde(default = "default_conflict")]
    conflict: String,
    // Declared, and refused while unimplemented. These two used to be decoded and dropped, which
    // meant a pack author could write a diagnostic requirement or a proof obligation, load green,
    // and get nothing — the field said the engine would do something it had no code for.
    #[serde(default)]
    required_diagnostics: Vec<String>,
    #[serde(default)]
    proof_obligations: Vec<String>,
    #[serde(default)]
    selecting_fixtures: Vec<SelectingFixture>,
}

/// Loaded neutral rule pack implementing [`RulePack`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedRulePack {
    pair: LanguagePair,
    digest: Digest,
    rules: Vec<RuleId>,
    loaded_rules: Vec<LoadedRule>,
    applies: BTreeMap<UnitId, Vec<RuleId>>,
    type_map: BTreeMap<String, String>,
    type_map_overrides: BTreeMap<String, BTreeMap<String, String>>,
    deferred_kinds: Vec<DeferredKind>,
    deferred_kind_set: BTreeSet<String>,
}

impl LoadedRulePack {
    /// Load and validate the embedded v0 rulepack mirror (fixture-gated).
    ///
    /// # Errors
    /// [`RulepackError`] on parse, schema, fixture, selection, undeclared-apply, or pair refusal.
    pub fn load_embedded() -> Result<Self, RulepackError> {
        Self::load_from_str(RULEPACK_V0_JSON)
    }

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
        let mut seen = BTreeSet::new();
        let mut rules = Vec::with_capacity(doc.rules.len());
        let mut loaded_rules = Vec::with_capacity(doc.rules.len());
        let mut previous_precedence: Option<i64> = None;
        for rule in doc.rules {
            if rule.id.is_empty() {
                return Err(RulepackError::Schema {
                    field: "rules[].id",
                });
            }
            if rule.version.is_empty() {
                return Err(RulepackError::Schema {
                    field: "rules[].version",
                });
            }
            if rule.precondition.is_empty() {
                return Err(RulepackError::Schema {
                    field: "rules[].precondition",
                });
            }
            if rule.construction.is_empty() {
                return Err(RulepackError::Schema {
                    field: "rules[].construction",
                });
            }
            if rule.selecting_fixtures.is_empty() {
                return Err(RulepackError::MissingSelectingFixture { rule: rule.id });
            }
            for fixture in &rule.selecting_fixtures {
                if fixture.id.is_empty() {
                    return Err(RulepackError::Schema {
                        field: "rules[].selecting_fixtures[].id",
                    });
                }
                if fixture.unit.is_empty() {
                    return Err(RulepackError::Schema {
                        field: "rules[].selecting_fixtures[].unit",
                    });
                }
            }
            if !seen.insert(rule.id.clone()) {
                return Err(RulepackError::Schema {
                    field: "rules(duplicate)",
                });
            }
            // Every field the wire shape carries must either drive behaviour or be refused.
            // These two carry no implementation, so a pack declaring them is told so rather than
            // loading green and receiving nothing.
            if !rule.required_diagnostics.is_empty() {
                return Err(RulepackError::UnimplementedSemantics {
                    rule: rule.id,
                    field: "required_diagnostics",
                });
            }
            if !rule.proof_obligations.is_empty() {
                return Err(RulepackError::UnimplementedSemantics {
                    rule: rule.id,
                    field: "proof_obligations",
                });
            }
            if rule.conflict != CONFLICT_REFUSE {
                return Err(RulepackError::UnknownConflictPolicy {
                    rule: rule.id,
                    policy: rule.conflict,
                });
            }
            // Declaration order IS the transform order — `port_engine_kernel::plan` refuses a
            // unit whose rules arrive out of declared position. `precedence` therefore has to
            // agree with it or the pack states an order that nothing obeys, and a reviewer
            // reading the precedences would be reading a fiction.
            if let Some(previous) = previous_precedence.filter(|p| rule.precedence <= *p) {
                return Err(RulepackError::PrecedenceDisagreesWithOrder {
                    rule: rule.id,
                    precedence: rule.precedence,
                    previous,
                });
            }
            previous_precedence = Some(rule.precedence);

            for capture in &rule.captures {
                if capture.is_empty() {
                    return Err(RulepackError::Schema {
                        field: "rules[].captures[]",
                    });
                }
            }

            let id = RuleId(rule.id);
            loaded_rules.push(LoadedRule {
                id: id.clone(),
                version: rule.version,
                precondition: rule.precondition,
                construction: rule.construction,
                captures: rule.captures,
                precedence: rule.precedence,
                conflict: rule.conflict,
                selecting_fixtures: rule.selecting_fixtures,
            });
            rules.push(id);
        }
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

        let mut deferred_kind_set = BTreeSet::new();
        for deferred in &doc.deferred_kinds {
            if deferred.kind.is_empty() {
                return Err(RulepackError::Schema {
                    field: "deferred_kinds[].kind",
                });
            }
            // A deferral without a reason is an omission wearing a label. The reason is what
            // makes it reviewable, and it is what travels in the digest.
            if deferred.reason.trim().is_empty() {
                return Err(RulepackError::Schema {
                    field: "deferred_kinds[].reason",
                });
            }
            if let Some(rule) = loaded_rules
                .iter()
                .find(|rule| rule.captures.contains(&deferred.kind))
            {
                return Err(RulepackError::DeferredKindAlsoCaptured {
                    kind: deferred.kind.clone(),
                    rule: rule.id.0.clone(),
                });
            }
            deferred_kind_set.insert(deferred.kind.clone());
        }

        // Digest the embedded bytes exactly — whitespace is part of the identity until a
        // canonicalizer lands with the forever specs/port-rules materializer.
        let digest = digest_bytes(json.as_bytes());
        Ok(Self {
            pair,
            digest,
            rules,
            loaded_rules,
            applies,
            type_map: doc.type_map,
            type_map_overrides: doc.type_map_overrides,
            deferred_kinds: doc.deferred_kinds,
            deferred_kind_set,
        })
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

    fn type_map_overrides(&self, construction: &str) -> Option<&BTreeMap<String, String>> {
        self.type_map_overrides.get(construction)
    }

    fn deferred_kinds(&self) -> &BTreeSet<String> {
        &self.deferred_kind_set
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

#[cfg(test)]
mod tests {
    use super::*;
    use port_engine_api::{Declaration, SourceModel, UnitId};
    use port_engine_hash::digest_bytes;

    struct TinyModel {
        units: Vec<UnitId>,
    }

    impl SourceModel for TinyModel {
        fn language(&self) -> &str {
            "go"
        }
        fn snapshot_digest(&self) -> Digest {
            Digest("snap".into())
        }
        fn units(&self) -> Vec<UnitId> {
            self.units.clone()
        }
        fn declarations(&self, unit: &UnitId) -> Option<Vec<Declaration>> {
            self.units.contains(unit).then(Vec::new)
        }
    }

    #[test]
    fn slice10_claims_fixture_gated_readiness() {
        assert!(w0_ready());
    }

    #[test]
    fn embedded_v0_loads_with_fixtures_and_digests_bytes() {
        let pack = LoadedRulePack::load_embedded().expect("embedded v0 must load");
        assert_eq!(pack.pair().source, "go");
        assert_eq!(pack.pair().target, "rust");
        assert_eq!(
            pack.digest(),
            digest_bytes(RULEPACK_V0_JSON.as_bytes()),
            "digest must be SHA-256 of embedded JSON bytes"
        );
        assert_eq!(
            pack.rules(),
            vec![
                RuleId("identity".into()),
                RuleId("canary_empty_unit".into())
            ]
        );
        assert_eq!(pack.selecting_fixture_count(), 2);
        for rule in pack.loaded_rules() {
            assert!(
                rule.selecting_fixtures
                    .iter()
                    .any(|fixture| fixture.selects),
                "every loaded rule must retain a positive selecting fixture"
            );
        }
        assert_eq!(
            pack.rules_for(&UnitId("example.com/b".into())),
            vec![
                RuleId("identity".into()),
                RuleId("canary_empty_unit".into())
            ]
        );
    }

    #[test]
    fn embedded_v0_plans_with_kernel() {
        let pack = LoadedRulePack::load_embedded().expect("load");
        let model = TinyModel {
            units: vec![
                UnitId("example.com/a".into()),
                UnitId("example.com/b".into()),
            ],
        };
        let plan = port_engine_kernel::plan(&model, &pack).expect("plan must succeed");
        assert_eq!(plan.steps.len(), 3);
        assert_eq!(plan.pair.source, "go");
    }

    #[test]
    fn refuses_undeclared_apply() {
        let json = r#"{
  "pair": {"source": "go", "target": "rust"},
  "rules": [{
    "id": "identity",
    "version": "0",
    "precondition": "unit_present",
    "construction": "pass_through",
    "selecting_fixtures": [{"id": "f", "unit": "u", "selects": true}]
  }],
  "applies": {"u": ["missing"]}
}"#;
        let err = LoadedRulePack::load_from_str(json).expect_err("undeclared apply");
        assert!(matches!(err, RulepackError::UndeclaredApply { .. }));
    }

    #[test]
    fn refuses_rule_without_selecting_fixture() {
        let json = r#"{
  "pair": {"source": "go", "target": "rust"},
  "rules": [{
    "id": "orphan",
    "version": "0",
    "precondition": "unit_present",
    "construction": "pass_through",
    "selecting_fixtures": []
  }],
  "applies": {}
}"#;
        let err = LoadedRulePack::load_from_str(json).expect_err("missing fixture must refuse");
        assert!(matches!(
            err,
            RulepackError::MissingSelectingFixture { rule } if rule == "orphan"
        ));
    }

    #[test]
    fn refuses_rule_with_omitted_selecting_fixtures_field() {
        let json = r#"{
  "pair": {"source": "go", "target": "rust"},
  "rules": [{
    "id": "bare",
    "version": "0",
    "precondition": "unit_present",
    "construction": "pass_through"
  }],
  "applies": {}
}"#;
        let err = LoadedRulePack::load_from_str(json).expect_err("omitted fixtures must refuse");
        assert!(matches!(
            err,
            RulepackError::MissingSelectingFixture { rule } if rule == "bare"
        ));
    }

    #[test]
    fn refuses_positive_fixture_that_does_not_select() {
        let json = r#"{
  "pair": {"source": "go", "target": "rust"},
  "rules": [{
    "id": "orphan",
    "version": "0",
    "precondition": "unit_present",
    "construction": "pass_through",
    "selecting_fixtures": [{"id": "positive", "unit": "u", "selects": true}]
  }],
  "applies": {}
}"#;
        let err = LoadedRulePack::load_from_str(json)
            .expect_err("a positive fixture must be selected by applies");
        assert!(matches!(
            err,
            RulepackError::FixtureExpectationMismatch {
                rule,
                fixture,
                expected: true,
                actual: false,
                ..
            } if rule == "orphan" && fixture == "positive"
        ));
    }

    #[test]
    fn refuses_negative_fixture_that_selects() {
        let json = r#"{
  "pair": {"source": "go", "target": "rust"},
  "rules": [{
    "id": "unexpected",
    "version": "0",
    "precondition": "unit_present",
    "construction": "pass_through",
    "selecting_fixtures": [
      {"id": "positive", "unit": "v", "selects": true},
      {"id": "negative", "unit": "u", "selects": false}
    ]
  }],
  "applies": {"u": ["unexpected"], "v": ["unexpected"]}
}"#;
        let err = LoadedRulePack::load_from_str(json)
            .expect_err("a negative fixture must not be selected by applies");
        assert!(matches!(
            err,
            RulepackError::FixtureExpectationMismatch {
                rule,
                fixture,
                expected: false,
                actual: true,
                ..
            } if rule == "unexpected" && fixture == "negative"
        ));
    }

    #[test]
    fn accepts_agreeing_negative_fixture_without_counting_it_as_selection() {
        let json = r#"{
  "pair": {"source": "go", "target": "rust"},
  "rules": [{
    "id": "conditional",
    "version": "0",
    "precondition": "unit_present",
    "construction": "pass_through",
    "selecting_fixtures": [
      {"id": "positive", "unit": "u", "selects": true},
      {"id": "negative", "unit": "v", "selects": false}
    ]
  }],
  "applies": {"u": ["conditional"]}
}"#;
        let pack = LoadedRulePack::load_from_str(json)
            .expect("an agreeing negative fixture must remain admissible");
        assert_eq!(pack.selecting_fixture_count(), 1);
        assert_eq!(pack.loaded_rules()[0].selecting_fixtures.len(), 2);
    }

    #[test]
    fn refuses_rule_with_only_negative_fixtures() {
        let json = r#"{
  "pair": {"source": "go", "target": "rust"},
  "rules": [{
    "id": "never",
    "version": "0",
    "precondition": "unit_present",
    "construction": "pass_through",
    "selecting_fixtures": [{"id": "negative", "unit": "u", "selects": false}]
  }],
  "applies": {}
}"#;
        let err = LoadedRulePack::load_from_str(json)
            .expect_err("every loaded rule needs a positive fixture");
        assert!(matches!(
            err,
            RulepackError::NoPositiveFixture {
                rule,
                fixture_count: 1,
            } if rule == "never"
        ));
    }

    // -----------------------------------------------------------------------------------------
    // Declared-but-ignored semantics. Each of these fields used to decode and drop.
    // -----------------------------------------------------------------------------------------

    /// A pack that declares a diagnostic requirement and gets nothing is worse than one that
    /// cannot declare it: the field reads as a promise the engine has no code to keep.
    #[test]
    fn refuses_declared_semantics_the_engine_does_not_implement() {
        for field in ["required_diagnostics", "proof_obligations"] {
            let json = format!(
                r#"{{"pair":{{"source":"go","target":"rust"}},"rules":[{{"id":"r","version":"0",
                   "precondition":"unit_present","construction":"pass_through","{field}":["x"],
                   "selecting_fixtures":[{{"id":"f","unit":"u","selects":true}}]}}],
                   "applies":{{"u":["r"]}}}}"#
            );
            let err = LoadedRulePack::load_from_str(&json)
                .expect_err("declared-but-unimplemented semantics must refuse");
            assert!(
                matches!(err, RulepackError::UnimplementedSemantics { field: f, .. } if f == field),
                "{field}: {err}"
            );
        }
    }

    /// Declaration order is the transform order — `plan` refuses a unit whose rules arrive out of
    /// declared position. A precedence that disagrees is a second ordering nothing obeys, and a
    /// reviewer reading it would be reading a fiction.
    #[test]
    fn refuses_precedence_that_disagrees_with_declaration_order() {
        let json = r#"{"pair":{"source":"go","target":"rust"},"rules":[
            {"id":"first","version":"0","precondition":"unit_present","construction":"pass_through",
             "precedence":10,"selecting_fixtures":[{"id":"a","unit":"u","selects":true}]},
            {"id":"second","version":"0","precondition":"unit_present","construction":"pass_through",
             "precedence":5,"selecting_fixtures":[{"id":"b","unit":"u","selects":true}]}],
            "applies":{"u":["first","second"]}}"#;
        let err = LoadedRulePack::load_from_str(json).expect_err("out-of-order precedence refuses");
        assert!(matches!(
            err,
            RulepackError::PrecedenceDisagreesWithOrder { ref rule, .. } if rule == "second"
        ));
    }

    #[test]
    fn refuses_a_conflict_policy_with_no_implementation() {
        let json = r#"{"pair":{"source":"go","target":"rust"},"rules":[
            {"id":"r","version":"0","precondition":"unit_present","construction":"pass_through",
             "conflict":"last_wins","selecting_fixtures":[{"id":"f","unit":"u","selects":true}]}],
            "applies":{"u":["r"]}}"#;
        let err = LoadedRulePack::load_from_str(json).expect_err("unimplemented policy refuses");
        assert!(matches!(err, RulepackError::UnknownConflictPolicy { .. }));
    }

    /// A deferral without a reason is an omission wearing a label.
    #[test]
    fn refuses_a_deferral_without_a_recorded_reason() {
        let json = r#"{"pair":{"source":"go","target":"rust"},
            "deferred_kinds":[{"kind":"var","reason":"   "}],
            "rules":[{"id":"r","version":"0","precondition":"unit_present",
             "construction":"pass_through","selecting_fixtures":[{"id":"f","unit":"u","selects":true}]}],
            "applies":{"u":["r"]}}"#;
        let err = LoadedRulePack::load_from_str(json).expect_err("reasonless deferral refuses");
        assert!(matches!(
            err,
            RulepackError::Schema {
                field: "deferred_kinds[].reason"
            }
        ));
    }

    #[test]
    fn refuses_a_kind_that_is_both_captured_and_deferred() {
        let json = r#"{"pair":{"source":"go","target":"rust"},
            "deferred_kinds":[{"kind":"const","reason":"not yet"}],
            "rules":[{"id":"r","version":"0","precondition":"unit_present","captures":["const"],
             "construction":"rust_const","selecting_fixtures":[{"id":"f","unit":"u","selects":true}]}],
            "applies":{"u":["r"]}}"#;
        let err = LoadedRulePack::load_from_str(json).expect_err("contradiction refuses");
        assert!(matches!(
            err,
            RulepackError::DeferredKindAlsoCaptured { .. }
        ));
    }

    /// A misspelled key used to parse clean and do nothing. `type_map_override` would have
    /// overridden no types at all while the load stayed green.
    #[test]
    fn refuses_an_unknown_key_rather_than_ignoring_it() {
        let json = r#"{"pair":{"source":"go","target":"rust"},"type_map_override":{},
            "rules":[{"id":"r","version":"0","precondition":"unit_present",
             "construction":"pass_through","selecting_fixtures":[{"id":"f","unit":"u","selects":true}]}],
            "applies":{"u":["r"]}}"#;
        let err = LoadedRulePack::load_from_str(json).expect_err("unknown key refuses");
        assert!(matches!(err, RulepackError::Parse { .. }), "{err}");
    }

    #[test]
    fn embedded_go_rust_pack_loads_with_captures_types_and_deferrals() {
        let pack = LoadedRulePack::load_embedded_go_rust().expect("go→rust pack must load");
        assert_eq!(pack.language_pair().source, "go");
        assert_eq!(pack.language_pair().target, "rust");

        let by_id: BTreeMap<&str, &LoadedRule> = pack
            .loaded_rules()
            .iter()
            .map(|rule| (rule.id.0.as_str(), rule))
            .collect();
        assert_eq!(by_id["go_struct"].captures, vec!["struct".to_owned()]);
        assert_eq!(by_id["go_func"].construction, "rust_fn_body");

        assert_eq!(pack.type_map().get("int").map(String::as_str), Some("i64"));
        assert_eq!(
            pack.type_map_overrides("rust_const")
                .and_then(|map| map.get("string"))
                .map(String::as_str),
            Some("&\'static str"),
            "a Go string constant is a &\'static str, not an owned String"
        );

        let deferred = pack.deferred();
        assert_eq!(deferred.len(), 1);
        assert_eq!(deferred[0].kind, "var");
        assert!(
            deferred[0].reason.len() > 40,
            "a deferral's reason is the record; it must say something"
        );
    }

    /// Neutrality fence: production sources must not carry corpus needles.
    #[test]
    fn production_source_forbids_corpus_leakage() {
        let src = include_str!("lib.rs");
        let production = src
            .split("#[cfg(test)]")
            .next()
            .expect("production section");
        for needle in [
            ["k", "8", "s", ".", "i", "o"].concat(),
            ["k", "ube", "rnete", "s"].concat(),
            ["k", "ube", "let"].concat(),
        ] {
            assert!(
                !production.contains(&needle),
                "rulepack production must not embed `{needle}`"
            );
        }
    }
}
