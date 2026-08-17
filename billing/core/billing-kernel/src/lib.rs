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
    SubtotalOverflow,
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
            Self::SubtotalOverflow => "invoice subtotal overflowed u128".to_owned(),
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

/// Aggregated totals for a finalized invoice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvoiceTotals {
    pub subtotal_micros: u128,
    pub tax_micros: u128,
    pub total_micros: u128,
}

/// Aggregate a slice of `LineItem`s into `InvoiceTotals`.
///
/// `tax_rate_basis_points` is the tax rate in basis points (1 bp = 0.01%).
/// Tax is computed as `round_half_up(subtotal × bps / 10_000)`.
///
/// Empty `lines` returns zero totals.
/// Any line failing `finalize_line` causes the whole aggregate to fail.
/// Arithmetic overflow returns `BillingError::SubtotalOverflow`.
pub fn aggregate_invoice(
    lines: &[LineItem],
    tax_rate_basis_points: u32,
) -> Result<InvoiceTotals, BillingError> {
    let mut subtotal_micros: u128 = 0;
    for line in lines {
        let line_subtotal = finalize_line(line)?;
        subtotal_micros = subtotal_micros
            .checked_add(line_subtotal)
            .ok_or(BillingError::SubtotalOverflow)?;
    }

    let bps = u128::from(tax_rate_basis_points);
    // round-half-up: (a × b + half_divisor) / divisor
    let numerator = subtotal_micros
        .checked_mul(bps)
        .ok_or(BillingError::SubtotalOverflow)?
        .checked_add(5_000)
        .ok_or(BillingError::SubtotalOverflow)?;
    let tax_micros = numerator / 10_000;

    let total_micros = subtotal_micros
        .checked_add(tax_micros)
        .ok_or(BillingError::SubtotalOverflow)?;

    Ok(InvoiceTotals {
        subtotal_micros,
        tax_micros,
        total_micros,
    })
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

    // ── aggregate_invoice acceptance tests ──────────────────────────────────

    // (a) empty line set returns zero totals
    #[test]
    fn aggregate_empty_lines_returns_zero() {
        let result = aggregate_invoice(&[], 1_000).unwrap();
        assert_eq!(
            result,
            InvoiceTotals {
                subtotal_micros: 0,
                tax_micros: 0,
                total_micros: 0,
            }
        );
    }

    // (b) any line missing tax_profile_ref fails the whole aggregate
    #[test]
    fn aggregate_rejects_line_without_tax_profile() {
        let lines = vec![
            line("L1", UsageUnit::Cpu, 10, 100, Some("profile-alpha")),
            line("L2", UsageUnit::Cpu, 5, 200, None),
            line("L3", UsageUnit::Cpu, 3, 50, Some("profile-alpha")),
        ];
        let err = aggregate_invoice(&lines, 1_000).unwrap_err();
        assert!(matches!(err, BillingError::NoTaxProfileRef { line_id } if line_id == "L2"));
    }

    // (c) multi-line subtotal sums correctly
    #[test]
    fn aggregate_multi_line_subtotal_correct() {
        // L1: qty=10, price=100 → 1_000
        // L2: qty=5, price=200 → 1_000
        // L3: qty=3, price=50  → 150
        // subtotal = 2_150; bps=0 → tax=0, total=2_150
        let lines = vec![
            line("L1", UsageUnit::Cpu, 10, 100, Some("p")),
            line("L2", UsageUnit::GpuHour, 5, 200, Some("p")),
            line("L3", UsageUnit::GibStorage, 3, 50, Some("p")),
        ];
        let result = aggregate_invoice(&lines, 0).unwrap();
        assert_eq!(result.subtotal_micros, 2_150);
        assert_eq!(result.tax_micros, 0);
        assert_eq!(result.total_micros, 2_150);
    }

    // (d) tax basis-point math + rounding is exact on representative cases
    #[test]
    fn aggregate_tax_basis_points_rounding_exact() {
        // Case 1: subtotal=1_000_000, bps=1_000 (10%) → tax=100_000 exact
        let lines = vec![line("L1", UsageUnit::Cpu, 1_000, 1_000, Some("p"))];
        let r = aggregate_invoice(&lines, 1_000).unwrap();
        assert_eq!(r.subtotal_micros, 1_000_000);
        assert_eq!(r.tax_micros, 100_000);
        assert_eq!(r.total_micros, 1_100_000);

        // Case 2: round-half-up: subtotal=3, bps=5_000 (50%)
        // exact = 3 * 5000 / 10000 = 1.5 → round-half-up → 2
        // (3 * 5000 + 5000) / 10000 = 20000/10000 = 2 ✓
        let lines2 = vec![line("L2", UsageUnit::Token, 1, 3, Some("p"))];
        let r2 = aggregate_invoice(&lines2, 5_000).unwrap();
        assert_eq!(r2.subtotal_micros, 3);
        assert_eq!(r2.tax_micros, 2);
        assert_eq!(r2.total_micros, 5);

        // Case 3: round-half-up: subtotal=3, bps=3_333
        // exact = 3 * 3333 / 10000 = 9999/10000 = 0.9999 → round-half-up → 1
        // (3 * 3333 + 5000) / 10000 = 14999/10000 = 1 ✓
        let lines3 = vec![line("L3", UsageUnit::Token, 1, 3, Some("p"))];
        let r3 = aggregate_invoice(&lines3, 3_333).unwrap();
        assert_eq!(r3.tax_micros, 1);

        // Case 4: bps=10_000 (100% tax) subtotal=500 → tax=500, total=1000
        let lines4 = vec![line("L4", UsageUnit::Request, 5, 100, Some("p"))];
        let r4 = aggregate_invoice(&lines4, 10_000).unwrap();
        assert_eq!(r4.subtotal_micros, 500);
        assert_eq!(r4.tax_micros, 500);
        assert_eq!(r4.total_micros, 1_000);
    }

    // (e) overflow yields SubtotalOverflow error
    #[test]
    fn aggregate_overflow_yields_error() {
        // Each line subtotal = (2^64-1)^2 = 2^128 - 2^65 + 1, which fits in u128.
        // Two such lines sum to 2^129 - 2^66 + 2, which overflows u128.
        let big_line = |id: &str| LineItem {
            line_id: id.into(),
            usage: UsageRecord {
                record_id: id.into(),
                tenant_id: "t1".into(),
                unit: UsageUnit::Cpu,
                quantity: u64::MAX,
                timestamp_unix_ms: 0,
            },
            unit_price_micros: u64::MAX,
            tax_profile_ref: Some("p".into()),
        };
        let err = aggregate_invoice(&[big_line("big1"), big_line("big2")], 0).unwrap_err();
        assert_eq!(err, BillingError::SubtotalOverflow);
    }

    // Additional: invalid line (zero quantity) propagated
    #[test]
    fn aggregate_propagates_zero_quantity_error() {
        let lines = vec![line("L1", UsageUnit::Cpu, 0, 100, Some("p"))];
        assert!(matches!(
            aggregate_invoice(&lines, 500),
            Err(BillingError::ZeroQuantity)
        ));
    }

    // Additional: zero tax rate
    #[test]
    fn aggregate_zero_tax_rate() {
        let lines = vec![line("L1", UsageUnit::Cpu, 4, 250, Some("p"))];
        let r = aggregate_invoice(&lines, 0).unwrap();
        assert_eq!(r.subtotal_micros, 1_000);
        assert_eq!(r.tax_micros, 0);
        assert_eq!(r.total_micros, 1_000);
    }
}
