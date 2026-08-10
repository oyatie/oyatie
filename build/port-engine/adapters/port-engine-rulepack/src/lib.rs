//! # port-engine-rulepack — fixture-gated neutral RulePack loader (W0-B Slice 10).
//!
//! ADR-0637 D1 / W0-B plan §Slice 5: rule SEMANTICS live in data under forever home
//! `specs/port-rules/**` (integ/specs). This adapter embeds a package-local v0 mirror and
//! implements [`RulePack`]. **Every loaded rule MUST carry ≥1 selecting fixture** — a rule with
//! an empty or missing fixture set cannot load (fail closed). Digest is SHA-256 of the embedded
//! JSON bytes via `port-engine-hash`. Neutral only — no corpus vocabulary.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;

use port_engine_api::{Digest, LanguagePair, RuleId, RulePack, UnitId};
use port_engine_hash::digest_bytes;
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

/// One loaded rule record (identity + fixture gate; semantics remain data).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedRule {
    /// Rule id.
    pub id: RuleId,
    /// Rule version string.
    pub version: String,
    /// Selecting fixtures (≥1 after load validation).
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

/// Wire shape for one rule. Optional W0-B §5.3 fields are accepted for forward schema
/// compatibility; selection gating only requires `id` / `version` / `selecting_fixtures`.
#[derive(Deserialize)]
#[allow(dead_code)]
struct RuleDocument {
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
    #[serde(default)]
    conflict: String,
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
}

impl LoadedRulePack {
    /// Load and validate the embedded v0 rulepack mirror (fixture-gated).
    ///
    /// # Errors
    /// [`RulepackError`] on parse/schema/missing-fixture/undeclared-apply/pair refusal.
    pub fn load_embedded() -> Result<Self, RulepackError> {
        Self::load_from_str(RULEPACK_V0_JSON)
    }

    /// Load from an in-memory JSON string (test hook / future specs materializer input).
    ///
    /// # Errors
    /// [`RulepackError`] on parse/schema/missing-fixture/undeclared-apply/pair refusal.
    pub fn load_from_str(json: &str) -> Result<Self, RulepackError> {
        let doc: RulepackDocument =
            serde_json::from_str(json).map_err(|err| RulepackError::Parse {
                detail: err.to_string(),
            })?;
        if doc.pair.source.is_empty() || doc.pair.target.is_empty() {
            return Err(RulepackError::Schema {
                field: "pair",
            });
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
            if rule.selecting_fixtures.is_empty() {
                return Err(RulepackError::MissingSelectingFixture {
                    rule: rule.id,
                });
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

    /// Loaded rule records in pack order (each has ≥1 selecting fixture).
    #[must_use]
    pub fn loaded_rules(&self) -> &[LoadedRule] {
        &self.loaded_rules
    }

    /// Total selecting fixtures across every loaded rule.
    #[must_use]
    pub fn selecting_fixture_count(&self) -> usize {
        self.loaded_rules
            .iter()
            .map(|r| r.selecting_fixtures.len())
            .sum()
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
    use port_engine_api::{SourceModel, UnitId};
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
                !rule.selecting_fixtures.is_empty(),
                "every loaded rule must retain selecting fixtures"
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
  "rules": [{"id": "bare", "version": "0"}],
  "applies": {}
}"#;
        let err = LoadedRulePack::load_from_str(json).expect_err("omitted fixtures must refuse");
        assert!(matches!(
            err,
            RulepackError::MissingSelectingFixture { rule } if rule == "bare"
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
