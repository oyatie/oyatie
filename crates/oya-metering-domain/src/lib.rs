//! Platform metering event kernel.
//!
//! One append-oriented metering shape is shared by SaaS, Cloud, Foundry,
//! Search, Ads, Marketplace, and vertical products so downstream billing and
//! FinOps do not need per-axis event dialects.

use std::collections::{BTreeMap, BTreeSet};

use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const METER_EVENT_SCHEMA_VERSION: u32 = 1;
const METER_EVENT_ID_PREFIX: &str = "mtr_";
const TENANT_ID_PREFIX: &str = "ten_";
const CAPABILITY_ID_PREFIX: &str = "cap.";
const IDEMPOTENCY_KEY_PREFIX: &str = "idem_";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MeterEventId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CapabilityId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IdempotencyKey {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PlaneTag {
    Control,
    Data,
    Analytics,
    Audit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AxisId {
    Saas,
    Foundry,
    Cloud,
    Search,
    Ads,
    Marketplace,
    Vertical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MeterUnitKind {
    Request,
    ByteIn,
    ByteOut,
    Millisecond,
    GpuSecond,
    LlmToken,
    ResourceSecond,
    StorageGbSecond,
    EgressGb,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MeterUnit {
    pub kind: MeterUnitKind,      // data_class: INTERNAL_ONLY
    pub quantity_microunits: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MeterUnits {
    pub units: Vec<MeterUnit>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterEventCreate {
    pub id: String,                     // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub capability_id: String,          // data_class: INTERNAL_ONLY
    pub plane: PlaneTag,                // data_class: INTERNAL_ONLY
    pub units: Vec<MeterUnit>,          // data_class: INTERNAL_ONLY
    pub source_axis: AxisId,            // data_class: INTERNAL_ONLY
    pub recorded_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub idempotency_key: String,        // data_class: INTERNAL_ONLY
    pub data_class: DataClass,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterEvent {
    pub id: Classified<MeterEventId>,  // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub capability_id: Classified<CapabilityId>, // data_class: INTERNAL_ONLY
    pub plane: Classified<PlaneTag>,   // data_class: INTERNAL_ONLY
    pub units: Classified<MeterUnits>, // data_class: INTERNAL_ONLY
    pub source_axis: Classified<AxisId>, // data_class: INTERNAL_ONLY
    pub recorded_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<IdempotencyKey>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MeteringError {
    InvalidMeterEventId,
    InvalidTenantId,
    InvalidCapabilityId,
    EmptyUnits,
    InvalidUnitQuantity,
    DuplicateUnitKind,
    InvalidRecordedAt,
    InvalidIdempotencyKey,
    InvalidDataClass,
    DuplicateMeterEvent,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Meter {
    events_by_id: BTreeMap<MeterEventId, MeterEvent>,
    events_by_idempotency: BTreeMap<IdempotencyKey, MeterEventId>,
}

impl MeterEventId {
    pub fn new(value: impl Into<String>) -> Result<Self, MeteringError> {
        prefixed_id(
            value.into(),
            METER_EVENT_ID_PREFIX,
            MeteringError::InvalidMeterEventId,
        )
        .map(|value| Self { value })
    }
}

impl CapabilityId {
    pub fn new(value: impl Into<String>) -> Result<Self, MeteringError> {
        prefixed_id(
            value.into(),
            CAPABILITY_ID_PREFIX,
            MeteringError::InvalidCapabilityId,
        )
        .map(|value| Self { value })
    }
}

impl IdempotencyKey {
    pub fn new(value: impl Into<String>) -> Result<Self, MeteringError> {
        prefixed_id(
            value.into(),
            IDEMPOTENCY_KEY_PREFIX,
            MeteringError::InvalidIdempotencyKey,
        )
        .map(|value| Self { value })
    }
}

impl MeterUnit {
    pub fn new(kind: MeterUnitKind, quantity_microunits: u64) -> Result<Self, MeteringError> {
        if quantity_microunits == 0 {
            return Err(MeteringError::InvalidUnitQuantity);
        }
        Ok(Self {
            kind,
            quantity_microunits,
        })
    }
}

impl MeterUnits {
    pub fn new(units: Vec<MeterUnit>) -> Result<Self, MeteringError> {
        if units.is_empty() {
            return Err(MeteringError::EmptyUnits);
        }
        let mut seen = BTreeSet::new();
        for unit in &units {
            if unit.quantity_microunits == 0 {
                return Err(MeteringError::InvalidUnitQuantity);
            }
            if !seen.insert(unit.kind) {
                return Err(MeteringError::DuplicateUnitKind);
            }
        }
        Ok(Self { units })
    }
}

impl MeterEvent {
    pub fn new(input: MeterEventCreate) -> Result<Self, MeteringError> {
        let id = MeterEventId::new(input.id)?;
        validate_tenant_id(&input.tenant_id)?;
        let capability_id = CapabilityId::new(input.capability_id)?;
        let units = MeterUnits::new(input.units)?;
        if input.recorded_at_epoch_seconds == 0 {
            return Err(MeteringError::InvalidRecordedAt);
        }
        let idempotency_key = IdempotencyKey::new(input.idempotency_key)?;
        let data_class = public_data_class(input.data_class)?;
        Ok(Self {
            id: internal(id),
            tenant_id: internal(input.tenant_id),
            capability_id: internal(capability_id),
            plane: internal(input.plane),
            units: internal(units),
            source_axis: internal(input.source_axis),
            recorded_at_epoch_seconds: internal(input.recorded_at_epoch_seconds),
            idempotency_key: internal(idempotency_key),
            data_class: public(data_class),
            schema_version: public(METER_EVENT_SCHEMA_VERSION),
        })
    }
}

impl Meter {
    pub fn record(&mut self, input: MeterEventCreate) -> Result<MeterEvent, MeteringError> {
        let event = MeterEvent::new(input)?;
        if let Some(existing_id) = self.events_by_idempotency.get(&event.idempotency_key.value) {
            return self
                .events_by_id
                .get(existing_id)
                .cloned()
                .ok_or(MeteringError::DuplicateMeterEvent);
        }
        if self.events_by_id.contains_key(&event.id.value) {
            return Err(MeteringError::DuplicateMeterEvent);
        }
        self.events_by_id
            .insert(event.id.value.clone(), event.clone());
        self.events_by_idempotency
            .insert(event.idempotency_key.value.clone(), event.id.value.clone());
        Ok(event)
    }

    pub fn get(&self, id: &MeterEventId) -> Option<&MeterEvent> {
        self.events_by_id.get(id)
    }

    pub fn events(&self) -> impl Iterator<Item = &MeterEvent> {
        self.events_by_id.values()
    }
}

pub fn meter_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, MeteringError> {
    public_data_class(data_class)
}

fn public_data_class(data_class: DataClass) -> Result<PrivacyDataClass, MeteringError> {
    let data_class =
        PrivacyDataClass::new(data_class).map_err(|_| MeteringError::InvalidDataClass)?;
    if data_class.data_class() == DataClass::Public {
        Ok(data_class)
    } else {
        Err(MeteringError::InvalidDataClass)
    }
}

fn validate_tenant_id(value: &str) -> Result<(), MeteringError> {
    if value.starts_with(TENANT_ID_PREFIX) && value.len() > TENANT_ID_PREFIX.len() {
        Ok(())
    } else {
        Err(MeteringError::InvalidTenantId)
    }
}

fn prefixed_id(value: String, prefix: &str, error: MeteringError) -> Result<String, MeteringError> {
    if value.starts_with(prefix) && value.len() > prefix.len() {
        Ok(value)
    } else {
        Err(error)
    }
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event_create() -> MeterEventCreate {
        MeterEventCreate {
            id: "mtr_cloud_001".to_string(),
            tenant_id: "ten_kr".to_string(),
            capability_id: "cap.cloud.billing.resource-hour".to_string(),
            plane: PlaneTag::Data,
            units: vec![
                MeterUnit::new(MeterUnitKind::ResourceSecond, 3_600_000_000)
                    .expect("unit fixture is valid"),
            ],
            source_axis: AxisId::Cloud,
            recorded_at_epoch_seconds: 1_700_000_000,
            idempotency_key: "idem_ten_kr_resource_001".to_string(),
            data_class: DataClass::Public,
        }
    }

    #[test]
    fn records_meter_event_and_replays_by_idempotency_key() {
        let mut meter = Meter::default();
        let first = meter.record(event_create()).expect("first record succeeds");
        let replay = meter
            .record(MeterEventCreate {
                id: "mtr_cloud_002".to_string(),
                ..event_create()
            })
            .expect("idempotency replay returns the original event");

        assert_eq!(first.id.value, replay.id.value);
        assert_eq!(meter.events().count(), 1);
    }

    #[test]
    fn rejects_empty_duplicate_or_zero_units() {
        let empty_error = MeterEvent::new(MeterEventCreate {
            units: Vec::new(),
            ..event_create()
        })
        .expect_err("meter events require units");
        assert_eq!(empty_error, MeteringError::EmptyUnits);

        let duplicate_error = MeterEvent::new(MeterEventCreate {
            units: vec![
                MeterUnit::new(MeterUnitKind::Request, 1).expect("unit valid"),
                MeterUnit::new(MeterUnitKind::Request, 2).expect("unit valid"),
            ],
            ..event_create()
        })
        .expect_err("unit kinds are unique per event");
        assert_eq!(duplicate_error, MeteringError::DuplicateUnitKind);

        let zero_error = MeterUnit::new(MeterUnitKind::Request, 0)
            .expect_err("zero microunits are not billable evidence");
        assert_eq!(zero_error, MeteringError::InvalidUnitQuantity);
    }

    #[test]
    fn rejects_non_public_metering_metadata_class_and_bad_capability_id() {
        let class_error = MeterEvent::new(MeterEventCreate {
            data_class: DataClass::Audit,
            ..event_create()
        })
        .expect_err("meter metadata class must be public privacy metadata");
        assert_eq!(class_error, MeteringError::InvalidDataClass);

        let capability_error = MeterEvent::new(MeterEventCreate {
            capability_id: "cloud.billing".to_string(),
            ..event_create()
        })
        .expect_err("capability id must use canonical capability prefix");
        assert_eq!(capability_error, MeteringError::InvalidCapabilityId);
    }
}
