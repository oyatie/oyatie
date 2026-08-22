//! Cloud FinOps analytics kernel.
//!
//! This crate owns the `cloud.finops.report` contract: per-tenant per-axis
//! cost allocation, per-resource breakdown, budget guardrails, anomaly
//! detection, and gross-margin evidence. It consumes the platform metering
//! vocabulary and cloud billing money/rate-card references while remaining
//! adapter-free.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use billing_domain::{CurrencyCode, Money, RateCardRef};
use billing_metering::{AxisId, MeterEvent, MeterEventId, MeterUnitKind};
use cell_region::RegionCode;
use compute_resource::{CloudResourceError, ResourceId};
use data_boundary_kernel::{Classified, DataClass, DataClassMatcher, PrivacyDataClass};

const FINOPS_SCHEMA_VERSION: u32 = 1;
const MILLION_MICROUNITS: u128 = 1_000_000;
const TENANT_ID_PREFIX: &str = "ten_";
const REPORT_ID_PREFIX: &str = "finr_";
const ALLOCATION_ID_PREFIX: &str = "fca_";
const BUDGET_ID_PREFIX: &str = "fbg_";
const RECOMMENDATION_ID_PREFIX: &str = "frec_";
pub const GA_GROSS_MARGIN_TARGET_BPS: u16 = 5_000;
pub const STABLE_GROSS_MARGIN_TARGET_BPS: u16 = 3_000;
pub const MAX_REPORT_WINDOW_SECONDS: u64 = 92 * 24 * 60 * 60;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FinopsReportId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CostAllocationId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BudgetId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecommendationId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FinopsPeriod {
    pub start_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub end_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct UnitRate {
    pub minor_units_per_million_microunits: u64, // data_class: INTERNAL_ONLY
    pub cost_of_revenue_minor_units_per_million_microunits: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct AnomalyPolicy {
    pub spend_growth_threshold_bps: u16, // data_class: INTERNAL_ONLY
    pub min_absolute_delta_minor_units: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CostAnomalyKind {
    SpendSpike,
    BudgetSoftLimit,
    BudgetHardLimit,
    MarginBelowTarget,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecommendationKind {
    InvestigateSpendSpike,
    PurchaseCommitment,
    DownsizeResource,
    ReviewRateCard,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RateCardLineCreate {
    pub rate_card_ref: String,          // data_class: INTERNAL_ONLY
    pub region: String,                 // data_class: PUBLIC
    pub axis: AxisId,                   // data_class: INTERNAL_ONLY
    pub unit_kind: MeterUnitKind,       // data_class: INTERNAL_ONLY
    pub currency: String,               // data_class: INTERNAL_ONLY
    pub rate: UnitRate,                 // data_class: INTERNAL_ONLY
    pub effective_period: FinopsPeriod, // data_class: INTERNAL_ONLY
    pub data_class: DataClass,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RateCardLine {
    pub rate_card_ref: Classified<RateCardRef>, // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,         // data_class: PUBLIC
    pub axis: Classified<AxisId>,               // data_class: INTERNAL_ONLY
    pub unit_kind: Classified<MeterUnitKind>,   // data_class: INTERNAL_ONLY
    pub currency: Classified<CurrencyCode>,     // data_class: INTERNAL_ONLY
    pub rate: Classified<UnitRate>,             // data_class: INTERNAL_ONLY
    pub effective_period: Classified<FinopsPeriod>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,        // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CostAllocationCreate {
    pub id: String,              // data_class: INTERNAL_ONLY
    pub region: String,          // data_class: PUBLIC
    pub resource_id: String,     // data_class: INTERNAL_ONLY
    pub rate_card_ref: String,   // data_class: INTERNAL_ONLY
    pub meter_event: MeterEvent, // data_class: INTERNAL_ONLY
    pub data_class: DataClass,   // data_class: FINANCIAL
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CostAllocation {
    pub id: Classified<CostAllocationId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,   // data_class: PUBLIC
    pub axis: Classified<AxisId>,         // data_class: INTERNAL_ONLY
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub meter_event_id: Classified<MeterEventId>, // data_class: INTERNAL_ONLY
    pub rate_card_ref: Classified<RateCardRef>, // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub actual_cost: Classified<Money>,   // data_class: FINANCIAL
    pub cost_of_revenue: Classified<Money>, // data_class: FINANCIAL
    pub gross_margin_bps: Classified<u16>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: FINANCIAL
    pub schema_version: Classified<u32>,  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AxisBudgetCreate {
    pub id: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub region: String,          // data_class: PUBLIC
    pub axis: AxisId,            // data_class: INTERNAL_ONLY
    pub period: FinopsPeriod,    // data_class: INTERNAL_ONLY
    pub budget: Money,           // data_class: FINANCIAL
    pub soft_threshold_bps: u16, // data_class: INTERNAL_ONLY
    pub hard_threshold_bps: u16, // data_class: INTERNAL_ONLY
    pub data_class: DataClass,   // data_class: FINANCIAL
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AxisBudget {
    pub id: Classified<BudgetId>,            // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub axis: Classified<AxisId>,            // data_class: INTERNAL_ONLY
    pub period: Classified<FinopsPeriod>,    // data_class: INTERNAL_ONLY
    pub budget: Classified<Money>,           // data_class: FINANCIAL
    pub soft_threshold_bps: Classified<u16>, // data_class: INTERNAL_ONLY
    pub hard_threshold_bps: Classified<u16>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: FINANCIAL
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AxisCostBreakdown {
    pub axis: AxisId,                        // data_class: INTERNAL_ONLY
    pub actual_cost: Money,                  // data_class: FINANCIAL
    pub cost_of_revenue: Money,              // data_class: FINANCIAL
    pub gross_margin_bps: u16,               // data_class: INTERNAL_ONLY
    pub budget: Option<Money>,               // data_class: FINANCIAL
    pub budget_utilization_bps: Option<u16>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceCostBreakdown {
    pub resource_id: ResourceId, // data_class: INTERNAL_ONLY
    pub axis: AxisId,            // data_class: INTERNAL_ONLY
    pub actual_cost: Money,      // data_class: FINANCIAL
    pub cost_of_revenue: Money,  // data_class: FINANCIAL
    pub gross_margin_bps: u16,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CostAnomaly {
    pub kind: CostAnomalyKind,           // data_class: INTERNAL_ONLY
    pub axis: AxisId,                    // data_class: INTERNAL_ONLY
    pub resource_id: Option<ResourceId>, // data_class: INTERNAL_ONLY
    pub actual_cost: Money,              // data_class: FINANCIAL
    pub baseline_cost: Option<Money>,    // data_class: FINANCIAL
    pub threshold_bps: u16,              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinopsRecommendation {
    pub id: RecommendationId,              // data_class: INTERNAL_ONLY
    pub kind: RecommendationKind,          // data_class: INTERNAL_ONLY
    pub axis: AxisId,                      // data_class: INTERNAL_ONLY
    pub resource_id: Option<ResourceId>,   // data_class: INTERNAL_ONLY
    pub evidence_anomaly: CostAnomalyKind, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinopsReportRequest {
    pub id: String,                            // data_class: INTERNAL_ONLY
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub region: String,                        // data_class: PUBLIC
    pub period: FinopsPeriod,                  // data_class: INTERNAL_ONLY
    pub baseline_period: Option<FinopsPeriod>, // data_class: INTERNAL_ONLY
    pub axes: Vec<AxisId>,                     // data_class: INTERNAL_ONLY
    pub anomaly_policy: AnomalyPolicy,         // data_class: INTERNAL_ONLY
    pub minimum_gross_margin_bps: u16,         // data_class: INTERNAL_ONLY
    pub data_class: DataClass,                 // data_class: FINANCIAL
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinopsReport {
    pub id: Classified<FinopsReportId>,   // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,   // data_class: PUBLIC
    pub period: Classified<FinopsPeriod>, // data_class: INTERNAL_ONLY
    pub axes: Classified<Vec<AxisId>>,    // data_class: INTERNAL_ONLY
    pub axis_costs: Classified<Vec<AxisCostBreakdown>>, // data_class: FINANCIAL
    pub resource_costs: Classified<Vec<ResourceCostBreakdown>>, // data_class: FINANCIAL
    pub anomalies: Classified<Vec<CostAnomaly>>, // data_class: FINANCIAL
    pub recommendations: Classified<Vec<FinopsRecommendation>>, // data_class: INTERNAL_ONLY
    pub total_cost: Classified<Money>,    // data_class: FINANCIAL
    pub total_cost_of_revenue: Classified<Money>, // data_class: FINANCIAL
    pub gross_margin_bps: Classified<u16>, // data_class: INTERNAL_ONLY
    pub minimum_gross_margin_bps: Classified<u16>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: FINANCIAL
    pub schema_version: Classified<u32>,  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudFinopsError {
    InvalidReportId,
    InvalidCostAllocationId,
    InvalidBudgetId,
    InvalidRecommendationId,
    InvalidTenantId,
    InvalidRegion,
    InvalidResourceId,
    ResourceTenantMismatch,
    ResourceRegionMismatch,
    InvalidRateCardRef,
    InvalidRateCardLine,
    InvalidCurrency,
    InvalidPeriod,
    InvalidAxisSet,
    InvalidBudget,
    InvalidBudgetThreshold,
    InvalidAnomalyPolicy,
    InvalidGrossMarginTarget,
    InvalidDataClass,
    InvalidMeterEvent,
    MissingRateCardLine,
    CurrencyMismatch,
    CostOverflow,
    NonIntegralCost,
    NegativeGrossMargin,
    DuplicateRateCardLine,
    DuplicateAllocation,
    DuplicateMeterEvent,
    DuplicateBudget,
    DuplicateReport,
    NoReportData,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudFinopsLedger {
    rate_card_lines: BTreeMap<RateCardLineKey, RateCardLine>,
    allocations: BTreeMap<CostAllocationId, CostAllocation>,
    meter_event_index: BTreeSet<MeterEventId>,
    budgets: BTreeMap<BudgetKey, AxisBudget>,
    reports: BTreeMap<FinopsReportId, FinopsReport>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct RateCardLineKey {
    rate_card_ref: RateCardRef,
    region: RegionCode,
    axis: AxisId,
    unit_kind: MeterUnitKind,
    effective_start: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct BudgetKey {
    tenant_id: String,
    region: RegionCode,
    axis: AxisId,
    period: FinopsPeriod,
}

impl FinopsReportId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudFinopsError> {
        prefixed_id(
            value.into(),
            REPORT_ID_PREFIX,
            CloudFinopsError::InvalidReportId,
        )
        .map(|value| Self { value })
    }
}

impl CostAllocationId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudFinopsError> {
        prefixed_id(
            value.into(),
            ALLOCATION_ID_PREFIX,
            CloudFinopsError::InvalidCostAllocationId,
        )
        .map(|value| Self { value })
    }
}

impl BudgetId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudFinopsError> {
        prefixed_id(
            value.into(),
            BUDGET_ID_PREFIX,
            CloudFinopsError::InvalidBudgetId,
        )
        .map(|value| Self { value })
    }
}

impl RecommendationId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudFinopsError> {
        prefixed_id(
            value.into(),
            RECOMMENDATION_ID_PREFIX,
            CloudFinopsError::InvalidRecommendationId,
        )
        .map(|value| Self { value })
    }
}

impl FinopsPeriod {
    pub fn new(start_epoch_seconds: u64, end_epoch_seconds: u64) -> Result<Self, CloudFinopsError> {
        if start_epoch_seconds == 0
            || end_epoch_seconds <= start_epoch_seconds
            || end_epoch_seconds - start_epoch_seconds > MAX_REPORT_WINDOW_SECONDS
        {
            return Err(CloudFinopsError::InvalidPeriod);
        }
        Ok(Self {
            start_epoch_seconds,
            end_epoch_seconds,
        })
    }

    pub const fn contains(self, timestamp: u64) -> bool {
        self.start_epoch_seconds <= timestamp && timestamp < self.end_epoch_seconds
    }

    pub const fn duration(self) -> u64 {
        self.end_epoch_seconds - self.start_epoch_seconds
    }
}

impl UnitRate {
    pub fn new(
        minor_units_per_million_microunits: u64,
        cost_of_revenue_minor_units_per_million_microunits: u64,
    ) -> Result<Self, CloudFinopsError> {
        if minor_units_per_million_microunits == 0 {
            return Err(CloudFinopsError::InvalidRateCardLine);
        }
        if cost_of_revenue_minor_units_per_million_microunits > minor_units_per_million_microunits {
            return Err(CloudFinopsError::NegativeGrossMargin);
        }
        Ok(Self {
            minor_units_per_million_microunits,
            cost_of_revenue_minor_units_per_million_microunits,
        })
    }
}

impl AnomalyPolicy {
    pub fn new(
        spend_growth_threshold_bps: u16,
        min_absolute_delta_minor_units: u64,
    ) -> Result<Self, CloudFinopsError> {
        if spend_growth_threshold_bps == 0 || min_absolute_delta_minor_units == 0 {
            return Err(CloudFinopsError::InvalidAnomalyPolicy);
        }
        Ok(Self {
            spend_growth_threshold_bps,
            min_absolute_delta_minor_units,
        })
    }
}

impl RateCardLine {
    pub fn new(input: RateCardLineCreate) -> Result<Self, CloudFinopsError> {
        let rate_card_ref = RateCardRef::new(input.rate_card_ref)
            .map_err(|_| CloudFinopsError::InvalidRateCardRef)?;
        let region = RegionCode::new(input.region).map_err(|_| CloudFinopsError::InvalidRegion)?;
        let currency =
            CurrencyCode::new(input.currency).map_err(|_| CloudFinopsError::InvalidCurrency)?;
        let data_class = validate_financial_class(input.data_class)?;
        validate_rate_line_period(input.effective_period)?;
        UnitRate::new(
            input.rate.minor_units_per_million_microunits,
            input
                .rate
                .cost_of_revenue_minor_units_per_million_microunits,
        )?;
        Ok(Self {
            rate_card_ref: internal(rate_card_ref),
            region: public(region),
            axis: internal(input.axis),
            unit_kind: internal(input.unit_kind),
            currency: internal(currency),
            rate: internal(input.rate),
            effective_period: internal(input.effective_period),
            data_class: financial(data_class),
            schema_version: public(FINOPS_SCHEMA_VERSION),
        })
    }

    fn key(&self) -> RateCardLineKey {
        RateCardLineKey {
            rate_card_ref: self.rate_card_ref.value.clone(),
            region: self.region.value.clone(),
            axis: self.axis.value,
            unit_kind: self.unit_kind.value,
            effective_start: self.effective_period.value.start_epoch_seconds,
        }
    }

    fn applies_to(
        &self,
        rate_card_ref: &RateCardRef,
        region: &RegionCode,
        axis: AxisId,
        unit_kind: MeterUnitKind,
        timestamp: u64,
    ) -> bool {
        &self.rate_card_ref.value == rate_card_ref
            && &self.region.value == region
            && self.axis.value == axis
            && self.unit_kind.value == unit_kind
            && self.effective_period.value.contains(timestamp)
    }
}

impl CostAllocation {
    fn from_meter_event(
        input: CostAllocationCreate,
        rate_card_lines: impl Iterator<Item = RateCardLine>,
    ) -> Result<Self, CloudFinopsError> {
        let id = CostAllocationId::new(input.id)?;
        let tenant_id = input.meter_event.tenant_id.value.clone();
        validate_tenant_id(&tenant_id)?;
        let region = RegionCode::new(input.region).map_err(|_| CloudFinopsError::InvalidRegion)?;
        let resource_id = resource_for(&input.resource_id, &tenant_id, &region)?;
        let rate_card_ref = RateCardRef::new(input.rate_card_ref)
            .map_err(|_| CloudFinopsError::InvalidRateCardRef)?;
        let axis = input.meter_event.source_axis.value;
        let timestamp = input.meter_event.recorded_at_epoch_seconds.value;
        if timestamp == 0 {
            return Err(CloudFinopsError::InvalidMeterEvent);
        }
        let data_class = validate_financial_class(input.data_class)?;
        let lines = rate_card_lines.collect::<Vec<_>>();
        let (actual_cost, cost_of_revenue) = price_units(
            &input.meter_event.units.value.units,
            &lines,
            &rate_card_ref,
            &region,
            axis,
            timestamp,
        )?;
        let gross_margin_bps = gross_margin_bps(&actual_cost, &cost_of_revenue)?;
        Ok(Self {
            id: internal(id),
            tenant_id: internal(tenant_id),
            region: public(region),
            axis: internal(axis),
            resource_id: internal(resource_id),
            meter_event_id: internal(input.meter_event.id.value),
            rate_card_ref: internal(rate_card_ref),
            occurred_at_epoch_seconds: internal(timestamp),
            actual_cost: financial_value(actual_cost),
            cost_of_revenue: financial_value(cost_of_revenue),
            gross_margin_bps: internal(gross_margin_bps),
            data_class: financial(data_class),
            schema_version: public(FINOPS_SCHEMA_VERSION),
        })
    }
}

impl AxisBudget {
    pub fn new(input: AxisBudgetCreate) -> Result<Self, CloudFinopsError> {
        let id = BudgetId::new(input.id)?;
        validate_tenant_id(&input.tenant_id)?;
        let region = RegionCode::new(input.region).map_err(|_| CloudFinopsError::InvalidRegion)?;
        validate_rate_line_period(input.period)?;
        validate_budget_thresholds(input.soft_threshold_bps, input.hard_threshold_bps)?;
        let data_class = validate_financial_class(input.data_class)?;
        if input.budget.minor_units == 0 {
            return Err(CloudFinopsError::InvalidBudget);
        }
        Ok(Self {
            id: internal(id),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            axis: internal(input.axis),
            period: internal(input.period),
            budget: financial_value(input.budget),
            soft_threshold_bps: internal(input.soft_threshold_bps),
            hard_threshold_bps: internal(input.hard_threshold_bps),
            data_class: financial(data_class),
            schema_version: public(FINOPS_SCHEMA_VERSION),
        })
    }

    fn key(&self) -> BudgetKey {
        BudgetKey {
            tenant_id: self.tenant_id.value.clone(),
            region: self.region.value.clone(),
            axis: self.axis.value,
            period: self.period.value,
        }
    }
}

impl CloudFinopsLedger {
    pub fn add_rate_card_line(
        &mut self,
        input: RateCardLineCreate,
    ) -> Result<RateCardLine, CloudFinopsError> {
        let line = RateCardLine::new(input)?;
        let key = line.key();
        if self.rate_card_lines.insert(key, line.clone()).is_some() {
            return Err(CloudFinopsError::DuplicateRateCardLine);
        }
        Ok(line)
    }

    pub fn set_budget(&mut self, input: AxisBudgetCreate) -> Result<AxisBudget, CloudFinopsError> {
        let budget = AxisBudget::new(input)?;
        let key = budget.key();
        if self.budgets.insert(key, budget.clone()).is_some() {
            return Err(CloudFinopsError::DuplicateBudget);
        }
        Ok(budget)
    }

    pub fn record_allocation(
        &mut self,
        input: CostAllocationCreate,
    ) -> Result<CostAllocation, CloudFinopsError> {
        let lines = self.rate_card_lines.values().cloned();
        let allocation = CostAllocation::from_meter_event(input, lines)?;
        if self.allocations.contains_key(&allocation.id.value) {
            return Err(CloudFinopsError::DuplicateAllocation);
        }
        if !self
            .meter_event_index
            .insert(allocation.meter_event_id.value.clone())
        {
            return Err(CloudFinopsError::DuplicateMeterEvent);
        }
        self.allocations
            .insert(allocation.id.value.clone(), allocation.clone());
        Ok(allocation)
    }

    pub fn generate_report(
        &mut self,
        request: FinopsReportRequest,
    ) -> Result<FinopsReport, CloudFinopsError> {
        let report = self.build_report(request)?;
        if self
            .reports
            .insert(report.id.value.clone(), report.clone())
            .is_some()
        {
            return Err(CloudFinopsError::DuplicateReport);
        }
        Ok(report)
    }

    pub fn allocations(&self) -> impl Iterator<Item = &CostAllocation> {
        self.allocations.values()
    }

    fn build_report(&self, request: FinopsReportRequest) -> Result<FinopsReport, CloudFinopsError> {
        let id = FinopsReportId::new(request.id)?;
        validate_tenant_id(&request.tenant_id)?;
        let region =
            RegionCode::new(request.region).map_err(|_| CloudFinopsError::InvalidRegion)?;
        validate_rate_line_period(request.period)?;
        validate_gross_margin_target(request.minimum_gross_margin_bps)?;
        let anomaly_policy = AnomalyPolicy::new(
            request.anomaly_policy.spend_growth_threshold_bps,
            request.anomaly_policy.min_absolute_delta_minor_units,
        )?;
        let axes = validated_axes(request.axes)?;
        let data_class = validate_financial_class(request.data_class)?;
        if let Some(baseline) = request.baseline_period {
            validate_rate_line_period(baseline)?;
            if baseline.duration() != request.period.duration() {
                return Err(CloudFinopsError::InvalidPeriod);
            }
        }
        let current = self.matching_allocations(&request.tenant_id, &region, request.period, &axes);
        if current.is_empty() {
            return Err(CloudFinopsError::NoReportData);
        }
        let baseline = request
            .baseline_period
            .map(|period| self.matching_allocations(&request.tenant_id, &region, period, &axes))
            .unwrap_or_default();
        let budgets = self.matching_budgets(&request.tenant_id, &region, request.period, &axes);
        let currency = current[0].actual_cost.value.currency.clone();
        let axis_costs = axis_breakdowns(&axes, &current, &budgets, &currency)?;
        let resource_costs = resource_breakdowns(&current, &currency)?;
        let total_cost = sum_money(
            current
                .iter()
                .map(|allocation| &allocation.actual_cost.value),
            &currency,
        )?;
        let total_cost_of_revenue = sum_money(
            current
                .iter()
                .map(|allocation| &allocation.cost_of_revenue.value),
            &currency,
        )?;
        let report_gross_margin_bps = gross_margin_bps(&total_cost, &total_cost_of_revenue)?;
        let anomalies = detect_anomalies(
            &axis_costs,
            &baseline,
            &budgets,
            &currency,
            anomaly_policy,
            request.minimum_gross_margin_bps,
        )?;
        let recommendations = recommendations_for(&anomalies)?;
        Ok(FinopsReport {
            id: internal(id),
            tenant_id: internal(request.tenant_id),
            region: public(region),
            period: internal(request.period),
            axes: internal(axes),
            axis_costs: financial_value(axis_costs),
            resource_costs: financial_value(resource_costs),
            anomalies: financial_value(anomalies),
            recommendations: internal(recommendations),
            total_cost: financial_value(total_cost),
            total_cost_of_revenue: financial_value(total_cost_of_revenue),
            gross_margin_bps: internal(report_gross_margin_bps),
            minimum_gross_margin_bps: internal(request.minimum_gross_margin_bps),
            data_class: financial(data_class),
            schema_version: public(FINOPS_SCHEMA_VERSION),
        })
    }

    fn matching_allocations(
        &self,
        tenant_id: &str,
        region: &RegionCode,
        period: FinopsPeriod,
        axes: &[AxisId],
    ) -> Vec<CostAllocation> {
        self.allocations
            .values()
            .filter(|allocation| {
                allocation.tenant_id.value == tenant_id
                    && &allocation.region.value == region
                    && period.contains(allocation.occurred_at_epoch_seconds.value)
                    && axes.contains(&allocation.axis.value)
            })
            .cloned()
            .collect()
    }

    fn matching_budgets(
        &self,
        tenant_id: &str,
        region: &RegionCode,
        period: FinopsPeriod,
        axes: &[AxisId],
    ) -> Vec<AxisBudget> {
        self.budgets
            .values()
            .filter(|budget| {
                budget.tenant_id.value == tenant_id
                    && &budget.region.value == region
                    && budget.period.value == period
                    && axes.contains(&budget.axis.value)
            })
            .cloned()
            .collect()
    }
}

fn axis_breakdowns(
    axes: &[AxisId],
    current: &[CostAllocation],
    budgets: &[AxisBudget],
    currency: &CurrencyCode,
) -> Result<Vec<AxisCostBreakdown>, CloudFinopsError> {
    axes.iter()
        .copied()
        .filter(|axis| {
            current
                .iter()
                .any(|allocation| allocation.axis.value == *axis)
        })
        .map(|axis| {
            let actual_cost = sum_money(
                current
                    .iter()
                    .filter(|allocation| allocation.axis.value == axis)
                    .map(|allocation| &allocation.actual_cost.value),
                currency,
            )?;
            let cost_of_revenue = sum_money(
                current
                    .iter()
                    .filter(|allocation| allocation.axis.value == axis)
                    .map(|allocation| &allocation.cost_of_revenue.value),
                currency,
            )?;
            let budget = budgets.iter().find(|budget| budget.axis.value == axis);
            let budget_utilization_bps = budget
                .map(|budget| ratio_bps(actual_cost.minor_units, budget.budget.value.minor_units))
                .transpose()?;
            Ok(AxisCostBreakdown {
                axis,
                gross_margin_bps: gross_margin_bps(&actual_cost, &cost_of_revenue)?,
                actual_cost,
                cost_of_revenue,
                budget: budget.map(|budget| budget.budget.value.clone()),
                budget_utilization_bps,
            })
        })
        .collect()
}

fn resource_breakdowns(
    current: &[CostAllocation],
    currency: &CurrencyCode,
) -> Result<Vec<ResourceCostBreakdown>, CloudFinopsError> {
    let mut keys = BTreeSet::new();
    for allocation in current {
        keys.insert((allocation.resource_id.value.clone(), allocation.axis.value));
    }
    keys.into_iter()
        .map(|(resource_id, axis)| {
            let actual_cost = sum_money(
                current
                    .iter()
                    .filter(|allocation| {
                        allocation.resource_id.value == resource_id && allocation.axis.value == axis
                    })
                    .map(|allocation| &allocation.actual_cost.value),
                currency,
            )?;
            let cost_of_revenue = sum_money(
                current
                    .iter()
                    .filter(|allocation| {
                        allocation.resource_id.value == resource_id && allocation.axis.value == axis
                    })
                    .map(|allocation| &allocation.cost_of_revenue.value),
                currency,
            )?;
            Ok(ResourceCostBreakdown {
                resource_id,
                axis,
                gross_margin_bps: gross_margin_bps(&actual_cost, &cost_of_revenue)?,
                actual_cost,
                cost_of_revenue,
            })
        })
        .collect()
}

fn detect_anomalies(
    axis_costs: &[AxisCostBreakdown],
    baseline: &[CostAllocation],
    budgets: &[AxisBudget],
    currency: &CurrencyCode,
    policy: AnomalyPolicy,
    minimum_gross_margin_bps: u16,
) -> Result<Vec<CostAnomaly>, CloudFinopsError> {
    let mut anomalies = Vec::new();
    for axis_cost in axis_costs {
        let baseline_cost = sum_money(
            baseline
                .iter()
                .filter(|allocation| allocation.axis.value == axis_cost.axis)
                .map(|allocation| &allocation.actual_cost.value),
            currency,
        )?;
        if baseline_cost.minor_units > 0
            && axis_cost.actual_cost.minor_units > baseline_cost.minor_units
        {
            let delta = axis_cost.actual_cost.minor_units - baseline_cost.minor_units;
            let growth_bps = ratio_bps(delta, baseline_cost.minor_units)?;
            if growth_bps >= policy.spend_growth_threshold_bps
                && delta >= policy.min_absolute_delta_minor_units
            {
                anomalies.push(CostAnomaly {
                    kind: CostAnomalyKind::SpendSpike,
                    axis: axis_cost.axis,
                    resource_id: None,
                    actual_cost: axis_cost.actual_cost.clone(),
                    baseline_cost: Some(baseline_cost),
                    threshold_bps: policy.spend_growth_threshold_bps,
                });
            }
        }
        if let Some(budget) = budgets
            .iter()
            .find(|budget| budget.axis.value == axis_cost.axis)
        {
            let utilization = ratio_bps(
                axis_cost.actual_cost.minor_units,
                budget.budget.value.minor_units,
            )?;
            if utilization >= budget.hard_threshold_bps.value {
                anomalies.push(CostAnomaly {
                    kind: CostAnomalyKind::BudgetHardLimit,
                    axis: axis_cost.axis,
                    resource_id: None,
                    actual_cost: axis_cost.actual_cost.clone(),
                    baseline_cost: Some(budget.budget.value.clone()),
                    threshold_bps: budget.hard_threshold_bps.value,
                });
            } else if utilization >= budget.soft_threshold_bps.value {
                anomalies.push(CostAnomaly {
                    kind: CostAnomalyKind::BudgetSoftLimit,
                    axis: axis_cost.axis,
                    resource_id: None,
                    actual_cost: axis_cost.actual_cost.clone(),
                    baseline_cost: Some(budget.budget.value.clone()),
                    threshold_bps: budget.soft_threshold_bps.value,
                });
            }
        }
        if axis_cost.gross_margin_bps < minimum_gross_margin_bps {
            anomalies.push(CostAnomaly {
                kind: CostAnomalyKind::MarginBelowTarget,
                axis: axis_cost.axis,
                resource_id: None,
                actual_cost: axis_cost.actual_cost.clone(),
                baseline_cost: Some(axis_cost.cost_of_revenue.clone()),
                threshold_bps: minimum_gross_margin_bps,
            });
        }
    }
    Ok(anomalies)
}

/// Derive a deterministic list of [`FinopsRecommendation`] values from a slice of
/// [`CostAnomaly`] values.
///
/// Mapping rules:
/// - [`CostAnomalyKind::SpendSpike`] → [`RecommendationKind::InvestigateSpendSpike`]; when the
///   anomaly also carries a `resource_id`, an additional [`RecommendationKind::DownsizeResource`]
///   recommendation is emitted for that resource.
/// - [`CostAnomalyKind::BudgetSoftLimit`] | [`CostAnomalyKind::BudgetHardLimit`] →
///   [`RecommendationKind::PurchaseCommitment`].
/// - [`CostAnomalyKind::MarginBelowTarget`] → [`RecommendationKind::ReviewRateCard`].
///
/// Recommendation IDs are minted deterministically from `id_seed` so that calling this
/// function twice with the same `anomalies` and `id_seed` produces identical output.
///
/// # Panics
///
/// Never panics; returns an empty `Vec` for an empty slice.
pub fn recommend_from_anomalies(
    anomalies: &[CostAnomaly],
    id_seed: u64,
) -> Vec<FinopsRecommendation> {
    let mut output = Vec::new();
    let mut slot: u64 = 0;
    for anomaly in anomalies {
        match anomaly.kind {
            CostAnomalyKind::SpendSpike => {
                let id_str = format!("{RECOMMENDATION_ID_PREFIX}s{id_seed}p{slot}");
                if let Ok(id) = RecommendationId::new(id_str) {
                    output.push(FinopsRecommendation {
                        id,
                        kind: RecommendationKind::InvestigateSpendSpike,
                        axis: anomaly.axis,
                        resource_id: anomaly.resource_id.clone(),
                        evidence_anomaly: anomaly.kind,
                    });
                }
                slot += 1;
                if anomaly.resource_id.is_some() {
                    let id_str = format!("{RECOMMENDATION_ID_PREFIX}s{id_seed}p{slot}");
                    if let Ok(id) = RecommendationId::new(id_str) {
                        output.push(FinopsRecommendation {
                            id,
                            kind: RecommendationKind::DownsizeResource,
                            axis: anomaly.axis,
                            resource_id: anomaly.resource_id.clone(),
                            evidence_anomaly: anomaly.kind,
                        });
                    }
                    slot += 1;
                }
            }
            CostAnomalyKind::BudgetSoftLimit | CostAnomalyKind::BudgetHardLimit => {
                let id_str = format!("{RECOMMENDATION_ID_PREFIX}s{id_seed}p{slot}");
                if let Ok(id) = RecommendationId::new(id_str) {
                    output.push(FinopsRecommendation {
                        id,
                        kind: RecommendationKind::PurchaseCommitment,
                        axis: anomaly.axis,
                        resource_id: anomaly.resource_id.clone(),
                        evidence_anomaly: anomaly.kind,
                    });
                }
                slot += 1;
            }
            CostAnomalyKind::MarginBelowTarget => {
                let id_str = format!("{RECOMMENDATION_ID_PREFIX}s{id_seed}p{slot}");
                if let Ok(id) = RecommendationId::new(id_str) {
                    output.push(FinopsRecommendation {
                        id,
                        kind: RecommendationKind::ReviewRateCard,
                        axis: anomaly.axis,
                        resource_id: anomaly.resource_id.clone(),
                        evidence_anomaly: anomaly.kind,
                    });
                }
                slot += 1;
            }
        }
    }
    output
}

fn recommendations_for(
    anomalies: &[CostAnomaly],
) -> Result<Vec<FinopsRecommendation>, CloudFinopsError> {
    anomalies
        .iter()
        .enumerate()
        .map(|(index, anomaly)| {
            let kind = match anomaly.kind {
                CostAnomalyKind::SpendSpike => RecommendationKind::InvestigateSpendSpike,
                CostAnomalyKind::BudgetSoftLimit | CostAnomalyKind::BudgetHardLimit => {
                    RecommendationKind::PurchaseCommitment
                }
                CostAnomalyKind::MarginBelowTarget => RecommendationKind::ReviewRateCard,
            };
            Ok(FinopsRecommendation {
                id: RecommendationId::new(format!("{RECOMMENDATION_ID_PREFIX}{}", index + 1))?,
                kind,
                axis: anomaly.axis,
                resource_id: anomaly.resource_id.clone(),
                evidence_anomaly: anomaly.kind,
            })
        })
        .collect()
}

fn price_units(
    units: &[billing_metering::MeterUnit],
    lines: &[RateCardLine],
    rate_card_ref: &RateCardRef,
    region: &RegionCode,
    axis: AxisId,
    timestamp: u64,
) -> Result<(Money, Money), CloudFinopsError> {
    let mut currency: Option<CurrencyCode> = None;
    let mut actual_minor_units = 0_u64;
    let mut cost_of_revenue_minor_units = 0_u64;
    for unit in units {
        let line = lines
            .iter()
            .find(|line| line.applies_to(rate_card_ref, region, axis, unit.kind, timestamp))
            .ok_or(CloudFinopsError::MissingRateCardLine)?;
        if let Some(existing) = &currency {
            if existing != &line.currency.value {
                return Err(CloudFinopsError::CurrencyMismatch);
            }
        } else {
            currency = Some(line.currency.value.clone());
        }
        actual_minor_units = actual_minor_units
            .checked_add(charge_minor_units(
                unit.quantity_microunits,
                line.rate.value.minor_units_per_million_microunits,
            )?)
            .ok_or(CloudFinopsError::CostOverflow)?;
        cost_of_revenue_minor_units = cost_of_revenue_minor_units
            .checked_add(charge_minor_units(
                unit.quantity_microunits,
                line.rate
                    .value
                    .cost_of_revenue_minor_units_per_million_microunits,
            )?)
            .ok_or(CloudFinopsError::CostOverflow)?;
    }
    let currency = currency.ok_or(CloudFinopsError::MissingRateCardLine)?;
    Ok((
        money(currency.clone(), actual_minor_units)?,
        money(currency, cost_of_revenue_minor_units)?,
    ))
}

fn charge_minor_units(
    quantity_microunits: u64,
    per_million_rate: u64,
) -> Result<u64, CloudFinopsError> {
    let product = u128::from(quantity_microunits)
        .checked_mul(u128::from(per_million_rate))
        .ok_or(CloudFinopsError::CostOverflow)?;
    if product % MILLION_MICROUNITS != 0 {
        return Err(CloudFinopsError::NonIntegralCost);
    }
    let value = product / MILLION_MICROUNITS;
    u64::try_from(value).map_err(|_| CloudFinopsError::CostOverflow)
}

fn gross_margin_bps(actual_cost: &Money, cost_of_revenue: &Money) -> Result<u16, CloudFinopsError> {
    ensure_same_currency(actual_cost, cost_of_revenue)?;
    if cost_of_revenue.minor_units > actual_cost.minor_units {
        return Err(CloudFinopsError::NegativeGrossMargin);
    }
    if actual_cost.minor_units == 0 {
        return Ok(10_000);
    }
    let margin = actual_cost.minor_units - cost_of_revenue.minor_units;
    ratio_bps(margin, actual_cost.minor_units)
}

fn ratio_bps(numerator: u64, denominator: u64) -> Result<u16, CloudFinopsError> {
    if denominator == 0 {
        return Err(CloudFinopsError::InvalidBudget);
    }
    let value = u128::from(numerator)
        .checked_mul(10_000)
        .ok_or(CloudFinopsError::CostOverflow)?
        / u128::from(denominator);
    u16::try_from(value.min(u128::from(u16::MAX))).map_err(|_| CloudFinopsError::CostOverflow)
}

fn sum_money<'a>(
    values: impl IntoIterator<Item = &'a Money>,
    currency: &CurrencyCode,
) -> Result<Money, CloudFinopsError> {
    let mut total = 0_u64;
    for value in values {
        if &value.currency != currency {
            return Err(CloudFinopsError::CurrencyMismatch);
        }
        total = total
            .checked_add(value.minor_units)
            .ok_or(CloudFinopsError::CostOverflow)?;
    }
    money(currency.clone(), total)
}

fn money(currency: CurrencyCode, minor_units: u64) -> Result<Money, CloudFinopsError> {
    Money::new(currency.value, minor_units).map_err(|_| CloudFinopsError::InvalidCurrency)
}

fn ensure_same_currency(a: &Money, b: &Money) -> Result<(), CloudFinopsError> {
    if a.currency == b.currency {
        Ok(())
    } else {
        Err(CloudFinopsError::CurrencyMismatch)
    }
}

fn resource_for(
    value: &str,
    tenant_id: &str,
    region: &RegionCode,
) -> Result<ResourceId, CloudFinopsError> {
    let resource_id = ResourceId::new(value.to_string()).map_err(map_resource_error)?;
    if resource_id.tenant_id().map_err(map_resource_error)? != tenant_id {
        return Err(CloudFinopsError::ResourceTenantMismatch);
    }
    if resource_id.region().map_err(map_resource_error)? != *region {
        return Err(CloudFinopsError::ResourceRegionMismatch);
    }
    Ok(resource_id)
}

fn map_resource_error(error: CloudResourceError) -> CloudFinopsError {
    match error {
        CloudResourceError::InvalidResourceId => CloudFinopsError::InvalidResourceId,
        CloudResourceError::ResourceIdTenantMismatch => CloudFinopsError::ResourceTenantMismatch,
        CloudResourceError::ResourceIdRegionMismatch => CloudFinopsError::ResourceRegionMismatch,
        _ => CloudFinopsError::InvalidResourceId,
    }
}

fn validated_axes(axes: Vec<AxisId>) -> Result<Vec<AxisId>, CloudFinopsError> {
    if axes.is_empty() {
        return Err(CloudFinopsError::InvalidAxisSet);
    }
    let mut seen = BTreeSet::new();
    let mut output = Vec::with_capacity(axes.len());
    for axis in axes {
        if !seen.insert(axis) {
            return Err(CloudFinopsError::InvalidAxisSet);
        }
        output.push(axis);
    }
    Ok(output)
}

fn validate_rate_line_period(period: FinopsPeriod) -> Result<(), CloudFinopsError> {
    FinopsPeriod::new(period.start_epoch_seconds, period.end_epoch_seconds).map(|_| ())
}

fn validate_budget_thresholds(soft: u16, hard: u16) -> Result<(), CloudFinopsError> {
    if soft == 0 || hard == 0 || soft > hard || hard > 10_000 {
        return Err(CloudFinopsError::InvalidBudgetThreshold);
    }
    Ok(())
}

fn validate_gross_margin_target(value: u16) -> Result<(), CloudFinopsError> {
    if value <= 10_000 {
        Ok(())
    } else {
        Err(CloudFinopsError::InvalidGrossMarginTarget)
    }
}

fn validate_tenant_id(value: &str) -> Result<(), CloudFinopsError> {
    if value.starts_with(TENANT_ID_PREFIX)
        && value.len() > TENANT_ID_PREFIX.len()
        && is_ascii_token(value)
    {
        Ok(())
    } else {
        Err(CloudFinopsError::InvalidTenantId)
    }
}

fn validate_financial_class(data_class: DataClass) -> Result<PrivacyDataClass, CloudFinopsError> {
    let data_class =
        PrivacyDataClass::new(data_class).map_err(|_| CloudFinopsError::InvalidDataClass)?;
    if DataClassMatcher::RegulatedFinancial.matches(data_class.data_class()) {
        Ok(data_class)
    } else {
        Err(CloudFinopsError::InvalidDataClass)
    }
}

fn prefixed_id(
    value: String,
    prefix: &str,
    error: CloudFinopsError,
) -> Result<String, CloudFinopsError> {
    if value.starts_with(prefix) && value.len() > prefix.len() && is_ascii_token(&value) {
        Ok(value)
    } else {
        Err(error)
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn financial<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Financial)
}

fn financial_value<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Financial)
}

fn is_ascii_token(value: &str) -> bool {
    value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':' | '/'))
}

#[cfg(test)]
mod tests {
    use billing_metering::{AxisId, MeterEventCreate, MeterUnit, MeterUnitKind, PlaneTag};

    use super::*;

    const TENANT: &str = "ten_alpha";
    const REGION: &str = "region-alpha1";
    const RESOURCE: &str = "oya:cloud:region-alpha1:ten_alpha:instance:vm-a";
    const RATE_CARD: &str = "rate/standard";

    fn period() -> FinopsPeriod {
        FinopsPeriod::new(1_000, 2_000).expect("period")
    }

    fn baseline_period() -> FinopsPeriod {
        FinopsPeriod::new(1, 1_001).expect("baseline period")
    }

    fn money_units(minor_units: u64) -> Money {
        Money::new("XTS", minor_units).expect("money")
    }

    fn meter_event(id: &str, axis: AxisId, quantity: u64, ts: u64) -> MeterEvent {
        MeterEvent::new(MeterEventCreate {
            id: id.to_string(),
            tenant_id: TENANT.to_string(),
            capability_id: "cap.cloud.compute.vm".to_string(),
            plane: PlaneTag::Data,
            units: vec![MeterUnit::new(MeterUnitKind::ResourceSecond, quantity).expect("unit")],
            source_axis: axis,
            recorded_at_epoch_seconds: ts,
            idempotency_key: format!("idem_{id}"),
            data_class: DataClass::Public,
        })
        .expect("meter event")
    }

    fn rate_line(axis: AxisId, unit_kind: MeterUnitKind, rate: UnitRate) -> RateCardLineCreate {
        RateCardLineCreate {
            rate_card_ref: RATE_CARD.to_string(),
            region: REGION.to_string(),
            axis,
            unit_kind,
            currency: "XTS".to_string(),
            rate,
            effective_period: FinopsPeriod::new(1, 3_000).expect("rate period"),
            data_class: DataClass::Financial,
        }
    }

    fn allocation(id: &str, event: MeterEvent) -> CostAllocationCreate {
        CostAllocationCreate {
            id: id.to_string(),
            region: REGION.to_string(),
            resource_id: RESOURCE.to_string(),
            rate_card_ref: RATE_CARD.to_string(),
            meter_event: event,
            data_class: DataClass::Financial,
        }
    }

    fn report_request() -> FinopsReportRequest {
        FinopsReportRequest {
            id: "finr_month".to_string(),
            tenant_id: TENANT.to_string(),
            region: REGION.to_string(),
            period: period(),
            baseline_period: Some(baseline_period()),
            axes: vec![AxisId::Cloud, AxisId::Saas],
            anomaly_policy: AnomalyPolicy::new(1_000, 100).expect("policy"),
            minimum_gross_margin_bps: STABLE_GROSS_MARGIN_TARGET_BPS,
            data_class: DataClass::Financial,
        }
    }

    fn ledger_with_rates() -> CloudFinopsLedger {
        let mut ledger = CloudFinopsLedger::default();
        ledger
            .add_rate_card_line(rate_line(
                AxisId::Cloud,
                MeterUnitKind::ResourceSecond,
                UnitRate::new(2_000, 800).expect("rate"),
            ))
            .expect("cloud rate");
        ledger
            .add_rate_card_line(rate_line(
                AxisId::Saas,
                MeterUnitKind::ResourceSecond,
                UnitRate::new(1_000, 600).expect("rate"),
            ))
            .expect("saas rate");
        ledger
    }

    #[test]
    fn reports_per_tenant_axis_resource_costs_with_budget_and_margin() {
        let mut ledger = ledger_with_rates();
        ledger
            .set_budget(AxisBudgetCreate {
                id: "fbg_cloud".to_string(),
                tenant_id: TENANT.to_string(),
                region: REGION.to_string(),
                axis: AxisId::Cloud,
                period: period(),
                budget: money_units(10_000),
                soft_threshold_bps: 8_000,
                hard_threshold_bps: 10_000,
                data_class: DataClass::Financial,
            })
            .expect("budget");
        ledger
            .record_allocation(allocation(
                "fca_base_cloud",
                meter_event("mtr_base_cloud", AxisId::Cloud, 1_000_000, 500),
            ))
            .expect("baseline");
        ledger
            .record_allocation(allocation(
                "fca_current_cloud",
                meter_event("mtr_current_cloud", AxisId::Cloud, 2_000_000, 1_500),
            ))
            .expect("current cloud");
        ledger
            .record_allocation(allocation(
                "fca_current_saas",
                meter_event("mtr_current_saas", AxisId::Saas, 1_000_000, 1_600),
            ))
            .expect("current saas");

        let report = ledger.generate_report(report_request()).expect("report");
        assert_eq!(report.total_cost.value, money_units(5_000));
        assert_eq!(report.total_cost_of_revenue.value, money_units(2_200));
        assert_eq!(report.gross_margin_bps.value, 5_600);
        assert_eq!(report.axis_costs.value.len(), 2);
        assert_eq!(report.resource_costs.value.len(), 2);
        assert_eq!(report.anomalies.value.len(), 1);
        assert_eq!(report.anomalies.value[0].kind, CostAnomalyKind::SpendSpike);
    }

    #[test]
    fn detects_budget_hard_limit_and_margin_below_target() {
        let mut ledger = ledger_with_rates();
        ledger
            .set_budget(AxisBudgetCreate {
                id: "fbg_cloud".to_string(),
                tenant_id: TENANT.to_string(),
                region: REGION.to_string(),
                axis: AxisId::Cloud,
                period: period(),
                budget: money_units(3_000),
                soft_threshold_bps: 8_000,
                hard_threshold_bps: 10_000,
                data_class: DataClass::Financial,
            })
            .expect("budget");
        ledger
            .record_allocation(allocation(
                "fca_base_cloud",
                meter_event("mtr_base_cloud", AxisId::Cloud, 1_000_000, 500),
            ))
            .expect("baseline");
        ledger
            .record_allocation(allocation(
                "fca_current_cloud",
                meter_event("mtr_current_cloud", AxisId::Cloud, 2_000_000, 1_500),
            ))
            .expect("current");

        let report = ledger
            .generate_report(FinopsReportRequest {
                axes: vec![AxisId::Cloud],
                minimum_gross_margin_bps: 7_000,
                ..report_request()
            })
            .expect("report");
        assert_eq!(
            report
                .anomalies
                .value
                .iter()
                .map(|anomaly| anomaly.kind)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([
                CostAnomalyKind::SpendSpike,
                CostAnomalyKind::BudgetHardLimit,
                CostAnomalyKind::MarginBelowTarget,
            ])
        );
        assert_eq!(report.recommendations.value.len(), 3);
    }

    #[test]
    fn rejects_non_integral_rate_and_negative_margin_rate() {
        let err = RateCardLine::new(rate_line(
            AxisId::Cloud,
            MeterUnitKind::ResourceSecond,
            UnitRate {
                minor_units_per_million_microunits: 2_000,
                cost_of_revenue_minor_units_per_million_microunits: 2_001,
            },
        ));
        assert_eq!(err.unwrap_err(), CloudFinopsError::NegativeGrossMargin);

        let mut ledger = CloudFinopsLedger::default();
        ledger
            .add_rate_card_line(rate_line(
                AxisId::Cloud,
                MeterUnitKind::ResourceSecond,
                UnitRate::new(1, 0).expect("fractional rate"),
            ))
            .expect("rate added");
        let allocation_error = ledger
            .record_allocation(allocation(
                "fca_fractional",
                meter_event("mtr_fractional", AxisId::Cloud, 1, 1_500),
            ))
            .unwrap_err();
        assert_eq!(allocation_error, CloudFinopsError::NonIntegralCost);
    }

    #[test]
    fn rejects_allocation_mismatches_and_duplicate_meter_events() {
        let mut ledger = ledger_with_rates();
        assert_eq!(
            ledger
                .record_allocation(CostAllocationCreate {
                    resource_id: "oya:cloud:region-alpha1:ten_other:instance:vm-a".to_string(),
                    ..allocation(
                        "fca_bad_tenant",
                        meter_event("mtr_bad_tenant", AxisId::Cloud, 1_000_000, 1_500),
                    )
                })
                .unwrap_err(),
            CloudFinopsError::ResourceTenantMismatch
        );
        ledger
            .record_allocation(allocation(
                "fca_good",
                meter_event("mtr_good", AxisId::Cloud, 1_000_000, 1_500),
            ))
            .expect("allocation");
        assert_eq!(
            ledger
                .record_allocation(allocation(
                    "fca_duplicate_event",
                    meter_event("mtr_good", AxisId::Cloud, 1_000_000, 1_600),
                ))
                .unwrap_err(),
            CloudFinopsError::DuplicateMeterEvent
        );
    }

    #[test]
    fn rejects_missing_rate_card_report_shape_and_budget_thresholds() {
        let mut ledger = CloudFinopsLedger::default();
        assert_eq!(
            ledger
                .record_allocation(allocation(
                    "fca_missing_rate",
                    meter_event("mtr_missing_rate", AxisId::Cloud, 1_000_000, 1_500),
                ))
                .unwrap_err(),
            CloudFinopsError::MissingRateCardLine
        );
        assert_eq!(
            AxisBudget::new(AxisBudgetCreate {
                id: "fbg_bad".to_string(),
                tenant_id: TENANT.to_string(),
                region: REGION.to_string(),
                axis: AxisId::Cloud,
                period: period(),
                budget: money_units(1_000),
                soft_threshold_bps: 10_000,
                hard_threshold_bps: 8_000,
                data_class: DataClass::Financial,
            })
            .unwrap_err(),
            CloudFinopsError::InvalidBudgetThreshold
        );
        let mut ledger = ledger_with_rates();
        assert_eq!(
            ledger
                .generate_report(FinopsReportRequest {
                    axes: vec![AxisId::Cloud, AxisId::Cloud],
                    ..report_request()
                })
                .unwrap_err(),
            CloudFinopsError::InvalidAxisSet
        );
    }

    #[test]
    fn rejects_non_financial_report_and_allocation_classes() {
        assert_eq!(
            FinopsReportId::new("report_bad").unwrap_err(),
            CloudFinopsError::InvalidReportId
        );
        assert_eq!(
            AxisBudget::new(AxisBudgetCreate {
                id: "fbg_bad_class".to_string(),
                tenant_id: TENANT.to_string(),
                region: REGION.to_string(),
                axis: AxisId::Cloud,
                period: period(),
                budget: money_units(1_000),
                soft_threshold_bps: 8_000,
                hard_threshold_bps: 10_000,
                data_class: DataClass::Public,
            })
            .unwrap_err(),
            CloudFinopsError::InvalidDataClass
        );
    }

    // ---- recommend_from_anomalies tests ----

    fn anomaly(kind: CostAnomalyKind, resource_id: Option<ResourceId>) -> CostAnomaly {
        CostAnomaly {
            kind,
            axis: AxisId::Cloud,
            resource_id,
            actual_cost: money_units(500),
            baseline_cost: None,
            threshold_bps: 1_000,
        }
    }

    fn resource_id() -> ResourceId {
        ResourceId::new(RESOURCE.to_string()).expect("resource id")
    }

    #[test]
    fn empty_input_yields_empty_vec() {
        let recs = recommend_from_anomalies(&[], 42);
        assert!(recs.is_empty());
    }

    #[test]
    fn spend_spike_without_resource_yields_investigate() {
        let anomalies = vec![anomaly(CostAnomalyKind::SpendSpike, None)];
        let recs = recommend_from_anomalies(&anomalies, 1);
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].kind, RecommendationKind::InvestigateSpendSpike);
        assert_eq!(recs[0].evidence_anomaly, CostAnomalyKind::SpendSpike);
        assert_eq!(recs[0].axis, AxisId::Cloud);
        assert!(recs[0].resource_id.is_none());
    }

    #[test]
    fn spend_spike_with_resource_yields_investigate_and_downsize() {
        let anomalies = vec![anomaly(CostAnomalyKind::SpendSpike, Some(resource_id()))];
        let recs = recommend_from_anomalies(&anomalies, 2);
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].kind, RecommendationKind::InvestigateSpendSpike);
        assert_eq!(recs[1].kind, RecommendationKind::DownsizeResource);
        assert_eq!(recs[1].evidence_anomaly, CostAnomalyKind::SpendSpike);
        assert!(recs[1].resource_id.is_some());
    }

    #[test]
    fn budget_soft_limit_yields_purchase_commitment() {
        let anomalies = vec![anomaly(CostAnomalyKind::BudgetSoftLimit, None)];
        let recs = recommend_from_anomalies(&anomalies, 3);
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].kind, RecommendationKind::PurchaseCommitment);
        assert_eq!(recs[0].evidence_anomaly, CostAnomalyKind::BudgetSoftLimit);
    }

    #[test]
    fn budget_hard_limit_yields_purchase_commitment() {
        let anomalies = vec![anomaly(CostAnomalyKind::BudgetHardLimit, None)];
        let recs = recommend_from_anomalies(&anomalies, 4);
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].kind, RecommendationKind::PurchaseCommitment);
        assert_eq!(recs[0].evidence_anomaly, CostAnomalyKind::BudgetHardLimit);
    }

    #[test]
    fn margin_below_target_yields_review_rate_card() {
        let anomalies = vec![anomaly(CostAnomalyKind::MarginBelowTarget, None)];
        let recs = recommend_from_anomalies(&anomalies, 5);
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].kind, RecommendationKind::ReviewRateCard);
        assert_eq!(recs[0].evidence_anomaly, CostAnomalyKind::MarginBelowTarget);
    }

    #[test]
    fn determinism_same_input_twice_produces_identical_output() {
        let anomalies = vec![
            anomaly(CostAnomalyKind::SpendSpike, Some(resource_id())),
            anomaly(CostAnomalyKind::BudgetHardLimit, None),
            anomaly(CostAnomalyKind::MarginBelowTarget, None),
        ];
        let first = recommend_from_anomalies(&anomalies, 7);
        let second = recommend_from_anomalies(&anomalies, 7);
        assert_eq!(first, second);
    }

    #[test]
    fn different_seeds_produce_different_ids_same_kinds() {
        let anomalies = vec![anomaly(CostAnomalyKind::BudgetSoftLimit, None)];
        let recs_a = recommend_from_anomalies(&anomalies, 10);
        let recs_b = recommend_from_anomalies(&anomalies, 99);
        assert_eq!(recs_a.len(), 1);
        assert_eq!(recs_b.len(), 1);
        // kinds match but IDs are distinct
        assert_eq!(recs_a[0].kind, recs_b[0].kind);
        assert_ne!(recs_a[0].id, recs_b[0].id);
    }

    #[test]
    fn all_anomaly_kinds_mixed_with_resource_scoped_spike() {
        let anomalies = vec![
            anomaly(CostAnomalyKind::SpendSpike, Some(resource_id())),
            anomaly(CostAnomalyKind::BudgetSoftLimit, None),
            anomaly(CostAnomalyKind::BudgetHardLimit, None),
            anomaly(CostAnomalyKind::MarginBelowTarget, None),
        ];
        let recs = recommend_from_anomalies(&anomalies, 100);
        // SpendSpike+resource→2, BudgetSoft→1, BudgetHard→1, Margin→1 = 5
        assert_eq!(recs.len(), 5);
        assert_eq!(recs[0].kind, RecommendationKind::InvestigateSpendSpike);
        assert_eq!(recs[1].kind, RecommendationKind::DownsizeResource);
        assert_eq!(recs[2].kind, RecommendationKind::PurchaseCommitment);
        assert_eq!(recs[3].kind, RecommendationKind::PurchaseCommitment);
        assert_eq!(recs[4].kind, RecommendationKind::ReviewRateCard);
        // all IDs are distinct
        let ids: BTreeSet<_> = recs.iter().map(|r| r.id.value.clone()).collect();
        assert_eq!(ids.len(), 5);
    }

    #[test]
    fn recommendation_ids_have_correct_prefix() {
        let anomalies = vec![anomaly(CostAnomalyKind::SpendSpike, None)];
        let recs = recommend_from_anomalies(&anomalies, 55);
        assert!(recs[0].id.value.starts_with(RECOMMENDATION_ID_PREFIX));
    }
}
