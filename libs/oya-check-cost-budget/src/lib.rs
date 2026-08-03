//! Foundry cost-budget kernel.
//!
//! Pure pre-flight budget contracts for provider invocation cost ceilings.

use std::collections::BTreeMap;

use data_boundary_kernel::{Classified, DataClass};

type ScopeKey = (String, String, String);
type TenantWindowKey = (String, String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BudgetError {
    InvalidTenantId,
    InvalidCapabilityId,
    InvalidWindowId,
    InvalidBudgetCeiling,
    MissingBudgetCeiling,
    NonPositiveAmount,
    PerInvocationLimitExceeded,
    TenantMonthlyLimitExceeded,
    CapabilityMonthlyLimitExceeded,
    ReservationNotFound,
    ReservationNotPending,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BudgetCeilingSource {
    Tenant,
    Capability,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BudgetWarning {
    RunningSpendThresholdReached,
    TenantSpendThresholdReached,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReservationStatus {
    Pending,
    Committed,
    Released,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BudgetScope {
    pub tenant_id: Classified<String>,
    pub capability_id: Classified<String>,
    pub window_id: Classified<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetCeiling {
    pub monthly_limit_micros: Classified<u64>,
    pub per_invocation_limit_micros: Classified<u64>,
    pub warning_threshold_percent: Classified<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetSnapshot {
    pub scope: BudgetScope,
    pub ceiling_source: Classified<BudgetCeilingSource>,
    pub ceiling: BudgetCeiling,
    pub running_spend_micros: Classified<u64>,
    pub committed_scope_spend_micros: Classified<u64>,
    pub pending_scope_spend_micros: Classified<u64>,
    pub committed_tenant_spend_micros: Classified<u64>,
    pub pending_tenant_spend_micros: Classified<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetDecision {
    pub allowed: Classified<bool>,
    pub warning: Classified<Option<BudgetWarning>>,
    pub projected_running_spend_micros: Classified<u64>,
    pub projected_tenant_spend_micros: Classified<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetReservation {
    pub reservation_id: Classified<String>,
    pub scope: BudgetScope,
    pub amount_micros: Classified<u64>,
    pub status: Classified<ReservationStatus>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetLedger {
    tenant_ceilings: BTreeMap<TenantWindowKey, BudgetCeiling>,
    capability_ceilings: BTreeMap<ScopeKey, BudgetCeiling>,
    committed_spend_micros: BTreeMap<ScopeKey, u64>,
    reservations: BTreeMap<String, BudgetReservation>,
    next_reservation_number: u64,
}

impl Default for BudgetLedger {
    fn default() -> Self {
        Self {
            tenant_ceilings: BTreeMap::new(),
            capability_ceilings: BTreeMap::new(),
            committed_spend_micros: BTreeMap::new(),
            reservations: BTreeMap::new(),
            next_reservation_number: 1,
        }
    }
}

impl BudgetScope {
    pub fn new(
        tenant_id: String,
        capability_id: String,
        window_id: String,
    ) -> Result<Self, BudgetError> {
        validate_tenant_id(&tenant_id)?;
        validate_capability_id(&capability_id)?;
        validate_window_id(&window_id)?;
        Ok(Self {
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            capability_id: Classified::new(capability_id, DataClass::InternalOnly),
            window_id: Classified::new(window_id, DataClass::InternalOnly),
        })
    }

    fn key(&self) -> ScopeKey {
        (
            self.tenant_id.value.clone(),
            self.capability_id.value.clone(),
            self.window_id.value.clone(),
        )
    }

    fn tenant_window_key(&self) -> TenantWindowKey {
        (self.tenant_id.value.clone(), self.window_id.value.clone())
    }
}

impl BudgetCeiling {
    pub fn new(
        monthly_limit_micros: u64,
        per_invocation_limit_micros: u64,
        warning_threshold_percent: u8,
    ) -> Result<Self, BudgetError> {
        if monthly_limit_micros == 0
            || per_invocation_limit_micros == 0
            || warning_threshold_percent == 0
            || warning_threshold_percent > 100
        {
            return Err(BudgetError::InvalidBudgetCeiling);
        }
        Ok(Self {
            monthly_limit_micros: Classified::new(monthly_limit_micros, DataClass::InternalOnly),
            per_invocation_limit_micros: Classified::new(
                per_invocation_limit_micros,
                DataClass::InternalOnly,
            ),
            warning_threshold_percent: Classified::new(
                warning_threshold_percent,
                DataClass::InternalOnly,
            ),
        })
    }
}

impl BudgetLedger {
    pub fn configure_tenant_ceiling(
        &mut self,
        tenant_id: String,
        window_id: String,
        ceiling: BudgetCeiling,
    ) -> Result<(), BudgetError> {
        validate_tenant_id(&tenant_id)?;
        validate_window_id(&window_id)?;
        self.tenant_ceilings.insert((tenant_id, window_id), ceiling);
        Ok(())
    }

    pub fn configure_capability_ceiling(
        &mut self,
        scope: BudgetScope,
        ceiling: BudgetCeiling,
    ) -> Result<(), BudgetError> {
        self.capability_ceilings.insert(scope.key(), ceiling);
        Ok(())
    }

    pub fn evaluate(
        &self,
        scope: &BudgetScope,
        amount_micros: u64,
    ) -> Result<BudgetDecision, BudgetError> {
        if amount_micros == 0 {
            return Err(BudgetError::NonPositiveAmount);
        }

        let snapshot = self.snapshot(scope)?;
        if amount_micros > snapshot.ceiling.per_invocation_limit_micros.value {
            return Err(BudgetError::PerInvocationLimitExceeded);
        }

        let projected_running_spend = snapshot
            .running_spend_micros
            .value
            .saturating_add(amount_micros);
        if projected_running_spend > snapshot.ceiling.monthly_limit_micros.value {
            return match snapshot.ceiling_source.value {
                BudgetCeilingSource::Tenant => Err(BudgetError::TenantMonthlyLimitExceeded),
                BudgetCeilingSource::Capability => Err(BudgetError::CapabilityMonthlyLimitExceeded),
            };
        }

        let projected_tenant_spend = snapshot
            .committed_tenant_spend_micros
            .value
            .saturating_add(snapshot.pending_tenant_spend_micros.value)
            .saturating_add(amount_micros);
        let tenant_ceiling = self
            .tenant_ceilings
            .get(&scope.tenant_window_key())
            .ok_or(BudgetError::MissingBudgetCeiling)?;
        if projected_tenant_spend > tenant_ceiling.monthly_limit_micros.value {
            return Err(BudgetError::TenantMonthlyLimitExceeded);
        }

        let warning = if threshold_reached(projected_running_spend, &snapshot.ceiling) {
            Some(BudgetWarning::RunningSpendThresholdReached)
        } else if matches!(
            snapshot.ceiling_source.value,
            BudgetCeilingSource::Capability
        ) && threshold_reached(projected_tenant_spend, tenant_ceiling)
        {
            Some(BudgetWarning::TenantSpendThresholdReached)
        } else {
            None
        };

        Ok(BudgetDecision {
            allowed: Classified::new(true, DataClass::InternalOnly),
            warning: Classified::new(warning, DataClass::InternalOnly),
            projected_running_spend_micros: Classified::new(
                projected_running_spend,
                DataClass::InternalOnly,
            ),
            projected_tenant_spend_micros: Classified::new(
                projected_tenant_spend,
                DataClass::InternalOnly,
            ),
        })
    }

    pub fn reserve(
        &mut self,
        scope: &BudgetScope,
        amount_micros: u64,
    ) -> Result<BudgetReservation, BudgetError> {
        self.evaluate(scope, amount_micros)?;
        let reservation_id = format!("res_{:012}", self.next_reservation_number);
        self.next_reservation_number += 1;
        let reservation = BudgetReservation {
            reservation_id: Classified::new(reservation_id.clone(), DataClass::InternalOnly),
            scope: scope.clone(),
            amount_micros: Classified::new(amount_micros, DataClass::InternalOnly),
            status: Classified::new(ReservationStatus::Pending, DataClass::InternalOnly),
        };
        self.reservations
            .insert(reservation_id, reservation.clone());
        Ok(reservation)
    }

    pub fn commit(&mut self, reservation_id: &str) -> Result<BudgetReservation, BudgetError> {
        let reservation = self
            .reservations
            .get_mut(reservation_id)
            .ok_or(BudgetError::ReservationNotFound)?;
        if reservation.status.value != ReservationStatus::Pending {
            return Err(BudgetError::ReservationNotPending);
        }
        reservation.status = Classified::new(ReservationStatus::Committed, DataClass::InternalOnly);
        let key = reservation.scope.key();
        *self.committed_spend_micros.entry(key).or_default() += reservation.amount_micros.value;
        Ok(reservation.clone())
    }

    pub fn release(&mut self, reservation_id: &str) -> Result<BudgetReservation, BudgetError> {
        let reservation = self
            .reservations
            .get_mut(reservation_id)
            .ok_or(BudgetError::ReservationNotFound)?;
        if reservation.status.value != ReservationStatus::Pending {
            return Err(BudgetError::ReservationNotPending);
        }
        reservation.status = Classified::new(ReservationStatus::Released, DataClass::InternalOnly);
        Ok(reservation.clone())
    }

    pub fn snapshot(&self, scope: &BudgetScope) -> Result<BudgetSnapshot, BudgetError> {
        let tenant_ceiling = self
            .tenant_ceilings
            .get(&scope.tenant_window_key())
            .ok_or(BudgetError::MissingBudgetCeiling)?;
        let capability_ceiling = self.capability_ceilings.get(&scope.key());
        let (ceiling_source, ceiling) = match capability_ceiling {
            Some(ceiling) => (BudgetCeilingSource::Capability, ceiling.clone()),
            None => (BudgetCeilingSource::Tenant, tenant_ceiling.clone()),
        };
        let committed_scope_spend = self.committed_scope_spend_micros(scope);
        let pending_scope_spend = self.pending_scope_spend_micros(scope);
        let committed_tenant_spend = self.committed_tenant_spend_micros(scope);
        let pending_tenant_spend = self.pending_tenant_spend_micros(scope);
        let running_spend = match ceiling_source {
            BudgetCeilingSource::Tenant => {
                committed_tenant_spend.saturating_add(pending_tenant_spend)
            }
            BudgetCeilingSource::Capability => {
                committed_scope_spend.saturating_add(pending_scope_spend)
            }
        };
        Ok(BudgetSnapshot {
            scope: scope.clone(),
            ceiling_source: Classified::new(ceiling_source, DataClass::InternalOnly),
            ceiling,
            running_spend_micros: Classified::new(running_spend, DataClass::InternalOnly),
            committed_scope_spend_micros: Classified::new(
                committed_scope_spend,
                DataClass::InternalOnly,
            ),
            pending_scope_spend_micros: Classified::new(
                pending_scope_spend,
                DataClass::InternalOnly,
            ),
            committed_tenant_spend_micros: Classified::new(
                committed_tenant_spend,
                DataClass::InternalOnly,
            ),
            pending_tenant_spend_micros: Classified::new(
                pending_tenant_spend,
                DataClass::InternalOnly,
            ),
        })
    }

    fn committed_scope_spend_micros(&self, scope: &BudgetScope) -> u64 {
        self.committed_spend_micros
            .get(&scope.key())
            .copied()
            .unwrap_or_default()
    }

    fn pending_scope_spend_micros(&self, scope: &BudgetScope) -> u64 {
        let scope_key = scope.key();
        self.reservations
            .values()
            .filter(|reservation| {
                reservation.status.value == ReservationStatus::Pending
                    && reservation.scope.key() == scope_key
            })
            .map(|reservation| reservation.amount_micros.value)
            .sum()
    }

    fn committed_tenant_spend_micros(&self, scope: &BudgetScope) -> u64 {
        let tenant_id = &scope.tenant_id.value;
        let window_id = &scope.window_id.value;
        self.committed_spend_micros
            .iter()
            .filter(|((spend_tenant_id, _, spend_window_id), _)| {
                spend_tenant_id == tenant_id && spend_window_id == window_id
            })
            .map(|(_, spend)| *spend)
            .sum()
    }

    fn pending_tenant_spend_micros(&self, scope: &BudgetScope) -> u64 {
        let tenant_id = &scope.tenant_id.value;
        let window_id = &scope.window_id.value;
        self.reservations
            .values()
            .filter(|reservation| {
                reservation.status.value == ReservationStatus::Pending
                    && reservation.scope.tenant_id.value == *tenant_id
                    && reservation.scope.window_id.value == *window_id
            })
            .map(|reservation| reservation.amount_micros.value)
            .sum()
    }
}

fn validate_tenant_id(tenant_id: &str) -> Result<(), BudgetError> {
    if !tenant_id.starts_with("ten_") {
        return Err(BudgetError::InvalidTenantId);
    }
    Ok(())
}

fn validate_capability_id(capability_id: &str) -> Result<(), BudgetError> {
    if !capability_id.starts_with("cap.") {
        return Err(BudgetError::InvalidCapabilityId);
    }
    Ok(())
}

fn validate_window_id(window_id: &str) -> Result<(), BudgetError> {
    if window_id.trim().is_empty() || window_id.contains('|') {
        return Err(BudgetError::InvalidWindowId);
    }
    Ok(())
}

fn threshold_reached(projected_spend_micros: u64, ceiling: &BudgetCeiling) -> bool {
    (projected_spend_micros as u128) * 100
        >= (ceiling.monthly_limit_micros.value as u128)
            * (ceiling.warning_threshold_percent.value as u128)
}
