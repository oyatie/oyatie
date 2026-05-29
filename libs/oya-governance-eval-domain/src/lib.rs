//! Foundry eval kernel.
//!
//! Pure capability-shaped eval set and publish-gate contracts.

use std::collections::{BTreeMap, BTreeSet};

use oya_data_boundary_kernel::{Classified, DataClass, OperationalDataClass};

pub const REQUIRED_LINGUISTIC_COHORT_LOCALES: [&str; 3] =
    ["lang-alpha1", "lang-beta1", "lang-gamma1"];
pub const REQUIRED_LINGUISTIC_COHORTS_MESSAGE: &str =
    "Eval set must include lang-alpha1, lang-beta1, and lang-gamma1 linguistic cohorts";
pub const REQUIRED_LINGUISTIC_COHORTS_DETAIL: &str =
    "missing lang-alpha1, lang-beta1, or lang-gamma1 locale cohort";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EvalMetric {
    ExactMatch,
    F1,
    Bleu,
    Rouge,
    HumanJudged,
    Composite,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AdversarialKind {
    PromptInjection,
    DataClassViolation,
    AutonomyBypass,
    ToolExfiltration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvalError {
    InvalidCapabilityId,
    EmptyVersion,
    EmptyCaseId,
    EmptyLocale,
    EmptyInputRef,
    EmptyExpectedRef,
    InvalidThreshold,
    EmptyEvalSet,
    UnsignedEvalSet,
    MissingAdversarialCoverage,
    MissingLinguisticCoverage,
    EvalSetNotFound,
    UnsignedEvalRun,
    EvalRunVersionMismatch,
    EvalRunBelowThreshold,
    MissingPassingEvalRun,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalCaseInput {
    pub case_id: String,
    pub locale: String,
    pub input_ref: String,
    pub expected_ref: String,
    pub adversarial_kind: Option<AdversarialKind>,
    pub deterministic_seed: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalSetInput {
    pub capability_id: String,
    pub version: String,
    pub metric: EvalMetric,
    pub min_pass_rate_percent: u8,
    pub min_p95_score_percent: u8,
    pub signed: bool,
    pub cases: Vec<EvalCaseInput>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalRunInput {
    pub capability_id: String,
    pub eval_set_version: String,
    pub pass_rate_percent: u8,
    pub p95_score_percent: u8,
    pub adversarial_passed: bool,
    pub linguistic_passed: bool,
    pub signed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalCase {
    pub case_id: Classified<String>,
    pub locale: Classified<String>,
    pub input_ref: Classified<String>,
    pub expected_ref: Classified<String>,
    pub adversarial_kind: Classified<Option<AdversarialKind>>,
    pub deterministic_seed: Classified<Option<u64>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalSet {
    pub capability_id: Classified<String>,
    pub version: Classified<String>,
    pub metric: Classified<EvalMetric>,
    pub min_pass_rate_percent: Classified<u8>,
    pub min_p95_score_percent: Classified<u8>,
    pub signed: Classified<bool>,
    pub cases: Classified<Vec<EvalCase>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalRun {
    pub capability_id: Classified<String>,
    pub eval_set_version: Classified<String>,
    pub pass_rate_percent: Classified<u8>,
    pub p95_score_percent: Classified<u8>,
    pub adversarial_passed: Classified<bool>,
    pub linguistic_passed: Classified<bool>,
    pub signed: Classified<bool>,
    pub passed: Classified<bool>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EvalGate {
    eval_sets: BTreeMap<String, EvalSet>,
    latest_runs: BTreeMap<String, EvalRun>,
}

impl EvalGate {
    pub fn register_eval_set(&mut self, input: EvalSetInput) -> Result<EvalSet, EvalError> {
        let eval_set = EvalSet::try_from(input)?;
        self.eval_sets
            .insert(eval_set.capability_id.value.clone(), eval_set.clone());
        Ok(eval_set)
    }

    pub fn record_run(&mut self, input: EvalRunInput) -> Result<EvalRun, EvalError> {
        validate_capability_id(&input.capability_id)?;
        validate_version(&input.eval_set_version)?;
        if !input.signed {
            return Err(EvalError::UnsignedEvalRun);
        }
        let eval_set = self
            .eval_sets
            .get(&input.capability_id)
            .ok_or(EvalError::EvalSetNotFound)?;
        if eval_set.version.value != input.eval_set_version {
            return Err(EvalError::EvalRunVersionMismatch);
        }
        let passed = input.pass_rate_percent >= eval_set.min_pass_rate_percent.value
            && input.p95_score_percent >= eval_set.min_p95_score_percent.value
            && input.adversarial_passed
            && input.linguistic_passed;
        if !passed {
            return Err(EvalError::EvalRunBelowThreshold);
        }
        let run = EvalRun {
            capability_id: Classified::new(input.capability_id, DataClass::InternalOnly),
            eval_set_version: Classified::new(input.eval_set_version, DataClass::InternalOnly),
            pass_rate_percent: Classified::new(
                input.pass_rate_percent,
                DataClass::BehavioralTenantProduct,
            ),
            p95_score_percent: Classified::new(
                input.p95_score_percent,
                DataClass::BehavioralTenantProduct,
            ),
            adversarial_passed: Classified::new(
                input.adversarial_passed,
                OperationalDataClass::Audit,
            ),
            linguistic_passed: Classified::new(
                input.linguistic_passed,
                OperationalDataClass::Audit,
            ),
            signed: Classified::new(input.signed, OperationalDataClass::Audit),
            passed: Classified::new(passed, OperationalDataClass::Audit),
        };
        self.latest_runs
            .insert(run.capability_id.value.clone(), run.clone());
        Ok(run)
    }

    pub fn assert_publish_ready(&self, capability_id: &str) -> Result<(), EvalError> {
        validate_capability_id(capability_id)?;
        self.eval_sets
            .get(capability_id)
            .ok_or(EvalError::EvalSetNotFound)?;
        let run = self
            .latest_runs
            .get(capability_id)
            .ok_or(EvalError::MissingPassingEvalRun)?;
        if !run.passed.value {
            return Err(EvalError::MissingPassingEvalRun);
        }
        Ok(())
    }
}

impl TryFrom<EvalSetInput> for EvalSet {
    type Error = EvalError;

    fn try_from(input: EvalSetInput) -> Result<Self, Self::Error> {
        validate_capability_id(&input.capability_id)?;
        validate_version(&input.version)?;
        validate_threshold(input.min_pass_rate_percent)?;
        validate_threshold(input.min_p95_score_percent)?;
        if !input.signed {
            return Err(EvalError::UnsignedEvalSet);
        }
        if input.cases.is_empty() {
            return Err(EvalError::EmptyEvalSet);
        }
        let cases = input
            .cases
            .into_iter()
            .map(EvalCase::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        validate_adversarial_coverage(&cases)?;
        validate_linguistic_coverage(&cases)?;
        Ok(Self {
            capability_id: Classified::new(input.capability_id, DataClass::InternalOnly),
            version: Classified::new(input.version, DataClass::InternalOnly),
            metric: Classified::new(input.metric, DataClass::InternalOnly),
            min_pass_rate_percent: Classified::new(
                input.min_pass_rate_percent,
                DataClass::InternalOnly,
            ),
            min_p95_score_percent: Classified::new(
                input.min_p95_score_percent,
                DataClass::InternalOnly,
            ),
            signed: Classified::new(input.signed, OperationalDataClass::Audit),
            cases: Classified::new(cases, OperationalDataClass::Audit),
        })
    }
}

impl TryFrom<EvalCaseInput> for EvalCase {
    type Error = EvalError;

    fn try_from(input: EvalCaseInput) -> Result<Self, Self::Error> {
        if input.case_id.trim().is_empty() {
            return Err(EvalError::EmptyCaseId);
        }
        if input.locale.trim().is_empty() {
            return Err(EvalError::EmptyLocale);
        }
        if input.input_ref.trim().is_empty() {
            return Err(EvalError::EmptyInputRef);
        }
        if input.expected_ref.trim().is_empty() {
            return Err(EvalError::EmptyExpectedRef);
        }
        Ok(Self {
            case_id: Classified::new(input.case_id, DataClass::InternalOnly),
            locale: Classified::new(input.locale, DataClass::InternalOnly),
            input_ref: Classified::new(input.input_ref, DataClass::InternalOnly),
            expected_ref: Classified::new(input.expected_ref, DataClass::InternalOnly),
            adversarial_kind: Classified::new(input.adversarial_kind, OperationalDataClass::Audit),
            deterministic_seed: Classified::new(input.deterministic_seed, DataClass::InternalOnly),
        })
    }
}

fn validate_capability_id(capability_id: &str) -> Result<(), EvalError> {
    if !capability_id.starts_with("cap.") {
        return Err(EvalError::InvalidCapabilityId);
    }
    Ok(())
}

fn validate_version(version: &str) -> Result<(), EvalError> {
    if version.trim().is_empty() {
        return Err(EvalError::EmptyVersion);
    }
    Ok(())
}

fn validate_threshold(threshold: u8) -> Result<(), EvalError> {
    if threshold == 0 || threshold > 100 {
        return Err(EvalError::InvalidThreshold);
    }
    Ok(())
}

fn validate_adversarial_coverage(cases: &[EvalCase]) -> Result<(), EvalError> {
    let actual = cases
        .iter()
        .filter_map(|case| case.adversarial_kind.value)
        .collect::<BTreeSet<_>>();
    let required = BTreeSet::from([
        AdversarialKind::PromptInjection,
        AdversarialKind::DataClassViolation,
        AdversarialKind::AutonomyBypass,
        AdversarialKind::ToolExfiltration,
    ]);
    if required.is_subset(&actual) {
        Ok(())
    } else {
        Err(EvalError::MissingAdversarialCoverage)
    }
}

fn validate_linguistic_coverage(cases: &[EvalCase]) -> Result<(), EvalError> {
    let actual = cases
        .iter()
        .map(|case| case.locale.value.as_str())
        .collect::<BTreeSet<_>>();
    if REQUIRED_LINGUISTIC_COHORT_LOCALES
        .iter()
        .all(|cohort| actual.contains(cohort))
    {
        Ok(())
    } else {
        Err(EvalError::MissingLinguisticCoverage)
    }
}
