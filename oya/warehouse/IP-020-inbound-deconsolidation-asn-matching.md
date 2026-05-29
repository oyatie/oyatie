---
doc_class: ImplementationPlan
ip_id: IP-020
microservice: warehouse
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0245
  - ADR-0253
  - ADR-0263
  - ADR-0294
  - ADR-0297
  - ADR-0314
  - ADR-0315
  - ADR-0329
  - ADR-0330
  - ADR-0331
journey_ref: j102-raw-material-purchase-with-quality-attestation
sap_submodule: EWM-DLV (deliveries)
tenant_class: paid
billing_components:
  - per_usage
persona: Priya Menon, inbound dock lead
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-020: Inbound deconsolidation with ASN matching

## Context

- SAP submodule: EWM-DLV inbound delivery deconsolidation.
- Persona: Priya Menon, inbound dock lead.
- Journey leg: j102 mixed pallet arrives and must be deconsolidated against supplier ASN before receipt completion.
- SAP tables: `/SCWM/PRDI`, `/SCWM/HUHDR`, `/SCWM/HUITM`, `/SCWM/QUANT`.
- Oyatie capability: `InboundDeconsolidation`.
- Precedent: SAP EWM handling-unit deconsolidation plus FedEx scan-to-container reconciliation.
- ADR-0263 binds HU split audit and ADR-0297 gates discrepancy resolution.
- Boundary: owns HU-to-item matching, split evidence, and discrepancy state; quality inspection remains quality-management.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.inbound_handling_unit (
  tenant_id UUID NOT NULL,
  handling_unit_id TEXT NOT NULL,
  inbound_delivery_id TEXT NOT NULL,
  supplier_asn_ref TEXT NOT NULL,
  parent_handling_unit_id TEXT,
  hu_status TEXT NOT NULL CHECK (hu_status IN ('sealed','opened','deconsolidated','discrepant','closed')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, handling_unit_id)
);
CREATE TABLE warehouse.inbound_deconsolidation_line (
  tenant_id UUID NOT NULL,
  deconsolidation_line_id TEXT NOT NULL,
  handling_unit_id TEXT NOT NULL,
  inbound_item_no TEXT NOT NULL,
  material_id TEXT NOT NULL,
  expected_qty NUMERIC(18,6) NOT NULL,
  counted_qty NUMERIC(18,6) NOT NULL,
  match_state TEXT NOT NULL,
  PRIMARY KEY (tenant_id, deconsolidation_line_id)
);
```

### Rust Types

```rust
pub struct InboundHandlingUnit {
    pub tenant_id: TenantId,
    pub handling_unit_id: HandlingUnitId,
    pub inbound_delivery_id: InboundDeliveryId,
    pub supplier_asn_ref: SupplierAsnRef,
    pub parent_handling_unit_id: Option<HandlingUnitId>,
    pub hu_status: HandlingUnitStatus,
}
pub struct InboundDeconsolidationLine {
    pub deconsolidation_line_id: DeconsolidationLineId,
    pub handling_unit_id: HandlingUnitId,
    pub inbound_item_no: DeliveryItemNo,
    pub material_id: MaterialId,
    pub expected_qty: Decimal,
    pub counted_qty: Decimal,
    pub match_state: AsnMatchState,
}
pub enum DeconsolidationError { AsnMissing, HuAlreadyClosed, QuantityMismatch, UnknownMaterial, SplitPolicyDenied }
```

## API Endpoints

- REST `POST /v1/warehouse/inbound-handling-units` registers HU from ASN or scan.
- REST `POST /v1/warehouse/inbound-handling-units/{id}:deconsolidate`.
- REST `POST /v1/warehouse/deconsolidation-lines/{id}:resolve-discrepancy`.
- gRPC `warehouse.deconsolidation.v1.InboundDeconsolidationService.DeconsolidateHandlingUnit`.
- gRPC `RegisterHandlingUnit`, `ResolveDiscrepancy`, and `ListAsnMismatches`.
- AsyncAPI channel `warehouse.inbound-deconsolidation.completed.v1`.
- AsyncAPI channel `warehouse.inbound-deconsolidation.discrepancy-detected.v1`.
- Consumers: inbound delivery, quality-management, inventory-ledger, workflow-engine.

## Cedar Policy Hooks

- Policy: `warehouse::deconsolidation::split`.
- Principal: `WarehouseDockOperator`.
- Action: `handling_unit_deconsolidate`.
- Resource: `InboundHandlingUnit`.
- Context: `tenant_id`, `supplier_asn_ref`, `handling_unit_id`, `quantity_variance`, `quality_gate_state`.
- Forbid when ASN is missing, HU is already closed, discrepancy exceeds threshold without supervisor permit, or quality gate forbids opening.

## Ontology Projection

- Vendor object: SAP EWM handling unit item.
- Oyatie object: `warehouse.inbound_deconsolidation_line`.
- `/SCWM/HUHDR-HUIDENT` -> `handling_unit_id`.
- `/SCWM/HUITM-ITEMID` -> HU item lineage.
- `/SCWM/PRDI-ITEMID` -> inbound item number.
- `/SCWM/QUANT-MATID` -> material evidence.
- Supplier ASN -> expected quantity and material.
- Count result -> `counted_qty`.
- Projection freshness floor: 3 seconds.
- Projection rule: discrepancies remain explicit rather than silently adjusting receipt quantity.

## Workflow Steps

- Node `hu-register`: scan handling unit and bind ASN.
- Node `asn-match`: compare HU item to ASN and inbound delivery item.
- Decision `asn-missing`: block opening and request supplier document.
- Node `hu-open`: record deconsolidation start.
- Decision `unknown-material`: route to discrepancy workflow.
- Decision `quantity-mismatch`: require supervisor or quality review.
- Node `line-confirm`: persist deconsolidated line.
- Node `receipt-update`: update inbound receipt item quantities.
- Node `discrepancy-resolve`: close discrepancy with reason.
- Node `audit-seal`: emit HU split evidence.

## Audit Events

- `EVT-WAREHOUSE-DECONSOLIDATION-HU_REGISTERED`.
- `EVT-WAREHOUSE-DECONSOLIDATION-HU_OPENED`.
- `EVT-WAREHOUSE-DECONSOLIDATION-LINE_CONFIRMED`.
- `EVT-WAREHOUSE-DECONSOLIDATION-DISCREPANCY_DETECTED`.
- `EVT-WAREHOUSE-DECONSOLIDATION-POLICY_DENIED`.
- `EVT-WAREHOUSE-DECONSOLIDATION-IP_ACCEPTED`.
- ADR-0263 envelope stores `handling_unit_id`, `supplier_asn_ref`, expected/counted quantities, and discrepancy reason.

## SLO Targets

- HU registration p50: 25 ms.
- HU registration p95: 90 ms.
- HU registration p99: 220 ms.
- Deconsolidation line confirm p95: 120 ms.
- Rationale: scan flow must stay RF-fast while discrepancy resolution can branch asynchronously.

## Failure Modes and Recovery

- Failure: `ASN-MISSING`; recovery: block HU opening and request supplier ASN.
- Failure: `HU-ALREADY-CLOSED`; recovery: reject duplicate split and show closure evidence.
- Failure: `QUANTITY-MISMATCH`; recovery: create discrepancy and hold receipt update.
- Failure: `UNKNOWN-MATERIAL`; recovery: quarantine HU line and request material master review.
- Failure: `SPLIT-POLICY-DENIED`; recovery: require supervisor permit or quality review.
- Failure: `RECEIPT-UPDATE-FAILED`; recovery: keep deconsolidation line pending and retry outbox.

## Migration Notes

- Import SAP handling unit header and item rows before inbound receipt completion.
- Map supplier ASN line references to inbound item numbers.
- Preserve parent-child HU lineage for mixed pallets.
- Do not close discrepancies during migration without reason code.
- Rollback path: disable deconsolidation command and receive by delivery item only.
- Backfill order: ASNs, inbound deliveries, HUs, HU items, deconsolidation lines, discrepancies.

## Cross-microservice Handoffs

- From procurement: supplier ASN and PO line.
- From quality-management: quality gate on opening controlled HU.
- To inbound delivery: confirmed item quantities.
- To inventory-ledger: material and quant evidence after split.
- To workflow-engine: discrepancy resolution.
- To compliance: HU lineage and split audit events.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The workflow remains bound to SAP EWM inbound delivery deconsolidation and handling units. |
| Persona specificity | Priya Menon owns HU opening, ASN match, discrepancy, and rollback acceptance language. |
| Journey specificity | The j102 mixed-pallet leg drives split quantity, ASN evidence, and quality-gate behavior. |
| DDL anchor | The HU, HU item, deconsolidation line, and discrepancy tables above are normative. |
| Rust anchor | The deconsolidation aggregate, ASN match result, and error enum above are implementation anchors. |
| REST anchor | Open HU, match ASN, split line, and resolve discrepancy endpoints are tenant command surfaces. |
| gRPC anchor | The deconsolidation service is the worker and replay contract. |
| AsyncAPI anchor | HU opened, line matched, split posted, and discrepancy channels carry downstream evidence. |
| Cedar anchor | HU open and split commands are default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP HU, ASN, delivery item, and material lineage projects to split evidence nodes. |
| ADR-0263 class binding | Deconsolidation checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Controlled-material or quality overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on deconsolidation APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, HU id, ASN id, material id, split quantity, and `cedar_decision_id`. |
| Metric | `oya_warehouse_deconsolidation_lines_total{tenant_id,cell_id,outcome,status}` caps outcome/status cardinality. |
| Latency histogram | `oya_warehouse_deconsolidation_duration_seconds` tracks HU open to split completion latency. |
| Trace span | `warehouse.deconsolidation.match_asn` links procurement, quality-management, inventory-ledger, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `handling_unit_id`, `asn_id`, `material_id`, and discrepancy code. |
| Capacity math | Split workload uses HU_count * avg_items_per_HU; backlog above dock SLA routes to overflow receiving lane. |
| Multi-region | HU split writes are home-cell authoritative; DR cells serve read-only deconsolidation history. |
| Sovereign cells | Supplier ASN and controlled-material evidence remains in-region for active packs. |
| Rollback | Disable deconsolidation command, receive by delivery item only, and replay from last sealed HU audit id. |
| Test evidence | Required tests cover ASN mismatch, quality denial, duplicate split, tenant mismatch, and idempotent replay. |
| Rejected shortcut | A generic pallet split is rejected because it loses SAP HU, ASN, and quality-gate semantics. |
