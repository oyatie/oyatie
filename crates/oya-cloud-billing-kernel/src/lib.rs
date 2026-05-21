//! Cloud billing kernel (M03-P03-IP-002 minimum viable kernel).
//!
//! Pure I/O-free model for usage records, billable line items, tax
//! treatment, and the admission rule that line items cannot be
//! finalized without their declared tax profile reference.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum UsageUnit {
    Cpu,
    GpuHour,
    GibStorage,
    GibEgress,
    Request,
    Token,
}

impl UsageUnit {
    pub fn name(self) -> &'static str {
        match self {
            Self::Cpu => "cpu-hour",
            Self::GpuHour => "gpu-hour",
            Self::GibStorage => "gib-storage-month",
            Self::GibEgress => "gib-egress",
            Self::Request => "request",
            Self::Token => "token",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageRecord {
    pub record_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,      // data_class: INTERNAL_ONLY
    pub unit: UsageUnit,        // data_class: PUBLIC
    pub quantity: u64,          // data_class: INTERNAL_ONLY
    pub timestamp_unix_ms: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineItem {
    pub line_id: String,                 // data_class: INTERNAL_ONLY
    pub usage: UsageRecord,              // data_class: INTERNAL_ONLY
    pub unit_price_micros: u64,          // data_class: INTERNAL_ONLY
    pub tax_profile_ref: Option<String>, // data_class: INTERNAL_ONLY
}

impl LineItem {
    pub fn subtotal_micros(&self) -> u128 {
        u128::from(self.usage.quantity) * u128::from(self.unit_price_micros)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BillingError {
    EmptyRecordId,
    EmptyTenantId,
    ZeroQuantity,
    NoTaxProfileRef { line_id: String },
}

impl BillingError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyRecordId => "usage record id is empty".to_owned(),
            Self::EmptyTenantId => "tenant id is empty".to_owned(),
            Self::ZeroQuantity => "usage quantity is zero".to_owned(),
            Self::NoTaxProfileRef { line_id } => {
                format!("line {line_id} cannot finalize without tax profile reference")
            }
        }
    }
}

pub fn validate_usage(record: &UsageRecord) -> Result<(), BillingError> {
    if record.record_id.is_empty() {
        return Err(BillingError::EmptyRecordId);
    }
    if record.tenant_id.is_empty() {
        return Err(BillingError::EmptyTenantId);
    }
    if record.quantity == 0 {
        return Err(BillingError::ZeroQuantity);
    }
    Ok(())
}

pub fn finalize_line(line: &LineItem) -> Result<u128, BillingError> {
    validate_usage(&line.usage)?;
    if line.tax_profile_ref.is_none() {
        return Err(BillingError::NoTaxProfileRef {
            line_id: line.line_id.clone(),
        });
    }
    Ok(line.subtotal_micros())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usage(id: &str, unit: UsageUnit, qty: u64) -> UsageRecord {
        UsageRecord {
            record_id: id.into(),
            tenant_id: "t1".into(),
            unit,
            quantity: qty,
            timestamp_unix_ms: 0,
        }
    }

    fn line(id: &str, unit: UsageUnit, qty: u64, price: u64, profile: Option<&str>) -> LineItem {
        LineItem {
            line_id: id.into(),
            usage: usage(id, unit, qty),
            unit_price_micros: price,
            tax_profile_ref: profile.map(String::from),
        }
    }

    #[test]
    fn subtotal_multiplies_quantity_and_price() {
        let l = line("L1", UsageUnit::Cpu, 5, 100, Some("profile-alpha"));
        assert_eq!(l.subtotal_micros(), 500);
    }

    #[test]
    fn finalize_valid_line_returns_subtotal() {
        let l = line("L1", UsageUnit::Cpu, 5, 100, Some("profile-alpha"));
        assert_eq!(finalize_line(&l).unwrap(), 500);
    }

    #[test]
    fn finalize_without_tax_profile_errors() {
        let l = line("L1", UsageUnit::Cpu, 5, 100, None);
        assert!(matches!(
            finalize_line(&l),
            Err(BillingError::NoTaxProfileRef { .. })
        ));
    }

    #[test]
    fn zero_quantity_errors() {
        let l = line("L1", UsageUnit::Cpu, 0, 100, Some("profile-alpha"));
        assert!(matches!(finalize_line(&l), Err(BillingError::ZeroQuantity)));
    }

    #[test]
    fn empty_record_id_errors() {
        let l = line("", UsageUnit::Cpu, 5, 100, Some("profile-alpha"));
        assert!(matches!(
            finalize_line(&l),
            Err(BillingError::EmptyRecordId)
        ));
    }

    #[test]
    fn empty_tenant_id_errors() {
        let mut l = line("L1", UsageUnit::Cpu, 5, 100, Some("profile-alpha"));
        l.usage.tenant_id = String::new();
        assert!(matches!(
            finalize_line(&l),
            Err(BillingError::EmptyTenantId)
        ));
    }

    #[test]
    fn usage_unit_names_distinct() {
        use std::collections::HashSet;
        let s: HashSet<_> = [
            UsageUnit::Cpu,
            UsageUnit::GpuHour,
            UsageUnit::GibStorage,
            UsageUnit::GibEgress,
            UsageUnit::Request,
            UsageUnit::Token,
        ]
        .iter()
        .map(|u| u.name())
        .collect();
        assert_eq!(s.len(), 6);
    }
}
