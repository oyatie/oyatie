//! Warehouse inventory domain foundation.
//!
//! This crate owns pure warehouse invariants for goods receipt, putaway stock
//! positioning, inventory reservation, pick confirmation, and cycle-count
//! reconciliation metadata. It does not perform durable persistence,
//! procurement three-way match, accounting ledger mutation, robotics/scanner
//! runtime I/O, carrier calls, shipping-label generation, Workflow execution,
//! runtime audit-chain emission, or cloud deployment.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// panic assertions to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const GOODS_RECEIPT_ID_PREFIX: &str = "gr_";
const PURCHASE_ORDER_ID_PREFIX: &str = "po_";
const INBOUND_LOAD_ID_PREFIX: &str = "iload_";
const TENANT_ID_PREFIX: &str = "ten_";
const LEGAL_ENTITY_ID_PREFIX: &str = "le_";
const WAREHOUSE_ID_PREFIX: &str = "wh_";
const ITEM_ID_PREFIX: &str = "item_";
const PUTAWAY_TASK_ID_PREFIX: &str = "ptask_";
const LOCATION_ID_PREFIX: &str = "loc_";
const BIN_ID_PREFIX: &str = "bin_";
const RESERVATION_ID_PREFIX: &str = "res_";
const OUTBOUND_ORDER_ID_PREFIX: &str = "so_";
const STOCK_POSITION_ID_PREFIX: &str = "stock_";
const PICK_TASK_ID_PREFIX: &str = "pick_";
const CYCLE_COUNT_ID_PREFIX: &str = "cc_";
const SOURCE_REF_PREFIX: &str = "src/";
const AUDIT_REF_PREFIX: &str = "audit/";
const WAREHOUSE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GoodsReceiptId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PurchaseOrderId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct InboundLoadId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TenantId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LegalEntityId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WarehouseId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ItemId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PutawayTaskId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LocationId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct StorageBinId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ReservationId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OutboundOrderId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct StockPositionId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PickTaskId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CycleCountId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceDocumentRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EvidenceRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GoodsReceiptState {
    Recorded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StockPositionState {
    Available,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InventoryReservationState {
    Reserved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InventoryPickState {
    Picked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CycleCountState {
    Reconciled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoodsReceiptInput {
    pub goods_receipt_id: String,       // data_class: INTERNAL_ONLY
    pub purchase_order_id: String,      // data_class: INTERNAL_ONLY
    pub inbound_load_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,        // data_class: INTERNAL_ONLY
    pub warehouse_id: String,           // data_class: INTERNAL_ONLY
    pub item_id: String,                // data_class: INTERNAL_ONLY
    pub expected_quantity: u32,         // data_class: FINANCIAL
    pub received_quantity: u32,         // data_class: FINANCIAL
    pub unit_of_measure: String,        // data_class: INTERNAL_ONLY
    pub purchase_order_ref: String,     // data_class: INTERNAL_ONLY
    pub receiving_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub received_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoodsReceiptRecord {
    pub goods_receipt_id: Classified<GoodsReceiptId>, // data_class: INTERNAL_ONLY
    pub purchase_order_id: Classified<PurchaseOrderId>, // data_class: INTERNAL_ONLY
    pub inbound_load_id: Classified<InboundLoadId>,   // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,              // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,   // data_class: INTERNAL_ONLY
    pub warehouse_id: Classified<WarehouseId>,        // data_class: INTERNAL_ONLY
    pub item_id: Classified<ItemId>,                  // data_class: INTERNAL_ONLY
    pub expected_quantity: Classified<u32>,           // data_class: FINANCIAL
    pub received_quantity: Classified<u32>,           // data_class: FINANCIAL
    pub quantity_variance: Classified<u32>,           // data_class: FINANCIAL
    pub unit_of_measure: Classified<String>,          // data_class: INTERNAL_ONLY
    pub purchase_order_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub receiving_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub received_at_epoch_seconds: Classified<u64>,   // data_class: INTERNAL_ONLY
    pub state: Classified<GoodsReceiptState>,         // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,          // data_class: INTERNAL_ONLY
    pub procurement_three_way_match_attached: Classified<bool>, // data_class: PUBLIC
    pub accounting_ledger_mutation_attached: Classified<bool>, // data_class: PUBLIC
    pub durable_inventory_write_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,              // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PutawayInput {
    pub putaway_task_id: String,            // data_class: INTERNAL_ONLY
    pub goods_receipt_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,            // data_class: INTERNAL_ONLY
    pub warehouse_id: String,               // data_class: INTERNAL_ONLY
    pub item_id: String,                    // data_class: INTERNAL_ONLY
    pub source_location_id: String,         // data_class: INTERNAL_ONLY
    pub target_bin_id: String,              // data_class: INTERNAL_ONLY
    pub receipt_recorded: bool,             // data_class: INTERNAL_ONLY
    pub quantity: u32,                      // data_class: FINANCIAL
    pub target_bin_remaining_capacity: u32, // data_class: FINANCIAL
    pub location_directive_ref: String,     // data_class: INTERNAL_ONLY
    pub putaway_evidence_ref: String,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StockPosition {
    pub stock_position_id: Classified<StockPositionId>, // data_class: INTERNAL_ONLY
    pub putaway_task_id: Classified<PutawayTaskId>,     // data_class: INTERNAL_ONLY
    pub goods_receipt_id: Classified<GoodsReceiptId>,   // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,     // data_class: INTERNAL_ONLY
    pub warehouse_id: Classified<WarehouseId>,          // data_class: INTERNAL_ONLY
    pub item_id: Classified<ItemId>,                    // data_class: INTERNAL_ONLY
    pub source_location_id: Classified<LocationId>,     // data_class: INTERNAL_ONLY
    pub target_bin_id: Classified<StorageBinId>,        // data_class: INTERNAL_ONLY
    pub on_hand_quantity: Classified<u32>,              // data_class: FINANCIAL
    pub available_quantity: Classified<u32>,            // data_class: FINANCIAL
    pub target_bin_remaining_capacity_after_putaway: Classified<u32>, // data_class: FINANCIAL
    pub location_directive_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub putaway_evidence_ref: Classified<EvidenceRef>,  // data_class: INTERNAL_ONLY
    pub state: Classified<StockPositionState>,          // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,            // data_class: INTERNAL_ONLY
    pub robotics_or_scanner_runtime_attached: Classified<bool>, // data_class: PUBLIC
    pub durable_inventory_write_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,    // data_class: PUBLIC
    pub schema_version: Classified<u32>,                // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryReservationInput {
    pub reservation_id: String,           // data_class: INTERNAL_ONLY
    pub outbound_order_id: String,        // data_class: INTERNAL_ONLY
    pub stock_position_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,          // data_class: INTERNAL_ONLY
    pub warehouse_id: String,             // data_class: INTERNAL_ONLY
    pub item_id: String,                  // data_class: INTERNAL_ONLY
    pub available_quantity: u32,          // data_class: FINANCIAL
    pub reserve_quantity: u32,            // data_class: FINANCIAL
    pub allocation_policy_ref: String,    // data_class: INTERNAL_ONLY
    pub reservation_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryReservation {
    pub reservation_id: Classified<ReservationId>, // data_class: INTERNAL_ONLY
    pub outbound_order_id: Classified<OutboundOrderId>, // data_class: INTERNAL_ONLY
    pub stock_position_id: Classified<StockPositionId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,           // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub warehouse_id: Classified<WarehouseId>,     // data_class: INTERNAL_ONLY
    pub item_id: Classified<ItemId>,               // data_class: INTERNAL_ONLY
    pub reserved_quantity: Classified<u32>,        // data_class: FINANCIAL
    pub available_after_reservation: Classified<u32>, // data_class: FINANCIAL
    pub allocation_policy_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub reservation_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<InventoryReservationState>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,       // data_class: INTERNAL_ONLY
    pub pick_release_allowed: Classified<bool>,    // data_class: PUBLIC
    pub carrier_network_call_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryPickInput {
    pub pick_task_id: String,      // data_class: INTERNAL_ONLY
    pub reservation_id: String,    // data_class: INTERNAL_ONLY
    pub outbound_order_id: String, // data_class: INTERNAL_ONLY
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,   // data_class: INTERNAL_ONLY
    pub warehouse_id: String,      // data_class: INTERNAL_ONLY
    pub item_id: String,           // data_class: INTERNAL_ONLY
    pub source_bin_id: String,     // data_class: INTERNAL_ONLY
    pub reservation_active: bool,  // data_class: INTERNAL_ONLY
    pub reserved_quantity: u32,    // data_class: FINANCIAL
    pub picked_quantity: u32,      // data_class: FINANCIAL
    pub pick_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryPickConfirmation {
    pub pick_task_id: Classified<PickTaskId>, // data_class: INTERNAL_ONLY
    pub reservation_id: Classified<ReservationId>, // data_class: INTERNAL_ONLY
    pub outbound_order_id: Classified<OutboundOrderId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,      // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub warehouse_id: Classified<WarehouseId>, // data_class: INTERNAL_ONLY
    pub item_id: Classified<ItemId>,          // data_class: INTERNAL_ONLY
    pub source_bin_id: Classified<StorageBinId>, // data_class: INTERNAL_ONLY
    pub picked_quantity: Classified<u32>,     // data_class: FINANCIAL
    pub pick_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<InventoryPickState>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,  // data_class: INTERNAL_ONLY
    pub shipment_release_allowed: Classified<bool>, // data_class: PUBLIC
    pub carrier_network_call_attached: Classified<bool>, // data_class: PUBLIC
    pub shipping_label_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,      // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CycleCountInput {
    pub cycle_count_id: String,     // data_class: INTERNAL_ONLY
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,    // data_class: INTERNAL_ONLY
    pub warehouse_id: String,       // data_class: INTERNAL_ONLY
    pub bin_id: String,             // data_class: INTERNAL_ONLY
    pub item_id: String,            // data_class: INTERNAL_ONLY
    pub book_quantity: u32,         // data_class: FINANCIAL
    pub counted_quantity: u32,      // data_class: FINANCIAL
    pub tolerance_quantity: u32,    // data_class: FINANCIAL
    pub count_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CycleCountReconciliation {
    pub cycle_count_id: Classified<CycleCountId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,          // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub warehouse_id: Classified<WarehouseId>,    // data_class: INTERNAL_ONLY
    pub bin_id: Classified<StorageBinId>,         // data_class: INTERNAL_ONLY
    pub item_id: Classified<ItemId>,              // data_class: INTERNAL_ONLY
    pub book_quantity: Classified<u32>,           // data_class: FINANCIAL
    pub counted_quantity: Classified<u32>,        // data_class: FINANCIAL
    pub variance_quantity: Classified<u32>,       // data_class: FINANCIAL
    pub tolerance_quantity: Classified<u32>,      // data_class: FINANCIAL
    pub within_tolerance: Classified<bool>,       // data_class: PUBLIC
    pub count_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<CycleCountState>,       // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,      // data_class: INTERNAL_ONLY
    pub accounting_adjustment_attached: Classified<bool>, // data_class: PUBLIC
    pub durable_inventory_write_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WarehouseDomainError {
    InvalidGoodsReceiptId,
    InvalidPurchaseOrderId,
    InvalidInboundLoadId,
    InvalidTenantId,
    InvalidLegalEntityId,
    InvalidWarehouseId,
    InvalidItemId,
    InvalidPutawayTaskId,
    InvalidLocationId,
    InvalidBinId,
    InvalidReservationId,
    InvalidOutboundOrderId,
    InvalidStockPositionId,
    InvalidPickTaskId,
    InvalidCycleCountId,
    InvalidSourceDocumentRef,
    InvalidEvidenceRef,
    InvalidQuantity,
    InvalidUnitOfMeasure,
    InvalidTimestamp,
    GoodsReceiptRequired,
    ReservationRequired,
    InsufficientBinCapacity,
    InsufficientAvailableQuantity,
    PickQuantityMismatch,
    CycleCountVarianceOutsideTolerance,
}

pub fn record_goods_receipt(
    input: GoodsReceiptInput,
) -> Result<GoodsReceiptRecord, WarehouseDomainError> {
    validate_goods_receipt_id(&input.goods_receipt_id)?;
    validate_purchase_order_id(&input.purchase_order_id)?;
    validate_inbound_load_id(&input.inbound_load_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_warehouse_id(&input.warehouse_id)?;
    validate_item_id(&input.item_id)?;
    validate_positive_quantity(input.expected_quantity)?;
    validate_positive_quantity(input.received_quantity)?;
    validate_unit_of_measure(&input.unit_of_measure)?;
    validate_source_ref(&input.purchase_order_ref)?;
    validate_evidence_ref(&input.receiving_evidence_ref)?;
    if input.received_at_epoch_seconds == 0 {
        return Err(WarehouseDomainError::InvalidTimestamp);
    }
    let quantity_variance = input.expected_quantity.abs_diff(input.received_quantity);
    let idempotency_key = format!(
        "warehouse:goods-receipt:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.warehouse_id, input.goods_receipt_id
    );

    Ok(GoodsReceiptRecord {
        goods_receipt_id: internal(GoodsReceiptId {
            value: input.goods_receipt_id,
        }),
        purchase_order_id: internal(PurchaseOrderId {
            value: input.purchase_order_id,
        }),
        inbound_load_id: internal(InboundLoadId {
            value: input.inbound_load_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        warehouse_id: internal(WarehouseId {
            value: input.warehouse_id,
        }),
        item_id: internal(ItemId {
            value: input.item_id,
        }),
        expected_quantity: financial(input.expected_quantity),
        received_quantity: financial(input.received_quantity),
        quantity_variance: financial(quantity_variance),
        unit_of_measure: internal(input.unit_of_measure),
        purchase_order_ref: internal(SourceDocumentRef {
            value: input.purchase_order_ref,
        }),
        receiving_evidence_ref: internal(EvidenceRef {
            value: input.receiving_evidence_ref,
        }),
        received_at_epoch_seconds: internal(input.received_at_epoch_seconds),
        state: internal(GoodsReceiptState::Recorded),
        idempotency_key: internal(idempotency_key),
        procurement_three_way_match_attached: public(false),
        accounting_ledger_mutation_attached: public(false),
        durable_inventory_write_attached: public(false),
        schema_version: public(WAREHOUSE_SCHEMA_VERSION),
    })
}

pub fn record_putaway(input: PutawayInput) -> Result<StockPosition, WarehouseDomainError> {
    validate_putaway_task_id(&input.putaway_task_id)?;
    validate_goods_receipt_id(&input.goods_receipt_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_warehouse_id(&input.warehouse_id)?;
    validate_item_id(&input.item_id)?;
    validate_location_id(&input.source_location_id)?;
    validate_bin_id(&input.target_bin_id)?;
    if !input.receipt_recorded {
        return Err(WarehouseDomainError::GoodsReceiptRequired);
    }
    validate_positive_quantity(input.quantity)?;
    if input.target_bin_remaining_capacity < input.quantity {
        return Err(WarehouseDomainError::InsufficientBinCapacity);
    }
    validate_source_ref(&input.location_directive_ref)?;
    validate_evidence_ref(&input.putaway_evidence_ref)?;
    let remaining_capacity = input.target_bin_remaining_capacity - input.quantity;
    let stock_position_id = format!("stock_{}_{}", input.target_bin_id, input.item_id);
    let idempotency_key = format!(
        "warehouse:putaway:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.warehouse_id, input.putaway_task_id
    );

    Ok(StockPosition {
        stock_position_id: internal(StockPositionId {
            value: stock_position_id,
        }),
        putaway_task_id: internal(PutawayTaskId {
            value: input.putaway_task_id,
        }),
        goods_receipt_id: internal(GoodsReceiptId {
            value: input.goods_receipt_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        warehouse_id: internal(WarehouseId {
            value: input.warehouse_id,
        }),
        item_id: internal(ItemId {
            value: input.item_id,
        }),
        source_location_id: internal(LocationId {
            value: input.source_location_id,
        }),
        target_bin_id: internal(StorageBinId {
            value: input.target_bin_id,
        }),
        on_hand_quantity: financial(input.quantity),
        available_quantity: financial(input.quantity),
        target_bin_remaining_capacity_after_putaway: financial(remaining_capacity),
        location_directive_ref: internal(SourceDocumentRef {
            value: input.location_directive_ref,
        }),
        putaway_evidence_ref: internal(EvidenceRef {
            value: input.putaway_evidence_ref,
        }),
        state: internal(StockPositionState::Available),
        idempotency_key: internal(idempotency_key),
        robotics_or_scanner_runtime_attached: public(false),
        durable_inventory_write_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(WAREHOUSE_SCHEMA_VERSION),
    })
}

pub fn reserve_inventory(
    input: InventoryReservationInput,
) -> Result<InventoryReservation, WarehouseDomainError> {
    validate_reservation_id(&input.reservation_id)?;
    validate_outbound_order_id(&input.outbound_order_id)?;
    validate_stock_position_id(&input.stock_position_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_warehouse_id(&input.warehouse_id)?;
    validate_item_id(&input.item_id)?;
    validate_positive_quantity(input.available_quantity)?;
    validate_positive_quantity(input.reserve_quantity)?;
    if input.reserve_quantity > input.available_quantity {
        return Err(WarehouseDomainError::InsufficientAvailableQuantity);
    }
    validate_source_ref(&input.allocation_policy_ref)?;
    validate_evidence_ref(&input.reservation_evidence_ref)?;
    let available_after_reservation = input.available_quantity - input.reserve_quantity;
    let idempotency_key = format!(
        "warehouse:reservation:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.warehouse_id, input.reservation_id
    );

    Ok(InventoryReservation {
        reservation_id: internal(ReservationId {
            value: input.reservation_id,
        }),
        outbound_order_id: internal(OutboundOrderId {
            value: input.outbound_order_id,
        }),
        stock_position_id: internal(StockPositionId {
            value: input.stock_position_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        warehouse_id: internal(WarehouseId {
            value: input.warehouse_id,
        }),
        item_id: internal(ItemId {
            value: input.item_id,
        }),
        reserved_quantity: financial(input.reserve_quantity),
        available_after_reservation: financial(available_after_reservation),
        allocation_policy_ref: internal(SourceDocumentRef {
            value: input.allocation_policy_ref,
        }),
        reservation_evidence_ref: internal(EvidenceRef {
            value: input.reservation_evidence_ref,
        }),
        state: internal(InventoryReservationState::Reserved),
        idempotency_key: internal(idempotency_key),
        pick_release_allowed: public(true),
        carrier_network_call_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(WAREHOUSE_SCHEMA_VERSION),
    })
}

pub fn confirm_inventory_pick(
    input: InventoryPickInput,
) -> Result<InventoryPickConfirmation, WarehouseDomainError> {
    validate_pick_task_id(&input.pick_task_id)?;
    validate_reservation_id(&input.reservation_id)?;
    validate_outbound_order_id(&input.outbound_order_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_warehouse_id(&input.warehouse_id)?;
    validate_item_id(&input.item_id)?;
    validate_bin_id(&input.source_bin_id)?;
    if !input.reservation_active {
        return Err(WarehouseDomainError::ReservationRequired);
    }
    validate_positive_quantity(input.reserved_quantity)?;
    validate_positive_quantity(input.picked_quantity)?;
    if input.picked_quantity != input.reserved_quantity {
        return Err(WarehouseDomainError::PickQuantityMismatch);
    }
    validate_evidence_ref(&input.pick_evidence_ref)?;
    let idempotency_key = format!(
        "warehouse:pick:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.warehouse_id, input.pick_task_id
    );

    Ok(InventoryPickConfirmation {
        pick_task_id: internal(PickTaskId {
            value: input.pick_task_id,
        }),
        reservation_id: internal(ReservationId {
            value: input.reservation_id,
        }),
        outbound_order_id: internal(OutboundOrderId {
            value: input.outbound_order_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        warehouse_id: internal(WarehouseId {
            value: input.warehouse_id,
        }),
        item_id: internal(ItemId {
            value: input.item_id,
        }),
        source_bin_id: internal(StorageBinId {
            value: input.source_bin_id,
        }),
        picked_quantity: financial(input.picked_quantity),
        pick_evidence_ref: internal(EvidenceRef {
            value: input.pick_evidence_ref,
        }),
        state: internal(InventoryPickState::Picked),
        idempotency_key: internal(idempotency_key),
        shipment_release_allowed: public(true),
        carrier_network_call_attached: public(false),
        shipping_label_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(WAREHOUSE_SCHEMA_VERSION),
    })
}

pub fn perform_cycle_count(
    input: CycleCountInput,
) -> Result<CycleCountReconciliation, WarehouseDomainError> {
    validate_cycle_count_id(&input.cycle_count_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_warehouse_id(&input.warehouse_id)?;
    validate_bin_id(&input.bin_id)?;
    validate_item_id(&input.item_id)?;
    validate_evidence_ref(&input.count_evidence_ref)?;
    let variance_quantity = input.book_quantity.abs_diff(input.counted_quantity);
    if variance_quantity > input.tolerance_quantity {
        return Err(WarehouseDomainError::CycleCountVarianceOutsideTolerance);
    }
    let idempotency_key = format!(
        "warehouse:cycle-count:{}:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.warehouse_id, input.cycle_count_id
    );

    Ok(CycleCountReconciliation {
        cycle_count_id: internal(CycleCountId {
            value: input.cycle_count_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        warehouse_id: internal(WarehouseId {
            value: input.warehouse_id,
        }),
        bin_id: internal(StorageBinId {
            value: input.bin_id,
        }),
        item_id: internal(ItemId {
            value: input.item_id,
        }),
        book_quantity: financial(input.book_quantity),
        counted_quantity: financial(input.counted_quantity),
        variance_quantity: financial(variance_quantity),
        tolerance_quantity: financial(input.tolerance_quantity),
        within_tolerance: public(true),
        count_evidence_ref: internal(EvidenceRef {
            value: input.count_evidence_ref,
        }),
        state: internal(CycleCountState::Reconciled),
        idempotency_key: internal(idempotency_key),
        accounting_adjustment_attached: public(false),
        durable_inventory_write_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(WAREHOUSE_SCHEMA_VERSION),
    })
}

fn validate_goods_receipt_id(value: &str) -> Result<(), WarehouseDomainError> {
    validate_prefixed_identifier(
        value,
        GOODS_RECEIPT_ID_PREFIX,
        WarehouseDomainError::InvalidGoodsReceiptId,
    )
}

fn validate_purchase_order_id(value: &str) -> Result<(), WarehouseDomainError> {
    validate_prefixed_identifier(
        value,
        PURCHASE_ORDER_ID_PREFIX,
        WarehouseDomainError::InvalidPurchaseOrderId,
    )
}

fn validate_inbound_load_id(value: &str) -> Result<(), WarehouseDomainError> {
    validate_prefixed_identifier(
        value,
        INBOUND_LOAD_ID_PREFIX,
        WarehouseDomainError::InvalidInboundLoadId,
    )
}

fn validate_tenant_id(value: &str) -> Result<(), WarehouseDomainError> {
    validate_prefixed_identifier(
        value,
        TENANT_ID_PREFIX,
        WarehouseDomainError::InvalidTenantId,
    )
}

fn validate_legal_entity_id(value: &str) -> Result<(), WarehouseDomainError> {
    validate_prefixed_identifier(
        value,
        LEGAL_ENTITY_ID_PREFIX,
        WarehouseDomainError::InvalidLegalEntityId,
    )
}

fn validate_warehouse_id(value: &str) -> Result<(), WarehouseDomainError> {
    validate_prefixed_identifier(
        value,
        WAREHOUSE_ID_PREFIX,
        WarehouseDomainError::InvalidWarehouseId,
    )
}

fn validate_item_id(value: &str) -> Result<(), WarehouseDomainError> {
    validate_prefixed_identifier(value, ITEM_ID_PREFIX, WarehouseDomainError::InvalidItemId)
}

fn validate_putaway_task_id(value: &str) -> Result<(), WarehouseDomainError> {
    validate_prefixed_identifier(
        value,
        PUTAWAY_TASK_ID_PREFIX,
        WarehouseDomainError::InvalidPutawayTaskId,
    )
}

fn validate_location_id(value: &str) -> Result<(), WarehouseDomainError> {
    validate_prefixed_identifier(
        value,
        LOCATION_ID_PREFIX,
        WarehouseDomainError::InvalidLocationId,
    )
}

fn validate_bin_id(value: &str) -> Result<(), WarehouseDomainError> {
    validate_prefixed_identifier(value, BIN_ID_PREFIX, WarehouseDomainError::InvalidBinId)
}

fn validate_reservation_id(value: &str) -> Result<(), WarehouseDomainError> {
    validate_prefixed_identifier(
        value,
        RESERVATION_ID_PREFIX,
        WarehouseDomainError::InvalidReservationId,
    )
}

fn validate_outbound_order_id(value: &str) -> Result<(), WarehouseDomainError> {
    validate_prefixed_identifier(
        value,
        OUTBOUND_ORDER_ID_PREFIX,
        WarehouseDomainError::InvalidOutboundOrderId,
    )
}

fn validate_stock_position_id(value: &str) -> Result<(), WarehouseDomainError> {
    validate_prefixed_identifier(
        value,
        STOCK_POSITION_ID_PREFIX,
        WarehouseDomainError::InvalidStockPositionId,
    )
}

fn validate_pick_task_id(value: &str) -> Result<(), WarehouseDomainError> {
    validate_prefixed_identifier(
        value,
        PICK_TASK_ID_PREFIX,
        WarehouseDomainError::InvalidPickTaskId,
    )
}

fn validate_cycle_count_id(value: &str) -> Result<(), WarehouseDomainError> {
    validate_prefixed_identifier(
        value,
        CYCLE_COUNT_ID_PREFIX,
        WarehouseDomainError::InvalidCycleCountId,
    )
}

fn validate_prefixed_identifier(
    value: &str,
    prefix: &str,
    error: WarehouseDomainError,
) -> Result<(), WarehouseDomainError> {
    if value == prefix
        || !value.starts_with(prefix)
        || has_unsafe_text(value)
        || value.contains('/')
        || value.contains("..")
    {
        return Err(error);
    }
    Ok(())
}

fn validate_source_ref(value: &str) -> Result<(), WarehouseDomainError> {
    validate_ref(
        value,
        SOURCE_REF_PREFIX,
        WarehouseDomainError::InvalidSourceDocumentRef,
    )
}

fn validate_evidence_ref(value: &str) -> Result<(), WarehouseDomainError> {
    validate_ref(
        value,
        AUDIT_REF_PREFIX,
        WarehouseDomainError::InvalidEvidenceRef,
    )
}

fn validate_ref(
    value: &str,
    prefix: &str,
    error: WarehouseDomainError,
) -> Result<(), WarehouseDomainError> {
    if value == prefix
        || !value.starts_with(prefix)
        || has_unsafe_text(value)
        || value.contains("..")
    {
        return Err(error);
    }
    let lowered = value.to_ascii_lowercase();
    if lowered.contains("token")
        || lowered.contains("secret")
        || lowered.contains("bearer")
        || lowered.contains("password")
        || lowered.contains("api-key")
        || lowered.contains("apikey")
    {
        return Err(error);
    }
    Ok(())
}

fn validate_positive_quantity(value: u32) -> Result<(), WarehouseDomainError> {
    if value == 0 {
        return Err(WarehouseDomainError::InvalidQuantity);
    }
    Ok(())
}

fn validate_unit_of_measure(value: &str) -> Result<(), WarehouseDomainError> {
    if value.is_empty()
        || value.len() > 8
        || has_unsafe_text(value)
        || !value
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
    {
        return Err(WarehouseDomainError::InvalidUnitOfMeasure);
    }
    Ok(())
}

fn has_unsafe_text(value: &str) -> bool {
    value.chars().any(char::is_whitespace) || value.chars().any(char::is_control)
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, PrivacyDataClass::internal_only())
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn financial<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Financial)
}
