//! Foundry evidence kernel.
//!
//! Pure append-only evidence records for capability invocation proof.

pub mod retention_policy;
pub use retention_policy::{
    RegulatorySchedule, RetentionDays, RetentionPolicy, RetentionPolicyError,
};
pub mod kr_acceptance;
pub use kr_acceptance::AcceptanceTestKind;

use std::collections::BTreeMap;

use data_boundary_kernel::{
    Classified, DataClass, OperationalDataClass, PrivacyDataClass,
    data_classes_from_privacy_data_classes, most_restrictive_privacy_data_class,
    privacy_data_classes_from,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EvidenceKind {
    CapabilityInvocation,
    ToolCall,
    ProviderCall,
    DataFlow,
    AutonomyDecision,
    ConsentCheck,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvidenceError {
    InvalidEvidenceId,
    InvalidTenantId,
    InvalidRunId,
    InvalidStepId,
    InvalidCapabilityId,
    EmptyFields,
    MissingDataClasses,
    InvalidDataClass,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceRecord {
    pub evidence_id: Classified<String>,
    pub run_id: Classified<String>,
    pub step_id: Classified<Option<String>>,
    pub tenant_id: Classified<String>,
    pub capability_id: Classified<String>,
    pub kind: Classified<EvidenceKind>,
    pub fields: Classified<BTreeMap<String, String>>,
    pub data_classes_touched: Classified<Vec<PrivacyDataClass>>,
    pub data_class: Classified<DataClass>,
    pub prev_hash: Classified<String>,
    pub hash: Classified<String>,
    pub timestamp_epoch_seconds: Classified<u64>,
    pub schema_version: Classified<u32>,
}

impl EvidenceRecord {
    pub fn touched_privacy_data_classes(&self) -> &[PrivacyDataClass] {
        &self.data_classes_touched.value
    }

    /// Legacy evidence-chain projection for consumers that still persist raw
    /// `DataClass` labels. Evidence records store typed privacy classes, so the
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
pub struct EvidenceChain {
    records: Classified<Vec<EvidenceRecord>>,
    next_evidence_number: Classified<u64>,
}

impl Default for EvidenceChain {
    fn default() -> Self {
        Self {
            records: Classified::new(Vec::new(), OperationalDataClass::Audit),
            next_evidence_number: Classified::new(1, DataClass::InternalOnly),
        }
    }
}

impl EvidenceChain {
    pub fn from_records(records: Vec<EvidenceRecord>) -> Result<Self, EvidenceError> {
        let mut chain = Self::default();
        for record in records {
            validate_record_shape(&record)?;
            chain.next_evidence_number.value = chain
                .next_evidence_number
                .value
                .max(extract_sequence(&record.evidence_id.value)?.saturating_add(1));
            chain.records.value.push(record);
        }
        Ok(chain)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn append(
        &mut self,
        tenant_id: String,
        run_id: String,
        step_id: Option<String>,
        capability_id: String,
        kind: EvidenceKind,
        fields: BTreeMap<String, String>,
        data_classes_touched: Vec<PrivacyDataClass>,
        timestamp_epoch_seconds: u64,
    ) -> Result<EvidenceRecord, EvidenceError> {
        self.append_with_privacy_data_classes(
            tenant_id,
            run_id,
            step_id,
            capability_id,
            kind,
            fields,
            data_classes_touched,
            timestamp_epoch_seconds,
        )
    }

    /// Compatibility append path for replay/config seams that still carry raw
    /// `DataClass` labels. Canonical evidence appends take `PrivacyDataClass`,
    /// and this path fails closed for operational markers and subject markers.
    #[allow(clippy::too_many_arguments)]
    pub fn try_append_legacy_data_classes_touched(
        &mut self,
        tenant_id: String,
        run_id: String,
        step_id: Option<String>,
        capability_id: String,
        kind: EvidenceKind,
        fields: BTreeMap<String, String>,
        data_classes_touched: Vec<DataClass>,
        timestamp_epoch_seconds: u64,
    ) -> Result<EvidenceRecord, EvidenceError> {
        let data_classes_touched = validate_privacy_data_classes(&data_classes_touched)?;
        self.append(
            tenant_id,
            run_id,
            step_id,
            capability_id,
            kind,
            fields,
            data_classes_touched,
            timestamp_epoch_seconds,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn append_with_privacy_data_classes(
        &mut self,
        tenant_id: String,
        run_id: String,
        step_id: Option<String>,
        capability_id: String,
        kind: EvidenceKind,
        fields: BTreeMap<String, String>,
        data_classes_touched: Vec<PrivacyDataClass>,
        timestamp_epoch_seconds: u64,
    ) -> Result<EvidenceRecord, EvidenceError> {
        validate_tenant_id(&tenant_id)?;
        validate_run_id(&run_id)?;
        if let Some(step_id) = &step_id {
            validate_step_id(step_id)?;
        }
        validate_capability_id(&capability_id)?;
        if fields.is_empty() {
            return Err(EvidenceError::EmptyFields);
        }
        if data_classes_touched.is_empty() {
            return Err(EvidenceError::MissingDataClasses);
        }
        self.append_validated(
            tenant_id,
            run_id,
            step_id,
            capability_id,
            kind,
            fields,
            data_classes_touched,
            timestamp_epoch_seconds,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn append_validated(
        &mut self,
        tenant_id: String,
        run_id: String,
        step_id: Option<String>,
        capability_id: String,
        kind: EvidenceKind,
        fields: BTreeMap<String, String>,
        data_classes_touched: Vec<PrivacyDataClass>,
        timestamp_epoch_seconds: u64,
    ) -> Result<EvidenceRecord, EvidenceError> {
        let evidence_id = format!("ev_{:012}", self.next_evidence_number.value);
        self.next_evidence_number.value += 1;
        let prev_hash = self
            .records
            .value
            .last()
            .map(|record| record.hash.value.clone())
            .unwrap_or_else(|| "GENESIS".to_string());
        let data_class = most_restrictive_data_class(&data_classes_touched)
            .ok_or(EvidenceError::MissingDataClasses)?;
        let hash = evidence_hash(EvidenceHashInput {
            evidence_id: &evidence_id,
            run_id: &run_id,
            step_id: step_id.as_deref(),
            tenant_id: &tenant_id,
            capability_id: &capability_id,
            kind,
            fields: &fields,
            data_classes_touched: &data_classes_touched,
            prev_hash: &prev_hash,
            timestamp_epoch_seconds,
        });
        let record = EvidenceRecord {
            evidence_id: Classified::new(evidence_id, DataClass::InternalOnly),
            run_id: Classified::new(run_id, DataClass::InternalOnly),
            step_id: Classified::new(step_id, DataClass::InternalOnly),
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            capability_id: Classified::new(capability_id, DataClass::InternalOnly),
            kind: Classified::new(kind, OperationalDataClass::Audit),
            fields: Classified::new(fields, OperationalDataClass::Audit),
            data_classes_touched: Classified::new(data_classes_touched, DataClass::InternalOnly),
            data_class: Classified::new(data_class, DataClass::InternalOnly),
            prev_hash: Classified::new(prev_hash, OperationalDataClass::Audit),
            hash: Classified::new(hash, OperationalDataClass::Audit),
            timestamp_epoch_seconds: Classified::new(
                timestamp_epoch_seconds,
                OperationalDataClass::Audit,
            ),
            schema_version: Classified::new(1, DataClass::InternalOnly),
        };
        self.records.value.push(record.clone());
        Ok(record)
    }

    pub fn records(&self) -> &[EvidenceRecord] {
        &self.records.value
    }

    pub fn root_hash(&self) -> Option<&str> {
        self.records
            .value
            .last()
            .map(|record| record.hash.value.as_str())
    }

    pub fn verify(&self) -> bool {
        let mut prev_hash = "GENESIS".to_string();
        for record in &self.records.value {
            if validate_record_shape(record).is_err() {
                return false;
            }
            if record.prev_hash.value != prev_hash {
                return false;
            }
            let expected = evidence_hash(EvidenceHashInput {
                evidence_id: &record.evidence_id.value,
                run_id: &record.run_id.value,
                step_id: record.step_id.value.as_deref(),
                tenant_id: &record.tenant_id.value,
                capability_id: &record.capability_id.value,
                kind: record.kind.value,
                fields: &record.fields.value,
                data_classes_touched: &record.data_classes_touched.value,
                prev_hash: &record.prev_hash.value,
                timestamp_epoch_seconds: record.timestamp_epoch_seconds.value,
            });
            if record.hash.value != expected {
                return false;
            }
            prev_hash = record.hash.value.clone();
        }
        true
    }
}

fn validate_record_shape(record: &EvidenceRecord) -> Result<(), EvidenceError> {
    validate_evidence_id(&record.evidence_id.value)?;
    validate_tenant_id(&record.tenant_id.value)?;
    validate_run_id(&record.run_id.value)?;
    if let Some(step_id) = &record.step_id.value {
        validate_step_id(step_id)?;
    }
    validate_capability_id(&record.capability_id.value)?;
    if record.fields.value.is_empty() {
        return Err(EvidenceError::EmptyFields);
    }
    if record.data_classes_touched.value.is_empty() {
        return Err(EvidenceError::MissingDataClasses);
    }
    let data_class = most_restrictive_data_class(&record.data_classes_touched.value)
        .ok_or(EvidenceError::MissingDataClasses)?;
    if record.data_class.value != data_class {
        return Err(EvidenceError::InvalidDataClass);
    }
    Ok(())
}

fn validate_privacy_data_classes(
    data_classes: &[DataClass],
) -> Result<Vec<PrivacyDataClass>, EvidenceError> {
    privacy_data_classes_from(data_classes).map_err(|_| EvidenceError::InvalidDataClass)
}

fn validate_evidence_id(evidence_id: &str) -> Result<(), EvidenceError> {
    let Some(sequence) = evidence_id.strip_prefix("ev_") else {
        return Err(EvidenceError::InvalidEvidenceId);
    };
    if sequence.len() != 12 || !sequence.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(EvidenceError::InvalidEvidenceId);
    }
    if sequence == "000000000000" {
        return Err(EvidenceError::InvalidEvidenceId);
    }
    Ok(())
}

fn validate_tenant_id(tenant_id: &str) -> Result<(), EvidenceError> {
    if !tenant_id.starts_with("ten_") {
        return Err(EvidenceError::InvalidTenantId);
    }
    Ok(())
}

fn validate_run_id(run_id: &str) -> Result<(), EvidenceError> {
    if !run_id.starts_with("run_") {
        return Err(EvidenceError::InvalidRunId);
    }
    Ok(())
}

fn validate_step_id(step_id: &str) -> Result<(), EvidenceError> {
    if !step_id.starts_with("step_") {
        return Err(EvidenceError::InvalidStepId);
    }
    Ok(())
}

fn validate_capability_id(capability_id: &str) -> Result<(), EvidenceError> {
    if !capability_id.starts_with("cap.") {
        return Err(EvidenceError::InvalidCapabilityId);
    }
    Ok(())
}

fn extract_sequence(evidence_id: &str) -> Result<u64, EvidenceError> {
    // ADR-0083 Tier 1: propagate the prefix-strip failure through the
    // existing `EvidenceError::InvalidEvidenceId` variant instead of
    // double-validating with `.expect()` after `validate_evidence_id`.
    validate_evidence_id(evidence_id)?;
    evidence_id
        .strip_prefix("ev_")
        .ok_or(EvidenceError::InvalidEvidenceId)?
        .parse()
        .map_err(|_| EvidenceError::InvalidEvidenceId)
}

fn most_restrictive_data_class(data_classes: &[PrivacyDataClass]) -> Option<DataClass> {
    most_restrictive_privacy_data_class(data_classes)
}

struct EvidenceHashInput<'a> {
    evidence_id: &'a str,
    run_id: &'a str,
    step_id: Option<&'a str>,
    tenant_id: &'a str,
    capability_id: &'a str,
    kind: EvidenceKind,
    fields: &'a BTreeMap<String, String>,
    data_classes_touched: &'a [PrivacyDataClass],
    prev_hash: &'a str,
    timestamp_epoch_seconds: u64,
}

fn evidence_hash(input: EvidenceHashInput<'_>) -> String {
    let mut material = format!(
        "{}|{}|{}|{}|{}|{}|{:?}|{}",
        input.prev_hash,
        input.evidence_id,
        input.run_id,
        input.step_id.unwrap_or("-"),
        input.tenant_id,
        input.capability_id,
        input.kind,
        input.timestamp_epoch_seconds,
    );
    for (key, value) in input.fields {
        material.push('|');
        material.push_str(key);
        material.push('=');
        material.push_str(value);
    }
    for data_class in input.data_classes_touched {
        material.push('|');
        material.push_str(data_class.data_class().pascal_label());
    }
    format!("fnv1a64:{:016x}", fnv1a64(material.as_bytes()))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
