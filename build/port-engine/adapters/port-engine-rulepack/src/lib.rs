//! # port-engine-rulepack — fixture-gated neutral RulePack loader (W0-B Slice 10).
//!
//! ADR-0637 D1 / W0-B plan §Slice 5: rule SEMANTICS live in data under forever home
//! `specs/port-rules/**` (integ/specs). This adapter embeds a package-local v0 mirror and
//! implements [`RulePack`]. **Every loaded rule MUST carry ≥1 positive selecting fixture**, and
//! every positive or negative fixture MUST agree with the selection derived from `applies`.
//! Missing, empty, or false fixtures cannot manufacture coverage. Digest is SHA-256 of the
//! embedded JSON bytes via `port-engine-hash`. Neutral only — no corpus vocabulary.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;

use port_engine_api::{Digest, LanguagePair, RuleId, RulePack, UnitId};
use port_engine_hash::digest_bytes;
use port_engine_transform::RuleConstruction;
use serde::Deserialize;

/// Embedded v0 mirror of forever `specs/port-rules/**` (integ/specs owns the live tree).
const RULEPACK_V0_JSON: &str = include_str!("rulepack-v0.json");

/// Fail-closed readiness gate. `true` once Slice 10 fixture-gated load is present.
pub const fn w0_ready() -> bool {
    true
}

/// One selecting fixture bound to a rule (W0-B plan §5.3 minimum shape).
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
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
    /// Precondition id (Slice 11 transform evaluates this).
    pub precondition: String,
    /// Construction id (Slice 11 transform applies this into RustIr).
    pub construction: String,
    /// Selection fixtures, including at least one validated positive fixture.
    pub selecting_fixtures: Vec<SelectingFixture>,
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
            Self::Pair(err) => write!(f, "rulepack language pair refused: {err}"),
        }
    }
}

impl std::error::Error for RulepackError {}

#[derive(Deserialize)]
struct RulepackDocument {
    pair: PairFields,
    rules: Vec<RuleDocument>,
    applies: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct PairFields {
    source: String,
    target: String,
}

/// Wire shape for one rule. Selection gating requires `id` / `version` / `selecting_fixtures`;
/// Slice 11 also requires non-empty `precondition` + `construction` for transform apply.
#[derive(Deserialize)]
struct RuleDocument {
    id: String,
    version: String,
    #[serde(default)]
    precondition: String,
    #[serde(default)]
    #[allow(dead_code)]
    captures: Vec<String>,
    #[serde(default)]
    construction: String,
    #[serde(default)]
    #[allow(dead_code)]
    precedence: i64,
    #[serde(default)]
    #[allow(dead_code)]
    conflict: String,
    #[serde(default)]
    #[allow(dead_code)]
    required_diagnostics: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
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
        let mut seen = std::collections::BTreeSet::new();
        let mut rules = Vec::with_capacity(doc.rules.len());
        let mut loaded_rules = Vec::with_capacity(doc.rules.len());
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
            let id = RuleId(rule.id);
            loaded_rules.push(LoadedRule {
                id: id.clone(),
                version: rule.version,
                precondition: rule.precondition,
                construction: rule.construction,
                selecting_fixtures: rule.selecting_fixtures,
            });
            rules.push(id);
        }
        let declared: std::collections::BTreeSet<&str> =
            rules.iter().map(|r| r.0.as_str()).collect();

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

        // Digest the embedded bytes exactly — whitespace is part of the identity until a
        // canonicalizer lands with the forever specs/port-rules materializer.
        let digest = digest_bytes(json.as_bytes());
        Ok(Self {
            pair,
            digest,
            rules,
            loaded_rules,
            applies,
        })
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

impl RuleConstruction for LoadedRulePack {
    fn construction(&self, rule: &RuleId) -> Option<&str> {
        self.rule(rule).map(|r| r.construction.as_str())
    }

    fn precondition(&self, rule: &RuleId) -> Option<&str> {
        self.rule(rule).map(|r| r.precondition.as_str())
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
