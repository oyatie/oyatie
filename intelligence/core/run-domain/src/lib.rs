//! Foundry run kernel.
//!
//! Pure typed records for capability invocation runs.

use std::collections::BTreeMap;

use data_boundary_kernel::{
    Classified, DataClass, OperationalDataClass, PrivacyDataClass,
    data_classes_from_privacy_data_classes, most_restrictive_privacy_data_class,
    privacy_data_classes_from,
};
use intelligence_capability_domain::AutonomyTier;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunState {
    Running,
    Succeeded,
    Failed,
    Cancelled,
    RejectedAutonomy,
    RejectedClass,
    RejectedBudget,
    RejectedLicense,
    RejectedPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunDisposition {
    Success,
    FailureClass,
    FailureProvider,
    FailureTimeout,
    FailureBudget,
    FailureAutonomy,
    FailureLicense,
    FailureAuthorization,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RunError {
    InvalidTenantId,
    InvalidCapabilityId,
    InvalidInitiatorId,
    InvalidRunHistory,
    MissingDataClasses,
    InvalidDataClass,
    EmptyRegion,
    EmptyIdempotencyKey,
    RunNotFound,
    RunNotRunning,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunStart {
    pub tenant_id: Classified<String>,
    pub capability_id: Classified<String>,
    pub initiator_user_id: Classified<String>,
    pub autonomy_tier_used: Classified<AutonomyTier>,
    pub data_classes_touched: Classified<Vec<PrivacyDataClass>>,
    pub region: Classified<String>,
    pub idempotency_key: Classified<String>,
    pub started_at_epoch_seconds: Classified<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Run {
    pub run_id: Classified<String>,
    pub tenant_id: Classified<String>,
    pub capability_id: Classified<String>,
    pub initiator_user_id: Classified<String>,
    pub autonomy_tier_used: Classified<AutonomyTier>,
    pub data_classes_touched: Classified<Vec<PrivacyDataClass>>,
    pub data_class: Classified<DataClass>,
    pub state: Classified<RunState>,
    pub disposition: Classified<Option<RunDisposition>>,
    pub region: Classified<String>,
    pub idempotency_key: Classified<String>,
    pub started_at_epoch_seconds: Classified<u64>,
    pub completed_at_epoch_seconds: Classified<Option<u64>>,
    pub schema_version: Classified<u32>,
}

impl RunStart {
    pub fn touched_privacy_data_classes(&self) -> &[PrivacyDataClass] {
        &self.data_classes_touched.value
    }

    /// Legacy run-start projection for ledger consumers that still persist raw
    /// `DataClass` labels. Run starts are constructed with typed privacy
    /// classes, making this projection lossless and incapable of introducing
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

impl Run {
    pub fn touched_privacy_data_classes(&self) -> &[PrivacyDataClass] {
        &self.data_classes_touched.value
    }

    /// Legacy run-ledger projection for ledger consumers that still persist raw
    /// `DataClass` labels. Runs store typed privacy classes, making this
    /// projection lossless and incapable of introducing operational or subject
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
pub struct RunLedger {
    runs: Classified<Vec<Run>>,
    latest_by_id: Classified<BTreeMap<String, usize>>,
    next_run_number: Classified<u64>,
}

impl Default for RunLedger {
    fn default() -> Self {
        Self {
            runs: Classified::new(Vec::new(), OperationalDataClass::Audit),
            latest_by_id: Classified::new(BTreeMap::new(), DataClass::InternalOnly),
            next_run_number: Classified::new(1, DataClass::InternalOnly),
        }
    }
}

impl RunStart {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: String,
        capability_id: String,
        initiator_user_id: String,
        autonomy_tier_used: AutonomyTier,
        data_classes_touched: Vec<PrivacyDataClass>,
        region: String,
        idempotency_key: String,
        started_at_epoch_seconds: u64,
    ) -> Result<Self, RunError> {
        Self::new_with_privacy_data_classes(
            tenant_id,
            capability_id,
            initiator_user_id,
            autonomy_tier_used,
            data_classes_touched,
            region,
            idempotency_key,
            started_at_epoch_seconds,
        )
    }

    /// Compatibility constructor for replay/config seams that still carry raw
    /// `DataClass` labels. Canonical run starts take `PrivacyDataClass`, and
    /// this path fails closed for operational markers and subject markers.
    #[allow(clippy::too_many_arguments)]
    pub fn try_from_legacy_data_classes_touched(
        tenant_id: String,
        capability_id: String,
        initiator_user_id: String,
        autonomy_tier_used: AutonomyTier,
        data_classes_touched: Vec<DataClass>,
        region: String,
        idempotency_key: String,
        started_at_epoch_seconds: u64,
    ) -> Result<Self, RunError> {
        let data_classes_touched = validate_privacy_data_classes(&data_classes_touched)?;
        Self::new(
            tenant_id,
            capability_id,
            initiator_user_id,
            autonomy_tier_used,
            data_classes_touched,
            region,
            idempotency_key,
            started_at_epoch_seconds,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_privacy_data_classes(
        tenant_id: String,
        capability_id: String,
        initiator_user_id: String,
        autonomy_tier_used: AutonomyTier,
        data_classes_touched: Vec<PrivacyDataClass>,
        region: String,
        idempotency_key: String,
        started_at_epoch_seconds: u64,
    ) -> Result<Self, RunError> {
        validate_tenant_id(&tenant_id)?;
        validate_capability_id(&capability_id)?;
        validate_initiator_id(&initiator_user_id)?;
        if data_classes_touched.is_empty() {
            return Err(RunError::MissingDataClasses);
        }
        Self::new_with_validated_privacy_data_classes(
            tenant_id,
            capability_id,
            initiator_user_id,
            autonomy_tier_used,
            data_classes_touched,
            region,
            idempotency_key,
            started_at_epoch_seconds,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_validated_privacy_data_classes(
        tenant_id: String,
        capability_id: String,
        initiator_user_id: String,
        autonomy_tier_used: AutonomyTier,
        data_classes_touched: Vec<PrivacyDataClass>,
        region: String,
        idempotency_key: String,
        started_at_epoch_seconds: u64,
    ) -> Result<Self, RunError> {
        if region.trim().is_empty() {
            return Err(RunError::EmptyRegion);
        }
        if idempotency_key.trim().is_empty() {
            return Err(RunError::EmptyIdempotencyKey);
        }
        Ok(Self {
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            capability_id: Classified::new(capability_id, DataClass::InternalOnly),
            initiator_user_id: Classified::new(initiator_user_id, DataClass::PiiIdentifying),
            autonomy_tier_used: Classified::new(autonomy_tier_used, DataClass::InternalOnly),
            data_classes_touched: Classified::new(data_classes_touched, DataClass::InternalOnly),
            region: Classified::new(region, DataClass::InternalOnly),
            idempotency_key: Classified::new(idempotency_key, DataClass::InternalOnly),
            started_at_epoch_seconds: Classified::new(
                started_at_epoch_seconds,
                DataClass::InternalOnly,
            ),
        })
    }
}

impl RunLedger {
    pub fn from_runs(runs: Vec<Run>) -> Result<Self, RunError> {
        let mut latest_by_id = BTreeMap::new();
        let mut max_run_number = 0;
        for (index, run) in runs.iter().enumerate() {
            validate_run_record(run)?;
            let run_number = parse_run_number(&run.run_id.value)?;
            if latest_by_id
                .insert(run.run_id.value.clone(), index)
                .is_some()
            {
                return Err(RunError::InvalidRunHistory);
            }
            max_run_number = max_run_number.max(run_number);
        }
        Ok(Self {
            runs: Classified::new(runs, OperationalDataClass::Audit),
            latest_by_id: Classified::new(latest_by_id, DataClass::InternalOnly),
            next_run_number: Classified::new(max_run_number + 1, DataClass::InternalOnly),
        })
    }

    pub fn start(&mut self, start: RunStart) -> Result<Run, RunError> {
        let run = self.build_run(start, RunState::Running, None, None)?;
        self.insert(run.clone());
        Ok(run)
    }

    pub fn reject(
        &mut self,
        start: RunStart,
        disposition: RunDisposition,
    ) -> Result<Run, RunError> {
        let state = rejected_state(disposition);
        let completed_at = Some(start.started_at_epoch_seconds.value);
        let run = self.build_run(start, state, Some(disposition), completed_at)?;
        self.insert(run.clone());
        Ok(run)
    }

    pub fn complete(
        &mut self,
        run_id: &str,
        disposition: RunDisposition,
        completed_at_epoch_seconds: u64,
    ) -> Result<Run, RunError> {
        let index = *self
            .latest_by_id
            .value
            .get(run_id)
            .ok_or(RunError::RunNotFound)?;
        let run = self
            .runs
            .value
            .get_mut(index)
            .ok_or(RunError::RunNotFound)?;
        if run.state.value != RunState::Running {
            return Err(RunError::RunNotRunning);
        }
        run.state = Classified::new(completed_state(disposition), DataClass::InternalOnly);
        run.disposition = Classified::new(Some(disposition), DataClass::InternalOnly);
        run.completed_at_epoch_seconds =
            Classified::new(Some(completed_at_epoch_seconds), DataClass::InternalOnly);
        Ok(run.clone())
    }

    pub fn runs(&self) -> &[Run] {
        &self.runs.value
    }

    fn build_run(
        &mut self,
        start: RunStart,
        state: RunState,
        disposition: Option<RunDisposition>,
        completed_at_epoch_seconds: Option<u64>,
    ) -> Result<Run, RunError> {
        let data_class = most_restrictive(&start.data_classes_touched.value)
            .ok_or(RunError::MissingDataClasses)?;
        let run_id = format!("run_{:012}", self.next_run_number.value);
        self.next_run_number.value += 1;
        Ok(Run {
            run_id: Classified::new(run_id, DataClass::InternalOnly),
            tenant_id: start.tenant_id,
            capability_id: start.capability_id,
            initiator_user_id: start.initiator_user_id,
            autonomy_tier_used: start.autonomy_tier_used,
            data_classes_touched: start.data_classes_touched,
            data_class: Classified::new(data_class, DataClass::InternalOnly),
            state: Classified::new(state, DataClass::InternalOnly),
            disposition: Classified::new(disposition, DataClass::InternalOnly),
            region: start.region,
            idempotency_key: start.idempotency_key,
            started_at_epoch_seconds: start.started_at_epoch_seconds,
            completed_at_epoch_seconds: Classified::new(
                completed_at_epoch_seconds,
                DataClass::InternalOnly,
            ),
            schema_version: Classified::new(1, DataClass::InternalOnly),
        })
    }

    fn insert(&mut self, run: Run) {
        self.latest_by_id
            .value
            .insert(run.run_id.value.clone(), self.runs.value.len());
        self.runs.value.push(run);
    }
}

fn rejected_state(disposition: RunDisposition) -> RunState {
    match disposition {
        RunDisposition::FailureBudget => RunState::RejectedBudget,
        RunDisposition::FailureAutonomy => RunState::RejectedAutonomy,
        RunDisposition::FailureClass => RunState::RejectedClass,
        RunDisposition::FailureLicense => RunState::RejectedLicense,
        RunDisposition::FailureAuthorization => RunState::RejectedPolicy,
        RunDisposition::Success
        | RunDisposition::FailureProvider
        | RunDisposition::FailureTimeout => RunState::Failed,
    }
}

fn completed_state(disposition: RunDisposition) -> RunState {
    match disposition {
        RunDisposition::Success => RunState::Succeeded,
        RunDisposition::FailureClass
        | RunDisposition::FailureProvider
        | RunDisposition::FailureTimeout
        | RunDisposition::FailureBudget
        | RunDisposition::FailureAutonomy
        | RunDisposition::FailureLicense
        | RunDisposition::FailureAuthorization => RunState::Failed,
    }
}

fn most_restrictive(data_classes: &[PrivacyDataClass]) -> Option<DataClass> {
    most_restrictive_privacy_data_class(data_classes)
}

fn validate_run_record(run: &Run) -> Result<(), RunError> {
    parse_run_number(&run.run_id.value)?;
    validate_tenant_id(&run.tenant_id.value)?;
    validate_capability_id(&run.capability_id.value)?;
    validate_initiator_id(&run.initiator_user_id.value)?;
    if run.data_classes_touched.value.is_empty() {
        return Err(RunError::InvalidRunHistory);
    }
    let data_class =
        most_restrictive(&run.data_classes_touched.value).ok_or(RunError::InvalidRunHistory)?;
    if run.data_class.value != data_class {
        return Err(RunError::InvalidRunHistory);
    }
    if run.region.value.trim().is_empty() || run.idempotency_key.value.trim().is_empty() {
        return Err(RunError::InvalidRunHistory);
    }
    if run.schema_version.value != 1 {
        return Err(RunError::InvalidRunHistory);
    }
    match (run.state.value, run.disposition.value) {
        (RunState::Running, None) if run.completed_at_epoch_seconds.value.is_none() => {}
        (RunState::Running, _) => return Err(RunError::InvalidRunHistory),
        (_, Some(disposition))
            if run.completed_at_epoch_seconds.value.is_some()
                && (run.state.value == completed_state(disposition)
                    || run.state.value == rejected_state(disposition)) => {}
        _ => return Err(RunError::InvalidRunHistory),
    }
    if run
        .completed_at_epoch_seconds
        .value
        .is_some_and(|completed_at| completed_at < run.started_at_epoch_seconds.value)
    {
        return Err(RunError::InvalidRunHistory);
    }
    Ok(())
}

fn parse_run_number(run_id: &str) -> Result<u64, RunError> {
    let Some(sequence) = run_id.strip_prefix("run_") else {
        return Err(RunError::InvalidRunHistory);
    };
    if sequence.len() != 12 || !sequence.chars().all(|character| character.is_ascii_digit()) {
        return Err(RunError::InvalidRunHistory);
    }
    sequence
        .parse::<u64>()
        .map_err(|_| RunError::InvalidRunHistory)
}

fn validate_privacy_data_classes(
    data_classes: &[DataClass],
) -> Result<Vec<PrivacyDataClass>, RunError> {
    privacy_data_classes_from(data_classes).map_err(|_| RunError::InvalidDataClass)
}

fn validate_tenant_id(tenant_id: &str) -> Result<(), RunError> {
    if !tenant_id.starts_with("ten_") {
        return Err(RunError::InvalidTenantId);
    }
    Ok(())
}

fn validate_capability_id(capability_id: &str) -> Result<(), RunError> {
    if !capability_id.starts_with("cap.") {
        return Err(RunError::InvalidCapabilityId);
    }
    Ok(())
}

fn validate_initiator_id(initiator_user_id: &str) -> Result<(), RunError> {
    if !initiator_user_id.starts_with("usr_") && !initiator_user_id.starts_with("svc_") {
        return Err(RunError::InvalidInitiatorId);
    }
    Ok(())
}
