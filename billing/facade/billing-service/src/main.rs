#[cfg(not(test))]
use billing_service::{config, observability};

#[cfg(not(test))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observability::init();
    let _cfg = config::load()?;
    // TODO(ADR-0478): wire subsystems
    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use billing_domain::{
        BillingAccount, BillingAccountCreate, BillingAccountState, BillingPeriod,
        CloudBillingError, InvoiceGenerate, InvoiceLineItemCreate, Money, TaxInvoiceFormat,
    };
    use billing_metering::{MeterUnit, MeterUnitKind};
    use billing_service::invoicing::{
        GenerateInvoiceRequest, InMemoryInvoiceService, InvoiceApplicationError,
    };
    use data_boundary_kernel::DataClass;

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
    fn generates_a_valid_invoice_and_rejects_empty_line_items() {
        let mut service = InMemoryInvoiceService::new();

        let result = service
            .generate(request())
            .expect("valid invoice is generated");

        assert_eq!(result.invoice.id.value.value, "inv_alpha_202605_001");
        assert_eq!(service.invoice_count(), 1);

        let mut invalid = request();
        invalid.invoice.id = "inv_alpha_202605_empty".to_owned();
        invalid.invoice.line_items.clear();
        invalid.invoice.subtotal =
            Money::new("OYC", 0).expect("zero money representation is valid");
        invalid.invoice.total = Money::new("OYC", 10_000).expect("money fixture valid");

        let error = service
            .generate(invalid)
            .expect_err("empty invoice is rejected");

        assert_eq!(
            error,
            InvoiceApplicationError::Domain(CloudBillingError::InvalidInvoiceLineItem)
        );
        assert_eq!(service.invoice_count(), 1);
    }
}
