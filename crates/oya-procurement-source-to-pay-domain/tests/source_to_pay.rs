use oya_procurement_source_to_pay_domain::{
    MoneyAmount, ProcurementDomainError, PurchaseOrderInput, PurchaseOrderState,
    PurchaseRequisitionInput, RequisitionState, SupplierQualificationInput,
    SupplierQualificationStatus, ThreeWayMatchInput, ThreeWayMatchState,
    approve_purchase_requisition, issue_purchase_order, perform_three_way_match, qualify_supplier,
};

fn usd(amount_minor: i64) -> MoneyAmount {
    MoneyAmount {
        amount_minor,
        currency: "USD".to_owned(),
    }
}

fn supplier_input() -> SupplierQualificationInput {
    SupplierQualificationInput {
        supplier_id: "sup_acme".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        supplier_profile_ref: "src/supplier/acme/profile".to_owned(),
        kyb_evidence_ref: "audit/procurement/supplier/acme/kyb".to_owned(),
        risk_screening_evidence_ref: "audit/procurement/supplier/acme/risk".to_owned(),
        approved_vendor_master_ref: "src/vendor-master/acme".to_owned(),
        qualified_at_epoch_seconds: 1_779_545_400,
    }
}

fn requisition_input(supplier_qualified: bool) -> PurchaseRequisitionInput {
    PurchaseRequisitionInput {
        requisition_id: "preq_laptops".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        requester_employee_id: "emp_requester".to_owned(),
        approver_employee_id: "emp_approver".to_owned(),
        supplier_id: "sup_acme".to_owned(),
        supplier_qualified,
        amount: usd(125_000),
        budget_ref: "src/budget/it-hardware/2026".to_owned(),
        policy_evidence_ref: "audit/procurement/preq_laptops/policy".to_owned(),
        approval_evidence_ref: "audit/procurement/preq_laptops/approval".to_owned(),
    }
}

fn purchase_order_input(requisition_approved: bool) -> PurchaseOrderInput {
    PurchaseOrderInput {
        purchase_order_id: "po_laptops".to_owned(),
        requisition_id: "preq_laptops".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        supplier_id: "sup_acme".to_owned(),
        requisition_approved,
        amount: usd(125_000),
        contract_ref: "src/contracts/acme/laptops-2026".to_owned(),
        approval_evidence_ref: "audit/procurement/preq_laptops/approval".to_owned(),
        issue_evidence_ref: "audit/procurement/po_laptops/issued".to_owned(),
    }
}

fn match_input(purchase_order_issued: bool) -> ThreeWayMatchInput {
    ThreeWayMatchInput {
        match_id: "pmatch_laptops".to_owned(),
        purchase_order_id: "po_laptops".to_owned(),
        receipt_id: "gr_laptops".to_owned(),
        invoice_id: "pinv_laptops".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        supplier_id: "sup_acme".to_owned(),
        purchase_order_issued,
        ordered_quantity: 10,
        received_quantity: 10,
        invoiced_quantity: 10,
        ordered_amount: usd(125_000),
        invoiced_amount: usd(125_000),
        receipt_evidence_ref: "audit/procurement/po_laptops/receipt".to_owned(),
        invoice_evidence_ref: "audit/procurement/po_laptops/invoice".to_owned(),
        accounting_handoff_evidence_ref: "audit/procurement/po_laptops/accounting-handoff"
            .to_owned(),
    }
}

#[test]
fn approved_requisition_issues_purchase_order_and_three_way_match() {
    let supplier = qualify_supplier(supplier_input()).unwrap();
    assert_eq!(
        supplier.status.value,
        SupplierQualificationStatus::Qualified
    );
    assert_eq!(supplier.supplier_id.value.value, "sup_acme");

    let requisition = approve_purchase_requisition(requisition_input(true)).unwrap();
    assert_eq!(requisition.state.value, RequisitionState::Approved);
    assert_eq!(requisition.amount.value.amount_minor, 125_000);

    let purchase_order = issue_purchase_order(purchase_order_input(true)).unwrap();
    assert_eq!(purchase_order.state.value, PurchaseOrderState::Issued);
    assert_eq!(purchase_order.requisition_id.value.value, "preq_laptops");

    let matched = perform_three_way_match(match_input(true)).unwrap();
    assert_eq!(matched.state.value, ThreeWayMatchState::Matched);
    assert!(matched.liability_draft_allowed.value);
    assert!(!matched.payment_execution_attached.value);
    assert!(!matched.inventory_mutation_attached.value);
    assert!(!matched.supplier_network_call_attached.value);
    assert!(!matched.cloud_deployment_attached.value);
}

#[test]
fn procurement_refuses_unqualified_supplier_and_unapproved_purchase_order() {
    assert_eq!(
        approve_purchase_requisition(requisition_input(false)),
        Err(ProcurementDomainError::SupplierQualificationRequired)
    );
    assert_eq!(
        issue_purchase_order(purchase_order_input(false)),
        Err(ProcurementDomainError::RequisitionApprovalRequired)
    );
    assert_eq!(
        perform_three_way_match(match_input(false)),
        Err(ProcurementDomainError::PurchaseOrderIssueRequired)
    );
}

#[test]
fn three_way_match_refuses_quantity_and_amount_mismatches() {
    let mut quantity_mismatch = match_input(true);
    quantity_mismatch.received_quantity = 9;
    assert_eq!(
        perform_three_way_match(quantity_mismatch),
        Err(ProcurementDomainError::QuantityMismatch)
    );

    let mut amount_mismatch = match_input(true);
    amount_mismatch.invoiced_amount = usd(126_000);
    assert_eq!(
        perform_three_way_match(amount_mismatch),
        Err(ProcurementDomainError::AmountMismatch)
    );
}

#[test]
fn procurement_validates_evidence_and_source_ref_boundaries() {
    let mut unsafe_supplier = supplier_input();
    unsafe_supplier.kyb_evidence_ref = "audit/procurement/secret-token".to_owned();
    assert_eq!(
        qualify_supplier(unsafe_supplier),
        Err(ProcurementDomainError::InvalidEvidenceRef)
    );

    let mut unsafe_requisition = requisition_input(true);
    unsafe_requisition.budget_ref = "src/../budget".to_owned();
    assert_eq!(
        approve_purchase_requisition(unsafe_requisition),
        Err(ProcurementDomainError::InvalidSourceDocumentRef)
    );

    let mut bad_money = purchase_order_input(true);
    bad_money.amount.currency = "usd".to_owned();
    assert_eq!(
        issue_purchase_order(bad_money),
        Err(ProcurementDomainError::InvalidMoney)
    );
}
