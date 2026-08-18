//! # port-engine-transform — plan → RustIr construction apply (W0-B Slice 11).
//!
//! ADR-0637 D1: the kernel plans; this core face applies rule **construction** / **precondition**
//! data (strings from the pack) into a deterministic [`RustIr`]. Unknown constructions refuse.
//! Neutral — no corpus vocabulary; unit ids are sanitized into Rust region names.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fmt;

use port_engine_api::{PortError, RuleId, SourceModel, TransformPlan, UnitId};
use port_engine_rust_ir::RustIr;

/// Fail-closed readiness gate. `true` once Slice 11 transform apply is present.
pub const fn w0_ready() -> bool {
    true
}

/// Known W0-B v0 construction ids (data vocabulary, not corpus).
pub const CONSTRUCTION_PASS_THROUGH: &str = "pass_through";
/// Empty canary construction — emits a minimal empty fn region for fixture coupling.
pub const CONSTRUCTION_EMPTY_CANARY: &str = "empty_canary";

/// Known W0-B v0 precondition ids.
pub const PRECONDITION_UNIT_PRESENT: &str = "unit_present";

/// Lookup table for per-rule construction / precondition strings (implemented by rulepack).
pub trait RuleConstruction {
    /// Construction id for `rule`, if the pack declares it.
    fn construction(&self, rule: &RuleId) -> Option<&str>;
    /// Precondition id for `rule`, if the pack declares it.
    fn precondition(&self, rule: &RuleId) -> Option<&str>;
}

/// Typed refusal from transform apply.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransformError {
    /// Pack did not declare construction/precondition for a planned rule.
    MissingSemantics {
        /// Rule missing semantics.
        rule: String,
        /// Which field was absent.
        field: &'static str,
    },
    /// Precondition evaluation refused.
    Precondition {
        /// Rule being applied.
        rule: String,
        /// Unit under transform.
        unit: String,
        /// Precondition id that failed.
        precondition: String,
    },
    /// Construction id is not one of the W0-B v0 known set.
    UnknownConstruction {
        /// Rule being applied.
        rule: String,
        /// Construction id found.
        construction: String,
    },
    /// IR / syn assembly refused.
    Ir(PortError),
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSemantics { rule, field } => {
                write!(f, "transform missing `{field}` for rule `{rule}`")
            }
            Self::Precondition {
                rule,
                unit,
                precondition,
            } => write!(
                f,
                "transform precondition `{precondition}` failed for rule `{rule}` unit `{unit}`"
            ),
            Self::UnknownConstruction { rule, construction } => write!(
                f,
                "transform unknown construction `{construction}` for rule `{rule}`"
            ),
            Self::Ir(err) => write!(f, "transform IR assembly failed: {err}"),
        }
    }
}

impl std::error::Error for TransformError {}

/// Sanitize a unit or rule id into a Rust-safe region / fn name segment.
#[must_use]
pub fn sanitize_ident(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push_str("unit");
    }
    if out.as_bytes().first().is_some_and(|b| b.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

/// Region id for one plan step: `{sanitized_unit}__{sanitized_rule}`.
#[must_use]
pub fn region_id_for(unit: &UnitId, rule: &RuleId) -> String {
    format!("{}__{}", sanitize_ident(&unit.0), sanitize_ident(&rule.0))
}

/// Apply `plan` constructions against `model` using pack semantics → deterministic [`RustIr`].
///
/// Step order is plan order. Each step becomes one IR region with a syn AST.
///
/// # Errors
/// [`TransformError`] on missing semantics, failed precondition, unknown construction, or IR refuse.
pub fn apply(
    plan: &TransformPlan,
    constructions: &dyn RuleConstruction,
    model: &dyn SourceModel,
) -> Result<RustIr, TransformError> {
    let model_units: BTreeSet<String> = model.units().into_iter().map(|u| u.0).collect();

    let mut region_names: Vec<String> = Vec::with_capacity(plan.steps.len());
    let mut sources: Vec<(String, String)> = Vec::with_capacity(plan.steps.len());

    for step in &plan.steps {
        let construction = constructions.construction(&step.rule).ok_or_else(|| {
            TransformError::MissingSemantics {
                rule: step.rule.0.clone(),
                field: "construction",
            }
        })?;
        let precondition = constructions.precondition(&step.rule).ok_or_else(|| {
            TransformError::MissingSemantics {
                rule: step.rule.0.clone(),
                field: "precondition",
            }
        })?;

        check_precondition(precondition, &step.unit, &step.rule, &model_units)?;

        let region = region_id_for(&step.unit, &step.rule);
        let src = construction_source(construction, &step.rule, &region)?;
        region_names.push(region.clone());
        sources.push((region, src));
    }

    let refs: Vec<&str> = region_names.iter().map(String::as_str).collect();
    let mut ir = RustIr::new(&refs);
    for (region, src) in sources {
        ir.set_file_from_str(&region, &src)
            .map_err(TransformError::Ir)?;
    }
    Ok(ir)
}

fn check_precondition(
    precondition: &str,
    unit: &UnitId,
    rule: &RuleId,
    model_units: &BTreeSet<String>,
) -> Result<(), TransformError> {
    match precondition {
        PRECONDITION_UNIT_PRESENT => {
            if model_units.contains(&unit.0) {
                Ok(())
            } else {
                Err(TransformError::Precondition {
                    rule: rule.0.clone(),
                    unit: unit.0.clone(),
                    precondition: precondition.to_owned(),
                })
            }
        }
        other => Err(TransformError::Precondition {
            rule: rule.0.clone(),
            unit: unit.0.clone(),
            precondition: other.to_owned(),
        }),
    }
}

fn construction_source(
    construction: &str,
    rule: &RuleId,
    region: &str,
) -> Result<String, TransformError> {
    match construction {
        CONSTRUCTION_PASS_THROUGH => Ok(format!("pub fn {region}() {{}}")),
        CONSTRUCTION_EMPTY_CANARY => Ok(format!("pub fn {region}_canary() {{}}")),
        other => Err(TransformError::UnknownConstruction {
            rule: rule.0.clone(),
            construction: other.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use port_engine_api::{Digest, LanguagePair, PlanStep, TargetIr};
    use std::collections::BTreeMap;

    struct Table {
        map: BTreeMap<&'static str, (&'static str, &'static str)>,
    }

    impl RuleConstruction for Table {
        fn construction(&self, rule: &RuleId) -> Option<&str> {
            self.map.get(rule.0.as_str()).map(|(c, _)| *c)
        }
        fn precondition(&self, rule: &RuleId) -> Option<&str> {
            self.map.get(rule.0.as_str()).map(|(_, p)| *p)
        }
    }

    struct Model {
        units: Vec<UnitId>,
    }

    impl SourceModel for Model {
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

    fn sample_plan() -> TransformPlan {
        TransformPlan {
            pair: LanguagePair {
                source: "go".into(),
                target: "rust".into(),
            },
            steps: vec![
                PlanStep {
                    unit: UnitId("example.com/a".into()),
                    rule: RuleId("identity".into()),
                },
                PlanStep {
                    unit: UnitId("example.com/b".into()),
                    rule: RuleId("identity".into()),
                },
                PlanStep {
                    unit: UnitId("example.com/b".into()),
                    rule: RuleId("canary_empty_unit".into()),
                },
            ],
        }
    }

    #[test]
    fn slice11_claims_transform_readiness() {
        assert!(w0_ready());
    }

    #[test]
    fn apply_builds_one_region_per_plan_step() {
        let table = Table {
            map: BTreeMap::from([
                (
                    "identity",
                    (CONSTRUCTION_PASS_THROUGH, PRECONDITION_UNIT_PRESENT),
                ),
                (
                    "canary_empty_unit",
                    (CONSTRUCTION_EMPTY_CANARY, PRECONDITION_UNIT_PRESENT),
                ),
            ]),
        };
        let model = Model {
            units: vec![
                UnitId("example.com/a".into()),
                UnitId("example.com/b".into()),
            ],
        };
        let ir = apply(&sample_plan(), &table, &model).expect("apply");
        assert_eq!(ir.regions().len(), 3);
        assert!(ir.file(&ir.regions()[0]).is_some());
    }

    #[test]
    fn refuses_unknown_construction() {
        let table = Table {
            map: BTreeMap::from([(
                "identity",
                ("not_a_real_construction", PRECONDITION_UNIT_PRESENT),
            )]),
        };
        let model = Model {
            units: vec![UnitId("example.com/a".into())],
        };
        let plan = TransformPlan {
            pair: LanguagePair {
                source: "go".into(),
                target: "rust".into(),
            },
            steps: vec![PlanStep {
                unit: UnitId("example.com/a".into()),
                rule: RuleId("identity".into()),
            }],
        };
        let err = apply(&plan, &table, &model).expect_err("unknown construction");
        assert!(matches!(err, TransformError::UnknownConstruction { .. }));
    }

    #[test]
    fn refuses_missing_unit_precondition() {
        let table = Table {
            map: BTreeMap::from([(
                "identity",
                (CONSTRUCTION_PASS_THROUGH, PRECONDITION_UNIT_PRESENT),
            )]),
        };
        let model = Model { units: vec![] };
        let plan = TransformPlan {
            pair: LanguagePair {
                source: "go".into(),
                target: "rust".into(),
            },
            steps: vec![PlanStep {
                unit: UnitId("example.com/a".into()),
                rule: RuleId("identity".into()),
            }],
        };
        let err = apply(&plan, &table, &model).expect_err("unit missing");
        assert!(matches!(err, TransformError::Precondition { .. }));
    }

    #[test]
    fn sanitize_ident_is_rust_safe() {
        assert_eq!(sanitize_ident("example.com/a"), "example_com_a");
        assert_eq!(sanitize_ident("9x"), "_9x");
    }
}
