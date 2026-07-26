//! In-memory invoice application boundary.

use billing_domain::{
    BillingAccount, CloudBillingError, CloudBillingLedger, Invoice, InvoiceGenerate,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerateInvoiceRequest {
    pub account: BillingAccount,
    pub invoice: InvoiceGenerate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerateInvoiceResult {
    pub invoice: Invoice,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvoiceApplicationError {
    Domain(CloudBillingError),
}

impl From<CloudBillingError> for InvoiceApplicationError {
    fn from(error: CloudBillingError) -> Self {
        Self::Domain(error)
    }
}

#[derive(Debug, Default)]
pub struct InMemoryInvoiceService {
    ledger: CloudBillingLedger,
}

impl InMemoryInvoiceService {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Generates and retains an issued invoice for this process lifetime.
    ///
    /// # Errors
    ///
    /// Returns a typed domain error when the request violates billing invariants
    /// or repeats an existing invoice identifier.
    pub fn generate(
        &mut self,
        request: GenerateInvoiceRequest,
    ) -> Result<GenerateInvoiceResult, InvoiceApplicationError> {
        let invoice = self
            .ledger
            .generate_invoice(&request.account, request.invoice)?;
        Ok(GenerateInvoiceResult { invoice })
    }

    #[must_use]
    pub fn invoice_count(&self) -> usize {
        self.ledger.invoices().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use billing_domain::{
        BillingAccountCreate, BillingAccountState, BillingPeriod, InvoiceId, InvoiceLineItemCreate,
        Money, TaxInvoiceFormat,
    };
    use billing_metering::{MeterUnit, MeterUnitKind};
    use oya_data_boundary_kernel::DataClass;

    fn request() -> GenerateInvoiceRequest {
        let account = BillingAccount::new(BillingAccountCreate {
            id: "ba_ten_alpha".to_owned(),
            tenant_id: "ten_alpha".to_owned(),
            region: "region-alpha".to_owned(),
            regional_pack: "oya-pack-electronic-tax".to_owned(),
            payment_method: "pm_card_001".to_owned(),
            credit_balance: Money::new("OYC", 10_000).expect("money fixture valid"),
            state: BillingAccountState::Active,
            data_class: DataClass::Financial,
            created_at_epoch_seconds: 1_700_000_000,
        })
        .expect("account fixture valid");
        let subtotal = Money::new("OYC", 100_000).expect("money fixture valid");
        GenerateInvoiceRequest {
            account,
            invoice: InvoiceGenerate {
                id: "inv_alpha_202605_001".to_owned(),
                billing_account_id: "ba_ten_alpha".to_owned(),
                tenant_id: "ten_alpha".to_owned(),
                regional_pack: "oya-pack-electronic-tax".to_owned(),
                period: BillingPeriod::new(1_700_000_000, 1_700_086_400)
                    .expect("period fixture valid"),
                line_items: vec![InvoiceLineItemCreate {
                    id: "ili_compute_001".to_owned(),
                    resource_id: "oya:cloud:region-alpha:ten_alpha:instance:api-001".to_owned(),
                    description: "instance api-001 resource seconds".to_owned(),
                    units: vec![
                        MeterUnit::new(MeterUnitKind::ResourceSecond, 3_600_000_000)
                            .expect("unit fixture valid"),
                    ],
                    subtotal: subtotal.clone(),
                    data_class: DataClass::Financial,
                }],
                subtotal,
                tax: Money::new("OYC", 10_000).expect("money fixture valid"),
                total: Money::new("OYC", 110_000).expect("money fixture valid"),
                tax_invoice_format: TaxInvoiceFormat::ElectronicTaxInvoice,
                tax_registration_id: "taxid/electronic/1234567890".to_owned(),
                issued_at_epoch_seconds: 1_700_086_500,
                due_at_epoch_seconds: 1_700_604_900,
                data_class: DataClass::Financial,
            },
        }
    }

    #[test]
    fn rejects_a_duplicate_invoice_id_without_mutating_the_ledger() {
        let mut service = InMemoryInvoiceService::new();
        service.generate(request()).expect("first invoice is valid");
        let invoice_id = InvoiceId::new("inv_alpha_202605_001").expect("invoice id fixture valid");
        let retained_before = service
            .ledger
            .get_invoice(&invoice_id)
            .cloned()
            .expect("first invoice retained");

        let mut duplicate = request();
        duplicate.invoice.line_items[0].description = "materially changed compute usage".to_owned();

        let error = service
            .generate(duplicate)
            .expect_err("duplicate is rejected");
        let retained_after = service
            .ledger
            .get_invoice(&invoice_id)
            .expect("original invoice remains retained");

        assert_eq!(
            error,
            InvoiceApplicationError::Domain(CloudBillingError::DuplicateInvoice)
        );
        assert_eq!(retained_after, &retained_before);
        assert_eq!(service.invoice_count(), 1);
    }
}
