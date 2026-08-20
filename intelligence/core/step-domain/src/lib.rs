//! Foundry step kernel.
//!
//! Pure typed records for per-run execution steps.

use std::collections::{BTreeMap, BTreeSet};

use oya_data_boundary_kernel::{
    Classified, DataClass, OperationalDataClass, PrivacyDataClass,
    data_classes_from_privacy_data_classes, most_restrictive_privacy_data_class,
    privacy_data_classes_from,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StepKind {
    ToolCall,
    ProviderCall,
    Reasoning,
    Retrieval,
    Cite,
    Wait,
    Branch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StepState {
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StepDisposition {
    Succeeded,
    FailedProvider,
    FailedTimeout,
    FailedBudget,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StepError {
    InvalidRunId,
    InvalidStepHistory,
    EmptyProviderKind,
    EmptyModelRef,
    MissingDataClasses,
    InvalidDataClass,
    StepNotFound,
    StepNotRunning,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepStart {
    pub run_id: Classified<String>,
    pub kind: Classified<StepKind>,
    pub provider_kind: Classified<String>,
    pub model_ref: Classified<Option<String>>,
    pub input_tokens: Classified<Option<u32>>,
    pub output_tokens: Classified<Option<u32>>,
    pub data_classes_touched: Classified<Vec<PrivacyDataClass>>,
    pub data_class: Classified<DataClass>,
    pub started_at_epoch_seconds: Classified<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Step {
    pub step_id: Classified<String>,
    pub run_id: Classified<String>,
    pub sequence: Classified<u32>,
    pub kind: Classified<StepKind>,
    pub provider_kind: Classified<String>,
    pub model_ref: Classified<Option<String>>,
    pub input_tokens: Classified<Option<u32>>,
    pub output_tokens: Classified<Option<u32>>,
    pub latency_ms: Classified<Option<u32>>,
    pub data_classes_touched: Classified<Vec<PrivacyDataClass>>,
    pub data_class: Classified<DataClass>,
    pub state: Classified<StepState>,
    pub disposition: Classified<Option<StepDisposition>>,
    pub started_at_epoch_seconds: Classified<u64>,
    pub completed_at_epoch_seconds: Classified<Option<u64>>,
    pub schema_version: Classified<u32>,
}

impl StepStart {
    pub fn touched_privacy_data_classes(&self) -> &[PrivacyDataClass] {
        &self.data_classes_touched.value
    }

    /// Legacy step-start projection for ledger consumers that still persist
    /// raw `DataClass` labels. Step starts are constructed with typed privacy
    /// classes, so this projection is lossless and cannot introduce
    /// operational or subject markers.
    pub fn legacy_touched_data_classes(&self) -> Vec<DataClass> {
        data_classes_from_privacy_data_classes(&self.data_classes_touched.value)
    }

    #[deprecated(
        note = "use touched_privacy_data_classes for canonical typed access or legacy_touched_data_classes for the compatibility projection"
    )]
    pub fn touched_data_classes(&self) -> Vec<DataClass> {
        self.legacy_touched_data_classes()
    }
}

impl Step {
    pub fn touched_privacy_data_classes(&self) -> &[PrivacyDataClass] {
        &self.data_classes_touched.value
    }

    /// Legacy step-ledger projection for ledger consumers that still persist
    /// raw `DataClass` labels. Steps store typed privacy classes, so this
    /// projection is lossless and cannot introduce operational or subject
    /// markers.
    pub fn legacy_touched_data_classes(&self) -> Vec<DataClass> {
        data_classes_from_privacy_data_classes(&self.data_classes_touched.value)
    }

    #[deprecated(
        note = "use touched_privacy_data_classes for canonical typed access or legacy_touched_data_classes for the compatibility projection"
    )]
    pub fn touched_data_classes(&self) -> Vec<DataClass> {
        self.legacy_touched_data_classes()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepLedger {
    steps: Classified<Vec<Step>>,
    latest_by_id: Classified<BTreeMap<String, usize>>,
    next_sequence_by_run: Classified<BTreeMap<String, u32>>,
}

impl Default for StepLedger {
    fn default() -> Self {
        Self {
            steps: Classified::new(Vec::new(), OperationalDataClass::Audit),
            latest_by_id: Classified::new(BTreeMap::new(), DataClass::InternalOnly),
            next_sequence_by_run: Classified::new(BTreeMap::new(), DataClass::InternalOnly),
        }
    }
}

impl StepStart {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        run_id: String,
        kind: StepKind,
        provider_kind: String,
        model_ref: Option<String>,
        input_tokens: Option<u32>,
        output_tokens: Option<u32>,
        data_classes_touched: Vec<PrivacyDataClass>,
        started_at_epoch_seconds: u64,
    ) -> Result<Self, StepError> {
        Self::new_with_privacy_data_classes(
            run_id,
            kind,
            provider_kind,
            model_ref,
            input_tokens,
            output_tokens,
            data_classes_touched,
            started_at_epoch_seconds,
        )
    }

    /// Compatibility constructor for replay/config seams that still carry raw
    /// `DataClass` labels. Canonical step starts take `PrivacyDataClass`, and
    /// this path fails closed for operational markers and subject markers.
    #[allow(clippy::too_many_arguments)]
    pub fn try_from_legacy_data_classes_touched(
        run_id: String,
        kind: StepKind,
        provider_kind: String,
        model_ref: Option<String>,
        input_tokens: Option<u32>,
        output_tokens: Option<u32>,
        data_classes_touched: Vec<DataClass>,
        started_at_epoch_seconds: u64,
    ) -> Result<Self, StepError> {
        let data_classes_touched = validate_privacy_data_classes(&data_classes_touched)?;
        Self::new(
            run_id,
            kind,
            provider_kind,
            model_ref,
            input_tokens,
            output_tokens,
            data_classes_touched,
            started_at_epoch_seconds,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_privacy_data_classes(
        run_id: String,
        kind: StepKind,
        provider_kind: String,
        model_ref: Option<String>,
        input_tokens: Option<u32>,
        output_tokens: Option<u32>,
        data_classes_touched: Vec<PrivacyDataClass>,
        started_at_epoch_seconds: u64,
    ) -> Result<Self, StepError> {
        validate_run_id(&run_id)?;
        if provider_kind.trim().is_empty() {
            return Err(StepError::EmptyProviderKind);
        }
        if model_ref
            .as_ref()
            .is_some_and(|model_ref| model_ref.trim().is_empty())
        {
            return Err(StepError::EmptyModelRef);
        }
        if data_classes_touched.is_empty() {
            return Err(StepError::MissingDataClasses);
        }
        Self::new_with_validated_privacy_data_classes(
            run_id,
            kind,
            provider_kind,
            model_ref,
            input_tokens,
            output_tokens,
            data_classes_touched,
            started_at_epoch_seconds,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_validated_privacy_data_classes(
        run_id: String,
        kind: StepKind,
        provider_kind: String,
        model_ref: Option<String>,
        input_tokens: Option<u32>,
        output_tokens: Option<u32>,
        data_classes_touched: Vec<PrivacyDataClass>,
        started_at_epoch_seconds: u64,
    ) -> Result<Self, StepError> {
        let data_class =
            most_restrictive(&data_classes_touched).ok_or(StepError::MissingDataClasses)?;
        Ok(Self {
            run_id: Classified::new(run_id, DataClass::InternalOnly),
            kind: Classified::new(kind, DataClass::InternalOnly),
            provider_kind: Classified::new(provider_kind, DataClass::InternalOnly),
            model_ref: Classified::new(model_ref, DataClass::InternalOnly),
            input_tokens: Classified::new(input_tokens, DataClass::BehavioralTenantProduct),
            output_tokens: Classified::new(output_tokens, DataClass::BehavioralTenantProduct),
            data_classes_touched: Classified::new(data_classes_touched, DataClass::InternalOnly),
            data_class: Classified::new(data_class, DataClass::InternalOnly),
            started_at_epoch_seconds: Classified::new(
                started_at_epoch_seconds,
                DataClass::InternalOnly,
            ),
        })
    }
}

impl StepLedger {
    pub fn from_steps(steps: Vec<Step>) -> Result<Self, StepError> {
        let mut latest_by_id = BTreeMap::new();
        let mut sequences_by_run: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
        let mut max_sequence_by_run: BTreeMap<String, u32> = BTreeMap::new();
        for (index, step) in steps.iter().enumerate() {
            validate_step_record(step)?;
            if latest_by_id
                .insert(step.step_id.value.clone(), index)
                .is_some()
            {
                return Err(StepError::InvalidStepHistory);
            }
            let sequences = sequences_by_run
                .entry(step.run_id.value.clone())
                .or_default();
            if !sequences.insert(step.sequence.value) {
                return Err(StepError::InvalidStepHistory);
            }
            max_sequence_by_run
                .entry(step.run_id.value.clone())
                .and_modify(|max_sequence| *max_sequence = (*max_sequence).max(step.sequence.value))
                .or_insert(step.sequence.value);
        }
        for (run_id, max_sequence) in &max_sequence_by_run {
            let Some(sequences) = sequences_by_run.get(run_id) else {
                return Err(StepError::InvalidStepHistory);
            };
            if (1..=*max_sequence).any(|sequence| !sequences.contains(&sequence)) {
                return Err(StepError::InvalidStepHistory);
            }
        }
        let next_sequence_by_run = max_sequence_by_run
            .into_iter()
            .map(|(run_id, max_sequence)| (run_id, max_sequence + 1))
            .collect();
        Ok(Self {
            steps: Classified::new(steps, OperationalDataClass::Audit),
            latest_by_id: Classified::new(latest_by_id, DataClass::InternalOnly),
            next_sequence_by_run: Classified::new(next_sequence_by_run, DataClass::InternalOnly),
        })
    }

    pub fn start(&mut self, start: StepStart) -> Result<Step, StepError> {
        validate_step_start(&start)?;
        let sequence = self.next_sequence(&start.run_id.value);
        let step_id = format!("step_{}_{sequence:06}", run_sequence(&start.run_id.value)?);
        let step = Step {
            step_id: Classified::new(step_id, DataClass::InternalOnly),
            run_id: start.run_id,
            sequence: Classified::new(sequence, DataClass::InternalOnly),
            kind: start.kind,
            provider_kind: start.provider_kind,
            model_ref: start.model_ref,
            input_tokens: start.input_tokens,
            output_tokens: start.output_tokens,
            latency_ms: Classified::new(None, DataClass::BehavioralTenantProduct),
            data_classes_touched: start.data_classes_touched,
            data_class: start.data_class,
            state: Classified::new(StepState::Running, DataClass::InternalOnly),
            disposition: Classified::new(None, DataClass::InternalOnly),
            started_at_epoch_seconds: start.started_at_epoch_seconds,
            completed_at_epoch_seconds: Classified::new(None, DataClass::InternalOnly),
            schema_version: Classified::new(1, DataClass::InternalOnly),
        };
        self.insert(step.clone());
        Ok(step)
    }

    pub fn complete(
        &mut self,
        step_id: &str,
        disposition: StepDisposition,
        latency_ms: u32,
        completed_at_epoch_seconds: u64,
    ) -> Result<Step, StepError> {
        let index = *self
            .latest_by_id
            .value
            .get(step_id)
            .ok_or(StepError::StepNotFound)?;
        let step = self
            .steps
            .value
            .get_mut(index)
            .ok_or(StepError::StepNotFound)?;
        if step.state.value != StepState::Running {
            return Err(StepError::StepNotRunning);
        }
        step.state = Classified::new(state_for(disposition), DataClass::InternalOnly);
        step.disposition = Classified::new(Some(disposition), DataClass::InternalOnly);
        step.latency_ms = Classified::new(Some(latency_ms), DataClass::BehavioralTenantProduct);
        step.completed_at_epoch_seconds =
            Classified::new(Some(completed_at_epoch_seconds), DataClass::InternalOnly);
        Ok(step.clone())
    }

    pub fn steps(&self) -> &[Step] {
        &self.steps.value
    }

    fn next_sequence(&mut self, run_id: &str) -> u32 {
        let next = self
            .next_sequence_by_run
            .value
            .entry(run_id.to_string())
            .or_insert(1);
        let sequence = *next;
        *next += 1;
        sequence
    }

    fn insert(&mut self, step: Step) {
        self.latest_by_id
            .value
            .insert(step.step_id.value.clone(), self.steps.value.len());
        self.steps.value.push(step);
    }
}

fn state_for(disposition: StepDisposition) -> StepState {
    match disposition {
        StepDisposition::Succeeded => StepState::Succeeded,
        StepDisposition::FailedProvider
        | StepDisposition::FailedTimeout
        | StepDisposition::FailedBudget => StepState::Failed,
        StepDisposition::Cancelled => StepState::Cancelled,
    }
}

fn validate_run_id(run_id: &str) -> Result<(), StepError> {
    run_sequence(run_id).map(|_| ())
}

fn validate_step_start(start: &StepStart) -> Result<(), StepError> {
    validate_run_id(&start.run_id.value)?;
    if start.provider_kind.value.trim().is_empty() {
        return Err(StepError::EmptyProviderKind);
    }
    if start
        .model_ref
        .value
        .as_ref()
        .is_some_and(|model_ref| model_ref.trim().is_empty())
    {
        return Err(StepError::EmptyModelRef);
    }
    let data_class =
        most_restrictive(&start.data_classes_touched.value).ok_or(StepError::MissingDataClasses)?;
    if start.data_class.value != data_class {
        return Err(StepError::InvalidStepHistory);
    }
    Ok(())
}

fn validate_step_record(step: &Step) -> Result<(), StepError> {
    validate_run_id(&step.run_id.value)?;
    if step.provider_kind.value.trim().is_empty() {
        return Err(StepError::InvalidStepHistory);
    }
    if step
        .model_ref
        .value
        .as_ref()
        .is_some_and(|model_ref| model_ref.trim().is_empty())
    {
        return Err(StepError::InvalidStepHistory);
    }
    if step.data_classes_touched.value.is_empty() {
        return Err(StepError::InvalidStepHistory);
    }
    let data_class =
        most_restrictive(&step.data_classes_touched.value).ok_or(StepError::InvalidStepHistory)?;
    if step.data_class.value != data_class {
        return Err(StepError::InvalidStepHistory);
    }
    if step.schema_version.value != 1 {
        return Err(StepError::InvalidStepHistory);
    }
    let (step_run_sequence, step_sequence) = parse_step_id(&step.step_id.value)?;
    if step_run_sequence != run_sequence(&step.run_id.value)?
        || step_sequence != step.sequence.value
    {
        return Err(StepError::InvalidStepHistory);
    }
    match (step.state.value, step.disposition.value) {
        (StepState::Running, None)
            if step.latency_ms.value.is_none()
                && step.completed_at_epoch_seconds.value.is_none() => {}
        (StepState::Running, _) => return Err(StepError::InvalidStepHistory),
        (_, Some(disposition))
            if step.state.value == state_for(disposition)
                && step.latency_ms.value.is_some()
                && step.completed_at_epoch_seconds.value.is_some() => {}
        _ => return Err(StepError::InvalidStepHistory),
    }
    if step
        .completed_at_epoch_seconds
        .value
        .is_some_and(|completed_at| completed_at < step.started_at_epoch_seconds.value)
    {
        return Err(StepError::InvalidStepHistory);
    }
    Ok(())
}

fn parse_step_id(step_id: &str) -> Result<(&str, u32), StepError> {
    let Some(rest) = step_id.strip_prefix("step_") else {
        return Err(StepError::InvalidStepHistory);
    };
    let Some((run_sequence, step_sequence)) = rest.split_once('_') else {
        return Err(StepError::InvalidStepHistory);
    };
    if run_sequence.len() != 12
        || !run_sequence
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return Err(StepError::InvalidStepHistory);
    }
    if step_sequence.len() != 6
        || !step_sequence
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return Err(StepError::InvalidStepHistory);
    }
    step_sequence
        .parse::<u32>()
        .map(|sequence| (run_sequence, sequence))
        .map_err(|_| StepError::InvalidStepHistory)
}

fn run_sequence(run_id: &str) -> Result<&str, StepError> {
    let Some(sequence) = run_id.strip_prefix("run_") else {
        return Err(StepError::InvalidRunId);
    };
    if sequence.is_empty() || !sequence.chars().all(|character| character.is_ascii_digit()) {
        return Err(StepError::InvalidRunId);
    }
    Ok(sequence)
}

fn most_restrictive(data_classes: &[PrivacyDataClass]) -> Option<DataClass> {
    most_restrictive_privacy_data_class(data_classes)
}

fn validate_privacy_data_classes(
    data_classes: &[DataClass],
) -> Result<Vec<PrivacyDataClass>, StepError> {
    privacy_data_classes_from(data_classes).map_err(|_| StepError::InvalidDataClass)
}
