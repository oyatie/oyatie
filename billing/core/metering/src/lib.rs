//! Platform metering event kernel.
//!
//! One append-oriented metering shape is shared by SaaS, Cloud, Foundry,
//! Search, Ads, Marketplace, and vertical products so downstream billing and
//! FinOps do not need per-axis event dialects.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

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

// ---------------------------------------------------------------------------
// Window rollup kernel
// ---------------------------------------------------------------------------

/// Composite key for a metering rollup bucket.
///
/// Ordered by `(tenant_id, capability_id, unit_kind)` so `BTreeMap` output is
/// stable and deterministic across runs.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RollupKey {
    pub tenant_id: String,
    pub capability_id: String,
    pub unit_kind: MeterUnitKind,
}

/// Result of a window rollup.
///
/// `totals` maps each `RollupKey` to the sum of `quantity_microunits` for all
/// events within the window.  Accumulation uses saturating addition so u64
/// overflow never panics; the value is capped at `u64::MAX`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MeterRollup {
    pub totals: BTreeMap<RollupKey, u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuotaDecision {
    pub used_microunits: u64,      // data_class: INTERNAL_ONLY
    pub requested_microunits: u64, // data_class: INTERNAL_ONLY
    pub limit_microunits: u64,     // data_class: INTERNAL_ONLY
    pub remaining_microunits: u64, // data_class: INTERNAL_ONLY
    pub allowed: bool,             // data_class: INTERNAL_ONLY
}

/// Aggregate all events in `meter` whose `recorded_at_epoch_seconds` falls
/// within the closed interval `[window_start_epoch_s, window_end_epoch_s]`.
///
/// Events are grouped by `(tenant_id, capability_id, MeterUnitKind)` and their
/// `quantity_microunits` values are summed with saturating arithmetic.
///
/// Returns an empty `MeterRollup` if no events fall in the window (including
/// when `window_end_epoch_s < window_start_epoch_s`).
pub fn rollup_window(
    meter: &Meter,
    window_start_epoch_s: u64,
    window_end_epoch_s: u64,
) -> MeterRollup {
    let mut totals: BTreeMap<RollupKey, u64> = BTreeMap::new();
    for event in meter.events() {
        let ts = event.recorded_at_epoch_seconds.value;
        if ts < window_start_epoch_s || ts > window_end_epoch_s {
            continue;
        }
        let tenant_id = event.tenant_id.value.clone();
        let capability_id = event.capability_id.value.value.clone();
        for unit in &event.units.value.units {
            let key = RollupKey {
                tenant_id: tenant_id.clone(),
                capability_id: capability_id.clone(),
                unit_kind: unit.kind,
            };
            let entry = totals.entry(key).or_insert(0);
            *entry = entry.saturating_add(unit.quantity_microunits);
        }
    }
    MeterRollup { totals }
}

/// Project quota availability from an already-materialized metering rollup.
///
/// This is a provider-neutral snapshot check. Authoritative admission still
/// needs an outer reserve/consume/release workflow with concurrency control and
/// audit emission.
pub fn check_quota(
    rollup: &MeterRollup,
    key: &RollupKey,
    requested_microunits: u64,
    limit_microunits: u64,
) -> QuotaDecision {
    let used_microunits = rollup.totals.get(key).copied().unwrap_or(0);
    let remaining_microunits = limit_microunits.saturating_sub(used_microunits);
    let allowed = requested_microunits > 0
        && used_microunits
            .checked_add(requested_microunits)
            .is_some_and(|projected| projected <= limit_microunits);

    QuotaDecision {
        used_microunits,
        requested_microunits,
        limit_microunits,
        remaining_microunits,
        allowed,
    }
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
    if let Some(segment) = value.strip_prefix(TENANT_ID_PREFIX)
        && is_canonical_tenant_segment(segment)
    {
        Ok(())
    } else {
        Err(MeteringError::InvalidTenantId)
    }
}

fn prefixed_id(value: String, prefix: &str, error: MeteringError) -> Result<String, MeteringError> {
    if value.starts_with(prefix) && value.len() > prefix.len() && is_safe_reference(&value) {
        Ok(value)
    } else {
        Err(error)
    }
}

fn is_canonical_tenant_segment(segment: &str) -> bool {
    !segment.is_empty()
        && !segment.starts_with('-')
        && !segment.ends_with('-')
        && !segment.contains("--")
        && segment.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
}

fn is_safe_reference(value: &str) -> bool {
    value == value.trim()
        && !value.is_empty()
        && !value.contains("//")
        && !value.bytes().any(|byte| {
            byte.is_ascii_control()
                || byte.is_ascii_whitespace()
                || matches!(byte, b'\\' | b'?' | b'#')
        })
        && value
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
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
            tenant_id: "ten_alpha".to_string(),
            capability_id: "cap.cloud.billing.resource-hour".to_string(),
            plane: PlaneTag::Data,
            units: vec![
                MeterUnit::new(MeterUnitKind::ResourceSecond, 3_600_000_000)
                    .expect("unit fixture is valid"),
            ],
            source_axis: AxisId::Cloud,
            recorded_at_epoch_seconds: 1_700_000_000,
            idempotency_key: "idem_ten_alpha_resource_001".to_string(),
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
        assert_eq!(first.tenant_id.value, "ten_alpha");
        assert_eq!(
            first.idempotency_key.value.value,
            "idem_ten_alpha_resource_001"
        );
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

    // -----------------------------------------------------------------------
    // Window rollup kernel tests
    // -----------------------------------------------------------------------

    /// Helper: record an event with explicit id, tenant, cap, ts, idem key, and units.
    fn record_event(
        meter: &mut Meter,
        id: &str,
        tenant_id: &str,
        capability_id: &str,
        ts: u64,
        idem: &str,
        units: Vec<MeterUnit>,
    ) {
        meter
            .record(MeterEventCreate {
                id: id.to_string(),
                tenant_id: tenant_id.to_string(),
                capability_id: capability_id.to_string(),
                plane: PlaneTag::Data,
                units,
                source_axis: AxisId::Cloud,
                recorded_at_epoch_seconds: ts,
                idempotency_key: idem.to_string(),
                data_class: DataClass::Public,
            })
            .expect("test event must record successfully");
    }

    #[test]
    fn rollup_window_includes_only_events_within_window() {
        let mut meter = Meter::default();
        let cap = "cap.cloud.compute.request";
        let tenant = "ten_alpha";

        // ts=100 inside [100, 200]
        record_event(
            &mut meter,
            "mtr_w001",
            tenant,
            cap,
            100,
            "idem_w001",
            vec![MeterUnit::new(MeterUnitKind::Request, 10).unwrap()],
        );
        // ts=200 inside [100, 200] (inclusive upper bound)
        record_event(
            &mut meter,
            "mtr_w002",
            tenant,
            cap,
            200,
            "idem_w002",
            vec![MeterUnit::new(MeterUnitKind::Request, 20).unwrap()],
        );
        // ts=99 outside (before window)
        record_event(
            &mut meter,
            "mtr_w003",
            tenant,
            cap,
            99,
            "idem_w003",
            vec![MeterUnit::new(MeterUnitKind::Request, 999).unwrap()],
        );
        // ts=201 outside (after window)
        record_event(
            &mut meter,
            "mtr_w004",
            tenant,
            cap,
            201,
            "idem_w004",
            vec![MeterUnit::new(MeterUnitKind::Request, 999).unwrap()],
        );

        let rollup = rollup_window(&meter, 100, 200);
        assert_eq!(rollup.totals.len(), 1);
        let key = RollupKey {
            tenant_id: tenant.to_string(),
            capability_id: cap.to_string(),
            unit_kind: MeterUnitKind::Request,
        };
        assert_eq!(
            rollup.totals[&key], 30,
            "only events at ts=100 and ts=200 count"
        );
    }

    #[test]
    fn rollup_window_sums_correctly_per_tenant_capability_unit_kind() {
        let mut meter = Meter::default();
        let cap = "cap.cloud.compute.bytes";
        let tenant = "ten_beta";
        let ts = 1_000;

        record_event(
            &mut meter,
            "mtr_s001",
            tenant,
            cap,
            ts,
            "idem_s001",
            vec![MeterUnit::new(MeterUnitKind::ByteOut, 100).unwrap()],
        );
        record_event(
            &mut meter,
            "mtr_s002",
            tenant,
            cap,
            ts,
            "idem_s002",
            vec![MeterUnit::new(MeterUnitKind::ByteOut, 200).unwrap()],
        );
        record_event(
            &mut meter,
            "mtr_s003",
            tenant,
            cap,
            ts,
            "idem_s003",
            vec![MeterUnit::new(MeterUnitKind::ByteOut, 300).unwrap()],
        );

        let rollup = rollup_window(&meter, 0, u64::MAX);
        let key = RollupKey {
            tenant_id: tenant.to_string(),
            capability_id: cap.to_string(),
            unit_kind: MeterUnitKind::ByteOut,
        };
        assert_eq!(rollup.totals[&key], 600, "three ByteOut events sum to 600");
    }

    #[test]
    fn rollup_window_keeps_distinct_unit_kinds_separate() {
        let mut meter = Meter::default();
        let cap = "cap.cloud.compute.mixed";
        let tenant = "ten_gamma";
        let ts = 500;

        record_event(
            &mut meter,
            "mtr_d001",
            tenant,
            cap,
            ts,
            "idem_d001",
            vec![
                MeterUnit::new(MeterUnitKind::Request, 5).unwrap(),
                MeterUnit::new(MeterUnitKind::ByteIn, 1024).unwrap(),
            ],
        );
        record_event(
            &mut meter,
            "mtr_d002",
            tenant,
            cap,
            ts,
            "idem_d002",
            vec![MeterUnit::new(MeterUnitKind::Request, 3).unwrap()],
        );

        let rollup = rollup_window(&meter, 0, u64::MAX);
        let req_key = RollupKey {
            tenant_id: tenant.to_string(),
            capability_id: cap.to_string(),
            unit_kind: MeterUnitKind::Request,
        };
        let byte_key = RollupKey {
            tenant_id: tenant.to_string(),
            capability_id: cap.to_string(),
            unit_kind: MeterUnitKind::ByteIn,
        };
        assert_eq!(rollup.totals[&req_key], 8, "Request: 5+3");
        assert_eq!(rollup.totals[&byte_key], 1024, "ByteIn: 1024 only");
        assert_eq!(
            rollup.totals.len(),
            2,
            "exactly two distinct unit kind keys"
        );
    }

    #[test]
    fn rollup_window_idempotent_replay_does_not_double_count() {
        let mut meter = Meter::default();
        let cap = "cap.cloud.compute.idem";
        let tenant = "ten_delta";
        let ts = 300;

        // First record
        record_event(
            &mut meter,
            "mtr_i001",
            tenant,
            cap,
            ts,
            "idem_i001",
            vec![MeterUnit::new(MeterUnitKind::Request, 50).unwrap()],
        );
        // Replay with different event id but same idempotency key — Meter deduplicates
        record_event(
            &mut meter,
            "mtr_i002",
            tenant,
            cap,
            ts,
            "idem_i001", // same key
            vec![MeterUnit::new(MeterUnitKind::Request, 50).unwrap()],
        );

        let rollup = rollup_window(&meter, 0, u64::MAX);
        let key = RollupKey {
            tenant_id: tenant.to_string(),
            capability_id: cap.to_string(),
            unit_kind: MeterUnitKind::Request,
        };
        assert_eq!(rollup.totals[&key], 50, "replay does not double-count");
        assert_eq!(
            meter.events().count(),
            1,
            "meter holds only one deduplicated event"
        );
    }

    #[test]
    fn rollup_window_saturates_on_overflow() {
        let mut meter = Meter::default();
        let cap = "cap.cloud.compute.overflow";
        let tenant = "ten_epsilon";
        let ts = 1;

        // Record two events whose quantities sum beyond u64::MAX
        record_event(
            &mut meter,
            "mtr_o001",
            tenant,
            cap,
            ts,
            "idem_o001",
            vec![MeterUnit::new(MeterUnitKind::LlmToken, u64::MAX).unwrap()],
        );
        record_event(
            &mut meter,
            "mtr_o002",
            tenant,
            cap,
            ts,
            "idem_o002",
            vec![MeterUnit::new(MeterUnitKind::LlmToken, 1).unwrap()],
        );

        let rollup = rollup_window(&meter, 0, u64::MAX);
        let key = RollupKey {
            tenant_id: tenant.to_string(),
            capability_id: cap.to_string(),
            unit_kind: MeterUnitKind::LlmToken,
        };
        assert_eq!(
            rollup.totals[&key],
            u64::MAX,
            "overflow saturates at u64::MAX, does not panic"
        );
    }

    #[test]
    fn rollup_window_empty_and_inverted_window_return_empty() {
        let mut meter = Meter::default();
        record_event(
            &mut meter,
            "mtr_e001",
            "ten_zeta",
            "cap.cloud.compute.empty",
            500,
            "idem_e001",
            vec![MeterUnit::new(MeterUnitKind::Request, 1).unwrap()],
        );

        // Empty window: no events in [600, 700]
        let rollup_no_match = rollup_window(&meter, 600, 700);
        assert!(
            rollup_no_match.totals.is_empty(),
            "no events in window → empty rollup"
        );

        // Inverted window: end < start
        let rollup_inverted = rollup_window(&meter, 700, 100);
        assert!(
            rollup_inverted.totals.is_empty(),
            "inverted window → empty rollup"
        );
    }

    #[test]
    fn rollup_window_output_is_stable_ordered() {
        let mut meter = Meter::default();
        // Insert events for two tenants and two capabilities, in non-sorted order
        for (id, tenant, cap, idem) in [
            ("mtr_ord001", "ten_zz", "cap.cloud.b", "idem_ord001"),
            ("mtr_ord002", "ten_aa", "cap.cloud.b", "idem_ord002"),
            ("mtr_ord003", "ten_aa", "cap.cloud.a", "idem_ord003"),
        ] {
            record_event(
                &mut meter,
                id,
                tenant,
                cap,
                1000,
                idem,
                vec![MeterUnit::new(MeterUnitKind::Request, 1).unwrap()],
            );
        }

        let rollup = rollup_window(&meter, 0, u64::MAX);
        let keys: Vec<_> = rollup.totals.keys().collect();
        // BTreeMap guarantees ascending order; verify it matches what we expect
        assert_eq!(keys[0].tenant_id, "ten_aa");
        assert_eq!(keys[0].capability_id, "cap.cloud.a");
        assert_eq!(keys[1].tenant_id, "ten_aa");
        assert_eq!(keys[1].capability_id, "cap.cloud.b");
        assert_eq!(keys[2].tenant_id, "ten_zz");
        assert_eq!(keys[2].capability_id, "cap.cloud.b");
    }

    #[test]
    fn quota_check_admits_only_projected_usage_within_limit() {
        let mut meter = Meter::default();
        record_event(
            &mut meter,
            "mtr_q001",
            "ten_alpha",
            "cap.cloud.compute.request",
            100,
            "idem_q001",
            vec![MeterUnit::new(MeterUnitKind::Request, 75).unwrap()],
        );

        let rollup = rollup_window(&meter, 0, u64::MAX);
        let key = RollupKey {
            tenant_id: "ten_alpha".to_string(),
            capability_id: "cap.cloud.compute.request".to_string(),
            unit_kind: MeterUnitKind::Request,
        };
        let decision = check_quota(&rollup, &key, 25, 100);

        assert!(decision.allowed);
        assert_eq!(decision.used_microunits, 75);
        assert_eq!(decision.remaining_microunits, 25);

        let denied = check_quota(&rollup, &key, 26, 100);

        assert!(!denied.allowed);
        assert_eq!(denied.remaining_microunits, 25);
        assert!(!check_quota(&rollup, &key, 0, 100).allowed);
        assert!(!check_quota(&rollup, &key, u64::MAX, u64::MAX).allowed);
    }

    #[test]
    fn quota_check_is_tenant_capability_and_unit_scoped() {
        let mut meter = Meter::default();
        record_event(
            &mut meter,
            "mtr_q002",
            "ten_alpha",
            "cap.cloud.compute.request",
            100,
            "idem_q002",
            vec![MeterUnit::new(MeterUnitKind::Request, 100).unwrap()],
        );

        let rollup = rollup_window(&meter, 0, u64::MAX);
        let key = RollupKey {
            tenant_id: "ten_beta".to_string(),
            capability_id: "cap.cloud.compute.request".to_string(),
            unit_kind: MeterUnitKind::Request,
        };
        let decision = check_quota(&rollup, &key, 1, 1);

        assert!(decision.allowed);
        assert_eq!(decision.used_microunits, 0);
        assert_eq!(decision.remaining_microunits, 1);

        let other_capability = RollupKey {
            tenant_id: "ten_alpha".to_string(),
            capability_id: "cap.cloud.storage.request".to_string(),
            unit_kind: MeterUnitKind::Request,
        };
        let other_unit = RollupKey {
            tenant_id: "ten_alpha".to_string(),
            capability_id: "cap.cloud.compute.request".to_string(),
            unit_kind: MeterUnitKind::ByteIn,
        };

        assert!(check_quota(&rollup, &other_capability, 1, 1).allowed);
        assert!(check_quota(&rollup, &other_unit, 1, 1).allowed);
    }
}
