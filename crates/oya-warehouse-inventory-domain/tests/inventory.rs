use oya_warehouse_inventory_domain::{
    CycleCountInput, CycleCountState, GoodsReceiptInput, GoodsReceiptState, InventoryPickInput,
    InventoryPickState, InventoryReservationInput, InventoryReservationState, PutawayInput,
    StockPositionState, WarehouseDomainError, confirm_inventory_pick, perform_cycle_count,
    record_goods_receipt, record_putaway, reserve_inventory,
};

fn goods_receipt_input() -> GoodsReceiptInput {
    GoodsReceiptInput {
        goods_receipt_id: "gr_laptops_20260523".to_owned(),
        purchase_order_id: "po_laptops".to_owned(),
        inbound_load_id: "iload_laptops".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        warehouse_id: "wh_main".to_owned(),
        item_id: "item_laptop".to_owned(),
        expected_quantity: 10,
        received_quantity: 10,
        unit_of_measure: "EA".to_owned(),
        purchase_order_ref: "src/procurement/po_laptops".to_owned(),
        receiving_evidence_ref: "audit/warehouse/gr_laptops/receiving".to_owned(),
        received_at_epoch_seconds: 1_779_546_600,
    }
}

fn putaway_input(receipt_recorded: bool) -> PutawayInput {
    PutawayInput {
        putaway_task_id: "ptask_laptops".to_owned(),
        goods_receipt_id: "gr_laptops_20260523".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        warehouse_id: "wh_main".to_owned(),
        item_id: "item_laptop".to_owned(),
        source_location_id: "loc_receiving".to_owned(),
        target_bin_id: "bin_a01".to_owned(),
        receipt_recorded,
        quantity: 10,
        target_bin_remaining_capacity: 25,
        location_directive_ref: "src/warehouse/location-directives/laptops".to_owned(),
        putaway_evidence_ref: "audit/warehouse/gr_laptops/putaway".to_owned(),
    }
}

fn reservation_input() -> InventoryReservationInput {
    InventoryReservationInput {
        reservation_id: "res_laptops_sales".to_owned(),
        outbound_order_id: "so_laptops".to_owned(),
        stock_position_id: "stock_bin_a01_laptops".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        warehouse_id: "wh_main".to_owned(),
        item_id: "item_laptop".to_owned(),
        available_quantity: 10,
        reserve_quantity: 4,
        allocation_policy_ref: "src/warehouse/allocation/laptops-fefo".to_owned(),
        reservation_evidence_ref: "audit/warehouse/so_laptops/reservation".to_owned(),
    }
}

fn pick_input(reservation_active: bool) -> InventoryPickInput {
    InventoryPickInput {
        pick_task_id: "pick_laptops_sales".to_owned(),
        reservation_id: "res_laptops_sales".to_owned(),
        outbound_order_id: "so_laptops".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        warehouse_id: "wh_main".to_owned(),
        item_id: "item_laptop".to_owned(),
        source_bin_id: "bin_a01".to_owned(),
        reservation_active,
        reserved_quantity: 4,
        picked_quantity: 4,
        pick_evidence_ref: "audit/warehouse/so_laptops/pick".to_owned(),
    }
}

fn cycle_count_input() -> CycleCountInput {
    CycleCountInput {
        cycle_count_id: "cc_bin_a01_laptops".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        warehouse_id: "wh_main".to_owned(),
        bin_id: "bin_a01".to_owned(),
        item_id: "item_laptop".to_owned(),
        book_quantity: 6,
        counted_quantity: 5,
        tolerance_quantity: 1,
        count_evidence_ref: "audit/warehouse/bin_a01/cycle-count".to_owned(),
    }
}

#[test]
fn goods_receipt_putaway_reservation_pick_and_cycle_count_flow() {
    let receipt = record_goods_receipt(goods_receipt_input()).unwrap();
    assert_eq!(receipt.state.value, GoodsReceiptState::Recorded);
    assert_eq!(receipt.received_quantity.value, 10);
    assert!(!receipt.procurement_three_way_match_attached.value);
    assert!(!receipt.accounting_ledger_mutation_attached.value);

    let stock = record_putaway(putaway_input(true)).unwrap();
    assert_eq!(stock.state.value, StockPositionState::Available);
    assert_eq!(stock.on_hand_quantity.value, 10);
    assert!(!stock.robotics_or_scanner_runtime_attached.value);
    assert!(!stock.cloud_deployment_attached.value);

    let reservation = reserve_inventory(reservation_input()).unwrap();
    assert_eq!(reservation.state.value, InventoryReservationState::Reserved);
    assert_eq!(reservation.reserved_quantity.value, 4);
    assert_eq!(reservation.available_after_reservation.value, 6);

    let pick = confirm_inventory_pick(pick_input(true)).unwrap();
    assert_eq!(pick.state.value, InventoryPickState::Picked);
    assert!(pick.shipment_release_allowed.value);
    assert!(!pick.carrier_network_call_attached.value);
    assert!(!pick.shipping_label_attached.value);

    let count = perform_cycle_count(cycle_count_input()).unwrap();
    assert_eq!(count.state.value, CycleCountState::Reconciled);
    assert_eq!(count.variance_quantity.value, 1);
    assert!(count.within_tolerance.value);
    assert!(!count.accounting_adjustment_attached.value);
    assert!(!count.durable_inventory_write_attached.value);
}

#[test]
fn warehouse_refuses_missing_receipt_and_inactive_reservation() {
    assert_eq!(
        record_putaway(putaway_input(false)),
        Err(WarehouseDomainError::GoodsReceiptRequired)
    );
    assert_eq!(
        confirm_inventory_pick(pick_input(false)),
        Err(WarehouseDomainError::ReservationRequired)
    );
}

#[test]
fn warehouse_refuses_capacity_shortage_and_over_reservation() {
    let mut no_capacity = putaway_input(true);
    no_capacity.target_bin_remaining_capacity = 9;
    assert_eq!(
        record_putaway(no_capacity),
        Err(WarehouseDomainError::InsufficientBinCapacity)
    );

    let mut over_reserved = reservation_input();
    over_reserved.reserve_quantity = 11;
    assert_eq!(
        reserve_inventory(over_reserved),
        Err(WarehouseDomainError::InsufficientAvailableQuantity)
    );
}

#[test]
fn warehouse_validates_refs_quantities_and_pick_counts() {
    let mut unsafe_receipt = goods_receipt_input();
    unsafe_receipt.receiving_evidence_ref = "audit/warehouse/secret-token".to_owned();
    assert_eq!(
        record_goods_receipt(unsafe_receipt),
        Err(WarehouseDomainError::InvalidEvidenceRef)
    );

    let mut bad_source = putaway_input(true);
    bad_source.location_directive_ref = "src/../warehouse".to_owned();
    assert_eq!(
        record_putaway(bad_source),
        Err(WarehouseDomainError::InvalidSourceDocumentRef)
    );

    let mut bad_pick = pick_input(true);
    bad_pick.picked_quantity = 5;
    assert_eq!(
        confirm_inventory_pick(bad_pick),
        Err(WarehouseDomainError::PickQuantityMismatch)
    );

    let mut bad_count = cycle_count_input();
    bad_count.tolerance_quantity = 0;
    assert_eq!(
        perform_cycle_count(bad_count),
        Err(WarehouseDomainError::CycleCountVarianceOutsideTolerance)
    );
}
