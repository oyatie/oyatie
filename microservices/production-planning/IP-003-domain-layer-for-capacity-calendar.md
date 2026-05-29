---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0244, ADR-0252, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-set
ip_id: IP-003
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: PP-CRP (Capacity Requirements Planning) + factory calendar (T247/TFACS)
tenant_class: substrate
persona: shop-floor-supervisor
---

# IP-003: Domain layer for capacity-calendar

## A. Intent

The capacity calendar defines, for each work-center / resource / production-line, the **available capacity per time bucket** (shift, day, week). MRP and finite scheduling read it to decide whether planned orders can be scheduled forward (within lead time) or must be flagged for capacity levelling. In SAP S/4HANA the equivalent lives across `T247` (factory calendar), `TFACD/TFACS` (calendar/holiday rules), `KAPA` (work-center capacity master), and `CR_CAP_AVAIL` (capacity availability per interval).

This IP implements the **domain layer** for `capacity-calendar`: pure types, interval algebra (merge/clip/subtract for shutdown overlays), capacity-availability projection, and the rolling-horizon iterator. No I/O.

### A.1 SAP equivalence delta

| SAP entity | Oyatie aggregate / value object |
|---|---|
| `T247` factory calendar | `FactoryCalendar` aggregate (per-plant) |
| `TFACS` shift definition | `ShiftPattern` value object |
| `KAPA` capacity master | `WorkCenterCapacity` aggregate root |
| `CR_CAP_AVAIL` availability | `CapacityInterval` value object stream |
| `CFAA` finite scheduling | `FiniteScheduler` use-case (NOT in domain — IP-009) |
| Setup time / processing time / teardown time | `SetupBlock`, `ProcessingBlock`, `TeardownBlock` enum-tagged intervals |

### A.2 Journey leg

Per `j101-multi-tier-supply-chain-formation`, after MRP explosion produces planned orders, **finite scheduling** assigns them to work-center capacity buckets. This domain owns the substrate that scheduling reads.

## B. Acceptance criteria

- **AC-1:** `FactoryCalendar::new(plant_code, base_days, exception_days)` rejects with `CalendarError::Overlap` if any exception_day overlaps another.
- **AC-2:** `WorkCenterCapacity::available_intervals(window)` returns a non-overlapping, time-sorted `Vec<CapacityInterval>` clipped to the requested window.
- **AC-3:** Interval algebra: `subtract(downtime: &CapacityInterval)` correctly produces 0/1/2 result intervals; tested for the 13 canonical interval-overlap cases (Allen's interval algebra: before/meets/overlaps/finished-by/contains/starts/equals/started-by/during/finishes/overlapped-by/met-by/after).
- **AC-4:** Tenant invariant: `WorkCenterCapacity.tenant_id` must match `FactoryCalendar.tenant_id`.
- **AC-5:** Determinism: re-projecting `available_intervals(window)` is pure-function (no internal mutation).
- **AC-6:** Capacity utilization > 120% on any sub-bucket flags an anomaly `CapacityAnomaly::Overload` (consumed by IP-022 alt-routing branch).
- **AC-7:** Cedar default-deny preserved at every public entry.
- **AC-8:** HLC ordering preserved on all interval boundaries.

## C. Verification

```bash
cargo test -p oya-production-planning-capacity-domain -- factory_calendar::
cargo test -p oya-production-planning-capacity-domain -- interval_algebra_allen_13_cases
cargo test -p oya-production-planning-capacity-domain -- available_intervals_respects_shutdowns
cargo test -p oya-production-planning-capacity-domain -- shift_pattern_24x7_three_shift
cargo test -p oya-production-planning-capacity-domain -- shift_pattern_5x2_day_only
cargo test -p oya-production-planning-capacity-domain -- exception_holiday_carves_out_capacity
cargo test -p oya-production-planning-capacity-domain -- overlapping_exception_days_rejected
cargo test -p oya-production-planning-capacity-domain -- cross_tenant_input_rejected
cargo test -p oya-production-planning-capacity-domain -- overload_anomaly_detected
cargo bench -p oya-production-planning-capacity-domain -- project_horizon_30d_workcenters_50
```

Coverage ≥ 95% line, ≥ 90% branch.

## D. Detailed mechanics

### D-1. Aggregate roots

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct FactoryCalendar {
    tenant_id: TenantId,
    plant_code: PlantCode,
    calendar_id: CalendarId,
    base_pattern: WeeklyPattern,          // Mon..Sun shift assignments
    exception_days: Vec<ExceptionDay>,
    timezone: TimeZoneId,                 // IANA, e.g., Asia/Seoul
    effective_from: Hlc,
    superseded_at: Option<Hlc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkCenterCapacity {
    tenant_id: TenantId,
    work_center_id: WorkCenterId,
    plant_code: PlantCode,
    calendar_id: CalendarId,              // factory calendar reference
    nominal_capacity_per_shift: Decimal,  // e.g., 480 minutes
    shifts: Vec<ShiftPattern>,
    downtime_overlays: Vec<DowntimeOverlay>, // maintenance / planned shutdown
    utilization_target_pct: Decimal,      // 0..=100
    effective_from: Hlc,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapacityInterval {
    start: Hlc,
    end: Hlc,
    available_minutes: Decimal,
    state: CapacityState,                  // Available / SetupOnly / DowntimePlanned / DowntimeUnplanned
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionDay {
    day: Date,
    classification: ExceptionClass,        // PublicHoliday / PlantHoliday / OvertimeAllowed / MaintenanceShutdown
    overrides: Option<DayPattern>,         // None = full closure; Some = custom shifts
}
```

### D-2. Interval algebra (Allen's 13 relations)

```rust
impl CapacityInterval {
    pub fn subtract(&self, downtime: &CapacityInterval) -> Vec<CapacityInterval> {
        use AllenRelation::*;
        match self.allen_relation_to(downtime) {
            Before | After | Meets | MetBy => vec![self.clone()],
            Equals | Starts | StartedBy | Finishes | FinishedBy | During => vec![],
            Contains => vec![
                CapacityInterval { start: self.start, end: downtime.start, .. },
                CapacityInterval { start: downtime.end, end: self.end, .. },
            ],
            Overlaps => vec![CapacityInterval { start: self.start, end: downtime.start, .. }],
            OverlappedBy => vec![CapacityInterval { start: downtime.end, end: self.end, .. }],
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AllenRelation { Before, After, Meets, MetBy, Equals, Starts, StartedBy,
                         Finishes, FinishedBy, During, Contains, Overlaps, OverlappedBy }
```

### D-3. Shift-pattern model

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ShiftPattern {
    shift_id: ShiftId,
    name: String,                          // "morning" / "afternoon" / "night" / "weekend-overtime"
    start_local: NaiveTime,
    end_local: NaiveTime,                  // may wrap past midnight
    break_minutes: Decimal,
    days_of_week: BitSet7,                 // Mon=0..Sun=6
    capacity_factor: Decimal,              // 0..=1.5 (overtime factor)
}
```

### D-4. Available-interval projection

```rust
impl WorkCenterCapacity {
    pub fn available_intervals(
        &self,
        calendar: &FactoryCalendar,
        window: Range<Hlc>,
    ) -> Result<Vec<CapacityInterval>, CapacityError> {
        if self.tenant_id != calendar.tenant_id { return Err(CapacityError::CrossTenant); }
        // 1. Start with nominal shift intervals over window
        let nominal = self.expand_shifts(calendar, &window)?;
        // 2. Subtract calendar exceptions (holidays, plant shutdowns)
        let after_holidays = nominal.into_iter()
            .flat_map(|iv| calendar.subtract_exceptions(iv, &window))
            .collect::<Vec<_>>();
        // 3. Subtract per-work-center downtime overlays (maintenance windows)
        let final_intervals = after_holidays.into_iter()
            .flat_map(|iv| self.subtract_downtime(iv))
            .collect::<Vec<_>>();
        Ok(final_intervals)
    }
}
```

### D-5. Anomaly detection

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum CapacityAnomaly {
    Overload { work_center_id: WorkCenterId, sub_bucket: Hlc, utilization_pct: Decimal },
    NegativeCapacity { interval: CapacityInterval },
    DowntimeOverlap { d1: DowntimeOverlay, d2: DowntimeOverlay },
}
```

### D-6. Typed errors

```rust
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CapacityError {
    #[error("cross-tenant: {a} vs {b}")] CrossTenant,
    #[error("calendar overlap: {d1:?} vs {d2:?}")] Overlap { d1: ExceptionDay, d2: ExceptionDay },
    #[error("shift wraps invalid: start={start} end={end}")] InvalidShiftWrap { start: NaiveTime, end: NaiveTime },
    #[error("downtime end <= start")] InvalidDowntimeOrder,
    #[error("capacity factor outside [0,1.5]: {f}")] InvalidCapacityFactor { f: Decimal },
}
```

### D-7. Audit-event class

`EVT-PRODUCTION_PLANNING-CAPACITY_CALENDAR-IP_ACCEPTED` per ADR-0263; emitted on calendar publish / capacity master edit / downtime overlay add.

### D-8. SLO contribution

In-process: `available_intervals` over a 30-day window across 50 work-centers ≤ 12ms P95. Feeds IP-009 (capacity usecase) and IP-020 (finite scheduling).

### D-9. Cross-µservice consumers

| Consumer | Mode | Purpose |
|---|---|---|
| `plant-maintenance` | publishes downtime overlays via AsyncAPI | maintenance windows shape capacity |
| `quality-management` | publishes inspection-hold overlays | failed-inspection lots block work-centers |
| `warehouse` | ontology read | staging-time feasibility |
| `costing` | ontology read | utilization → variance |

## E. Failure modes & recovery

### E-1. Calendar overlap on construction
**Detection:** `CalendarError::Overlap`.
**Behaviour:** Constructor rejects; aggregate never instantiated.
**Recovery:** Operator resolves the offending day (typically a holiday + plant-shutdown overlap); runbook `runbooks/factory-calendar-overlap.md`.

### E-2. Downtime overlay sneaks past tenant boundary
**Detection:** `CapacityError::CrossTenant`.
**Behaviour:** Projection rejects; security audit emitted.
**Recovery:** Re-check principal token + AsyncAPI consumer scope.

### E-3. Shift wraps past midnight incorrectly
**Detection:** `CapacityError::InvalidShiftWrap`.
**Behaviour:** Aggregate construction fails.
**Recovery:** Operator splits the wrap into a 23:00–24:00 + 00:00–06:00 pair.

### E-4. Utilization > 120% sustained
**Detection:** `CapacityAnomaly::Overload` repeated across 3+ sub-buckets in horizon.
**Behaviour:** Anomaly attached to projection; IP-022 alt-routing branch consumes.
**Recovery:** Capacity-leveling workflow runs (IP-020).

## F. Migration

Phase 1: domain.
Phase 2 (IP-009): usecase wiring.
Phase 3 (IP-013): persistence + AsyncAPI dispatch.

Rollback: feature flag `production_planning_capacity_v1` → false.

## G. References

- ADR-0105, ADR-0244, ADR-0252, ADR-0263, ADR-0297, ADR-0315.
- Allen, J. F. (1983). "Maintaining knowledge about temporal intervals." Comm. ACM 26(11): 832–843.
- SAP Help: PP-CRP (`CR01`/`CR02` work-center master, `T247` calendar).
- Benchmarks: SAP PP-CRP | Oracle Fusion Manufacturing capacity | Siemens Opcenter APS | Dassault DELMIA Quintiq | PlanetTogether.

## H. Out-of-scope

- Finite scheduling algorithm (IP-020 + IP-022).
- Persistence / outbox (IP-013).
- Maintenance-window source (owned by `plant-maintenance`).

— end IP-003 —
